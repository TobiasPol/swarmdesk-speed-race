use core::arch::wasm32::*;

#[inline(always)]
fn is_cont(b: u8) -> bool {
    (b as i8) < -64
}

#[inline(always)]
fn in_range(b: u8, lo: u8, hi: u8) -> bool {
    b.wrapping_sub(lo) <= hi - lo
}

fn is_valid(input: &[u8]) -> bool {
    let n = input.len();
    let p = input.as_ptr();
    let mut i = 0;
    while i < n {
        let b = unsafe { *p.add(i) };
        if b < 0x80 {
            if i + 16 <= n {
                let block = unsafe { v128_load(p.add(i) as *const v128) };
                let mask = i8x16_bitmask(block);
                i += if mask == 0 { 16 } else { mask.trailing_zeros() as usize };
            } else {
                i += 1;
            }
            continue;
        }
        if b < 0xC2 {
            return false;
        }
        if b < 0xE0 {
            if i + 1 >= n || !is_cont(unsafe { *p.add(i + 1) }) {
                return false;
            }
            i += 2;
            continue;
        }
        if b < 0xF0 {
            if i + 2 >= n {
                return false;
            }
            let lo = if b == 0xE0 { 0xA0 } else { 0x80 };
            let hi = if b == 0xED { 0x9F } else { 0xBF };
            let second = unsafe { *p.add(i + 1) };
            let third = unsafe { *p.add(i + 2) };
            if !in_range(second, lo, hi) || !is_cont(third) {
                return false;
            }
            i += 3;
            continue;
        }
        if b > 0xF4 || i + 3 >= n {
            return false;
        }
        let lo = if b == 0xF0 { 0x90 } else { 0x80 };
        let hi = if b == 0xF4 { 0x8F } else { 0xBF };
        let second = unsafe { *p.add(i + 1) };
        let third = unsafe { *p.add(i + 2) };
        let fourth = unsafe { *p.add(i + 3) };
        if !in_range(second, lo, hi) || !is_cont(third) || !is_cont(fourth) {
            return false;
        }
        i += 4;
    }
    true
}

pub fn solve(input: &[u8], output: &mut [u8]) -> usize {
    output[0] = if is_valid(input) { 1 } else { 0 };
    1
}
