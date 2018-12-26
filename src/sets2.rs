extern crate base64;
extern crate hex;

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

pub fn detect_mode<F>(oracle: F) -> Mode
where
    F: Fn(&[u8]) -> Vec<u8>,
{
    let plaintext: [u8; 1024] = [0; 1024];
    let cyphertext = oracle(&plaintext);
    if cyphertext[128..128 + 8] == cyphertext[128 + 16..128 + 16 + 8] {
        Mode::ECB
    } else {
        Mode::CBC
    }
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
            assert_eq!(Mode::CBC, detect_mode(|x| encryption_oracle(x, Mode::CBC)),)
        }
        for _ in 0..10 {
            assert_eq!(Mode::ECB, detect_mode(|x| encryption_oracle(x, Mode::ECB)),)
        }
    }

}
