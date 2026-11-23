//! First occurrence of a needle in a haystack.
//! Input: the needle length m as 4 little-endian bytes, then m needle bytes, then the
//! haystack. Output: the index of the first occurrence as 4 little-endian bytes, or
//! 0xFFFFFFFF if there is none. An empty needle gives 0.

use super::{Case, Target};
use crate::rng::Rng;

pub const TARGET: Target = Target {
    name: "find_substring",
    summary: "index of the first occurrence of a needle in a haystack, or 0xFFFFFFFF",
    reference,
    generate,
    out_cap: |_| 4,
};

fn reference(input: &[u8]) -> Vec<u8> {
    let m = u32::from_le_bytes([input[0], input[1], input[2], input[3]]) as usize;
    let needle = &input[4..4 + m];
    let haystack = &input[4 + m..];
    let mut found = u32::MAX;
    if m <= haystack.len() {
        for start in 0..=haystack.len() - m {
            if &haystack[start..start + m] == needle {
                found = start as u32;
                break;
            }
        }
    }
    found.to_le_bytes().to_vec()
}

fn build(needle: &[u8], haystack: &[u8]) -> Vec<u8> {
    let mut input = (needle.len() as u32).to_le_bytes().to_vec();
    input.extend_from_slice(needle);
    input.extend_from_slice(haystack);
    input
}

fn case(class: &'static str, needle: &[u8], haystack: &[u8]) -> Case {
    Case {
        class,
        input: build(needle, haystack),
    }
}

/// Random bytes drawn from the first `alphabet` byte values.
fn symbols(rng: &mut Rng, len: usize, alphabet: u64) -> Vec<u8> {
    (0..len).map(|_| rng.below(alphabet) as u8).collect()
}

fn located_cases(rng: &mut Rng) -> Vec<Case> {
    let mut cases = Vec::new();
    for _ in 0..2 {
        let hay_len = rng.range(30, 400);
        let hay = rng.bytes(hay_len);
        let k = rng.range(1, 12);
        let mid = rng.range(1, hay.len() - k);
        cases.push(case("needle at the start", &hay[..k], &hay));
        cases.push(case("needle in the middle", &hay[mid..mid + k], &hay));
        cases.push(case("needle at the end", &hay[hay.len() - k..], &hay));
        let mut absent = symbols(rng, k, 128);
        absent.push(0xFF);
        cases.push(case(
            "needle absent",
            &absent,
            &symbols(rng, hay.len(), 128),
        ));
    }
    cases
}

fn edge_cases(rng: &mut Rng) -> Vec<Case> {
    let (short_len, long_len, any_len) = (rng.range(0, 10), rng.range(11, 30), rng.range(1, 100));
    let short = rng.bytes(short_len);
    let long_needle = rng.bytes(long_len);
    let any = rng.bytes(any_len);
    let three = rng.bytes(3);
    vec![
        case("needle longer than haystack", &long_needle, &short),
        case("empty needle", b"", &any),
        case("empty needle and haystack", b"", b""),
        case("empty haystack", &three, b""),
    ]
}

fn repetitive_cases(rng: &mut Rng) -> Vec<Case> {
    let mut cases = Vec::new();
    for _ in 0..2 {
        let n = rng.range(1000, 30_000);
        let k = rng.range(3, 60);
        let mut hay = vec![b'a'; n];
        hay.push(b'b');
        let mut needle = vec![b'a'; k];
        needle.push(b'b');
        cases.push(case("repetitive haystack", &needle, &hay));
        needle[k] = b'c';
        cases.push(case("repetitive haystack, absent", &needle, &hay));
    }
    cases
}

fn alphabet_cases(rng: &mut Rng) -> Vec<Case> {
    let mut cases = Vec::new();
    for alphabet in 2..=4 {
        let hay_len = rng.range(200, 3000);
        let hay = symbols(rng, hay_len, alphabet);
        let k = rng.range(3, 10);
        let at = rng.range(0, hay.len() - k);
        cases.push(case("small alphabet", &hay[at..at + k], &hay));
        let other = symbols(rng, k + 6, alphabet);
        cases.push(case("small alphabet", &other, &hay));
    }
    cases
}

fn large_cases(rng: &mut Rng) -> Vec<Case> {
    let hay_len = rng.range(40_000, 65_536);
    let hay = rng.bytes(hay_len);
    let n = hay.len();
    let near_end = rng.range(n - 200, n - 20);
    let mut absent = hay[1000..1020].to_vec();
    absent[5] ^= 0x55;
    vec![
        case("large haystack", &hay[near_end..near_end + 16], &hay),
        case("large haystack, absent", &absent, &symbols(rng, n, 100)),
    ]
}

fn generate(rng: &mut Rng) -> Vec<Case> {
    let mut cases = located_cases(rng);
    cases.extend(edge_cases(rng));
    cases.extend(repetitive_cases(rng));
    cases.extend(alphabet_cases(rng));
    cases.extend(large_cases(rng));
    cases
}

#[cfg(test)]
mod tests {
    use super::{build, generate, reference, TARGET};
    use crate::rng::Rng;

    fn find(needle: &[u8], haystack: &[u8]) -> u32 {
        let out = reference(&build(needle, haystack));
        u32::from_le_bytes([out[0], out[1], out[2], out[3]])
    }

    #[test]
    fn known_positions() {
        assert_eq!(find(b"abc", b"abcdef"), 0);
        assert_eq!(find(b"cd", b"abcdef"), 2);
        assert_eq!(find(b"ef", b"abcdef"), 4);
        assert_eq!(find(b"abab", b"aabababab"), 1);
        assert_eq!(find(b"aab", b"aaaaaab"), 4);
    }

    #[test]
    fn not_found_and_empty() {
        assert_eq!(find(b"xyz", b"abcdef"), u32::MAX);
        assert_eq!(find(b"abcdefg", b"abcdef"), u32::MAX);
        assert_eq!(find(b"a", b""), u32::MAX);
        assert_eq!(find(b"", b"abc"), 0);
        assert_eq!(find(b"", b""), 0);
    }

    #[test]
    fn generator_shape() {
        for seed in 0..5 {
            let cases = generate(&mut Rng::new(seed));
            assert!((20..=40).contains(&cases.len()));
            assert!(cases.iter().map(|c| c.input.len()).sum::<usize>() < 1_000_000);
            for c in &cases {
                assert_eq!(reference(&c.input).len(), (TARGET.out_cap)(c.input.len()));
            }
        }
    }
}
