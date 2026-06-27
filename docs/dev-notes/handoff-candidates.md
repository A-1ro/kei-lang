# HANDOFF.md candidates

`gh pr merge` 後の Sonnet hook が自動追記する候補集。HANDOFF.md に昇格
させたいエントリは人間レビューを経て本体に移し、ここから削除する。

## PR #70: chore: bump version to 0.4.0 — 2026-06-27

(no design-decision candidates for this PR)

<!-- 判断根拠:
     PR #70 はバージョン文字列の機械的置換のみ。
     ただし以下 2 点は将来の混乱防止として参考記録を残す。

     (a) MCP golden (tests/mcp/*.response.json) の serverInfo.version は
         env!("CARGO_PKG_VERSION") 由来なので、ワークスペース version 変更時は
         UPDATE_GOLDEN=1 cargo test -p kei_mcp --test golden_mcp の再生成が必須。
         しかし PR body にそのまま記載されており、HANDOFF.md に昇格するほど
         埋もれた情報ではないと判断。

     (b) バージョン管理対象外: runtime/(独立 npm パッケージ) と editors/vscode は
         Cargo workspace version とは独立して管理される。skills/kei/SKILL.md の
         バージョン言及は機能 PR 側で更新する慣例(本 PR ではなく PR #69 が担当)。
         これらも PR body に明記されているため HANDOFF.md 昇格不要と判断。
-->
