fn value(c: u8) -> u32 {
    match c {
        b'A'..=b'Z' => (c - b'A') as u32,
        b'a'..=b'z' => (c - b'a') as u32 + 26,
        b'0'..=b'9' => (c - b'0') as u32 + 52,
        b'+' => 62,
        b'/' => 63,
        _ => 0,
    }
}

pub fn solve(input: &[u8], output: &mut [u8]) -> usize {
    let mut i = 0;
    let mut o = 0;
    while i + 4 <= input.len() {
        let bits = (value(input[i]) << 18)
            | (value(input[i + 1]) << 12)
            | (value(input[i + 2]) << 6)
            | value(input[i + 3]);
        output[o] = (bits >> 16) as u8;
        o += 1;
        if input[i + 2] != b'=' {
            output[o] = (bits >> 8) as u8;
            o += 1;
        }
        if input[i + 3] != b'=' {
            output[o] = bits as u8;
            o += 1;
        }
        i += 4;
    }
    o
}
