#!/usr/bin/env bash
# Acts on a verdict: merges an ACCEPTED pull request, posts the verdict as a comment and
# sends it, signed, to the site. Run by the deciding workflow's report job, which never
# builds or runs anything from the pull request.
#
# Safe to run twice for the same verdict: a pull request that is already merged at the judged
# commit is not merged or commented on again, and the site records a verdict once.
#
#   scripts/ci-report.sh <final.json>
set -euo pipefail
: "${PR:?}" "${REPO:?}" "${GH_TOKEN:?}"
file="${1:?usage: ci-report.sh <final.json>}"
here=$(cd "$(dirname "$0")" && pwd)
PROBLEM="${PROBLEM:-speed-race}"
printf '%s' "$PR" | grep -Eqx '[1-9][0-9]{0,6}' || { echo "bad pull request number"; exit 1; }
[ -s "$file" ] || { mkdir -p "$(dirname "$file")"; echo '{"verdict":"FAULT","reason":"the check produced no verdict"}' > "$file"; }

# Every field is read through a strict pattern before it is used.
field() { jq -r --arg k "$1" '.[$k] // "" | tostring' "$file"; }
pick() { field "$1" | grep -Ex "$2" || true; }
verdict=$(pick verdict 'ACCEPTED|REJECTED|FAULT')
verdict=${verdict:-FAULT}
reason=$(field reason | tr -cd '[:alnum:] .,:()<>_/`-' | cut -c1-200)
head=$(pick head_sha '[0-9a-f]{40}')
target=$(pick target '[a-z0-9_]{1,40}')
step=$(pick step '[1-9][0-9]{0,8}')
author=$(pick author '[A-Za-z0-9][A-Za-z0-9-]{0,38}')
candidate=$(pick candidate_fuel '[0-9]{1,18}')
holder=$(pick holder_fuel '[0-9]{1,18}')
seed=$(pick seed '[0-9]{1,20}')
cases=$(pick cases '[0-9]{1,6}')
run_url=$(pick run_url "https://github\.com/$REPO/actions/runs/[0-9]+")

merged=false
repeat=false
if [ "$verdict" = ACCEPTED ] && [ -n "$head" ] && [ -n "$target" ] && [ -n "$step" ]; then
  state=$(gh api "repos/$REPO/pulls/$PR" --jq '[(.merged | tostring), .head.sha] | join(" ")' 2>/dev/null || true)
  if [ "$state" = "true $head" ]; then
    # This verdict was acted on before. Only the site may still be missing it.
    merged=true
    repeat=true
  # Merged through the pull request, as a merge commit. GitHub checks that the pull request
  # is still open and that its head is the commit that was judged (`sha`). The merge commit is
  # written by this workflow's identity; a squash would instead write the contributor's
  # account email into public history. The guard allowed exactly one commit, named after the
  # branch, so that is all that enters history. A conflict means another file became the
  # record holder in the meantime. The commit message is read back by reconcile.sh.
  elif gh api -X PUT "repos/$REPO/pulls/$PR/merge" -f sha="$head" -f merge_method=merge \
    -f commit_title="step-$step: new $target record by $author" \
    -f commit_message="Fuel $candidate against $holder for the previous record holder. Seed $seed." >/dev/null 2>&1; then
    merged=true
  else
    verdict=STALE
    reason="the record moved while this was checked: rebase your one commit onto main and push again"
  fi
fi

line="**$verdict**"
[ -n "$reason" ] && line="$line ($reason)"
body="$line"
[ -n "$target" ] && body="$body"$'\n\n'"Target \`$target\`, step-$step."
if [ -n "$candidate" ] && [ -n "$holder" ] && [ "$holder" != 0 ] && [ "$candidate" != 0 ]; then
  less=$(awk -v c="$candidate" -v h="$holder" 'BEGIN { printf "%.1f", (1 - c / h) * 100 }')
  body="$body"$'\n'"Fuel $candidate against $holder for the record holder ($less percent less). Seed $seed, $cases inputs."
fi
[ "$merged" = true ] && body="$body"$'\n\n'"Merged. This file is the new record holder."
[ -n "$run_url" ] && body="$body"$'\n\n'"[The run]($run_url)"
[ "$repeat" = true ] || gh api -X POST "repos/$REPO/issues/$PR/comments" -f body="$body" >/dev/null
echo "$line"

# Tell the site. If it cannot be told, this job fails so the gap is visible, and the hourly
# catch-up workflow sends the record again.
if [ -n "${SWARMDESK_URL:-}" ] && [ -n "${VERDICT_SECRET:-}" ] && [ -n "$author" ] && [ -n "$head" ]; then
  payload=$(mktemp)
  jq -cn --arg problem "$PROBLEM" --argjson pr "$PR" --arg head_sha "$head" --arg author "$author" \
    --arg step "$step" --arg target "$target" --arg verdict "$verdict" --arg reason "$reason" \
    --arg candidate "$candidate" --arg holder "$holder" --arg seed "$seed" --argjson merged "$merged" \
    --arg run_url "$run_url" \
    '{problem: $problem, pr: $pr, head_sha: $head_sha, author: $author, step: ($step | tonumber? // null),
      target: $target, verdict: $verdict, reason: $reason, candidate_fuel: ($candidate | tonumber? // 0),
      holder_fuel: ($holder | tonumber? // 0), seed: $seed, merged: $merged, run_url: $run_url}' > "$payload"
  "$here/send-verdict.sh" "$payload"
fi
