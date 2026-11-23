#!/usr/bin/env bash
# Catches the site up. Re-sends the verdict of every recent record, rebuilt from what GitHub
# holds: the record commit the deciding workflow wrote. The site records a verdict once, so
# sending one it already has changes nothing.
#
# Only records are caught up. A lost REJECTED changes nothing that anyone depends on.
#
#   scripts/reconcile.sh        (LIMIT=<n> looks at the n most recent pull requests and commits)
set -euo pipefail
: "${REPO:?}" "${GH_TOKEN:?}" "${DEFAULT_BRANCH:?}" "${SWARMDESK_URL:?}" "${VERDICT_SECRET:?}"
here=$(cd "$(dirname "$0")" && pwd)
PROBLEM="${PROBLEM:-speed-race}"
LIMIT="${LIMIT:-30}"
printf '%s' "$LIMIT" | grep -Eqx '[1-9][0-9]?' || { echo "LIMIT must be 1 to 99"; exit 2; }
work=$(mktemp -d)

gh api "repos/$REPO/pulls?state=closed&base=$DEFAULT_BRANCH&sort=updated&direction=desc&per_page=$LIMIT" \
  --jq '.[] | select(.merged_at != null) | [.number, .head.ref, .head.sha, .user.login, .merge_commit_sha, .merged_at] | @tsv' \
  | sort -n > "$work/merged.tsv"

sent=0
failed=0
LOGIN='^[A-Za-z0-9][A-Za-z0-9-]{0,38}$'
WHEN='^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$'
TITLE='^step-([0-9]+): new ([a-z0-9_]{1,40}) record by ([A-Za-z0-9-]+)$'
FUEL='^Fuel ([0-9]{1,18}) against ([0-9]{1,18}) for the previous record holder\. Seed ([0-9]{1,20})\.$'

# send <pr> <head> <author> <step> <target> <candidate fuel> <holder fuel> <seed> <merged at>
send() {
  jq -cn --arg problem "$PROBLEM" --argjson pr "$1" --arg head_sha "$2" --arg author "$3" \
    --argjson step "$4" --arg target "$5" --argjson candidate "$6" --argjson holder "$7" \
    --arg seed "$8" --arg merged_at "$9" \
    '{problem: $problem, pr: $pr, head_sha: $head_sha, author: $author, step: $step, target: $target,
      verdict: "ACCEPTED", reason: "", candidate_fuel: $candidate, holder_fuel: $holder, seed: $seed,
      merged: true, merged_at: $merged_at, run_url: ""}' > "$work/payload.json"
  echo "pull request $1 (step-$4, $5):"
  if "$here/send-verdict.sh" "$work/payload.json"; then sent=$((sent + 1)); else failed=$((failed + 1)); fi
}

# Records from before the workflow wrote the record commit itself: true merges of the pull
# request. An author name alone can be imitated, so the commit must also be signed by GitHub
# (it signs only commits it creates), be a true merge, and have the judged commit as its
# second parent.
while IFS=$'\t' read -r pr branch head author merge merged_at; do
  # Every value is matched against a strict pattern before it is used.
  [[ "$pr" =~ ^[1-9][0-9]{0,6}$ && "$branch" =~ ^step-([1-9][0-9]{0,8})$ ]] || continue
  step="${BASH_REMATCH[1]}"
  [[ "$head" =~ ^[0-9a-f]{40}$ && "$merge" =~ ^[0-9a-f]{40}$ ]] || continue
  [[ "$author" =~ $LOGIN && "$merged_at" =~ $WHEN ]] || continue
  gh api "repos/$REPO/commits/$merge" > "$work/commit.json" 2>/dev/null || continue
  proof=$(jq -r --arg head "$head" '[
      .author.login == "github-actions[bot]",
      .committer.login == "web-flow",
      .commit.verification.verified == true,
      (.parents | length) == 2,
      .parents[1].sha == $head
    ] | all' "$work/commit.json")
  [ "$proof" = true ] || continue
  title=$(jq -r '.commit.message' "$work/commit.json" | sed -n 1p)
  fuel=$(jq -r '.commit.message' "$work/commit.json" | sed -n 3p)
  [[ "$title" =~ $TITLE ]] || continue
  [ "${BASH_REMATCH[1]}" = "$step" ] && [ "${BASH_REMATCH[3]}" = "$author" ] || continue
  target="${BASH_REMATCH[2]}"
  [[ "$fuel" =~ $FUEL ]] || continue
  send "$pr" "$head" "$author" "$step" "$target" "${BASH_REMATCH[1]}" "${BASH_REMATCH[2]}" "${BASH_REMATCH[3]}" "$merged_at"
done < "$work/merged.tsv"

# Records the workflow wrote itself: one commit on main with one parent, made and signed by
# GitHub for the workflow's identity, that changes exactly one record holder file. Only the
# workflow can make such a commit, and it writes every word of the message from values it
# has matched against strict patterns, so the message can be read back.
gh api "repos/$REPO/commits?sha=$DEFAULT_BRANCH&per_page=$LIMIT" \
  --jq '.[] | select((.parents | length) == 1) | .sha' | tac > "$work/single.txt"
while read -r sha; do
  [[ "$sha" =~ ^[0-9a-f]{40}$ ]] || continue
  gh api "repos/$REPO/commits/$sha" > "$work/commit.json" 2>/dev/null || continue
  proof=$(jq -r '[
      .author.login == "github-actions[bot]",
      .committer.login == "web-flow",
      .commit.verification.verified == true,
      (.parents | length) == 1,
      (.files | length) == 1,
      .files[0].status == "modified"
    ] | all' "$work/commit.json")
  [ "$proof" = true ] || continue
  [[ "$(jq -r '.files[0].filename' "$work/commit.json")" =~ ^targets/([a-z0-9_]{1,40})/holder\.rs$ ]] || continue
  changed="${BASH_REMATCH[1]}"
  merged_at=$(jq -r '.commit.committer.date' "$work/commit.json")
  [[ "$merged_at" =~ $WHEN ]] || continue
  title=$(jq -r '.commit.message' "$work/commit.json" | sed -n 1p)
  fuel=$(jq -r '.commit.message' "$work/commit.json" | sed -n 3p)
  from=$(jq -r '.commit.message' "$work/commit.json" | sed -n 4p)
  [[ "$title" =~ $TITLE ]] || continue
  step="${BASH_REMATCH[1]}"
  target="${BASH_REMATCH[2]}"
  author="${BASH_REMATCH[3]}"
  [ "$target" = "$changed" ] && [[ "$author" =~ $LOGIN && "$step" =~ ^[1-9][0-9]{0,8}$ ]] || continue
  [[ "$from" =~ ^Pull\ request\ \#([1-9][0-9]{0,6}),\ commit\ ([0-9a-f]{40})\.$ ]] || continue
  pr="${BASH_REMATCH[1]}"
  head="${BASH_REMATCH[2]}"
  [[ "$fuel" =~ $FUEL ]] || continue
  send "$pr" "$head" "$author" "$step" "$target" "${BASH_REMATCH[1]}" "${BASH_REMATCH[2]}" "${BASH_REMATCH[3]}" "$merged_at"
done < "$work/single.txt"

echo "records sent: $sent, not delivered: $failed"
[ "$failed" = 0 ]
