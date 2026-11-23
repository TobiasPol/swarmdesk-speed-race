//! CRC-32 (IEEE 802.3, reflected, polynomial 0xEDB88320).
//! Output: the checksum as 4 bytes, little-endian.

use super::{random_cases, Case, Target};
use crate::rng::Rng;

pub const TARGET: Target = Target {
    name: "crc32",
    summary: "CRC-32 checksum of the input, as 4 little-endian bytes",
    reference,
    generate,
    out_cap: |_| 4,
};

fn reference(input: &[u8]) -> Vec<u8> {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in input {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                0xEDB8_8320 ^ (crc >> 1)
            } else {
                crc >> 1
            };
        }
    }
    (crc ^ 0xFFFF_FFFF).to_le_bytes().to_vec()
}

fn generate(rng: &mut Rng) -> Vec<Case> {
    let mut cases = vec![Case {
        class: "empty",
        input: Vec::new(),
    }];
    cases.extend(random_cases(rng, "tiny", 8, 1, 16));
    cases.extend(random_cases(rng, "medium", 8, 17, 1024));
    cases.extend(random_cases(rng, "between", 3, 1025, 4095));
    cases.extend(random_cases(rng, "large", 6, 4096, 65_536));
    let len = rng.range(2048, 8192);
    cases.push(Case {
        class: "all zero",
        input: vec![0; len],
    });
    cases.push(Case {
        class: "all ones",
        input: vec![0xFF; len],
    });
    cases
}

#[cfg(test)]
mod tests {
    use super::reference;

    #[test]
    fn known_vector() {
        assert_eq!(reference(b"123456789"), 0xCBF4_3926u32.to_le_bytes());
    }

    #[test]
    fn empty_input() {
        assert_eq!(reference(b""), [0, 0, 0, 0]);
    }
}
