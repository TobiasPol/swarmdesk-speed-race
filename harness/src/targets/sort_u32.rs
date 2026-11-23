//! Sort unsigned 32-bit integers.
//! Input: n values, 4 bytes each, little-endian. Output: the same values, ascending.

use super::{Case, Target};
use crate::rng::Rng;

pub const TARGET: Target = Target {
    name: "sort_u32",
    summary: "sort little-endian u32 values ascending",
    reference,
    generate,
    out_cap: |len| len,
};

fn to_values(input: &[u8]) -> Vec<u32> {
    input
        .chunks(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

fn to_bytes(values: &[u32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}

fn reference(input: &[u8]) -> Vec<u8> {
    let mut values = to_values(input);
    values.sort();
    to_bytes(&values)
}

fn random_values(rng: &mut Rng, count: usize) -> Vec<u32> {
    (0..count).map(|_| rng.next_u64() as u32).collect()
}

fn case(class: &'static str, values: &[u32]) -> Case {
    Case {
        class,
        input: to_bytes(values),
    }
}

fn sized_cases(rng: &mut Rng, class: &'static str, n: usize, lo: usize, hi: usize) -> Vec<Case> {
    (0..n)
        .map(|_| {
            let count = rng.range(lo, hi);
            case(class, &random_values(rng, count))
        })
        .collect()
}

fn structured_cases(rng: &mut Rng) -> Vec<Case> {
    let sorted_len = rng.range(1000, 8000);
    let mut sorted = random_values(rng, sorted_len);
    sorted.sort();
    let mut reversed = sorted.clone();
    reversed.reverse();
    let equal_len = rng.range(500, 5000);
    let equal_value = rng.next_u64() as u32;
    let distinct_len = rng.range(2, 5);
    let distinct = random_values(rng, distinct_len);
    let few_len = rng.range(1000, 8000);
    let few: Vec<u32> = (0..few_len)
        .map(|_| distinct[rng.below(distinct.len() as u64) as usize])
        .collect();
    vec![
        case("already sorted", &sorted),
        case("reverse sorted", &reversed),
        case("all equal", &vec![equal_value; equal_len]),
        case("few distinct", &few),
    ]
}

fn generate(rng: &mut Rng) -> Vec<Case> {
    let mut cases = vec![Case {
        class: "empty",
        input: Vec::new(),
    }];
    cases.push(case("single", &random_values(rng, 1)));
    cases.extend(sized_cases(rng, "tiny", 6, 2, 16));
    cases.extend(sized_cases(rng, "medium", 5, 17, 1024));
    cases.extend(sized_cases(rng, "large", 3, 4096, 32_768));
    cases.extend(structured_cases(rng));
    cases
}

#[cfg(test)]
mod tests {
    use super::{generate, reference, to_bytes, TARGET};
    use crate::rng::Rng;

    #[test]
    fn sorts_known_values() {
        assert_eq!(reference(&to_bytes(&[3, 1, 2])), to_bytes(&[1, 2, 3]));
        assert_eq!(
            reference(&to_bytes(&[u32::MAX, 0, 256, 1])),
            to_bytes(&[0, 1, 256, u32::MAX])
        );
    }

    #[test]
    fn empty_input() {
        assert!(reference(b"").is_empty());
    }

    #[test]
    fn little_endian_layout() {
        assert_eq!(reference(&[1, 0, 0, 0]), [1, 0, 0, 0]);
        assert_eq!(
            reference(&[0, 1, 0, 0, 255, 0, 0, 0]),
            [255, 0, 0, 0, 0, 1, 0, 0]
        );
    }

    #[test]
    fn generator_shape() {
        let cases = generate(&mut Rng::new(1));
        assert!((20..=40).contains(&cases.len()));
        assert!(cases.iter().map(|c| c.input.len()).sum::<usize>() < 1_000_000);
        for c in &cases {
            assert_eq!(c.input.len() % 4, 0);
            assert_eq!((TARGET.out_cap)(c.input.len()), reference(&c.input).len());
        }
    }
}
