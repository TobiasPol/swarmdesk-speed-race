fn is_cont(b: u8) -> bool {
    b >= 0x80 && b <= 0xBF
}

fn sequence_len(input: &[u8], i: usize) -> usize {
    let b = input[i];
    let (len, lo, hi) = match b {
        0xC2..=0xDF => (2, 0x80, 0xBF),
        0xE0 => (3, 0xA0, 0xBF),
        0xE1..=0xEC | 0xEE..=0xEF => (3, 0x80, 0xBF),
        0xED => (3, 0x80, 0x9F),
        0xF0 => (4, 0x90, 0xBF),
        0xF1..=0xF3 => (4, 0x80, 0xBF),
        0xF4 => (4, 0x80, 0x8F),
        _ => return 0,
    };
    if i + len > input.len() {
        return 0;
    }
    let second = input[i + 1];
    if second < lo || second > hi {
        return 0;
    }
    let mut k = 2;
    while k < len {
        if !is_cont(input[i + k]) {
            return 0;
        }
        k += 1;
    }
    len
}

fn is_valid(input: &[u8]) -> bool {
    let mut i = 0;
    while i < input.len() {
        if input[i] < 0x80 {
            i += 1;
            continue;
        }
        let len = sequence_len(input, i);
        if len == 0 {
            return false;
        }
        i += len;
    }
    true
}

pub fn solve(input: &[u8], output: &mut [u8]) -> usize {
    output[0] = if is_valid(input) { 1 } else { 0 };
    1
}
