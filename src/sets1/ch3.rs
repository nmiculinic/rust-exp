// https://cryptopals.com/sets/1/challenges/3
extern crate base64;
extern crate hex;
use std::collections::HashMap;

#[allow(dead_code)]
pub fn test_xor(a: &Vec<u8>, c: u8) -> String {
    String::from_utf8(a.iter().map(|x| x ^ c).collect()).unwrap()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ch2() {
        let target =
            hex::decode("1b37373331363f78151b7f2b783431333d78397828372d363c78373e783a393b3736")
                .unwrap();
        // http://letterfrequency.org/
        println!("{:?}", freq_analysis(&target));
        for i in 0..126 {
            println!("{} : {}", i, test_xor(&target, i))
        }
        assert_eq!(
            String::from("Cooking MC\'s like a pound of bacon"),
            test_xor(&target, 88)
        )
    }
}
