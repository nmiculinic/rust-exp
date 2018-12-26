extern crate base64;
extern crate hamming;
extern crate hex;
extern crate serde_cbor;

use rv::dist::Categorical;
use rv::traits::*;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;

pub fn load_base64_file<P: AsRef<std::path::Path>>(path: P) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut f = File::open(path)?;
    let mut data = String::new();
    f.read_to_string(&mut data)?;
    data = data.replace(" ", "").replace("\n", "");
    let data = base64::decode(&data)?;
    Ok(data)
}

pub fn load_hex_strings<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
    let mut f = File::open(path)?;
    let mut data = String::new();
    f.read_to_string(&mut data).unwrap();
    let mut all: Vec<Vec<u8>> = Vec::new();
    data.split_whitespace()
        .filter(|x| x.len() > 0)
        .map(|x| hex::decode(x).unwrap())
        .for_each(|x| all.push(x));
    Ok(all)
}

pub fn load_letter_frequency<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<rv::dist::Categorical, Box<dyn Error>> {
    let f = File::open(path)?;
    let freq: HashMap<u8, u32> = serde_cbor::from_reader(f)?;
    let total_cnt: u32 = freq.iter().map(|(_, x)| x).sum();
    let mut weights: [f64; 256] = [1.0 / total_cnt as f64; 256];
    for (k, v) in freq {
        weights[k as usize] = v as f64;
    }
    Ok(Categorical::new(&weights)?)
}
