fn get(buf: &[u8], i: usize) -> u32 {
    u32::from_le_bytes([buf[4 * i], buf[4 * i + 1], buf[4 * i + 2], buf[4 * i + 3]])
}

fn set(buf: &mut [u8], i: usize, value: u32) {
    let bytes = value.to_le_bytes();
    buf[4 * i] = bytes[0];
    buf[4 * i + 1] = bytes[1];
    buf[4 * i + 2] = bytes[2];
    buf[4 * i + 3] = bytes[3];
}

fn swap(buf: &mut [u8], a: usize, b: usize) {
    let x = get(buf, a);
    let y = get(buf, b);
    set(buf, a, y);
    set(buf, b, x);
}

fn sift_down(buf: &mut [u8], mut root: usize, n: usize) {
    loop {
        let mut child = 2 * root + 1;
        if child >= n {
            break;
        }
        if child + 1 < n && get(buf, child) < get(buf, child + 1) {
            child += 1;
        }
        if get(buf, root) >= get(buf, child) {
            break;
        }
        swap(buf, root, child);
        root = child;
    }
}

pub fn solve(input: &[u8], output: &mut [u8]) -> usize {
    let n = input.len() / 4;
    let mut i = 0;
    while i < 4 * n {
        output[i] = input[i];
        i += 1;
    }
    let mut start = n / 2;
    while start > 0 {
        start -= 1;
        sift_down(output, start, n);
    }
    let mut end = n;
    while end > 1 {
        end -= 1;
        swap(output, 0, end);
        sift_down(output, 0, end);
    }
    4 * n
}
