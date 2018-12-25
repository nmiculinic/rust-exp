// https://cryptopals.com/sets/1/challenges/3
extern crate base64;
extern crate hex;
use std::collections::HashMap;

#[allow(dead_code)]
pub const COMMON_LETTERS: &'static str = " eariotnEARIOTN";

#[allow(dead_code)]
pub fn test_xor(a: &Vec<u8>, c: u8) -> Result<String, std::string::FromUtf8Error> {
    String::from_utf8(a.iter().map(|x| x ^ c).collect())
}

#[allow(dead_code)]
pub fn freq_analysis(a: &Vec<u8>) -> HashMap<u8, u32> {
    let mut freq: HashMap<u8, u32> = HashMap::new();
    for i in a {
        let c = freq.entry(*i).or_insert(0);
        *c += 1;
    }
    freq
}

#[allow(dead_code)]
pub fn test_common_letters(a: &Vec<u8>, letters: &[u8]) -> Vec<String> {
    let top_freq: (u8, u32) = freq_analysis(&a)
        .iter()
        .map(|(x, y)| (*x, *y))
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .unwrap();

    let mut all = Vec::new();
    for common in letters {
        match test_xor(&a, common ^ top_freq.0) {
            Ok(x) => all.push(x),
            _ => (),
        }
    }
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ch3() {
        let target =
            hex::decode("1b37373331363f78151b7f2b783431333d78397828372d363c78373e783a393b3736")
                .unwrap();
        // http://letterfrequency.org/
        assert!(test_common_letters(&target, COMMON_LETTERS.as_bytes())
            .contains(&String::from("Cooking MC\'s like a pound of bacon")));
        assert_eq!(
            String::from("Cooking MC\'s like a pound of bacon"),
            test_xor(&target, 88).unwrap()
        );
    }
}
