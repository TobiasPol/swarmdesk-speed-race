use core::arch::wasm32::*;

#[inline(always)]
unsafe fn encode12(src: *const u8, dst: *mut u8) {
    let raw = v128_load(src as *const v128);
    let l = i8x16_shuffle::<1, 0, 2, 1, 4, 3, 5, 4, 7, 6, 8, 7, 10, 9, 11, 10>(raw, raw);
    let a = v128_and(l, u32x4_splat(0x0fc0fc00));
    let a = i16x8_q15mulr_sat(a, u32x4_splat(0x02000020));
    let b = v128_and(l, u32x4_splat(0x003f03f0));
    let b = i16x8_mul(b, u32x4_splat(0x01000010));
    let idx = v128_or(v128_and(a, u16x8_splat(63)), b);
    let r = u8x16_sub_sat(idx, u8x16_splat(51));
    let lt = u8x16_lt(idx, u8x16_splat(26));
    let r = v128_or(r, v128_and(lt, u8x16_splat(13)));
    let tbl = i8x16(71, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -19, -16, 65, 0, 0);
    let out = i8x16_add(idx, u8x16_swizzle(tbl, r));
    v128_store(dst as *mut v128, out);
}

const ALPHABET: [u8; 64] = [
    65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87,
    88, 89, 90, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113,
    114, 115, 116, 117, 118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 43, 47,
];

pub fn solve(input: &[u8], output: &mut [u8]) -> usize {
    let n = input.len();
    let src = input.as_ptr();
    let dst = output.as_mut_ptr();
    let mut i = 0usize;
    let mut o = 0usize;
    unsafe {
        while i + 52 <= n {
            encode12(src.add(i), dst.add(o));
            encode12(src.add(i + 12), dst.add(o + 16));
            encode12(src.add(i + 24), dst.add(o + 32));
            encode12(src.add(i + 36), dst.add(o + 48));
            i += 48;
            o += 64;
        }
        while i + 16 <= n {
            encode12(src.add(i), dst.add(o));
            i += 12;
            o += 16;
        }
        while i + 3 <= n {
            let v = ((*src.add(i) as u32) << 16) | ((*src.add(i + 1) as u32) << 8) | *src.add(i + 2) as u32;
            *dst.add(o) = ALPHABET[(v >> 18) as usize];
            *dst.add(o + 1) = ALPHABET[((v >> 12) & 63) as usize];
            *dst.add(o + 2) = ALPHABET[((v >> 6) & 63) as usize];
            *dst.add(o + 3) = ALPHABET[(v & 63) as usize];
            i += 3;
            o += 4;
        }
        let left = n - i;
        if left == 1 {
            let v = (*src.add(i) as u32) << 16;
            *dst.add(o) = ALPHABET[(v >> 18) as usize];
            *dst.add(o + 1) = ALPHABET[((v >> 12) & 63) as usize];
            *dst.add(o + 2) = b'=';
            *dst.add(o + 3) = b'=';
            o += 4;
        } else if left == 2 {
            let v = ((*src.add(i) as u32) << 16) | ((*src.add(i + 1) as u32) << 8);
            *dst.add(o) = ALPHABET[(v >> 18) as usize];
            *dst.add(o + 1) = ALPHABET[((v >> 12) & 63) as usize];
            *dst.add(o + 2) = ALPHABET[((v >> 6) & 63) as usize];
            *dst.add(o + 3) = b'=';
            o += 4;
        }
    }
    o
}
