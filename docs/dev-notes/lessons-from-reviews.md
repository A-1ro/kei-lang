# Lessons from PR reviews

`gh pr merge` 後の Sonnet hook が自動追記する蓄積。SessionStart hook が
直近 N 件を Opus の system context に流す。卒業した教訓は SKILL.md / spec /
CLAUDE.md に落として、ここからは削除してよい。

## PR #71: chore: add Claude Code automation skills and hooks — 2026-06-27

> **Note**: 本 PR のマージで post-merge-lessons agent が初回発火したが、子セッションの
> permission(don't-ask mode で書き込み deny)により本ファイルへの追記に失敗した。
> 以下のメタ教訓は失敗事象そのものから手動で抽出した(PR #71 自体はレビュー無しで
> マージされたので外部由来の教訓は無い)。同 PR #72 で permission allow を追加し、
> 次回以降は自動追記される。

- **Pattern**: hook 子セッションは親と permission が独立 — 書き込みパスを明示 allow する
  **Source**: PR #71 マージ後の post-merge agent hook blocking error
  **Lesson**: `type: agent` hook で起動する子セッションは、親セッションの permission を継承しない(don't-ask mode で Edit/Write/Bash の書き込みが deny される)。Hook で書き込ませたいパスは `.claude/settings.json` の `permissions.allow` に明示追加する必要がある(例: `"Edit(docs/dev-notes/**)"`, `"Write(docs/dev-notes/**)"`)。これを忘れると hook は **静かに発火するが何も書かれない** 状態になり、「動いていない」ように見える(blocking error は親セッションには届くが、log 経路を知らないと気付けない)。

- **Pattern**: hook の watcher は新規 settings.json でも即時反映される
  **Source**: PR #71 マージ後、現セッション中に発火が観測された
  **Lesson**: 「settings.json が session start 時に存在しなかった場合は watcher が認識しない」と CLAUDE Code の挙動を仮定していたが、実際は新規 settings.json でも即時に watcher が認識する(少なくとも `.claude/settings.json` の場合)。hook 検証時に「次セッションから効く」前提で動作確認を省略すると、初回マージで予期せぬ振る舞いが出る可能性がある。新規 hook は merge 前に必ず pipe-test で完全検証する。

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

## PR #72: fix(hooks): grant dev-notes write permission and recover PR #71 loop — 2026-06-27

(no actionable patterns)

## PR #72 (auditor re-run): kei-invariant-auditor PostToolUse — 2026-06-27

(no actionable patterns — hook triggered by invariant auditor `git diff --stat` tool use, not by `gh pr merge`; most recent merged PR #72 already documented above)

## PR #79: chore(hooks): auto-run kei-code-review on gh pr create — 2026-06-28

- **Pattern**: `gh pr view --json` の fields に `author` を含め忘れる
  **Source**: A-1ro (owner) — general discussion (kei-code-review finding #1, CONFIRMED/correctness)
  **Lesson**: hook prompt で dependabot PR をスキップする条件として `author.login` を参照する場合、`gh pr view <N> --json` に `author` を明示指定しないとフィールドが null になりスキップ条件が常に無効になる。`--json` に渡す fields リストは実際に参照する全フィールドを列挙すること。

- **Pattern**: `--json` で取得した未使用フィールドを残さない
  **Source**: A-1ro (owner) — general discussion (kei-code-review finding #2, CONFIRMED/correctness)
  **Lesson**: hook スクリプトや prompt で `gh pr view --json body,...` のように `body` を取得しながら skip ロジックで一度も参照しないと dead field になる。使わないフィールドは `--json` から除いてノイズを減らす。逆に参照するフィールドは必ず取得リストに含めること(finding #1 と対になる教訓)。

- **Pattern**: draft PR チェックはセッション起動前に行う
  **Source**: A-1ro (owner) — general discussion (kei-code-review finding #3, PLAUSIBLE/pitfalls)
  **Lesson**: 現行の hook は draft PR でも Sonnet 子セッションを起動してから `gh pr view` で draft 判定しスキップしている。セッション起動コストを避けるには、親フック側(`tool_response.stdout` の URL 確定直後)で `gh pr view <N> --json isDraft` を叩いて draft なら子セッションを起動しない分岐を入れる方が望ましい。

## PR #81: feat(skills): kei-dogfood — auto-file next-version Issues from feedback — 2026-06-28

- **Pattern**: `gh issue create --label` は複数ラベルをカンマ区切りで渡せない
  **Source**: A-1ro (owner) — `.claude/skills/kei-dogfood/SKILL.md` line 292 (CONFIRMED/correctness)
  **Lesson**: `gh issue create --label "dogfood, from-v0.4, fix-chain"` のようにカンマ区切り文字列を 1 つの `--label` に渡すと、それを単一ラベル名として扱い 404/422 で失敗する。複数ラベルは `--label "dogfood" --label "from-v0.4" --label "fix-chain"` のように flag を繰り返すか、動的に `label_args` 配列を組み立てること。

- **Pattern**: `gh issue list --milestone` はマイルストーン番号で指定する
  **Source**: A-1ro (owner) — `.claude/skills/kei-dogfood/SKILL.md` line 268 (PLAUSIBLE/correctness)
  **Lesson**: `gh issue list --milestone v0.5` のようにタイトル文字列でフィルタすると、大文字小文字・空白の差異で silently 0 件になる。事前に `gh api repos/:owner/:repo/milestones --jq ".[] | select(.title==\"$milestone\") | .number"` でマイルストーン番号を解決し、番号で指定することで確実に dedup できる。

- **Pattern**: 自然言語の承認ゲートはLLMが短絡しうる
  **Source**: A-1ro (owner) — `.claude/skills/kei-dogfood/SKILL.md` line 286 (PLAUSIBLE/pitfalls)
  **Lesson**: 「承認前に `gh issue create` を絶対実行しない」を自然言語指示のみで縛ると、LLM が好意的な曖昧メッセージを承認とみなして gate を抜ける可能性がある。`permissions.allow` から当該コマンドを外してパーミッションプロンプトをバックストップにするか、SKILL.md に `# HARD GATE` ブロックを設けて禁止コマンドを明示するなど機械的な防線を追加すること。

- **Pattern**: `gh issue comment` の出力はコメント URL アンカーを含まない
  **Source**: A-1ro (owner) — `.claude/skills/kei-dogfood/SKILL.md` line 293 (PLAUSIBLE/pitfalls)
  **Lesson**: `gh issue comment <N> --body "..."` はデフォルトで Issue URL(`https://github.com/.../issues/N`)しか返さず、`#issuecomment-<id>` アンカーが付かない。ディープリンクを出力に含めたい場合は `--json url --jq '.url'` を追加して comment アンカー付き URL を取得すること(`gh` v2.17+ 必須)。

## PR #82: chore(skills): plan-then-delegate を実装タスクで常時発火に緩める — 2026-06-28

- **Pattern**: skill トリガーキーワードは複合語に絞る
  **Source**: A-1ro (owner) — `.claude/skills/plan-then-delegate/SKILL.md` (inline review comment, PLAUSIBLE/pitfalls)
  **Lesson**: `「対応」` のような単語を単体でスキル発火トリガーに追加すると、「〇〇の質問に対応して」「設計案に対応した…」など編集意図のない文脈でも誤発火する。トリガーキーワードは `「レビュー対応」「CI 対応」` のように複合語・修飾語付きに限定し、汎用単語は登録しないこと。

## PR #82 (altitude-finder re-run): kei-code-review PostToolUse — 2026-06-28

(no actionable patterns — hook triggered by `grep` tool call inside kei-code-review altitude finder for PR #83 (OPEN, not merged); most recent merged PR #82 already documented above)

## PR #82 (verifier re-run): kei-code-review PostToolUse — 2026-06-28

(no actionable patterns — hook triggered by Bash tool call (`mkdir`/`cat` creating `requires_old_lambda.kei` scratch file) inside kei-code-review verifier subagent for PR #83 (OPEN, not merged); most recent merged PR #82 already documented above)

## PR #83: feat: v0.4 remaining — M24 stock e2e + M25 lambdas + M26 Money notice — 2026-06-28

- **Pattern**: `old(lambda内式)` はラムダパラメータ参照を禁止する
  **Source**: A-1ro (owner) — crates/kei_emit/src/emit.rs:1205 (1st pass review, CONFIRMED/correctness 🔴)
  **Lesson**: `collect_old_exprs` がラムダ body を横断するとき、ラムダパラメータを参照する式(例: `old(p.qty)`)を巻き上げると未定義参照の TS が生成され `ReferenceError` で全実行が死ぬ。`check` 側にラムダパラム参照禁止ガードを設け、`emit` 側でも `old(...)` 中のラムダスコープ変数を検知して KEI-E4002 を発火させること。

- **Pattern**: AST 拡張後は pbt.rs の eval も追従させる
  **Source**: A-1ro (owner) — crates/kei_check/src/pbt.rs ~L980 (1st pass review, CONFIRMED/correctness 🔴)
  **Lesson**: `Expr::Lambda` を AST に追加した場合、`pbt.rs` の `eval_list_method` も `Expr::Lambda` を受け取る arm を追加しないと、generative 検証が `[bounded]` → `[runtime]` に静かに劣化する(反例検出が無効化される)。AST ノード追加時は必ず pbt.rs の eval 網羅性を確認すること。

- **Pattern**: spec §2.5 はパーサ実装に先行させる(0引数ラムダ)
  **Source**: A-1ro (owner) — crates/kei_syntax/src/parser.rs:1631 (1st pass review, CONFIRMED/invariant 🔴)
  **Lesson**: `() => expr` を受理するパーサ実装が先に入り spec §2.5 に「0引数ラムダ禁止」が追記されないと、不変条件 #4(spec-first)違反になる。新文法の実装前に spec を更新し、禁止構文はパーサでエラーにする(golden で固定する)こと。

- **Pattern**: 修正コードが新バグを持ち込む連鎖を想定してレビューを重ねる
  **Source**: A-1ro (owner) — general discussion (2nd pass review summary)
  **Lesson**: 1st pass で潰した F0/F1/F3 の修正コード自体が N0/N1/N3 という新規バグを持ち込んだ(0引数ラムダ修正の cascade、fix 文面の実現不能指示、`old()` 二段防御の片側漏れ)。修正 PR では必ず再レビューを 1 回追加し、「修正コードが新バグを入れていないか」を独立に確認すること。

- **Pattern**: fix 文面(Agent Repair Protocol)の実現可能性を検証する
  **Source**: A-1ro (owner) — crates/kei_check/src/check.rs:1600 (2nd pass review, CONFIRMED/correctness 🔴)
  **Lesson**: Diagnostic の `fix` フィールドに書く修正指示(例: 「Pass through the lambda parameter, or compute it outside」)は、その指示が実際に Kei の文法・制約下で実現可能かを確認してから書く。arity-1 制約やキャプチャ禁止により両案が実現不能な場合、誤誘導になり Agent Repair Protocol が機能しなくなる。

- **Pattern**: examples 内 `Money.zero` は spec §2.4 と整合させる
  **Source**: A-1ro (owner) — examples/contracts/withdraw.kei + transfer.kei (2nd pass review, CONFIRMED/invariant 🔴)
  **Lesson**: spec §2.4 で `Money.zero` を廃止・無効化した場合、examples/ 内のすべての `Money.zero` 参照も同時に更新する(MCP 配信時に spec と examples が矛盾するドキュメントを返してしまう)。spec 変更後は `grep -r "Money.zero" examples/` で残存参照を即座に検出・修正すること。

- **Pattern**: TS 予約語はラムダパラメータ名として素通りさせない
  **Source**: A-1ro (owner) — crates/kei_emit/src/emit.rs:935 (3rd pass review, CONFIRMED/pitfalls 🔴)
  **Lesson**: `class`/`var`/`null`/`this` 等の TypeScript 予約語がラムダパラメータ名として使われると、emit した TS が `tsc` で parse 不能になる。check.rs でラムダパラメータ名を検証する際に TS 予約語リストと照合し、予約語なら KEI-E 系エラーを発火させること(golden テストで `err_type_lambda_param_ts_reserved` として固定済み)。

- **Pattern**: `old(...)` walker は check と emit で重複しないよう構造化する
  **Source**: A-1ro (owner) — crates/kei_check/src/check.rs:3549 (3rd pass review, follow-up PR 行き)
  **Lesson**: `old(...)` を走査する walker が check.rs と emit.rs に 2 本同形で存在すると、一方に変更を加えたとき他方が同期漏れになる(M25 全体の構造的問題)。follow-up PR で共通 walker を `kei_check` または新クレートに切り出し、双方から参照する形に整理すること。

## PR #84: chore: bump version to 0.4.2 — 2026-06-29

(no actionable patterns)

## PR #84 (issue-check re-run): post-merge-lessons PostToolUse — 2026-06-29

(no actionable patterns — hook triggered by Bash tool call checking issue states (`gh issue view` loop for issues #54–#62), not by `gh pr merge`; most recent merged PR #84 already documented above)

## PR #87: chore(deps): bump lsp-server from 0.7.9 to 0.8.0 — 2026-07-03 merged

(no actionable patterns — dependabot dependency bump, no reviews or discussion)

## PR #88: chore(deps): bump the npm-minor-patch group across 3 directories with 2 updates — 2026-07-03 merged

(no actionable patterns — dependabot dependency bump, no reviews or discussion)

## PR #102: docs: v0.5 ロードマップ + v1.0 到達戦略(Workers + Hono API) — 2026-07-03 merged

- **Pattern**: Source of Truth 一覧の欠番を残さない
  **Source**: A-1ro (owner) — CLAUDE.md:18 (inline, CONFIRMED/correctness)
  **Lesson**: 新バージョンのロードマップを追加して CLAUDE.md の Source of Truth 一覧を編集する際は、既存の中間バージョン(例: `docs/kei-roadmap-v0.4.md`)への参照が抜け落ちていないか確認し、v0.3 → v0.5 のような欠番を作らない — 一覧に無いロードマップは次セッションのエージェントが /goal 契約の参照先を解決できなくなる。なお CLAUDE.md は auto-fix 除外パスのため、この種の修正は人間側で反映してもらう。

## PR #103: feat: M28 論理積 && を追加 (#91) — 2026-07-03 (merge 未完了: base branch policy によりブロック、レビュー教訓のみ先行記録)

- **Pattern**: Prec renumber は全 emit 呼び出し側へ波及
  **Source**: A-1ro (owner) — crates/kei_check/src/check.rs (inline, CONFIRMED/correctness)
  **Lesson**: `bin_prec` の優先順位を renumber したら(例: Mul 5→6)、同ファイル内の postfix 系 `child(…, N)` 呼び出し(base/callee/expr)も必ず新しい数値に更新する — 片方だけ更新すると `ensures result == -(a * b)` の契約テキスト描画で括弧が落ち、再パース時に別 AST になる表示忠実性 regression を生む。kei_fmt 側と kei_check 側の両方を grep して突き合わせること(PR #69 教訓の再発)。

## PR #103: feat: M28 論理積 && を追加 (#91) — 2026-07-03 merged

(merge 完了を確認 — 2026-07-03T10:53:07Z、admin squash merge。レビュー教訓「Prec renumber は全 emit 呼び出し側へ波及」は直上の先行記録セクションに記載済みのため重複追記なし。新規の actionable pattern なし)

## PR #105: chore: bump version to 0.4.3 — 2026-07-03 merged

(no actionable patterns — version bump PR, no inline comments / discussion / reviews)

## PR #105: chore: bump version to 0.4.3 — 2026-07-03 merged

(no actionable patterns — hook re-ran on a non-merge command; latest merged PR is still #105, a version-bump PR already recorded above with no review activity)

## PR #105: chore: bump version to 0.4.3 — 2026-07-03 merged

(no actionable patterns — hook re-ran on a non-merge command (cargo check of a scratchpad repair.kei on the M29 PR branch); latest merged PR is still #105, a version-bump PR already recorded above with no review activity)

## PR #105: chore: bump version to 0.4.3 — 2026-07-03 merged

(no actionable patterns — hook re-ran on a non-merge command (cargo check of repair2.kei in a missing scratchpad worktree); latest merged PR is still #105, a version-bump PR already recorded above with no review activity)

## PR #106: feat: M29 List.contains を追加 (#92) — 2026-07-04 merged

(no actionable patterns — CI (clippy/fmt/test) all green, admin squash-merge; no inline review comments, no discussion comments, no reviews)

## PR #106: feat: M29 List.contains を追加 (#92) — 2026-07-04 merged

(no actionable patterns — hook re-ran on a non-merge command (cargo transpile of a tagged-string concat repro in scratchpad); latest merged PR is still #106, already recorded above with no inline comments / discussion / reviews)

(no actionable patterns — hook re-ran 2026-07-04 on a non-merge command (`cargo run -p kei_cli -- check` of a String.map lambda repro in the wt-m30 scratchpad worktree); latest merged PR is still #106, already recorded above with no review activity)

## PR #108: feat: M30 文字列 stdlib 段階1 (#107) — 2026-07-04 merged

(no actionable patterns — CI (clippy/fmt/test) all green, admin squash-merge; no inline review comments, no discussion comments, no reviews)

## PR #110: chore: bump version to 0.4.4 — 2026-07-04 merged

(no actionable patterns — version bump PR, CI (clippy/fmt/test) all green, admin squash-merge; no inline review comments, no discussion comments, no reviews)

## PR #111: feat: M34 MCP 検証経路強化 — kei_check generative + opaque import 可視化 (#89, #90) — 2026-07-05 merged

(no actionable patterns — hook ran on a non-merge command (git fetch + cargo check of feat/m34-mcp-verification branch, EXIT=0); latest merged PR #111 has no inline review comments, no discussion comments, no reviews)

## PR #111: feat: M34 MCP 検証経路強化 — kei_check generative + opaque import 可視化 (#89, #90) — 2026-07-05 merged

(no actionable patterns — hook ran on a non-merge command (gh api contents lookup of PR #111 head tools.rs); PR #111 has 0 inline review comments, 0 discussion comments, 0 reviews; already recorded in prior sections)

## PR #111: feat: M34 MCP 検証経路強化 — kei_check generative + opaque import 可視化 (#89, #90) — 2026-07-05 merged

(no actionable patterns — hook ran on a non-merge command (`cargo test --workspace`, 35 suites all ok, exit=0); latest merged PR is still #111, which has 0 inline review comments, 0 discussion comments, 0 reviews; already recorded in prior sections)

## PR #111: feat: M34 MCP 検証経路強化 — kei_check generative + opaque import 可視化 (#89, #90) — 2026-07-05 merged

(no actionable patterns — hook ran on a status-check command (PR #111 merge state + PR #112 build check), not `gh pr merge`; latest merged PR is still #111, which has 0 inline review comments, 0 discussion comments, 0 reviews; already recorded in prior sections)

## PR #112: fix: M34 レビュー対応 — generative スキップ可視化と応答の構造化 — 2026-07-05 merged

- **Pattern**: ゲート条件と診断生成のタイミング不一致
  **Source**: A-1ro (inline, crates/kei_mcp/src/tools.rs:273)
  **Lesson**: 「静的エラーなし」を条件にする場合は、後段(PBT/generative)が積む診断(KEI-E4005 など)を評価対象から除外し、check.rs 側の「PBT 実行前クリーン」判定と厳密に一致させること — さもないと反例 1 件で skipped 一覧が丸ごと消え、不可視スキップが再発する。
- **Pattern**: 検証パイプラインの二重実行
  **Source**: A-1ro (inline, crates/kei_mcp/src/tools.rs:275)
  **Lesson**: CheckOptions 経由で generative 実行済みのモジュールに対して MCP 側で run_module_with_limit_reporting を再実行しない — 最悪 10,000 ケース × 2 になる。kei_check に `GenerativeRun` の結果を返す経路を足し、1 回の実行から outcomes と skipped の両方を得る形が本筋。
- **Pattern**: usize::MAX を JSON sentinel に使わない
  **Source**: A-1ro (inline, crates/kei_check/src/pbt.rs:294)
  **Lesson**: JSON 応答に載せる数値 sentinel は JS の `Number.MAX_SAFE_INTEGER`(2^53-1)以内(例: 9007199254740991)に収めるか、overflow 時に不正確である旨をスキーマにドキュメント化する — `usize::MAX` は JS クライアントで丸まる。

## PR #112: fix: M34 レビュー対応 — generative スキップ可視化と応答の構造化 — 2026-07-05 merged

(no actionable patterns — hook ran on a non-merge command (scratchpad transpile test `fold_old.kei` via kei_emit example, exit=0); latest merged PR is still #112, whose review lessons are already recorded in the section above)

## PR #112: fix: M34 レビュー対応 — generative スキップ可視化と応答の構造化 — 2026-07-05 merged

(no actionable patterns — hook ran on a non-merge command (kei_emit transpile scratch test for fold+old counter swap, exit=0); latest merged PR is still #112, whose 3 inline review comments (A-1ro) are already recorded in the prior PR #112 section; 0 discussion comments, no new review activity)

## PR #112: fix: M34 レビュー対応 — generative スキップ可視化と応答の構造化 — 2026-07-05 merged

(no actionable patterns — hook ran on a non-merge command (scratchpad lambda-capture repro `result_capture.kei`, cargo run failed: no Cargo.toml in scratchpad); latest merged PR is still #112, whose 3 inline review lessons (A-1ro) are already recorded above; no new review activity)

## PR #114: feat: M31 ラムダの読み取り専用キャプチャ (#59 後続 / dogfood critical) — 2026-07-05 merged

- **Pattern**: 意味論反転時の stale コメント残留
  **Source**: A-1ro (inline, crates/kei_check/src/check.rs:2607)
  **Lesson**: あるフィールドやフラグの役割を反転させる変更(例: `lambda_floor` が M25「キャプチャ禁止の隔離壁」→ M31「今ラムダ中かのフラグ」)を入れたら、`grep` で旧意味論を語るコメント(「キャプチャ禁止」等)を全箇所洗い出して一括更新すること — 改変行の直近だけ直すと、未改変箇所の古いコメント(L2381/L2537)を信じた次の変更が capture 意味論を壊す。
- **Pattern**: golden 削除は承認記録を PR に残す
  **Source**: A-1ro (general discussion)
  **Lesson**: `tests/golden/` の削除・転用(不変条件1)はセッション内承認だけでなく、契約根拠(roadmap/spec の該当節)を添えた承認記録コメントを PR に残す運用を継続すること — 後から監査可能になる。

## PR #114: feat: M31 ラムダの読み取り専用キャプチャ (#59 後続 / dogfood critical) — 2026-07-05 merged

(no actionable patterns — hook ran on a non-merge command (CI 完了待ち: `gh run list --branch main`、直近 2 run とも completed success); latest merged PR is still #114, whose 2 review lessons (A-1ro inline + general discussion) are already recorded in the section above; no new review activity)

## PR #116: chore: bump version to 0.4.5 — 2026-07-05 merged

(no actionable patterns — version-bump PR; 0 inline review comments, 0 discussion comments, no reviews)

## PR #116: chore: bump version to 0.4.5 — 2026-07-05 merged

(no actionable patterns — hook ran on a non-merge command (scratchpad enum-spread transpile repro `enum_spread.kei`, transpiled successfully); latest merged PR is still #116, a version-bump PR already recorded above with no review activity)

(2026-07-06 再実行: hook が非マージコマンド(`feat/m32-record-spread` の `cargo check` 実確認、EXIT=0)で発火。最新マージ PR は依然 #116 で、レビュー活動なしを再確認 — 新規パターンなし)

## PR #116: chore: bump version to 0.4.5 — 2026-07-05 merged

(no actionable patterns)

## PR #116: chore: bump version to 0.4.5 — 2026-07-05 merged

(no actionable patterns — hook ran on a non-merge command (M32 record-spread repro: `repro_enum_spread.kei` の enum variant への spread が期待通り診断+fix 提案を返すことを確認); latest merged PR is still #116 (version-bump, 0 inline comments, 0 discussion, no reviews), already recorded above)

## PR #116: chore: bump version to 0.4.5 — 2026-07-05 merged

(no actionable patterns — hook ran on a non-merge command (M32 worktree での `..p` 2 ドット spread near-miss の `kei check` 実確認、KEI-E0101 を確認); latest merged PR is still #116 (version-bump PR, 0 inline comments, 0 discussion comments, no reviews — 再確認済み), already recorded above)

## PR #117: feat: M32 record spread — 差分更新構文 (#97) — 2026-07-06 merged

- **Pattern**: enum variant spread silent corruption
  **Source**: A-1ro (kei-code-review), crates/kei_check/src/check.rs:3149
  **Lesson**: TS 側 enum 値は `{ kind, fields }` の 2 層構造なので、enum variant リテラルへの機能拡張(spread 等)は emit 後の実行値まで検証すること — tsc の excess-property check は spread リテラルを通すため型チェックでは silent data corruption を検出できない。合意設計(spread の型 = コンストラクタの record 型)から逸脱せず、variant リテラルでの spread は KEI-E2004 で拒否し golden で固定する。
- **Pattern**: parser recovery logic copy-paste drift
  **Source**: A-1ro (kei-code-review), crates/kei_syntax/src/parser.rs:1475
  **Lesson**: parser の区切り/リカバリ処理(skip_newlines → eat → error + recover)を新分岐に足すときは逐語コピペせず、`expect_record_lit_separator` のようなヘルパーに共通化して 2 経路のドリフトを防ぐ。

## PR #117: feat: M32 record spread — 差分更新構文 (#97) — 2026-07-06 merged

(no new actionable patterns — hook ran on a non-merge command (`git show` で M33 map stage1 の error golden JSON 確認); latest merged PR is #117 で、2 件の CONFIRMED 指摘(enum variant spread silent corruption / parser recovery copy-paste drift)は直上のセクションに記録済み)

## PR #117: feat: M32 record spread — 差分更新構文 (#97) — 2026-07-06 merged

(no actionable patterns — hook ran on a non-merge command (M33 scratchpad での expected_ty leak テストケース `t1.kei`/`t2.kei` の `kei check --json` 実行); latest merged PR is still #117, whose 2 lessons (enum variant spread silent corruption / parser recovery copy-paste drift) are already recorded above — no new review activity found)

## PR #117: feat: M32 record spread — 差分更新構文 (#97) — 2026-07-06 merged

(no actionable patterns — hook ran on a non-merge command (M33 scratchpad での nested-position `Map.empty()` 型推論ケース `t4.kei`/`t5.kei` の `kei check` 実行、両方 exit=0); latest merged PR is still #117 with 2 inline comments / 0 discussion comments / 1 review — its 2 lessons (enum variant spread silent corruption / parser recovery copy-paste drift) are already recorded above, no new review activity)

## PR #117: feat: M32 record spread — 差分更新構文 (#97) — 2026-07-06 merged

(no actionable patterns — hook ran on a non-merge command (M33 scratchpad での `t6.kei`/`t7.kei` の `kei check` 実行: Option<Map> 型不一致が leak せず KEI-E2001、未注釈 `Map.empty().set(...)` が KEI-E2012 を正しく報告); latest merged PR is still #117 with 2 inline comments / 0 discussion comments / 1 review — its 2 lessons (enum variant spread silent corruption / parser recovery copy-paste drift) are already recorded above, no new review activity)

## PR #117: feat: M32 record spread — 差分更新構文 (#97) — 2026-07-06 merged

(no actionable patterns — hook ran on a non-merge command (M33 scratchpad での enum `Map` shadow ケース `enum_map.kei`(exit=0)と match-arm 内 `Map.empty()` の KEI-E2012 ケース `match_e2012.kei`(exit=1)の `kei check --json` 実行); latest merged PR is still #117, whose 2 lessons (enum variant spread silent corruption / parser recovery copy-paste drift) are already recorded above — no new review activity)

## PR #117: feat: M32 record spread — 差分更新構文 (#97) — 2026-07-06 merged

(no actionable patterns — hook ran on a non-merge command (M33 scratchpad での `t8.kei`: `let n: Int = Map.empty().size` の `kei check` 実行(exit=0)と spec §7.3 の E2012 文言確認); latest merged PR is still #117 with 2 inline comments / 0 discussion comments / 1 review — its 2 lessons (enum variant spread silent corruption / parser recovery copy-paste drift) are already recorded above, no new review activity)

## PR #117: feat: M32 record spread — 差分更新構文 (#97) — 2026-07-06 merged

(no actionable patterns — hook ran on a non-merge command (M33 scratchpad での `t9.kei`: ユーザー定義 enum `Map` vs emit rewrite テスト、KEI-E2006 「type 'Map' takes 2 type argument(s)」で check 失敗し transpile 未生成); latest merged PR is still #117 with 2 inline comments / 0 discussion comments / 1 review — its 2 lessons (enum variant spread silent corruption / parser recovery copy-paste drift) are already recorded above, no new review activity)

## PR #117: feat: M32 record spread — 差分更新構文 (#97) — 2026-07-06 merged

(no actionable patterns — hook ran on a non-merge command (worktree main の check.rs での `expected_ty` grep、hit 0 件); latest merged PR is still #117 with 2 inline comments / 0 discussion comments / 1 review — its 2 lessons (enum variant spread silent corruption / parser recovery copy-paste drift) are already recorded above, no new review activity)

## PR #117: feat: M32 record spread — 差分更新構文 (#97) — 2026-07-06 merged

(no actionable patterns — hook ran on a non-merge command (M33 scratchpad での `two_empty.kei`: 2-arm match の両腕 `Map.empty()` ケースの `kei check --json` 実行、診断 + `false_` rename suggestion を出力); latest merged PR is still #117, whose 2 lessons (enum variant spread silent corruption / parser recovery copy-paste drift) are already recorded above — no new review activity)

## PR #117: feat: M32 record spread — 差分更新構文 (#97) — 2026-07-06 merged

(no actionable patterns — hook ran on a non-merge command (M33 scratchpad での `nested_leak.kei`: fold 内 `Map.empty()` と receiver-position `Map.empty().size` の `kei check --json` 実行、diagnostics 0 件・exit=0); latest merged PR is still #117 with 2 inline comments / 0 discussion comments / 1 review — its 2 lessons (enum variant spread silent corruption / parser recovery copy-paste drift) are already recorded above, no new review activity)

## PR #117: feat: M32 record spread — 差分更新構文 (#97) — 2026-07-06 merged

(no actionable patterns — hook ran on a non-merge command (M33 scratchpad での `shadow_map.kei`: ユーザー定義 enum `Map` の match テスト、`let m = Map.empty();` の `;` が KEI-E0001 "unexpected character ';'" で check 失敗 exit=1); latest merged PR is still #117 with 2 inline comments / 0 discussion comments / 1 review — its 2 lessons (enum variant spread silent corruption / parser recovery copy-paste drift) are already recorded above, no new review activity)

## PR #117: feat: M32 record spread — 差分更新構文 (#97) — 2026-07-06 merged

(no actionable patterns — hook ran on a non-merge command (M33 scratchpad での `leak.kei` チェック: `fn` / `;` を使った Rust 風構文が KEI-E0101 "expected a declaration ... found identifier 'fn'" と KEI-E0001 "unexpected character ';'" で check 失敗 exit=1、Kei は `func` + セミコロンなし); latest merged PR is still #117 — its 2 lessons (enum variant spread silent corruption / parser recovery copy-paste drift) are already recorded above, no new review activity since)

## PR #117: feat: M32 record spread — 差分更新構文 (#97) — 2026-07-06 merged

(no actionable patterns — hook ran on a non-merge command (M33 scratchpad での `two_empty.kei`: Option scrutinee の 2-arm match で両腕 `Map.empty()` の `kei check --json` 実行、KEI-E2012 「'Map.empty()' requires a type annotation」を line 6 で報告); latest merged PR is still #117 with 2 inline comments / 0 discussion comments / 1 review — its 2 lessons (enum variant spread silent corruption / parser recovery copy-paste drift) are already recorded above, no new review activity)

## PR #117: feat: M32 record spread — 差分更新構文 (#97) — 2026-07-06 merged

(no actionable patterns — hook ran on a non-merge command (M33 scratchpad での `shadow_map.kei`: ユーザー定義 enum `Map` の match テスト、`;` を除いた版で `kei check --json` が diagnostics 0 件・exit=0); latest merged PR is still #117 with 2 inline comments / 0 discussion comments / 1 review — its 2 lessons (enum variant spread silent corruption / parser recovery copy-paste drift) are already recorded above, no new review activity)

## PR #117: feat: M32 record spread — 差分更新構文 (#97) — 2026-07-06 merged

(no actionable patterns — hook ran on a non-merge command (M33 scratchpad での cross-module テストプロジェクト作成: `proj/lib/types.kei` の `record Cache { entries: Map<Bool, Int> }` と `proj/app/main.kei` の `import lib.types { Cache }` + `c.entries.get(true)`、ファイル作成のみで check 未実行); latest merged PR is still #117 with 2 inline comments / 0 discussion comments / 1 review — its 2 lessons (enum variant spread silent corruption / parser recovery copy-paste drift) are already recorded above, no new review activity)

## PR #117: feat: M32 record spread — 差分更新構文 (#97) — 2026-07-06 merged

(no actionable patterns — hook ran on a non-merge command (M33 scratchpad での `cargo build -p kei_cli --quiet`、build-exit:0); latest merged PR is still #117 with 2 inline comments / 0 discussion comments / 1 review — its 2 lessons (enum variant spread silent corruption / parser recovery copy-paste drift) are already recorded above, no new review activity)

## PR #117: feat: M32 record spread — 差分更新構文 (#97) — 2026-07-06 merged

(no actionable patterns — hook ran on a non-merge command (M33 scratchpad での cross-module チェック実行: `proj/lib/types.kei` は `Map<Bool, Int>` が KEI-E2011 "Map key type must be 'Int', 'String', or a tagged type over them; found 'Bool'" で exit=1、一方 importer 側 `proj/app/main.kei` は exit=0 — 定義モジュールのエラーが import 経由で伝播していない点は dogfood 側の観察事項); latest merged PR is still #117 — its 2 lessons (enum variant spread silent corruption / parser recovery copy-paste drift) are already recorded above, no new review activity since)

## PR #117: feat: M32 record spread — 差分更新構文 (#97) — 2026-07-06 merged

(no actionable patterns — hook ran on a non-merge command (M33 scratchpad での `kei build $SP/proj` 全体ビルド: `lib/types.kei` の `Map<Bool, Int>` が KEI-E2011 で報告され "1 error(s) in 1 file(s); no output written"・dist 未生成、ただし build-exit:0 と表示された点は `head` パイプにより exit code が head 側のものになった観察上の注意); latest merged PR is still #117 with 2 inline comments / 0 discussion comments / 1 review — its 2 lessons (enum variant spread silent corruption / parser recovery copy-paste drift) are already recorded above, no new review activity since)

## PR #118: feat: M33 Map<K, V> 段階1 (#95) — 2026-07-06 merged

- **Pattern**: expected_ty save/restore 規約の非対称
  **Source**: A-1ro inline (crates/kei_check/src/check.rs:2556)
  **Lesson**: 新しい infer 経路(map_method 等)で引数を infer するときは、`infer_call` と同じ `expected_ty` の set/save-restore を必ず適用する — 引数の宣言型が既知なら `expected_ty` を立て(false E2012 防止)、経路冒頭で外側の stale な `expected_ty` をクリアする(誤診断 E2001 防止)。struct フィールドの規約コメント通り「対象式の infer 直後に None へ戻す」を全経路で守ること。
- **Pattern**: checker/emit の権威情報乖離
  **Source**: A-1ro inline (crates/kei_emit/src/emit.rs)
  **Lesson**: 検査器が名前衝突ガード(lookup_scope / env.kinds 優先)付きで解決する構文を emit 側で純粋な構文一致で書き換えてはいけない — 検査器の判定結果を span 集合(map_op_spans 方式)で emit に渡すか、衝突する名前を E で予約して両者の前提を揃える(M9 で List の構文ヒューリスティックを排した方針を踏襲)。
- **Pattern**: spec と実装の乖離は実装側を直す
  **Source**: general discussion (A-1ro 承認記録)
  **Lesson**: spec 本文(§7.3 期待型の 3 位置限定など)とチェッカー実装が乖離したら、レビューで承認された spec を正としマージ前に実装側を spec に揃える修正コミットを積む。
