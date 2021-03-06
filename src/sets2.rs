extern crate base64;
extern crate hex;
extern crate pretty_hex;
use pretty_hex::*;

use super::io::*;
use openssl::symm::{decrypt, encrypt, Cipher};
use rand::prelude::*;
use std::path::PathBuf;

pub fn pkcs_7_padding(data: &[u8], padding_size: usize) -> Vec<u8> {
    let mut sol = Vec::from(data);
    let amount = padding_size - (data.len() % padding_size);
    for _ in 0..amount {
        sol.push(amount as u8);
    }
    sol
}

#[derive(Eq, PartialEq, Debug)]
pub enum Mode {
    CBC,
    ECB,
}

pub fn encryption_oracle(data: &[u8], mode: Mode) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut key: [u8; 16] = [0; 16];
    let mut iv: [u8; 16] = [0; 16];
    rng.fill_bytes(&mut key);
    rng.fill_bytes(&mut iv);

    let mut d: Vec<u8> = Vec::new();
    for _ in 0..rng.gen_range(5, 11) {
        d.push(rng.next_u32() as u8);
    }
    data.iter().for_each(|x| d.push(*x));
    for _ in 0..rng.gen_range(5, 11) {
        d.push(rng.next_u32() as u8);
    }
    let cipher = match mode {
        Mode::CBC => Cipher::aes_128_cbc(),
        Mode::ECB => Cipher::aes_128_ecb(),
    };
    encrypt(cipher, &key, Some(&iv), &d).unwrap()
}

pub fn detect_mode<F>(oracle: &F) -> Mode
where
    F: Fn(&[u8]) -> Vec<u8>,
{
    let plaintext: [u8; 1024] = [0; 1024];
    let cyphertext = oracle(&plaintext);
    const offset: usize = 512;
    if cyphertext[offset..offset + 8] == cyphertext[offset + 16..offset + 16 + 8] {
        Mode::ECB
    } else {
        Mode::CBC
    }
}

pub fn find_block_size<F>(oracle: F) -> (usize, usize)
where
    F: Fn(&[u8]) -> Vec<u8>,
{
    let mut v: Vec<u8> = Vec::new();
    let sz = oracle(&v).len();
    let mut padding_sz = 0;
    while oracle(&v).len() == sz {
        padding_sz += 1;
        v.push(0);
    }
    let sz = oracle(&v).len();
    let mut block_size = 0;
    while oracle(&v).len() == sz {
        block_size += 1;
        v.push(0);
    }
    let target_size = sz - padding_sz - block_size;
    (block_size, target_size)
}
pub fn byte_at_time_ecb_simple<F>(oracle: &F) -> Vec<u8>
where
    F: Fn(&[u8]) -> Vec<u8>,
{
    let (block_size, plaintext_size) = find_block_size(oracle);
    println!("block size: {}, {}", block_size, plaintext_size);
    assert_eq!(detect_mode(&oracle), Mode::ECB);

    let mut sol = Vec::new();
    let mut padding: Vec<u8> = Vec::with_capacity(block_size * 256 + 2 * block_size);
    for i in 0..plaintext_size {
        padding.clear();
        {
            let pad = |cnt: usize, arr: &mut Vec<u8>| {
                if cnt > sol.len() {
                    for _ in 0..cnt - sol.len() {
                        arr.push(0);
                    }
                    arr.append(&mut sol.clone());
                } else {
                    arr.append(&mut Vec::from(&sol[sol.len() - cnt..]));
                }
            };
            for blk_id in 0..256 {
                pad(block_size - 1, &mut padding);
                padding.push(blk_id as u8);
            }

            for _ in 0..(2 * block_size - 1 - (i % block_size)) % block_size {
                padding.push(0);
            }
        }
        let cyphertext = oracle(&padding);
        let mut target: Option<u8> = None;
        let target_block = 256 + (sol.len() / block_size);
        for blk_id in 0..256 {
            if &cyphertext[blk_id * block_size..(blk_id + 1) * block_size]
                == &cyphertext[target_block * block_size..(target_block + 1) * block_size]
            {
                match target {
                    Some(x) => panic!("Collision between {} and {}", x, blk_id),
                    None => target = Some(blk_id as u8),
                };
            }
        }
        sol.push(target.unwrap());
    }
    sol
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_ch9() {
        assert_eq!(
            pkcs_7_padding(b"YELLOW SUBMARINE", 20),
            b"YELLOW SUBMARINE\x04\x04\x04\x04",
        )
    }

    #[test]
    fn test_ch10() {
        let data = load_base64_file(PathBuf::from(
            env!("CARGO_MANIFEST_DIR").to_owned() + "/data/10.in",
        ))
        .unwrap();

        let cipher = Cipher::aes_128_cbc();
        let key = b"YELLOW SUBMARINE";
        let iv = b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
        let pt = String::from_utf8(decrypt(cipher, key, Some(iv), &data).unwrap()).unwrap();
        // println!("{}", pt);
        // assert!(false) // TODO: Implementet CBC on your own
    }

    #[test]
    fn test_ch11() {
        for _ in 0..10 {
            assert_eq!(Mode::CBC, detect_mode(&|x| encryption_oracle(x, Mode::CBC)),)
        }
        for _ in 0..10 {
            assert_eq!(Mode::ECB, detect_mode(&|x| encryption_oracle(x, Mode::ECB)),)
        }
    }

    #[test]
    fn test_ch12() {
        let data = load_base64_file(PathBuf::from(
            env!("CARGO_MANIFEST_DIR").to_owned() + "/data/12.in",
        ))
        .unwrap();

        println!("plaintext size: {}", data.len());
        let mut rng = rand::thread_rng();
        let mut key: [u8; 16] = [0; 16];
        rng.fill_bytes(&mut key);
        let key = key; // make key immutable
        let cipher = Cipher::aes_128_ecb();

        let oracle = |x: &[u8]| {
            let mut v = Vec::from(x);
            v.append(&mut data.clone());
            // println!("data:\n{:?}", v.hex_dump());
            encrypt(cipher, &key, None, &v).unwrap()
        };
        assert_eq!(data, byte_at_time_ecb_simple(&oracle))
    }

}
