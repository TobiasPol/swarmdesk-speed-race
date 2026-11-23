# sort_u32

Sort a list of unsigned 32-bit integers in ascending order.

## Interface

- Input: n values of 4 bytes each, little-endian. The length is always a multiple of 4.
- Output: the same values sorted ascending, 4 bytes each, little-endian. The function returns the number of bytes written (the input length).
- Output buffer size: the input length.

## Inputs

- Empty input and a single value.
- Tiny (2 to 16 values), medium (17 to 1024 values) and large (4096 to 32768 values) random values.
- Already sorted, reverse sorted, all equal, and a few distinct values repeated.

## Rules

See [../../CONTRIBUTING.md](../../CONTRIBUTING.md).
