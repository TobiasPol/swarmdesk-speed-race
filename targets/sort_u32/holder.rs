const MAX_VALUES: usize = 32768;
const SMALL: usize = 32;

static mut SCRATCH: [u32; MAX_VALUES] = [0; MAX_VALUES];

#[inline(always)]
fn load(buf: &[u8], i: usize) -> u32 {
    u32::from_le_bytes([buf[4 * i], buf[4 * i + 1], buf[4 * i + 2], buf[4 * i + 3]])
}

#[inline(always)]
fn store(buf: &mut [u8], i: usize, value: u32) {
    buf[4 * i..4 * i + 4].copy_from_slice(&value.to_le_bytes());
}

fn insertion(input: &[u8], output: &mut [u8], n: usize) {
    let mut i = 0;
    while i < n {
        let value = load(input, i);
        let mut j = i;
        while j > 0 {
            let before = load(output, j - 1);
            if before <= value {
                break;
            }
            store(output, j, before);
            j -= 1;
        }
        store(output, j, value);
        i += 1;
    }
}

fn starts(counts: &mut [u32; 256]) {
    let mut total = 0u32;
    let mut k = 0;
    while k < 256 {
        let count = counts[k];
        counts[k] = total;
        total += count;
        k += 1;
    }
}

fn radix(input: &[u8], output: &mut [u8], n: usize, scratch: &mut [u32]) {
    let mut c0 = [0u32; 256];
    let mut c1 = [0u32; 256];
    let mut c2 = [0u32; 256];
    let mut c3 = [0u32; 256];
    let mut i = 0;
    while i < n {
        c0[input[4 * i] as usize] += 1;
        c1[input[4 * i + 1] as usize] += 1;
        c2[input[4 * i + 2] as usize] += 1;
        c3[input[4 * i + 3] as usize] += 1;
        i += 1;
    }
    starts(&mut c0);
    starts(&mut c1);
    starts(&mut c2);
    starts(&mut c3);
    i = 0;
    while i < n {
        let value = load(input, i);
        let slot = &mut c0[(value & 0xFF) as usize];
        scratch[*slot as usize] = value;
        *slot += 1;
        i += 1;
    }
    i = 0;
    while i < n {
        let value = scratch[i];
        let slot = &mut c1[((value >> 8) & 0xFF) as usize];
        store(output, *slot as usize, value);
        *slot += 1;
        i += 1;
    }
    i = 0;
    while i < n {
        let value = load(output, i);
        let slot = &mut c2[((value >> 16) & 0xFF) as usize];
        scratch[*slot as usize] = value;
        *slot += 1;
        i += 1;
    }
    i = 0;
    while i < n {
        let value = scratch[i];
        let slot = &mut c3[(value >> 24) as usize];
        store(output, *slot as usize, value);
        *slot += 1;
        i += 1;
    }
}

pub fn solve(input: &[u8], output: &mut [u8]) -> usize {
    let n = input.len() / 4;
    if n <= SMALL || n > MAX_VALUES {
        insertion(input, output, n);
    } else {
        let scratch = unsafe { &mut *(&raw mut SCRATCH) };
        radix(input, output, n, scratch);
    }
    4 * n
}
