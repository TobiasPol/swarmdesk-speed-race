//! Base64 decoding (RFC 4648): standard alphabet, `=` padding, no line breaks.
//! Only valid input is generated; behaviour on invalid input is not tested.
//! Output buffer: input length / 4 * 3 bytes.

use super::{random_cases, Case, Target};
use crate::rng::Rng;

pub const TARGET: Target = Target {
    name: "base64_decode",
    summary: "the bytes encoded by a valid padded base64 text",
    reference,
    generate,
    out_cap: |len| len / 4 * 3,
};

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn sextet(symbol: u8) -> u32 {
    ALPHABET.iter().position(|&a| a == symbol).unwrap_or(0) as u32
}

fn reference(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    for group in input.chunks(4) {
        let padding = group.iter().filter(|&&b| b == b'=').count();
        let mut bits = 0u32;
        for &symbol in group {
            bits = (bits << 6) | sextet(symbol);
        }
        let bytes = [(bits >> 16) as u8, (bits >> 8) as u8, bits as u8];
        out.extend_from_slice(&bytes[..3 - padding]);
    }
    out
}

/// Encoding used only to build inputs.
fn encode(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    for chunk in data.chunks(3) {
        let mut group = [0u8; 3];
        group[..chunk.len()].copy_from_slice(chunk);
        let bits = (u32::from(group[0]) << 16) | (u32::from(group[1]) << 8) | u32::from(group[2]);
        for i in 0..4 {
            let symbol = if i <= chunk.len() {
                ALPHABET[((bits >> (18 - 6 * i)) & 63) as usize]
            } else {
                b'='
            };
            out.push(symbol);
        }
    }
    out
}

fn encoded_cases(
    rng: &mut Rng,
    class: &'static str,
    count: usize,
    lo: usize,
    hi: usize,
) -> Vec<Case> {
    let cases = random_cases(rng, class, count, lo, hi);
    cases
        .into_iter()
        .map(|c| Case {
            class,
            input: encode(&c.input),
        })
        .collect()
}

fn generate(rng: &mut Rng) -> Vec<Case> {
    let mut cases = vec![Case {
        class: "empty",
        input: Vec::new(),
    }];
    for (class, len) in [
        ("decoded length 1", 1),
        ("decoded length 2", 2),
        ("decoded length 3", 3),
    ] {
        cases.extend(encoded_cases(rng, class, 2, len, len));
    }
    cases.extend(encoded_cases(rng, "tiny", 5, 4, 16));
    cases.extend(encoded_cases(rng, "medium", 6, 17, 1024));
    cases.extend(encoded_cases(rng, "large", 3, 4096, 65_536));
    let long_len = rng.range(1000, 3000);
    cases.push(Case {
        class: "decoded length 1 mod 3",
        input: encode(&rng.bytes(long_len / 3 * 3 + 1)),
    });
    cases.push(Case {
        class: "decoded length 2 mod 3",
        input: encode(&rng.bytes(long_len / 3 * 3 + 2)),
    });
    cases
}

#[cfg(test)]
mod tests {
    use super::{encode, generate, reference, TARGET};
    use crate::rng::Rng;

    #[test]
    fn rfc4648_vectors() {
        let vectors: [(&[u8], &[u8]); 7] = [
            (b"", b""),
            (b"Zg==", b"f"),
            (b"Zm8=", b"fo"),
            (b"Zm9v", b"foo"),
            (b"Zm9vYg==", b"foob"),
            (b"Zm9vYmE=", b"fooba"),
            (b"Zm9vYmFy", b"foobar"),
        ];
        for (input, expected) in vectors {
            assert_eq!(reference(input), expected);
        }
    }

    #[test]
    fn uses_plus_and_slash() {
        assert_eq!(reference(b"+/+/"), [0xFB, 0xFF, 0xBF]);
    }

    #[test]
    fn round_trip() {
        let data = Rng::new(3).bytes(1000);
        assert_eq!(reference(&encode(&data)), data);
    }

    #[test]
    fn generator_shape() {
        let cases = generate(&mut Rng::new(1));
        assert!((20..=40).contains(&cases.len()));
        assert!(cases.iter().map(|c| c.input.len()).sum::<usize>() < 1_000_000);
        for c in &cases {
            assert_eq!(c.input.len() % 4, 0);
            assert!(reference(&c.input).len() <= (TARGET.out_cap)(c.input.len()));
        }
    }
}
