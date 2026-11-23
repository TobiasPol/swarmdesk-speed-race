#!/usr/bin/env bash
# Judges your working copy of targets/<target>/holder.rs against the record holder on main,
# with the same checker the deciding workflow runs.
#
#   scripts/check.sh <target> [seed]
set -euo pipefail
target="${1:-}"
if ! printf '%s' "$target" | grep -Eqx '[a-z0-9_]{1,40}'; then
  echo "usage: scripts/check.sh <target> [seed]" >&2
  echo "targets: $(ls targets | tr '\n' ' ')" >&2
  exit 2
fi
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
file="targets/$target/holder.rs"
[ -f "$file" ] || { echo "no such target: $target" >&2; exit 2; }

# The record holder is the file on the main branch of the repository you forked.
remote=origin
git remote | grep -qx upstream && remote=upstream
git fetch --quiet "$remote" main
holder="$(mktemp -d)/holder.rs"
git show "$remote/main:$file" > "$holder"

cargo build --release --locked --quiet
./target/release/sdcheck judge --target "$target" --candidate "$file" --holder "$holder" ${2:+--seed "$2"}
