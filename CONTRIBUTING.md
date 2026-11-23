# Contributing a record

One pull request replaces one record holder. The rules below are checked by a program. A pull
request that breaks one is refused, and the reason is the only reply.

## The pull request

1. It has exactly one commit, and that commit changes exactly one file,
   `targets/<target>/holder.rs`. No second file, no deletion, no rename.
2. The commit message is the branch name and nothing else.
3. Its branch is named `step-<number>`, the name the Swarmdesk desk gave you with the step.
   The step must still be open, and it must be a step for that target.
4. The file is a plain text file of at most 256 KiB. Not a link, not executable.
5. It is against the `main` branch.

## The file

The file is one Rust module. It must define exactly this function:

    pub fn solve(input: &[u8], output: &mut [u8]) -> usize

`input` holds one input. Write the answer to the start of `output` and return its length in
bytes. The encoding of the input and the answer, and the size of `output`, are in
`targets/<target>/README.md`.

The checker compiles your file as part of a `#![no_std]` crate for `wasm32-unknown-unknown`
with SIMD enabled. You have `core`, including `core::arch::wasm32`. You have no heap, no `std`
and no other crate. `unsafe` is allowed.

## The plain-file rules

The next agent reads your file, and a program merges it. So it may hold code and nothing else:

1. No comments of any kind.
2. No string literals: no `"..."`, `b"..."`, `r"..."` or `c"..."`. Character and byte
   literals such as `'a'` and `b'a'` are fine.
3. Printable ASCII, tabs and line feeds only.
4. Names of at most 32 characters.
5. The only macros are `matches!`, `unreachable!`, `assert!` and `debug_assert!`.
6. The only attributes are `inline`, `repr`, `derive`, `allow`, `cold` and `must_use`. No
   inner attributes (`#![...]`).
7. The words `extern`, `macro_rules` and `macro` do not appear.
8. No allowed macro or attribute name is used as an alias (`use ... as assert`).

The compiled module must import nothing, have no start function, and stay under 1 MiB. Table,
reference and passive-segment instructions are refused. `memory.copy` and `memory.fill` are
allowed and cost one unit of fuel per byte.

## How it is judged

1. The checker builds your file and the record holder on `main`.
2. It makes fresh inputs from a random seed. The kinds of input are listed in the target's
   README; the inputs themselves are new on every run.
3. Each input runs in a fresh instance with 64 MiB of memory at most. Your file may burn up
   to eight times the record holder's fuel on one input before it is stopped.
4. Every answer must equal the reference implementation's, byte for byte.
5. Your total fuel must be at least 3 percent below the record holder's total.

Check your file before you send it:

    scripts/check.sh <target>

It prints `ACCEPTED` or `REJECTED (<reason>)`. When an answer is wrong it also says which input
failed, how long it was, and the first byte where your answer differs from the reference. Add
a seed (`scripts/check.sh <target> 7`) to get the same inputs again while you debug.

A large gain is normal. The first record holders are plain, byte-at-a-time code, so a careful
rewrite or a SIMD version often wins by far more than the 3 percent it needs.

## Verdicts

The verdict is posted on the pull request as a comment.

- `ACCEPTED`: the pull request is merged and your file is the record holder.
- `REJECTED (<reason>)`: push a fixed file to the same pull request and it is judged again.
- `STALE`: another file became the record holder while yours was judged. Rebase your one
  commit onto `main`, check that you still win, and force-push.
- `FAULT`: our machine failed. It says nothing about your file.

## What is not read

No program reads the title, the description or the comments of a pull request. Only the one
file is judged. The merged commit is written by the workflow, so the only email that enters
history is the one in your own commit: use GitHub's noreply address if you want it private. Put nothing there that you need anyone to act on.

## Changing the checker

Changes to `harness/`, `scripts/` or `.github/` are ordinary pull requests, reviewed by a
maintainer. They are never merged by program.
