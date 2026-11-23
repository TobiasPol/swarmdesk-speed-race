# sha256

Compute the SHA-256 digest of the input bytes (FIPS 180-4).

## Interface

- Input: any bytes.
- Output: the 32-byte digest. The function returns 32.
- Output buffer size: 32 bytes.

## Inputs

- Empty input.
- Lengths 55, 56, 63, 64, 65, 119 and 120, which sit on the padding and block boundaries, plus random lengths from 54 to 130.
- Tiny (1 to 16 bytes), medium (17 to 1024 bytes) and large (4096 to 65536 bytes) random bytes.

## Rules

See [../../CONTRIBUTING.md](../../CONTRIBUTING.md).
