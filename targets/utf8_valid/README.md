# utf8_valid

Decide whether the input is valid UTF-8.

## Interface

- Input: any bytes.
- Output: 1 byte, 1 if the input is valid UTF-8, else 0. The function returns 1.
- Valid means what `std::str::from_utf8` accepts: no overlong encodings, no surrogates (U+D800 to U+DFFF), no code points above U+10FFFF, no truncated sequences.
- Output buffer size: 1 byte.

## Inputs

About half of the inputs are valid.
- Valid: empty, ASCII, and text mixing 1-, 2-, 3- and 4-byte sequences, up to 12000 characters.
- Invalid: random bytes, valid text with one corrupted byte, overlong encodings (such as C0 80), surrogates (ED A0 80 and up), values above U+10FFFF (F4 90 80 80 and up), and text whose final sequence is cut short.

## Rules

See [../../CONTRIBUTING.md](../../CONTRIBUTING.md).
