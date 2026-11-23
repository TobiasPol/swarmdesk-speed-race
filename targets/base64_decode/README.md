# base64_decode

Decode base64 text back into bytes.

## Interface

- Input: valid standard base64 (A-Z, a-z, 0-9, +, /), padded with `=` to a multiple of 4, with no whitespace or line breaks.
- Output: the decoded bytes. The function returns the number of bytes written.
- Output buffer size: input length / 4 * 3 bytes.
- Only valid input is generated. Behaviour on invalid input is not tested.

## Inputs

Every input is made by encoding random bytes.
- Empty input.
- Decoded lengths of 1, 2 and 3 bytes, and longer inputs whose decoded length is 1 or 2 modulo 3 (padding with `==` and `=`).
- Tiny, medium and large inputs (up to 65536 decoded bytes).

## Rules

See [../../CONTRIBUTING.md](../../CONTRIBUTING.md).
