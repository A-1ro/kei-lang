# Lessons from PR reviews

`gh pr merge` 後の Sonnet hook が自動追記する蓄積。SessionStart hook が
直近 N 件を Opus の system context に流す。卒業した教訓は SKILL.md / spec /
CLAUDE.md に落として、ここからは削除してよい。

## PR #69: [codex] Add v0.4 roadmap and operator support — 2026-06-27

- **Pattern**: JS `%` の意味論を確認してから展開する
  **Source**: A-1ro (owner) — crates/kei_emit/src/emit.rs:1086
  **Lesson**: JavaScript の `%` 演算子は ECMA-262 §6.1.6.1.5 で `a - trunc(a/b) * b`(被除数と同符号の truncated remainder)として定義されており Kei 仕様と一致するため、`BinOp::Rem` は `Div` 同様に `lhs % rhs` をそのまま emit すれば足り、IIFE や手動展開は不要。

- **Pattern**: operand を複数回 emit しない
  **Source**: chatgpt-codex-connector[bot] (P2) — crates/kei_emit/src/emit.rs:1084
  **Lesson**: 二項演算を手動展開するとき `lhs`/`rhs` を式中に 2 回以上 emit すると、extern 呼び出し等の副作用を伴う式で observable な二重評価が起き、観測可能な動作変化を生む — 必ず各オペランドを 1 回だけ emit すること。

- **Pattern**: GFM 表セル内の `|` はバックティック内でもエスケープ必須
  **Source**: A-1ro (owner) — spec/kei-spec-v0.1.md:141 / docs/kei-roadmap-v0.4.md:15
  **Lesson**: GFM テーブルではインラインコード(バックティック)の中でも `|` はセル区切りとして解釈されるため、`||` は必ず `\|\|` と書く — spec/kei-spec-v0.1.md:130 の既存行がリポジトリの慣習であり、新規追加行もこれに倣う。

- **Pattern**: `eval_binary` に短絡演算子の arm を追加しない
  **Source**: A-1ro (owner) — crates/kei_check/src/pbt.rs:557
  **Lesson**: `BinOp::Or`(および `Implies`)は `eval_expr` が短絡評価するため `eval_binary` には到達しない — `eval_binary` に `(Or, Bool, Bool)` arm を追加すると既存コメント「Or/Implies はここには来ない」と矛盾する到達不能コードになるので、短絡演算子の処理は `eval_expr` 側に統一する。

- **Pattern**: 演算子の Prec 変更は全 emit 呼び出し側に波及する
  **Source**: A-1ro (owner) — crates/kei_emit/src/emit.rs:1069
  **Lesson**: `Prec::Implication` のような新しい優先度を追加してある演算子に割り当てる場合、`emit_contract_check`・`emit_call` 引数・`RecordLit` フィールドなど既存のすべての emit 呼び出し側が渡す `Prec` 値も同時に更新しないと、不要な括弧が生成コードに増殖する。
