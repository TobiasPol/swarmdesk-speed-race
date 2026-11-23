#!/usr/bin/env bash
# Acts on a verdict: merges an ACCEPTED pull request, posts the verdict as a comment and
# sends it, signed, to the site. Run by the deciding workflow's report job, which never
# builds or runs anything from the pull request.
#
#   scripts/ci-report.sh <final.json>
set -euo pipefail
: "${PR:?}" "${REPO:?}" "${GH_TOKEN:?}" "${DEFAULT_BRANCH:?}"
file="${1:?usage: ci-report.sh <final.json>}"
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
if [ "$verdict" = ACCEPTED ] && [ -n "$head" ] && [ -n "$target" ] && [ -n "$step" ]; then
  # A merge commit made here, by this workflow's own identity, with the judged commit as its
  # second parent. GitHub then marks the pull request as merged. A squash through the pull
  # request API would instead write the contributor's account email into public history.
  # Naming the commit, not the branch, merges exactly what was judged. A conflict means
  # another file became the record holder in the meantime.
  if gh api -X POST "repos/$REPO/merges" -f base="$DEFAULT_BRANCH" -f head="$head" \
    -f commit_message="step-$step: new $target record by $author

Fuel $candidate against $holder for the previous record holder. Seed $seed." >/dev/null 2>&1; then
    merged=true
  else
    verdict=STALE
    reason="the record moved while this was checked: bring your branch up to date with main and push again"
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
gh api -X POST "repos/$REPO/issues/$PR/comments" -f body="$body" >/dev/null
echo "$line"

# Tell the site, signed with the secret only this job and the site hold.
if [ -n "${SWARMDESK_URL:-}" ] && [ -n "${VERDICT_SECRET:-}" ] && [ -n "$author" ] && [ -n "$head" ]; then
  printf '%s' "$SWARMDESK_URL" | grep -Eqx 'https://[a-z0-9.-]+' || { echo "bad site address"; exit 0; }
  payload=$(jq -cn --arg problem "$PROBLEM" --argjson pr "$PR" --arg head_sha "$head" --arg author "$author" \
    --arg step "$step" --arg target "$target" --arg verdict "$verdict" --arg reason "$reason" \
    --arg candidate "$candidate" --arg holder "$holder" --arg seed "$seed" --argjson merged "$merged" \
    --arg run_url "$run_url" \
    '{problem: $problem, pr: $pr, head_sha: $head_sha, author: $author, step: ($step | tonumber? // null),
      target: $target, verdict: $verdict, reason: $reason, candidate_fuel: ($candidate | tonumber? // 0),
      holder_fuel: ($holder | tonumber? // 0), seed: $seed, merged: $merged, run_url: $run_url}')
  signature=$(printf '%s' "$payload" | openssl dgst -sha256 -hmac "$VERDICT_SECRET" -hex | sed 's/^.* //')
  curl -sS --proto '=https' --max-time 20 -X POST -H 'Content-Type: application/json' \
    -H "X-Swarmdesk-Signature: sha256=$signature" --data-binary "$payload" \
    "$SWARMDESK_URL/api/verdicts" || echo "the site could not be reached"
fi
