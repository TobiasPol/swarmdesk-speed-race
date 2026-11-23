fn matches_at(needle: &[u8], haystack: &[u8], start: usize) -> bool {
    let mut k = 0;
    while k < needle.len() {
        if haystack[start + k] != needle[k] {
            return false;
        }
        k += 1;
    }
    true
}

fn find(needle: &[u8], haystack: &[u8]) -> u32 {
    if needle.len() > haystack.len() {
        return 0xFFFF_FFFF;
    }
    let mut start = 0;
    while start + needle.len() <= haystack.len() {
        if matches_at(needle, haystack, start) {
            return start as u32;
        }
        start += 1;
    }
    0xFFFF_FFFF
}

pub fn solve(input: &[u8], output: &mut [u8]) -> usize {
    let m = u32::from_le_bytes([input[0], input[1], input[2], input[3]]) as usize;
    let result = find(&input[4..4 + m], &input[4 + m..]).to_le_bytes();
    output[0] = result[0];
    output[1] = result[1];
    output[2] = result[2];
    output[3] = result[3];
    4
}
