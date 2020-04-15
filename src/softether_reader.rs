use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct SoftEtherError {
    pub msg: String,
}

impl fmt::Display for SoftEtherError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for SoftEtherError {
    fn description(&self) -> &str {
        &self.msg
    }
}

pub struct SoftEtherReader;

impl SoftEtherReader {    
    pub fn decode_packets(src: &str) -> Result<f64, Box<dyn Error>> {
        let ret = String::from(src)
            .replace(",", "")
            .replace(" パケット", "")
            .replace(" packets", "")
            .replace(" 数据包", "")
            .parse()?;
        Ok(ret)
    }

    pub fn decode_bytes(src: &str) -> Result<f64, Box<dyn Error>> {
        let ret = String::from(src)
            .replace(",", "")
            .replace(" バイト", "")
            .replace(" bytes", "")
            .replace(" 字节", "")
            .parse()?;
        Ok(ret)
    }

    pub fn decode_connections(src: &str) -> Result<(f64, f64), Box<dyn Error>> {
        if !src.contains('/') {
            Ok((0.0, 0.0))
        } else {
            let src: Vec<_> = src.split('/').collect();
            let ret0: f64 = src[0].trim().parse()?;
            let ret1: f64 = src[1].trim().parse()?;
            Ok((ret0, ret1))
        }
    }
}

