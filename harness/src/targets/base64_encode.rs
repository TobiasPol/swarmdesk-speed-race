//! Base64 encoding (RFC 4648): standard alphabet, `=` padding, no line breaks.
//! Output buffer: 4 * ceil(n / 3) bytes.

use super::{random_cases, Case, Target};
use crate::rng::Rng;

pub const TARGET: Target = Target {
    name: "base64_encode",
    summary: "standard base64 text of the input, with padding",
    reference,
    generate,
    out_cap: |len| 4 * len.div_ceil(3),
};

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn reference(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    for chunk in input.chunks(3) {
        let mut group = [0u8; 3];
        group[..chunk.len()].copy_from_slice(chunk);
        let bits = (u32::from(group[0]) << 16) | (u32::from(group[1]) << 8) | u32::from(group[2]);
        for i in 0..4 {
            // A chunk of n bytes carries n + 1 meaningful sextets.
            if i <= chunk.len() {
                out.push(ALPHABET[((bits >> (18 - 6 * i)) & 63) as usize]);
            } else {
                out.push(b'=');
            }
        }
    }
    out
}

fn generate(rng: &mut Rng) -> Vec<Case> {
    let mut cases = vec![Case {
        class: "empty",
        input: Vec::new(),
    }];
    for (class, len) in [("length 1", 1), ("length 2", 2), ("length 3", 3)] {
        cases.extend(random_cases(rng, class, 2, len, len));
    }
    cases.extend(random_cases(rng, "tiny", 5, 4, 16));
    cases.extend(random_cases(rng, "medium", 6, 17, 1024));
    cases.extend(random_cases(rng, "between", 3, 1025, 4095));
    cases.extend(random_cases(rng, "large", 3, 4096, 65_536));
    cases.push(Case {
        class: "all ones",
        input: vec![0xFF; rng.range(300, 2000)],
    });
    cases
}

#[cfg(test)]
mod tests {
    use super::{generate, reference, TARGET};
    use crate::rng::Rng;

    #[test]
    fn rfc4648_vectors() {
        let vectors: [(&[u8], &[u8]); 7] = [
            (b"", b""),
            (b"f", b"Zg=="),
            (b"fo", b"Zm8="),
            (b"foo", b"Zm9v"),
            (b"foob", b"Zm9vYg=="),
            (b"fooba", b"Zm9vYmE="),
            (b"foobar", b"Zm9vYmFy"),
        ];
        for (input, expected) in vectors {
            assert_eq!(reference(input), expected);
        }
    }

    #[test]
    fn uses_plus_and_slash() {
        assert_eq!(reference(&[0xFB, 0xFF, 0xBF]), b"+/+/");
    }

    #[test]
    fn generator_shape() {
        let cases = generate(&mut Rng::new(1));
        assert!((20..=40).contains(&cases.len()));
        assert!(cases.iter().map(|c| c.input.len()).sum::<usize>() < 1_000_000);
        for c in &cases {
            assert_eq!((TARGET.out_cap)(c.input.len()), reference(&c.input).len());
        }
    }
}
