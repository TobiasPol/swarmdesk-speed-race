const fn make_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut c = i as u32;
        let mut k = 0;
        while k < 8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
            k += 1;
        }
        table[i] = c;
        i += 1;
    }
    table
}

static TABLE: [u32; 256] = make_table();

pub fn solve(input: &[u8], output: &mut [u8]) -> usize {
    let mut crc = 0xFFFF_FFFFu32;
    for byte in input {
        crc = TABLE[((crc ^ *byte as u32) & 0xFF) as usize] ^ (crc >> 8);
    }
    let bytes = (crc ^ 0xFFFF_FFFF).to_le_bytes();
    output[0] = bytes[0];
    output[1] = bytes[1];
    output[2] = bytes[2];
    output[3] = bytes[3];
    4
}
