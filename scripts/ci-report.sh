#!/usr/bin/env bash
# Acts on a verdict: writes the record for an ACCEPTED pull request, posts the verdict as a
# comment and sends it, signed, to the site. Run by the deciding workflow's report job, which
# never builds or runs anything from the pull request.
#
# Safe to run twice for the same verdict: a record that main already holds through this pull
# request is not written or commented on again, and the site records a verdict once.
#
#   scripts/ci-report.sh <final.json>
set -euo pipefail
: "${PR:?}" "${REPO:?}" "${GH_TOKEN:?}" "${DEFAULT_BRANCH:?}"
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
runs=$(pick runs '[0-9]{1,3}')
faulty=$(pick holder_faulty 'true|false')
candidate_blob=$(pick candidate_blob '[0-9a-f]{40}')
holder_blob=$(pick holder_blob '[0-9a-f]{40}')
# The judged file itself, kept by the judge job next to its verdict.
bytes="$(dirname "$file")/candidate.rs"
run_url=$(pick run_url "https://github\.com/$REPO/actions/runs/[0-9]+")

merged=false
repeat=false
record_sha=""
api() { gh api -H "X-GitHub-Api-Version: 2022-11-28" "$@"; }
# The content hash of the record holder file at a commit.
holder_at() { api "repos/$REPO/contents/targets/$target/holder.rs?ref=$1" --jq .sha 2>/dev/null; }

# Writes the record as one commit on top of main whose only change is the judged file. A pull
# request is never merged: a merge combines the request with whatever main holds, and the
# result can be a file nobody judged. Here the file that lands is the file that was judged,
# by content hash, and it lands only if main still holds the file it was compared against.
#
# Returns 0 when main holds the judged file through this pull request, 1 when the record
# moved (STALE), 2 when the record could not be written (FAULT).
write_record() {
  local main now last blob tree new commit message
  message="step-$step: new $target record by $author"$'\n\n'
  message+="Fuel $candidate against $holder for the previous record holder. Seed $seed."$'\n'
  message+="Pull request #$PR, commit $head."
  for _ in 1 2 3 4; do
    main=$(api "repos/$REPO/git/ref/heads/$DEFAULT_BRANCH" --jq .object.sha) || return 2
    now=$(holder_at "$main") || return 2
    if [ "$now" = "$candidate_blob" ]; then
      # Main holds this very file already. That is this pull request's record only if the
      # commit that put it there names this pull request: an identical copy earns nothing.
      last=$(api "repos/$REPO/commits?sha=$main&path=targets/$target/holder.rs&per_page=1" \
        --jq '.[0].commit.message' | sed -n 4p) || return 2
      [ "$last" = "Pull request #$PR, commit $head." ] || return 1
      repeat=true
      return 0
    fi
    [ "$now" = "$holder_blob" ] || return 1
    base64 -w0 "$bytes" > "$bytes.b64"
    blob=$(jq -n --rawfile content "$bytes.b64" '{content: $content, encoding: "base64"}' \
      | api -X POST "repos/$REPO/git/blobs" --input - --jq .sha) || return 2
    [ "$blob" = "$candidate_blob" ] || return 2
    tree=$(api "repos/$REPO/git/commits/$main" --jq .tree.sha) || return 2
    new=$(jq -n --arg base "$tree" --arg path "targets/$target/holder.rs" --arg sha "$blob" \
      '{base_tree: $base, tree: [{path: $path, mode: "100644", type: "blob", sha: $sha}]}' \
      | api -X POST "repos/$REPO/git/trees" --input - --jq .sha) || return 2
    commit=$(jq -n --arg message "$message" --arg tree "$new" --arg parent "$main" \
      '{message: $message, tree: $tree, parents: [$parent]}' \
      | api -X POST "repos/$REPO/git/commits" --input - --jq .sha) || return 2
    # Not forced: GitHub refuses this when main is no longer the commit this one sits on.
    if jq -n --arg sha "$commit" '{sha: $sha, force: false}' \
      | api -X PATCH "repos/$REPO/git/refs/heads/$DEFAULT_BRANCH" --input - >/dev/null 2>&1; then
      [ "$(holder_at "$commit")" = "$candidate_blob" ] || return 2
      record_sha="$commit"
      return 0
    fi
    # Main moved. If it was another function's record, this file is unchanged: try again.
  done
  return 1
}

if [ "$verdict" = ACCEPTED ] && [ -n "$head" ] && [ -n "$target" ] && [ -n "$step" ] && [ -n "$author" ]; then
  outcome=2
  if [ -n "$candidate_blob" ] && [ -n "$holder_blob" ] && [ -s "$bytes" ]; then
    write_record && outcome=0 || outcome=$?
  fi
  case "$outcome" in
    0) merged=true ;;
    1)
      verdict=STALE
      reason="the record moved while this was checked: rebase your one commit onto main and push again"
      ;;
    *)
      verdict=FAULT
      reason="the record could not be written"
      ;;
  esac
fi

line="**$verdict**"
[ -n "$reason" ] && line="$line ($reason)"
body="$line"
[ -n "$target" ] && body="$body"$'\n\n'"Target \`$target\`, step-$step."
if [ "$faulty" = true ]; then
  body="$body"$'\n'"The record holder gave a wrong answer on at least one input, so a correct file replaces it whatever its fuel. Seed $seed, $cases inputs in ${runs:-1} runs."
elif [ -n "$candidate" ] && [ -n "$holder" ] && [ "$holder" != 0 ] && [ "$candidate" != 0 ]; then
  less=$(awk -v c="$candidate" -v h="$holder" 'BEGIN { printf "%.1f", (1 - c / h) * 100 }')
  body="$body"$'\n'"Fuel $candidate against $holder for the record holder ($less percent less). Seed $seed, $cases inputs in ${runs:-1} runs."
fi
if [ "$merged" = true ]; then
  body="$body"$'\n\n'"This file is the new record holder."
  [ -n "$record_sha" ] && body="$body Its record is commit $record_sha on \`$DEFAULT_BRANCH\`."
  body="$body"$'\n\n'"This pull request is closed, not merged: the workflow writes the record itself, so that exactly the file that was judged lands."
fi
[ -n "$run_url" ] && body="$body"$'\n\n'"[The run]($run_url)"
[ "$repeat" = true ] || gh api -X POST "repos/$REPO/issues/$PR/comments" -f body="$body" >/dev/null
# The request has done its work. Closing it also takes it out of its author's open count.
[ "$merged" = true ] && api -X PATCH "repos/$REPO/pulls/$PR" -f state=closed >/dev/null 2>&1 || true
[ -n "$record_sha" ] && [ -n "${GITHUB_OUTPUT:-}" ] && echo "recorded=true" >> "$GITHUB_OUTPUT"
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
