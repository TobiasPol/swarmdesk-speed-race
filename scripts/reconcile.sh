#!/usr/bin/env bash
# Catches the site up. Re-sends the verdict of every recently merged record, rebuilt from what
# GitHub holds: the pull request and the merge commit the deciding workflow wrote. The site
# records a verdict once, so sending one it already has changes nothing.
#
# Only records are caught up. A lost REJECTED changes nothing that anyone depends on.
#
#   scripts/reconcile.sh        (LIMIT=<n> looks at the n most recently updated pull requests)
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
while IFS=$'\t' read -r pr branch head author merge merged_at; do
  # Every value is matched against a strict pattern before it is used.
  [[ "$pr" =~ ^[1-9][0-9]{0,6}$ && "$branch" =~ ^step-([1-9][0-9]{0,8})$ ]] || continue
  step="${BASH_REMATCH[1]}"
  [[ "$head" =~ ^[0-9a-f]{40}$ && "$merge" =~ ^[0-9a-f]{40}$ ]] || continue
  [[ "$author" =~ ^[A-Za-z0-9][A-Za-z0-9-]{0,38}$ ]] || continue
  [[ "$merged_at" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$ ]] || continue

  # Only a merge commit written by the deciding workflow carries a verdict to rebuild. An
  # author name alone can be imitated, so the commit must also be signed by GitHub itself
  # (it signs only commits it creates), be a true merge, and have the judged commit as its
  # second parent.
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
  [[ "$title" =~ ^step-([0-9]+):\ new\ ([a-z0-9_]{1,40})\ record\ by\ ([A-Za-z0-9-]+)$ ]] || continue
  [ "${BASH_REMATCH[1]}" = "$step" ] && [ "${BASH_REMATCH[3]}" = "$author" ] || continue
  target="${BASH_REMATCH[2]}"
  [[ "$fuel" =~ ^Fuel\ ([0-9]{1,18})\ against\ ([0-9]{1,18})\ for\ the\ previous\ record\ holder\.\ Seed\ ([0-9]{1,20})\.$ ]] || continue

  jq -cn --arg problem "$PROBLEM" --argjson pr "$pr" --arg head_sha "$head" --arg author "$author" \
    --argjson step "$step" --arg target "$target" --argjson candidate "${BASH_REMATCH[1]}" \
    --argjson holder "${BASH_REMATCH[2]}" --arg seed "${BASH_REMATCH[3]}" --arg merged_at "$merged_at" \
    '{problem: $problem, pr: $pr, head_sha: $head_sha, author: $author, step: $step, target: $target,
      verdict: "ACCEPTED", reason: "", candidate_fuel: $candidate, holder_fuel: $holder, seed: $seed,
      merged: true, merged_at: $merged_at, run_url: ""}' > "$work/payload.json"
  echo "pull request $pr (step-$step, $target):"
  if "$here/send-verdict.sh" "$work/payload.json"; then sent=$((sent + 1)); else failed=$((failed + 1)); fi
done < "$work/merged.tsv"

echo "records sent: $sent, not delivered: $failed"
[ "$failed" = 0 ]
