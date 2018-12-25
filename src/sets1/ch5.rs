// https://cryptopals.com/sets/1/challenges/5
extern crate base64;
extern crate hex;
use super::ch2::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ch5() {
        const DATA: &'static str = "Burning 'em, if you ain't quick and nimble
I go crazy when I hear a cymbal";
        const KEY: &'static str = "ICE";

        let g = repeating_xor(&Vec::from(DATA.as_bytes()), &Vec::from(KEY.as_bytes())).unwrap();
        assert_eq!(
            "0b3637272a2b2e63622c2e69692a23693a2a3c6324202d623d63343c2a26226324272765272a282b2f20430a652e2c652a3124333a653e2b2027630c692b20283165286326302e27282f",
            hex::encode(g),
        )
    }
}
