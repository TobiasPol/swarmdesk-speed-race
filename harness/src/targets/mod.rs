//! The functions being raced. Each target has a reference implementation that
//! defines the right answer, and a generator that makes fresh inputs from a seed.

use crate::rng::Rng;

mod base64_decode;
mod base64_encode;
mod crc32;
mod find_substring;
mod parse_ints;
mod sha256;
mod sort_u32;
mod utf8_valid;

pub struct Case {
    /// The kind of input, named in a verdict when a submission fails on it.
    pub class: &'static str,
    pub input: Vec<u8>,
}

pub struct Target {
    pub name: &'static str,
    pub summary: &'static str,
    /// The right answer for an input.
    pub reference: fn(&[u8]) -> Vec<u8>,
    /// Fresh inputs for one run.
    pub generate: fn(&mut Rng) -> Vec<Case>,
    /// The size of the output buffer a submission is given for an input of this length.
    pub out_cap: fn(usize) -> usize,
}

pub fn all() -> &'static [Target] {
    &[
        crc32::TARGET,
        sort_u32::TARGET,
        utf8_valid::TARGET,
        base64_encode::TARGET,
        base64_decode::TARGET,
        find_substring::TARGET,
        parse_ints::TARGET,
        sha256::TARGET,
    ]
}

pub fn find(name: &str) -> Option<&'static Target> {
    all().iter().find(|t| t.name == name)
}

/// Helper for generators: `count` inputs of random bytes with lengths in `lo..=hi`.
pub fn random_cases(
    rng: &mut Rng,
    class: &'static str,
    count: usize,
    lo: usize,
    hi: usize,
) -> Vec<Case> {
    (0..count)
        .map(|_| {
            let len = rng.range(lo, hi);
            Case {
                class,
                input: rng.bytes(len),
            }
        })
        .collect()
}
