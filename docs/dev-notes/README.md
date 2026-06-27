# 開発ノート(自動蓄積)

このディレクトリは Kei の開発ワークフローで回す**自己改善ループ**の蓄積先。
コミットして共有する想定(手動編集も歓迎、ただし bot 由来の追記は残す)。

## ファイル

- `lessons-from-reviews.md` — マージされた PR のレビュー指摘から抽出した「次のPRで踏まない教訓」。
  `gh pr merge` 直後に Sonnet hook(`.claude/hooks/post-merge-lessons.sh`)が追記する。
- `handoff-candidates.md` — `HANDOFF.md` に昇格させる価値がありそうな**設計判断の候補**。
  同じく `gh pr merge` 直後の Sonnet hook(`.claude/hooks/post-merge-handoff.sh`)が追記する。

## ライフサイクル

1. **蓄積**: `gh pr merge <N>` 実行直後、PostToolUse hook が走り Sonnet がレビューと diff を読む
2. **注入**: 次セッション開始時、SessionStart hook(`.claude/hooks/session-start-learnings.sh`)が
   両ファイルの直近項目を Opus の system context に流す
3. **卒業**:
   - `handoff-candidates.md` のエントリは、内容を確認の上 `HANDOFF.md` 本体に統合 → 元エントリを削除
   - `lessons-from-reviews.md` のエントリは、`skills/kei/SKILL.md` や spec / CLAUDE.md に
     落とすべきものは落とす。残しておくこと自体に価値があれば残す(参照ログ)

## 編集ルール

- bot が追記した形式(`## PR #<N>: <title> — <date>` 始まり)は壊さない
- 古いエントリの削除は OK(卒業させたら消す)
- 「これは違うな」と思った bot 追記は人間判断で削除/書き換えして構わない
- 機密(認証情報など)が混ざっていたら即削除
