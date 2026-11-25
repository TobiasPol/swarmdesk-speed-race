use core::arch::wasm32::*;

#[inline(always)]
unsafe fn decode_block(input: *const u8, output: *mut u8) {
    let ascii = v128_load(input.cast());
    let high_nibbles = u8x16_shr(ascii, 4);
    let offsets = u8x16(0, 0, 19, 4, 191, 191, 185, 185, 0, 0, 0, 0, 0, 0, 0, 0);
    let slash = u8x16_eq(ascii, u8x16_splat(b'/'));
    let adjustment = v128_and(slash, u8x16_splat(3));
    let digits = u8x16_sub(
        u8x16_add(ascii, u8x16_swizzle(offsets, high_nibbles)),
        adjustment,
    );

    let first = v128_and(digits, u16x8_splat(63));
    let pairs = v128_or(u16x8_shl(first, 6), u16x8_shr(digits, 8));
    let words = i32x4_dot_i16x8(pairs, i16x8(4096, 1, 4096, 1, 4096, 1, 4096, 1));
    let bytes = u8x16_shuffle::<2, 1, 0, 6, 5, 4, 10, 9, 8, 14, 13, 12, 3, 3, 3, 3>(words, words);
    v128_store(output.cast(), bytes);
}

const fn make_decode_table() -> [u8; 256] {
    let mut values = [0; 256];
    let mut i = 0;
    while i < 26 {
        values[b'A' as usize + i] = i as u8;
        values[b'a' as usize + i] = 26 + i as u8;
        i += 1;
    }
    i = 0;
    while i < 10 {
        values[b'0' as usize + i] = 52 + i as u8;
        i += 1;
    }
    values[b'+' as usize] = 62;
    values[b'/' as usize] = 63;
    values
}

static DECODE: [u8; 256] = make_decode_table();

#[inline(always)]
fn decode_quad(quad: &[u8; 4]) -> [u8; 3] {
    let [a, b, c, d] = quad.map(|byte| DECODE[byte as usize]);
    [(a << 2) | (b >> 4), (b << 4) | (c >> 2), (c << 6) | d]
}

pub fn solve(mut input: &[u8], output: &mut [u8]) -> usize {
    if input.is_empty() {
        return 0;
    }

    let mut written = 0;
    while input.len() >= 24 {
        unsafe { decode_block(input.as_ptr(), output.as_mut_ptr().add(written)) };
        input = &input[16..];
        written += 12;
    }

    let (rest, last) = input.split_last_chunk::<4>().unwrap();
    for quad in rest.as_chunks::<4>().0 {
        output[written..written + 3].copy_from_slice(&decode_quad(quad));
        written += 3;
    }

    let tail = decode_quad(last);
    let count = 3 - usize::from(last[3] == b'=') - usize::from(last[2] == b'=');
    output[written..written + count].copy_from_slice(&tail[..count]);
    written + count
}
