// https://cryptopals.com/sets/1/challenges/2
extern crate base64;
extern crate hex;

#[allow(dead_code)]
pub fn fixed_xor(a: &Vec<u8>, b: &Vec<u8>) -> Result<Vec<u8>, String> {
    if a.len() != b.len() {
        return Err(String::from("different length"));
    }
    Ok(a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ch2() {
        assert_eq!(
            hex::decode("746865206b696420646f6e277420706c6179").unwrap(),
            fixed_xor(
                &hex::decode("1c0111001f010100061a024b53535009181c").unwrap(),
                &hex::decode("686974207468652062756c6c277320657965").unwrap(),
            )
            .unwrap(),
        )
    }
}
