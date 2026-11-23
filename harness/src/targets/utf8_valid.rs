//! UTF-8 validity. Output: 1 byte, 1 if the input is valid UTF-8, else 0.

use super::{random_cases, Case, Target};
use crate::rng::Rng;

pub const TARGET: Target = Target {
    name: "utf8_valid",
    summary: "1 if the input is valid UTF-8, else 0",
    reference,
    generate,
    out_cap: |_| 1,
};

fn reference(input: &[u8]) -> Vec<u8> {
    vec![u8::from(std::str::from_utf8(input).is_ok())]
}

/// Valid UTF-8 made of `count` characters. `width` is 1 to 4 for characters of
/// that many bytes only, or 0 for a mix of all four.
fn text(rng: &mut Rng, count: usize, width: usize) -> Vec<u8> {
    let mut out = Vec::new();
    let mut buf = [0u8; 4];
    while out.len() < count {
        let w = if width == 0 { rng.range(1, 4) } else { width };
        let (lo, hi) = [
            (0x20, 0x7E),
            (0x80, 0x7FF),
            (0x800, 0xFFFF),
            (0x10000, 0x10FFFF),
        ][w - 1];
        let code = rng.range(lo, hi) as u32;
        if let Some(c) = char::from_u32(code) {
            out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
        }
    }
    out
}

/// Valid text, then one of the `bad` sequences, then more valid text.
fn with_bad(rng: &mut Rng, bad: &[&[u8]]) -> Vec<u8> {
    let (before, after) = (rng.range(0, 40), rng.range(0, 40));
    let choice = bad[rng.below(bad.len() as u64) as usize];
    let mut out = text(rng, before, 0);
    out.extend_from_slice(choice);
    out.extend(text(rng, after, 0));
    out
}

fn corrupted(rng: &mut Rng) -> Vec<u8> {
    let count = rng.range(5, 200);
    let mut bytes = text(rng, count, 0);
    let at = rng.below(bytes.len() as u64) as usize;
    bytes[at] = rng.byte();
    if std::str::from_utf8(&bytes).is_ok() {
        bytes[at] = 0xFF;
    }
    bytes
}

fn truncated(rng: &mut Rng) -> Vec<u8> {
    let count = rng.range(0, 100);
    let mut bytes = text(rng, count, 0);
    bytes.extend_from_slice("\u{20AC}\u{1F600}".as_bytes());
    bytes.pop();
    bytes
}

fn invalid_cases(rng: &mut Rng) -> Vec<Case> {
    let overlong: [&[u8]; 4] = [
        &[0xC0, 0x80],
        &[0xC1, 0xBF],
        &[0xE0, 0x80, 0x80],
        &[0xF0, 0x80, 0x80, 0x80],
    ];
    let surrogate: [&[u8]; 3] = [
        &[0xED, 0xA0, 0x80],
        &[0xED, 0xBF, 0xBF],
        &[0xED, 0xB0, 0x80],
    ];
    let too_high: [&[u8]; 3] = [
        &[0xF4, 0x90, 0x80, 0x80],
        &[0xF5, 0x80, 0x80, 0x80],
        &[0xF7, 0xBF, 0xBF, 0xBF],
    ];
    let mut cases = random_cases(rng, "random bytes", 3, 1, 64);
    for _ in 0..3 {
        cases.push(Case {
            class: "one corrupted byte",
            input: corrupted(rng),
        });
    }
    for _ in 0..2 {
        cases.push(Case {
            class: "overlong encoding",
            input: with_bad(rng, &overlong),
        });
        cases.push(Case {
            class: "surrogate",
            input: with_bad(rng, &surrogate),
        });
        cases.push(Case {
            class: "above U+10FFFF",
            input: with_bad(rng, &too_high),
        });
        cases.push(Case {
            class: "truncated final sequence",
            input: truncated(rng),
        });
    }
    cases
}

fn valid_cases(rng: &mut Rng) -> Vec<Case> {
    let mut cases = vec![Case {
        class: "empty",
        input: Vec::new(),
    }];
    for (class, count, width) in [("ascii", 40, 1), ("ascii", 3000, 1), ("2-byte", 200, 2)] {
        cases.push(Case {
            class,
            input: text(rng, count, width),
        });
    }
    for (class, count, width) in [("3-byte", 200, 3), ("4-byte", 200, 4), ("ascii", 10, 1)] {
        cases.push(Case {
            class,
            input: text(rng, count, width),
        });
    }
    for count in [8, 60, 500, 4000, 12_000] {
        cases.push(Case {
            class: "multi-byte mix",
            input: text(rng, count, 0),
        });
    }
    cases
}

fn generate(rng: &mut Rng) -> Vec<Case> {
    let mut cases = valid_cases(rng);
    cases.extend(invalid_cases(rng));
    cases
}

#[cfg(test)]
mod tests {
    use super::{generate, reference, TARGET};
    use crate::rng::Rng;

    #[test]
    fn known_valid() {
        assert_eq!(reference(b""), [1]);
        assert_eq!(reference(b"plain ascii"), [1]);
        assert_eq!(reference("h\u{e9}llo \u{20AC} \u{1F600}".as_bytes()), [1]);
        assert_eq!(reference(&[0xF4, 0x8F, 0xBF, 0xBF]), [1]);
    }

    #[test]
    fn known_invalid() {
        assert_eq!(reference(&[0xC0, 0x80]), [0]);
        assert_eq!(reference(&[0xED, 0xA0, 0x80]), [0]);
        assert_eq!(reference(&[0xF4, 0x90, 0x80, 0x80]), [0]);
        assert_eq!(reference(&[0xE2, 0x82]), [0]);
        assert_eq!(reference(&[0x80]), [0]);
    }

    #[test]
    fn classes_have_the_right_answer() {
        for seed in 0..10 {
            let cases = generate(&mut Rng::new(seed));
            assert!((20..=40).contains(&cases.len()));
            assert!(cases.iter().map(|c| c.input.len()).sum::<usize>() < 1_000_000);
            let valid = cases.iter().filter(|c| reference(&c.input) == [1]).count();
            assert!(valid * 100 >= cases.len() * 40 && valid * 100 <= cases.len() * 60);
            for c in &cases {
                assert_eq!((TARGET.out_cap)(c.input.len()), 1);
                let must_be_invalid = !matches!(c.class, "empty" | "ascii" | "2-byte" | "3-byte")
                    && !matches!(c.class, "4-byte" | "multi-byte mix" | "random bytes");
                if must_be_invalid {
                    assert_eq!(reference(&c.input), [0], "{}", c.class);
                }
            }
        }
    }
}
