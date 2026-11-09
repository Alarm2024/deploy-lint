#!/usr/bin/env bash
# Post Preflight findings as inline pull request review comments (file + line).
# Never posts secret values — messages come from Preflight's masked reason text only.
set -euo pipefail

if [[ "${GITHUB_EVENT_NAME:-}" != "pull_request" ]]; then
  echo "Not a pull_request event; skipping PR comments."
  exit 0
fi

PR_NUMBER="${PR_NUMBER:-${GITHUB_EVENT_PULL_REQUEST_NUMBER:-}}"
REPO="${GITHUB_REPOSITORY:?}"

if [[ -z "$PR_NUMBER" ]]; then
  echo "PR_NUMBER not set; skipping PR comments."
  exit 0
fi

if [[ "$#" -eq 0 ]]; then
  echo "No files to scan; skipping PR comments."
  exit 0
fi

MERGED="$(mktemp)"
trap 'rm -f "$MERGED"' EXIT
echo '{"findings":[]}' >"$MERGED"

for f in "$@"; do
  if [[ ! -f "$f" ]]; then
    continue
  fi
  ONE="$(mktemp)"
  cargo run --quiet -- --json "$f" >"$ONE"
  jq -s '{findings: (.[0].findings + .[1].findings)}' "$MERGED" "$ONE" >"${MERGED}.next"
  mv "${MERGED}.next" "$MERGED"
  rm -f "$ONE"
done

FINDING_COUNT="$(jq '.findings | length' "$MERGED")"

if [[ "$FINDING_COUNT" -eq 0 ]]; then
  echo "No Preflight findings; no PR comments to post."
  exit 0
fi

COMMENTS_JSON="$(jq -c '
  [.findings[] |
    {
      path: (.file | gsub("\\\\"; "/")),
      line: .line,
      side: "RIGHT",
      body: ("**" + .rule_id + "** (" + .severity + "): " + .reason + "\n\nFix: " + .fix_hint)
    }]
' "$MERGED")"

BATCH_SIZE=45
TOTAL="$(jq 'length' <<<"$COMMENTS_JSON")"
OFFSET=0
BATCH=1

while [[ "$OFFSET" -lt "$TOTAL" ]]; do
  SLICE="$(jq -c ".[$OFFSET:$((OFFSET + BATCH_SIZE))]" <<<"$COMMENTS_JSON")"
  BODY="Preflight findings (batch ${BATCH})"
  jq -n \
    --arg event COMMENT \
    --arg body "$BODY" \
    --argjson comments "$SLICE" \
    '{event: $event, body: $body, comments: $comments}' \
    | gh api --method POST "repos/${REPO}/pulls/${PR_NUMBER}/reviews" --input -
  OFFSET=$((OFFSET + BATCH_SIZE))
  BATCH=$((BATCH + 1))
done

echo "Posted Preflight review comments for ${FINDING_COUNT} finding(s)."
