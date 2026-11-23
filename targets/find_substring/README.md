# find_substring

Find the first occurrence of a needle in a haystack.

## Interface

- Input: 4 bytes holding the needle length m (little-endian), then m needle bytes, then the haystack (the rest of the input).
- Output: 4 bytes, little-endian: the index of the first occurrence of the needle in the haystack, or 0xFFFFFFFF if there is none. An empty needle gives 0. The function returns 4.
- Output buffer size: 4 bytes.

## Inputs

- The needle at the start, in the middle and at the end of the haystack, and absent.
- A needle longer than the haystack, an empty needle, an empty haystack, and both empty.
- Repetitive haystacks (many `a` then `b`, with a needle such as `aaa...ab`), found and absent.
- Small alphabets of 2 to 4 symbols.
- Large haystacks of up to 65536 bytes.

## Rules

See [../../CONTRIBUTING.md](../../CONTRIBUTING.md).
