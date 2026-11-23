# crc32

Compute the CRC-32 checksum (IEEE 802.3) of the input bytes.

## Interface

- Input: any bytes, length 0 to 65536.
- Output: the checksum as 4 bytes, little-endian. The function returns 4.
- Algorithm: reflected, polynomial 0xEDB88320, initial value 0xFFFFFFFF, final XOR 0xFFFFFFFF. The check value for `123456789` is 0xCBF43926.
- Output buffer size: 4 bytes.

## Inputs

- Empty input.
- Tiny (1 to 16 bytes), medium (17 to 1024 bytes) and large (4096 to 65536 bytes) random bytes.
- All zero bytes and all 0xFF bytes, 2048 to 8192 long.

## Rules

See [../../CONTRIBUTING.md](../../CONTRIBUTING.md).
