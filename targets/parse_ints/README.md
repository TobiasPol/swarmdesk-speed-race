# parse_ints

Parse newline-separated decimal numbers into 32-bit integers.

## Interface

- Input: unsigned decimal integers of 1 to 10 ASCII digits each, value at most 4294967295, leading zeros allowed, separated by single `\n` bytes, with no trailing newline. An empty input means no numbers.
- Output: each value as 4 bytes, little-endian, in order. The function returns 4 times the number of values.
- Output buffer size: (input length + 1) / 2 * 4 bytes.

## Inputs

- Empty input and a single number (including 0 and 4294967295).
- Small numbers, numbers of all digit counts, and many 4294967295 values.
- Numbers padded with leading zeros up to 10 digits.
- Many numbers: up to 10000 per input.

## Rules

See [../../CONTRIBUTING.md](../../CONTRIBUTING.md).
