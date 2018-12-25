extern crate serde;
extern crate serde_cbor;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

fn main() {
    let mut file = File::open("data/shakespeare-hamlet.txt").unwrap();
    let mut buff: [u8; 1024] = [0; 1024];
    let mut total = HashMap::new();

    loop {
        match file.read(&mut buff) {
            Ok(0) => break,
            Ok(n) => {
                for i in 0..n {
                    let c = total.entry(buff[i]).or_insert(0);
                    *c += 1;
                }
            }
            Err(x) => panic!(x),
        }
    }
    let mut out = File::create("data/letter_freq").unwrap();
    serde_cbor::to_writer(&mut out, &total).unwrap();
}
