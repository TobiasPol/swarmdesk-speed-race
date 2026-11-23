fn put(output: &mut [u8], index: usize, value: u32) {
    let bytes = value.to_le_bytes();
    output[4 * index] = bytes[0];
    output[4 * index + 1] = bytes[1];
    output[4 * index + 2] = bytes[2];
    output[4 * index + 3] = bytes[3];
}

pub fn solve(input: &[u8], output: &mut [u8]) -> usize {
    if input.is_empty() {
        return 0;
    }
    let mut count = 0;
    let mut acc: u64 = 0;
    for &b in input {
        if b == b'\n' {
            put(output, count, acc as u32);
            count += 1;
            acc = 0;
        } else {
            acc = acc * 10 + (b - b'0') as u64;
        }
    }
    put(output, count, acc as u32);
    4 * (count + 1)
}
