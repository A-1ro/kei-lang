#!/usr/bin/env bash
# session-start-learnings: at session start, inject the most recent lessons
# and HANDOFF.md candidates into Opus's system context via additionalContext.
#
# Wired via .claude/settings.json -> SessionStart hook (no matcher).

set -uo pipefail

cd "$(git rev-parse --show-toplevel 2>/dev/null)" || { echo '{}'; exit 0; }

LESSONS=docs/dev-notes/lessons-from-reviews.md
CANDS=docs/dev-notes/handoff-candidates.md

# Extract the N most recent "## PR #..." sections (top-of-file = newest IF we
# prepend, but our hook appends. So tail of file = newest). We pull the last
# N sections via reverse-scan.
extract_recent_sections() {
  local file="$1"
  local n="$2"
  [ -f "$file" ] || return 0
  awk -v n="$n" '
    /^## PR #/ { headings[++h] = NR }
    { lines[NR] = $0 }
    END {
      if (h == 0) exit
      start = headings[h - n + 1]
      if (start == "" || h < n) start = headings[1]
      for (i = start; i <= NR; i++) print lines[i]
    }
  ' "$file"
}

recent_lessons=$(extract_recent_sections "$LESSONS" 5)
recent_cands=$(extract_recent_sections "$CANDS" 3)

if [ -z "$recent_lessons" ] && [ -z "$recent_cands" ]; then
  echo '{}'
  exit 0
fi

context='# Kei 自己改善ループ — 直近の蓄積(自動注入)

`gh pr merge` 後の post-merge hook が蓄積した、過去 PR からの教訓と
HANDOFF.md 候補です。今のセッションで Kei コンパイラを編集するときに
**同じ指摘を二度踏まない**ためのリマインダとして参照してください。
卒業した項目は `docs/dev-notes/` 配下から削除して構いません。'

if [ -n "$recent_lessons" ]; then
  context="${context}

## Lessons from recent PR reviews

${recent_lessons}"
fi

if [ -n "$recent_cands" ]; then
  context="${context}

## Pending HANDOFF.md candidates

${recent_cands}"
fi

jq -cn --arg ctx "$context" \
  '{hookSpecificOutput:{hookEventName:"SessionStart",additionalContext:$ctx}}'
