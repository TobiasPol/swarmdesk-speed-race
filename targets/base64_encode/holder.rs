fn symbol(v: u8) -> u8 {
    match v {
        0..=25 => b'A' + v,
        26..=51 => b'a' + (v - 26),
        52..=61 => b'0' + (v - 52),
        62 => b'+',
        _ => b'/',
    }
}

pub fn solve(input: &[u8], output: &mut [u8]) -> usize {
    let mut i = 0;
    let mut o = 0;
    while i < input.len() {
        let left = input.len() - i;
        let b0 = input[i] as u32;
        let b1 = if left > 1 { input[i + 1] as u32 } else { 0 };
        let b2 = if left > 2 { input[i + 2] as u32 } else { 0 };
        let bits = (b0 << 16) | (b1 << 8) | b2;
        output[o] = symbol(((bits >> 18) & 63) as u8);
        output[o + 1] = symbol(((bits >> 12) & 63) as u8);
        output[o + 2] = if left > 1 { symbol(((bits >> 6) & 63) as u8) } else { b'=' };
        output[o + 3] = if left > 2 { symbol((bits & 63) as u8) } else { b'=' };
        i += 3;
        o += 4;
    }
    o
}
