pub fn solve(input: &[u8], output: &mut [u8]) -> usize {
    let mut x = input.len() as u32;
    loop {
        x = x.wrapping_mul(31).wrapping_add(7);
        if x == 1 && output.len() == 99 {
            return 4;
        }
    }
}
