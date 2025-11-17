use crate::meta::{get_github_ips, Ip};
use anyhow::Result;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::net::{Shutdown, SocketAddr, TcpStream, ToSocketAddrs};
use std::path::Path;
use std::time::{Duration, Instant};

mod meta;

#[tokio::main]
async fn main() -> Result<()> {
    let my_str = include_str!("../meta.json");
    let ips = get_github_ips(my_str)?;
    if let Some(fastest_ip) = get_fastest_ip(&ips) {
        println!("Fastest ip is {}", fastest_ip.0);
        update_hosts_file(&fastest_ip.0)?;
    } else {
        println!("No ip found");
    }
    Ok(())
}

fn get_fastest_ip(ips: &[Ip]) -> Option<Ip> {
    let res = ips
        .par_iter()
        .map(|ip: &Ip| {
            format!("{}:{}", &ip.0, 443)
                .to_socket_addrs()
                .unwrap()
                .next()
                .unwrap()
        })
        .filter(|&addr: &SocketAddr| addr.is_ipv4())
        .map(|addr| {
            let start = Instant::now();
            if let Ok(stream) = TcpStream::connect_timeout(&addr, Duration::from_millis(100)) {
                stream
                    .shutdown(Shutdown::Both)
                    .expect("shutdown call failed");
                (start.elapsed().as_millis(), addr.ip().to_string())
            } else {
                (start.elapsed().as_millis(), addr.ip().to_string())
            }
        })
        .min();
    match res {
        Some(t) => Some(Ip(t.1)),
        None => None,
    }
}
/// 更新 hosts 文件
fn update_hosts_file(ip: &str) -> Result<()> {
    let hosts_path = get_hosts_path();

    // 读取现有内容
    let content = fs::read_to_string(&hosts_path)?;

    // 构造新的 hosts 条目
    let github_entries = [
        format!("{} github.com", ip),
        format!("{} assets-cdn.github.com", ip),
        format!("{} github.global.ssl.fastly.net", ip),
    ];

    // 移除旧的 GitHub 条目
    let mut new_content = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.contains("github.com")
            && !trimmed.contains("github.global.ssl.fastly.net")
            && !trimmed.is_empty()
        {
            new_content.push_str(line);
            new_content.push('\n');
        }
    }

    // 添加新的条目
    new_content.push_str("\n# GitHub Fastest IP (Auto Updated)\n");
    for entry in &github_entries {
        new_content.push_str(&format!("{}\n", entry));
    }

    // 写入文件
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&hosts_path)?;
    file.write_all(new_content.as_bytes())?;

    println!("已更新 hosts 文件: {}", hosts_path.display());
    Ok(())
}

/// 获取 hosts 文件路径
fn get_hosts_path() -> std::path::PathBuf {
    #[cfg(windows)]
    {
        Path::new(r"C:\Windows\System32\drivers\etc\hosts").to_path_buf()
    }
    #[cfg(not(windows))]
    {
        Path::new("/etc/hosts").to_path_buf()
    }
}
