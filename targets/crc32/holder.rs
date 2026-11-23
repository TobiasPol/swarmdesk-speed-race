const fn make_tables() -> [[u32; 256]; 16] {
    let mut t = [[0u32; 256]; 16];
    let mut i = 0;
    while i < 256 {
        let mut c = i as u32;
        let mut k = 0;
        while k < 8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
            k += 1;
        }
        t[0][i] = c;
        i += 1;
    }
    let mut j = 1;
    while j < 16 {
        let mut i = 0;
        while i < 256 {
            let prev = t[j - 1][i];
            t[j][i] = (prev >> 8) ^ t[0][(prev & 0xFF) as usize];
            i += 1;
        }
        j += 1;
    }
    t
}

static T: [[u32; 256]; 16] = make_tables();

pub fn solve(input: &[u8], output: &mut [u8]) -> usize {
    let mut crc = 0xFFFF_FFFFu32;
    let mut chunks = input.chunks_exact(16);
    for chunk in &mut chunks {
        let first = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) ^ crc;
        crc = T[15][(first & 0xFF) as usize]
            ^ T[14][((first >> 8) & 0xFF) as usize]
            ^ T[13][((first >> 16) & 0xFF) as usize]
            ^ T[12][(first >> 24) as usize]
            ^ T[11][chunk[4] as usize]
            ^ T[10][chunk[5] as usize]
            ^ T[9][chunk[6] as usize]
            ^ T[8][chunk[7] as usize]
            ^ T[7][chunk[8] as usize]
            ^ T[6][chunk[9] as usize]
            ^ T[5][chunk[10] as usize]
            ^ T[4][chunk[11] as usize]
            ^ T[3][chunk[12] as usize]
            ^ T[2][chunk[13] as usize]
            ^ T[1][chunk[14] as usize]
            ^ T[0][chunk[15] as usize];
    }
    for byte in chunks.remainder() {
        crc = T[0][((crc ^ *byte as u32) & 0xFF) as usize] ^ (crc >> 8);
    }
    let bytes = (crc ^ 0xFFFF_FFFF).to_le_bytes();
    output[..4].copy_from_slice(&bytes);
    4
}
