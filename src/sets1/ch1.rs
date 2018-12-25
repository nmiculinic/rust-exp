// https://cryptopals.com/sets/1/challenges/1
extern crate base64;
extern crate hex;

#[allow(dead_code)]
pub fn hex_to_base64(a: &str) -> String {
    base64::encode(&hex::decode(a).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ch1() {
        assert_eq!(
            "SSdtIGtpbGxpbmcgeW91ciBicmFpbiBsaWtlIGEgcG9pc29ub3VzIG11c2hyb29t", 
            hex_to_base64("49276d206b696c6c696e6720796f757220627261696e206c696b65206120706f69736f6e6f7573206d757368726f6f6d")
        )
    }
}
