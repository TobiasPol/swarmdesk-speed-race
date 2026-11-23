#!/usr/bin/env bash
# Judges one pull request. The deciding workflow runs this from a checkout of the BASE
# branch, so every line here is reviewed code. The only thing taken from the pull request
# is the bytes of one file, fetched by content hash. The fork is never checked out.
#
# Always leaves "$OUT/final.json" behind: a verdict, a reason and what was judged.
set -euo pipefail
: "${PR:?}" "${REPO:?}" "${DEFAULT_BRANCH:?}" "${GH_TOKEN:?}" "${OUT:?}" "${RUN_URL:?}"
API=https://api.github.com
PROBLEM="${PROBLEM:-speed-race}"
mkdir -p "$OUT/api"

printf '%s' "$PR" | grep -Eqx '[1-9][0-9]{0,6}' || { echo "bad pull request number"; exit 1; }

# write <verdict> <reason> [json object with more fields]
write() {
  jq -n --arg verdict "$1" --arg reason "$2" --argjson pr "$PR" --arg run_url "$RUN_URL" \
    --argjson more "${3:-{\}}" '$more + {verdict: $verdict, reason: $reason, pr: $pr, run_url: $run_url}' \
    > "$OUT/final.json"
  echo "$1 ($2)"
}
write FAULT "the check stopped before it reached a verdict"

get() { # <api path> <file> [accept]
  curl -sSf --proto '=https' --max-redirs 0 --max-filesize 20000000 \
    -H "Authorization: Bearer $GH_TOKEN" -H "Accept: ${3:-application/vnd.github+json}" \
    -H "X-GitHub-Api-Version: 2022-11-28" -o "$2" "$API/repos/$REPO/$1"
}

get "pulls/$PR" "$OUT/api/pr.json"
get "pulls/$PR/files?per_page=100" "$OUT/api/files.json"
head=$(jq -r '.head.sha // ""' "$OUT/api/pr.json")
printf '%s' "$head" | grep -Eqx '[0-9a-f]{40}' || { write REJECTED "the pull request has no head commit"; exit 0; }
get "git/trees/$head?recursive=1" "$OUT/api/tree.json"
get "pulls/$PR/commits?per_page=5" "$OUT/api/commits.json"

# One place decides what a pull request may be: exactly one record holder file, replaced.
if ! guard=$(node .github/scripts/pr-guard.mjs pr "$OUT/api/pr.json" "$OUT/api/files.json" \
  "$OUT/api/tree.json" "$OUT/api/commits.json" "$REPO" "$DEFAULT_BRANCH"); then
  write REJECTED "$(jq -r '.refused // "the pull request was refused"' <<<"$guard")"
  exit 0
fi
target=$(jq -r .target <<<"$guard")
step=$(jq -r .step <<<"$guard")
blob=$(jq -r .blob <<<"$guard")
meta=$(jq -c '{target, step, head_sha: .head, author}' <<<"$guard")

# The step must be open at the desk. Without a desk address the repository judges on its own.
if [ -n "${SWARMDESK_URL:-}" ]; then
  printf '%s' "$SWARMDESK_URL" | grep -Eqx 'https://[a-z0-9.-]+' || { write FAULT "bad desk address" "$meta"; exit 0; }
  if ! curl -sSf --proto '=https' --max-time 20 -o "$OUT/api/steps.json" "$SWARMDESK_URL/p/$PROBLEM/steps.json"; then
    write FAULT "the desk could not be reached" "$meta"
    exit 0
  fi
  if ! open=$(node .github/scripts/pr-guard.mjs step "$OUT/api/steps.json" "$step" "$target"); then
    write REJECTED "$(jq -r '.refused // "the step was refused"' <<<"$open")" "$meta"
    exit 0
  fi
fi

# The content arrives by its hash, as bytes, into a file this script creates.
get "git/blobs/$blob" "$OUT/candidate.rs" "application/vnd.github.raw"

cargo build --release --locked --quiet
set +e
env -u GH_TOKEN ./target/release/sdcheck judge --target "$target" \
  --candidate "$OUT/candidate.rs" --holder "targets/$target/holder.rs" --json "$OUT/verdict.json"
set -e
if [ ! -s "$OUT/verdict.json" ]; then
  write FAULT "the checker produced no verdict" "$meta"
  exit 0
fi
jq -c --argjson meta "$meta" --argjson pr "$PR" --arg run_url "$RUN_URL" \
  '. + $meta + {pr: $pr, run_url: $run_url}' "$OUT/verdict.json" > "$OUT/final.json"
