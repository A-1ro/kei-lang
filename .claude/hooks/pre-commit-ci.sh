#!/usr/bin/env bash
# pre-commit CI: enforce kei-verify (fmt / clippy / test) before any `git commit`
# initiated through Claude Code. Wired via .claude/settings.json -> PreToolUse
# hook with `if: Bash(git commit*)`.
#
# Output protocol: print JSON to stdout on failure, route cargo output to stderr
# so Claude Code parses only the structured deny.

set -uo pipefail

cd "$(git rev-parse --show-toplevel 2>/dev/null)" || exit 0

deny() {
  jq -cn --arg reason "$1" \
    '{hookSpecificOutput:{hookEventName:"PreToolUse",permissionDecision:"deny",permissionDecisionReason:$reason}}'
  exit 0
}

step() {
  local name="$1"
  shift
  if ! "$@" >&2; then
    deny "pre-commit CI failed at ${name}. Reproduce: $*"
  fi
}

step fmt    cargo fmt --all -- --check
step clippy cargo clippy --workspace --all-targets -- -D warnings
step test   cargo test --workspace

# cargo test で e2e の package-lock.json が変化することがあるので working tree を復元
# (staged 状態は触らない。意図して lockfile を更新した場合は事前に staging されている前提)
git checkout -- tests/cli/projects/app/package-lock.json tests/e2e/package-lock.json 2>/dev/null || true

exit 0
