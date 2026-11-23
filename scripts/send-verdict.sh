#!/usr/bin/env bash
# Sends one verdict to the site, signed with the secret only this repository's workflows and
# the site hold. Tries four times over about a minute, because the site may be briefly
# unreachable. The site records a verdict once, so sending one twice is harmless.
#
#   scripts/send-verdict.sh <payload.json>
#
# Exit 0: the site has the verdict. Exit 1: it could not be delivered. Exit 2: bad setup.
set -euo pipefail
: "${SWARMDESK_URL:?}" "${VERDICT_SECRET:?}"
file="${1:?usage: send-verdict.sh <payload.json>}"
printf '%s' "$SWARMDESK_URL" | grep -Eqx 'https://[a-z0-9.-]+' || { echo "bad site address"; exit 2; }
payload=$(cat "$file")
signature=$(printf '%s' "$payload" | openssl dgst -sha256 -hmac "$VERDICT_SECRET" -hex | sed 's/^.* //')
answer=$(mktemp)

for wait in 0 5 15 45; do
  sleep "$wait"
  code=$(curl -sS --proto '=https' --max-time 20 -o "$answer" -w '%{http_code}' -X POST \
    -H 'Content-Type: application/json' -H "X-Swarmdesk-Signature: sha256=$signature" \
    --data-binary "$payload" "$SWARMDESK_URL/api/verdicts" 2>/dev/null || echo 000)
  if [ "$code" = 200 ]; then
    changed=$(jq -r '.changed // "unknown"' "$answer" 2>/dev/null || echo unknown)
    echo "the site has the verdict (new to it: $changed)"
    exit 0
  fi
  # A refusal will not change on a second try.
  case "$code" in 4??) echo "the site refused the verdict ($code)"; exit 1 ;; esac
  echo "the site answered $code"
done
echo "the site could not be told"
exit 1
