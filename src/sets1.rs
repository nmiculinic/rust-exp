extern crate base64;
extern crate hamming;
extern crate hex;
extern crate serde_cbor;

use super::io::*;
use rv::dist::Categorical;
use rv::traits::*;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub fn hex_to_base64(a: &str) -> String {
    base64::encode(&hex::decode(a).unwrap())
}

pub fn repeating_xor(a: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    if key.len() == 0 {
        return Err(String::from("key len is zero"));
    }
    Ok(a.iter()
        .zip(key.iter().cycle())
        .map(|(x, y)| x ^ y)
        .collect())
}

pub fn fixed_xor(text: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    if text.len() != key.len() {
        return Err(String::from("different length"));
    }
    repeating_xor(text, key)
}

pub fn single_letter_xor(a: &[u8], key: u8) -> Result<String, std::string::FromUtf8Error> {
    String::from_utf8(a.iter().map(|x| x ^ key).collect())
}

#[allow(dead_code)]
pub fn freq_analysis(a: &[u8]) -> HashMap<u8, usize> {
    let mut freq: HashMap<u8, usize> = HashMap::new();
    for i in a {
        let c = freq.entry(*i).or_insert(0);
        *c += 1;
    }
    freq
}

pub fn freq_to_dist(freq: &HashMap<u8, usize>, xor: u8) -> std::io::Result<Categorical> {
    let total_cnt: usize = freq.iter().map(|(_, x)| x).sum();
    let mut values: [f64; 256] = [1.0 / total_cnt as f64; 256];
    for (k, v) in freq {
        values[(k ^ xor) as usize] = (*v) as f64;
    }
    Categorical::new(&values)
}

pub fn most_likely_xor(
    freq: &HashMap<u8, usize>,
    letter_distribution: &Categorical,
) -> Result<(u8, f64), Box<dyn Error>> {
    let mut best = (0, std::f64::INFINITY);
    let (target_letter, _) = letter_distribution
        .ln_weights
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .unwrap();
    let mut freqvec = freq
        .iter()
        .map(|(a, b)| (*a, *b))
        .collect::<Vec<(u8, usize)>>();
    freqvec.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
    for (t, _) in freqvec.iter().take(10) {
        let candidate = t ^ target_letter as u8;
        match freq_to_dist(&freq, candidate) {
            Ok(other) => {
                let dist = letter_distribution.kl(&other);
                if dist.is_finite() && dist < best.1 {
                    best = (candidate, dist);
                }
            }
            Err(_) => (),
        }
    }
    Ok(best)
}

pub fn auto_single_byte_xor(
    data: &[u8],
    letter_distribution: &Categorical,
) -> Result<(String, u8, f64), Box<dyn Error>> {
    let (key, score) = most_likely_xor(&freq_analysis(data), letter_distribution)?;
    Ok((single_letter_xor(data, key)?, key, score))
}

pub fn edit_distance(a: &[u8], b: &[u8]) -> u32 {
    let sz: usize = 1 + (a.len() + 1) * (b.len() + 1) as usize;
    let mut dp: Vec<u32> = vec![std::u32::MAX; sz];
    let idx = |x: usize, y: usize| x * b.len() + y;

    for i in 0..a.len() {
        dp[idx(i, 0)] = i as u32;
    }
    for i in 0..b.len() {
        dp[idx(0, i)] = i as u32;
    }
    for i in 1..a.len() + 1 {
        for j in 1..b.len() + 1 {
            // match
            if a[i - 1] == b[j - 1] {
                dp[idx(i, j)] = dp[idx(i - 1, j - 1)];
            }

            // mismatch
            dp[idx(i, j)] = std::cmp::min(dp[idx(i, j)], 1 + dp[idx(i - 1, j - 1)]);

            // deletion|insertion depending how you look at it
            dp[idx(i, j)] = std::cmp::min(dp[idx(i, j)], 1 + dp[idx(i, j - 1)]);
            dp[idx(i, j)] = std::cmp::min(dp[idx(i, j)], 1 + dp[idx(i - 1, j)]);
        }
    }
    dp[idx(a.len(), b.len())]
}

pub fn normalized_keysize_score(data: &[u8], size: usize) -> f64 {
    let num_blocks = data.len() / size;
    if num_blocks < 2 {
        return std::f64::INFINITY;
    }
    let mut total = 0;
    let mut total_op = 0; // I'm lazy
    for i in 0..num_blocks - 1 {
        for j in i + 1..num_blocks {
            total += hamming::distance(
                &data[i * size..(i + 1) * size],
                &data[j * size..(j + 1) * size],
            );
            total_op += 1;
        }
    }
    total as f64 / (size * total_op) as f64
}

pub fn auto_known_multi_byte_xor(
    data: &[u8],
    letter_distribution: &Categorical,
    key_len: usize,
) -> Result<(String, Vec<u8>, f64), Box<dyn Error>> {
    let mut v: Vec<Vec<u8>> = Vec::new();
    for _ in 0..key_len {
        v.push(Vec::new())
    }
    for (i, elem) in data.iter().enumerate() {
        v[i % key_len].push(*elem);
    }
    let v = v; // make immutable

    let mut key: Vec<u8> = Vec::new();
    for i in 0..key_len {
        let (_, key_elem, _) = auto_single_byte_xor(&v[i], &letter_distribution)?;
        key.push(key_elem);
    }
    let key = key; // make immutable

    let s = String::from_utf8(repeating_xor(&data, &key)?)?;
    let current_dist = freq_to_dist(&freq_analysis(s.as_bytes()), 0)?;
    Ok((s, key, letter_distribution.kl(&current_dist)))
}

pub fn auto_multi_byte_xor<T>(
    data: &[u8],
    letter_distribution: &Categorical,
    key_len_range: T,
) -> Result<(String, Vec<u8>, f64), Box<dyn Error>>
where
    T: IntoIterator<Item = usize>,
{
    let mut candidate_sizes: Vec<(usize, f64)> = key_len_range
        .into_iter()
        .map(|x| (x, normalized_keysize_score(&data, x)))
        .collect();
    candidate_sizes.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());

    let mut candidates: Vec<(String, Vec<u8>, f64)> = Vec::new();
    for (x, _) in candidate_sizes.iter().take(5) {
        candidates.push(auto_known_multi_byte_xor(&data, &letter_distribution, *x)?);
    }
    candidates.sort_by(|(_, _, a), (_, _, b)| a.partial_cmp(b).unwrap());
    Ok(candidates[0].clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    // https://cryptopals.com/sets/1/challenges/1
    #[test]
    fn test_ch1() {
        assert_eq!(
            "SSdtIGtpbGxpbmcgeW91ciBicmFpbiBsaWtlIGEgcG9pc29ub3VzIG11c2hyb29t", 
            hex_to_base64("49276d206b696c6c696e6720796f757220627261696e206c696b65206120706f69736f6e6f7573206d757368726f6f6d")
        )
    }

    // https://cryptopals.com/sets/1/challenges/2
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

    // https://cryptopals.com/sets/1/challenges/3
    #[test]
    fn test_ch3() -> Result<(), Box<dyn Error>> {
        let target =
            hex::decode("1b37373331363f78151b7f2b783431333d78397828372d363c78373e783a393b3736")
                .unwrap();

        let letter_distribution = load_default_letter_freq()?;
        let (s, key, prob) = auto_single_byte_xor(&target, &letter_distribution)?;
        println!("{}, {}: Prob: {}", key, s, prob);
        assert_eq!(
            String::from("Cooking MC\'s like a pound of bacon"),
            single_letter_xor(&target, 88).unwrap()
        );
        assert_eq!(String::from("Cooking MC\'s like a pound of bacon"), s);
        Ok(())
    }

    #[test]
    fn test_ch4() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("data/sets1/4.txt");
        let data = load_hex_strings(d).unwrap();
        let letter_distribution = load_default_letter_freq().unwrap();
        let mut all = Vec::new();
        for it in data {
            match auto_single_byte_xor(&it, &letter_distribution) {
                Ok((s, _, score)) => {
                    println!("{} :{}", score, s);
                    all.push(s);
                }
                Err(_) => (),
            };
        }
        println!("{}", all.join("\n"));
        assert!(all.contains(&String::from("Now that the party is jumping\n")));
    }

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

    #[test]
    fn test_ch6() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("data/sets1/6.in");
        let data = load_base64_file(d).unwrap();
        let (s, key, score) =
            auto_multi_byte_xor(&data, &load_default_letter_freq().unwrap(), 2..40).unwrap();
        println!("{} {:?}:\n{}", score, key, s);

        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("data/sets1/6.out");
        let mut f = File::open(d).unwrap();
        let mut result = String::new();
        f.read_to_string(&mut result).unwrap();

        assert_eq!(s, result);
    }

    fn load_default_letter_freq() -> Result<Categorical, Box<dyn Error>> {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("data/letter_freq");
        let freq = load_letter_frequency(d)?;
        let mut top: Vec<(String, f64)> = freq
            .ln_weights
            .iter()
            .enumerate()
            .map(|(k, v)| (String::from_utf8(vec![k as u8]), *v))
            .filter(|(k, _)| k.is_ok())
            .map(|(k, v)| (k.unwrap(), v))
            .collect();

        top.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
        top.truncate(20);
        for (k, v) in top {
            println!("{}: {}", k, v);
        }
        Ok(freq)
    }

    #[test]
    fn test_edit_distance() {
        assert_eq!(
            edit_distance(
                String::from("this is a test").as_bytes(),
                String::from("wokka wokka!!!").as_bytes(),
            ),
            14
        )
    }

    #[test]
    fn test_hamming_distance() {
        assert_eq!(
            hamming::distance(
                String::from("this is a test").as_bytes(),
                String::from("wokka wokka!!!").as_bytes(),
            ),
            37
        )
    }
}
