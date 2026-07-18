#!/usr/bin/env bash
# file: 依存(@kei/runtime, @kei/hono)の dist/ が無ければ install + build する。
# 参照: docs/dogfood/2026-07-13-v0.9.0-todo-workers-api.md のギャップ1、対応案(a)。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"

ensure_built() {
  local dir="$1"
  if [ ! -f "$dir/dist/index.js" ]; then
    echo "[ensure-deps-built] $dir/dist/index.js が無いのでビルドします"
    (cd "$dir" && npm install --no-audit --no-fund && npm run build)
  fi
}

ensure_built "$ROOT/runtime"
ensure_built "$ROOT/tests/cli/packages/kei-hono"
