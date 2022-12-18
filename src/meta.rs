
use serde::{Deserialize};


#[derive(Debug, Deserialize)]
pub struct Meta {
   web: MaskedIps,
   api: MaskedIps,
   git: MaskedIps,
}

type MaskedIps = Vec<String>;
type IpStr = String;

#[derive(Debug)]
pub struct Ip(pub IpStr);

impl From<&String> for Ip {
   fn from(value: &String) -> Self {
      let ip_str = value.split('/').next().unwrap();
      Ip(ip_str.to_string())
   }
}


impl Meta {
   fn new(str: &str) -> Self {
      serde_json::from_str(str).unwrap()
   }
}

pub fn get_github_ips(meta_path: &str) -> anyhow::Result<Vec<Ip>> {
   let meta = Meta::new(meta_path);
   let ips: Vec<Ip> =
      meta.git.iter().map(|x| x.into())
         .chain(meta.api.iter().map(|x| x.into()))
         .chain(meta.web.iter().map(|x| x.into()))
         .collect();
   Ok(ips)
}

#[cfg(test)]
mod tests {
   use crate::meta::{Ip, Meta};
   
   #[test]
   fn new_meta_should_work() {
      let my_str = include_str!("../meta.json");
      let meta = Meta::new(my_str);
      
      assert!(!meta.web.is_empty());
   }
   
   #[test]
   fn parse_to_ip_should_work() {
      let my_str = include_str!("../meta.json");
      let meta = Meta::new(my_str);
      let ips: Vec<Ip> = meta.web.iter().map(|x| Ip(x.to_string())).collect();
      
      assert!(!ips.is_empty())
   }
}