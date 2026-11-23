//! Parse decimal integers.
//! Input: unsigned decimal integers of 1 to 10 digits (value at most 4294967295, leading
//! zeros allowed), separated by single `\n` bytes, no trailing newline. Empty input means
//! no numbers. Output: each value as 4 little-endian bytes, in order.

use super::{Case, Target};
use crate::rng::Rng;

pub const TARGET: Target = Target {
    name: "parse_ints",
    summary: "parse newline-separated decimal numbers into little-endian u32 values",
    reference,
    generate,
    out_cap: |len| len.div_ceil(2) * 4,
};

fn reference(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    if input.is_empty() {
        return out;
    }
    for line in input.split(|&b| b == b'\n') {
        let mut value = 0u64;
        for &digit in line {
            value = value * 10 + u64::from(digit - b'0');
        }
        out.extend_from_slice(&(value as u32).to_le_bytes());
    }
    out
}

fn join(numbers: &[String]) -> Vec<u8> {
    numbers.join("\n").into_bytes()
}

fn plain(values: &[u32]) -> Vec<u8> {
    join(&values.iter().map(u32::to_string).collect::<Vec<_>>())
}

fn random_value(rng: &mut Rng) -> u32 {
    (rng.next_u64() as u32) >> rng.below(32)
}

fn values(rng: &mut Rng, count: usize, max: u64) -> Vec<u32> {
    (0..count).map(|_| rng.below(max) as u32).collect()
}

fn padded(rng: &mut Rng, count: usize) -> Vec<u8> {
    let numbers: Vec<String> = (0..count)
        .map(|_| {
            let value = random_value(rng);
            let width = rng.range(value.to_string().len(), 10);
            format!("{value:0width$}")
        })
        .collect();
    join(&numbers)
}

fn many(rng: &mut Rng, count: usize) -> Vec<u8> {
    plain(&(0..count).map(|_| random_value(rng)).collect::<Vec<_>>())
}

fn generate(rng: &mut Rng) -> Vec<Case> {
    let mut cases = vec![Case {
        class: "empty",
        input: Vec::new(),
    }];
    for value in [0, 7, 4_294_967_295, random_value(rng), random_value(rng)] {
        cases.push(Case {
            class: "single number",
            input: plain(&[value]),
        });
    }
    for _ in 0..4 {
        let count = rng.range(2, 40);
        cases.push(Case {
            class: "small numbers",
            input: plain(&values(rng, count, 100)),
        });
    }
    for count in [2, 10, 300] {
        cases.push(Case {
            class: "maximum values",
            input: plain(&vec![u32::MAX; count]),
        });
    }
    for count in [1, 5, 50, 500] {
        cases.push(Case {
            class: "leading zeros",
            input: padded(rng, count),
        });
    }
    cases.push(Case {
        class: "all zeros",
        input: join(&vec!["0000000000".to_string(); 200]),
    });
    for count in [10, 100, 1000, 10_000, 10_000] {
        cases.push(Case {
            class: "many numbers",
            input: many(rng, count),
        });
    }
    cases
}

#[cfg(test)]
mod tests {
    use super::{generate, join, reference, TARGET};
    use crate::rng::Rng;

    fn words(out: &[u8]) -> Vec<u32> {
        out.chunks(4)
            .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect()
    }

    #[test]
    fn known_values() {
        assert_eq!(words(&reference(b"0\n4294967295\n007")), [0, u32::MAX, 7]);
        assert_eq!(words(&reference(b"12345")), [12345]);
        assert_eq!(
            words(&reference(b"0000000001\n1000000000")),
            [1, 1_000_000_000]
        );
    }

    #[test]
    fn empty_input() {
        assert!(reference(b"").is_empty());
    }

    #[test]
    fn join_has_no_trailing_newline() {
        assert_eq!(join(&["1".into(), "2".into()]), b"1\n2");
    }

    #[test]
    fn generator_shape() {
        let cases = generate(&mut Rng::new(1));
        assert!((20..=40).contains(&cases.len()));
        assert!(cases.iter().map(|c| c.input.len()).sum::<usize>() < 1_000_000);
        for c in &cases {
            assert!(reference(&c.input).len() <= (TARGET.out_cap)(c.input.len()));
        }
    }
}
