# base64_encode

Encode the input bytes as base64 text.

## Interface

- Input: any bytes.
- Output: the standard base64 alphabet (A-Z, a-z, 0-9, +, /) as ASCII, padded with `=` to a multiple of 4, with no line breaks (RFC 4648 section 4). The function returns the number of bytes written.
- Output buffer size: 4 * ceil(n / 3) bytes for an input of n bytes.

## Inputs

- Empty input and lengths 1, 2 and 3 (all three padding cases).
- Tiny (4 to 16 bytes), medium (17 to 1024 bytes) and large (4096 to 65536 bytes) random bytes.
- A run of 0xFF bytes, which uses the `/` symbol heavily.

## Rules

See [../../CONTRIBUTING.md](../../CONTRIBUTING.md).
