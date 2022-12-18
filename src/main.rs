#![feature(addr_parse_ascii)]

use std::net::{Shutdown, SocketAddr, TcpStream, ToSocketAddrs};

use std::time::{Duration, Instant};
use crate::meta::{get_github_ips, Ip};

use anyhow::Result;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

mod meta;

#[tokio::main]
async fn main() -> Result<()> {
   let my_str = include_str!("../meta.json");
   let ips = get_github_ips(my_str)?;
   if let Some(fastest_ip) = get_fastest_ip(&ips) {
      println!("Fastest ip is {}", fastest_ip.0);
   } else {
      println!("No ip found");
   }
   Ok(())
}


fn get_fastest_ip(ips: &[Ip]) -> Option<Ip> {
   let res = ips.par_iter().map(|ip:&Ip| format!("{}:{}", &ip.0, 443).to_socket_addrs().unwrap().next().unwrap())
      .filter(|&addr: &SocketAddr| addr.is_ipv4())
      .map(|addr| {
         let start = Instant::now();
         if let Ok(stream) = TcpStream::connect_timeout(&addr, Duration::from_millis(100)) {
            stream.shutdown(Shutdown::Both).expect("shutdown call failed");
            (start.elapsed().as_millis(), addr.ip().to_string())
         } else {
            (start.elapsed().as_millis(), addr.ip().to_string())
         }
      }).min();
   match res {
      Some(t) => Some(Ip(t.1)),
      None => None
   }
}
