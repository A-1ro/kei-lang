# HANDOFF.md candidates

`gh pr merge` 後の Sonnet hook が自動追記する候補集。HANDOFF.md に昇格
させたいエントリは人間レビューを経て本体に移し、ここから削除する。

## PR #71: chore: add Claude Code automation skills and hooks — 2026-06-27

> **Note**: 本 PR のマージで post-merge-handoff agent が初回発火したが、子セッションの
> permission(don't-ask mode で書き込み deny)により本ファイルへの追記に失敗した。
> 以下 4 候補は agent の最終応答(blocking error report)から手動で復元したもの。
> 同 PR #72 で permission allow を追加し、次回以降は自動追記される。

### Candidate: post-merge agent は type:command + claude --print ではなく type:agent を使う
**Why this matters for HANDOFF.md**: 外部 CLI (`claude -p`) が将来別課金になったときに自己改善ループが無効化されないようにする設計判断。
**Draft entry**:
> Hooks から Sonnet サブエージェントを呼ぶときは Claude Code 内蔵の `type: agent` を使う(Workflow と同じプール)。`type: command` で `claude --print` を呼ぶ案もあるが、CLI の課金体系が将来変わったときに hooks が無効化されるリスクがあるため避ける。長い prompt は別ファイル(`.claude/hooks/*.prompt.md`)に切り出し、agent に Read させる戦略。

### Candidate: pre-commit-ci.sh は cargo test 後に e2e package-lock.json を working tree のみ復元する
**Why this matters for HANDOFF.md**: `cargo test --workspace` が `tests/cli/projects/app/package-lock.json` と `tests/e2e/package-lock.json` を変更する副作用への暗黙対処を明文化する。
**Draft entry**:
> e2e テスト(`tests/e2e/`, `tests/cli/projects/app/`)は npm/npx を呼ぶ過程で lockfile を変更することがある。`cargo test --workspace` の後に `git status` を見ると意図せぬ差分が出る。pre-commit-ci hook は `git checkout --` で **working tree のみ** 復元する(staged 状態は触らない)。意図して lockfile を更新したい場合は事前に staging する規約。

### Candidate: .claude/settings.json はチェックイン(`.local.json` ではない)
**Why this matters for HANDOFF.md**: CLAUDE.md 不変条件「fmt/clippy/test 全パスが完了条件」と整合させるためのプロジェクト規律。
**Draft entry**:
> `.claude/settings.json` をチェックイン(`.gitignore` 対象の `.local.json` ではない)するのは、CLAUDE.md の「fmt/clippy/test 全パスが Milestone 完了条件」を Claude Code 経由の commit すべてに強制するため。これにより別マシン/将来の自分/共同編集者にも同じ品質ゲートが効く。「自分専用にしたい」場合だけ `.local.json` に分離する余地はあるが、本リポジトリではプロジェクト規律として共有版を採用。

### Candidate: kei-dogfood は plugin SKILL.md と MCP の **両方** 接続が必須
**Why this matters for HANDOFF.md**: 取説経由は 2 つあって、片方だけだとドッグフードが成立しないという設計上の前提を明示する。
**Draft entry**:
> Kei の「取説経由の正当な推論経路」は 2 つ — (a) plugin の `skills/kei/SKILL.md`(取説の入口)と (b) MCP の 4 ツール(対話的な検索・検証)。kei-dogfood スキルは両方の接続を必須条件として要求する。SKILL.md だけだと検査できず、MCP だけだとサブエージェントが「何があるか」を知らないまま走る。どちらが欠けてもドッグフードの結果が無意味になる。

## PR #72: fix(hooks): grant dev-notes write permission and recover PR #71 loop — 2026-06-27

### Candidate: type:agent 子セッションは親の permission を継承しない — 書き込みパスを明示 allow する必要がある
**Why this matters for HANDOFF.md**: hook が「動いているのに何も書かれない」状態になる最大の落とし穴であり、将来 hook を追加するたびに踏むリスクがある。
**Draft entry** (lift verbatim if approved):
> `type: agent` hook で起動する子セッションは、親セッションの permission を一切継承しない。don't-ask mode で Edit / Write / Bash の書き込みが暗黙 deny される。Hook に書き込み操作をさせる場合は `.claude/settings.json` の `permissions.allow` に対象パスを明示する(例: `"Edit(docs/dev-notes/**)"`, `"Write(docs/dev-notes/**)"`)。この設定が抜けていると hook は **静かに発火するが何も書かれない** 状態になり、デバッグが困難。blocking error は親セッションのトランスクリプトには届くが、通常の操作では気付きにくい。

### Candidate: hook 用の permissions.allow は最小スコープで付与する
**Why this matters for HANDOFF.md**: 将来 hook パスが増えるたびに「とりあえず `Bash(*)`」で広げようとする誘惑があるが、それは品質ゲートの意味を損なう。
**Draft entry** (lift verbatim if approved):
> Hook が書き込みを必要とするパスには **最小スコープ** の permission を付与する方針。例えば post-merge agent が `docs/dev-notes/` に書き込むなら `"Edit(docs/dev-notes/**)"` と `"Write(docs/dev-notes/**)"` だけを許可し、`Bash(*)`(全 Bash 許可)や `Edit(*)`(全編集許可)には広げない。settings.json はチェックイン対象なので、広い permission を入れると全共同編集者のセッションに影響する。

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

## PR #72 (re-check / PostToolUse audit session): kei-invariant-auditor M19 監査 — 2026-06-27

> **Note**: このセクションは `gh pr merge` ではなく `kei-invariant-auditor` セッション内の
> `PostToolUse`(Bash: `git diff --stat $(git merge-base main HEAD)..HEAD`)で発火した
> post-merge-handoff hook によって追記された。マージ対象 PR は未確定(M19 / #54 が WIP)。
> 最新マージ済み PR は #72(既記録)。以下は監査セッションが観察した設計判断の候補。

### Candidate: コメントは AST ノードに持たせず `ParseResult.comments` 副チャネルに退避する
**Why this matters for HANDOFF.md**: パーサ・フォーマッタ間の責務分離の根拠を知らないと、将来「なぜ AST にコメントがないのか」という疑問に誤った答えを出しやすい。
**Draft entry** (lift verbatim if approved):
> M19 以降、`//` 行コメントは `Comment` トークンとして採取されるが、AST ノードには **一切持たせない**。代わりに `ParseResult.comments` にソース順で並べる副チャネルを採用。理由: (1) コメントは文法上「どの AST ノードに属するか」が一意に定まらない(前の文末か次の文の前か)。(2) proptest や codegen などコメントに関心のない消費者が AST を使う経路では不要なデータを持ち込まない。(3) フォーマッタは行番号ベースで leading / trailing を自力で再構築できるため副チャネルで十分。新しくコメントを処理するコードを書くときは `ParseResult.comments` を参照し、AST ノードを拡張しようとしないこと。

### Candidate: `format_module` は意図的にコメントを失う(proptest 用純粋経路)
**Why this matters for HANDOFF.md**: `format_module` と `format_source` の使い分けを知らないと「コメントが消えるバグ」と誤解される。
**Draft entry** (lift verbatim if approved):
> `kei_fmt` には 2 つの公開 API がある。`format_module(&Module) -> String` はコメントを **意図的に失う**。proptest / codegen など純粋な AST 入力経路向けで、コメントを引数に取らない設計。`format_source(&str) -> Result<String, _>` は内部でパースし `ParseResult.comments` を使ってコメントを保持する。CLI の `kei fmt` は後者を経由する。`format_module` を使って「コメントが消えた」という報告があっても仕様どおりなので修正しないこと。

### Candidate: フォーマッタの冪等条件は M19 以降も変わらない — コメントは位置が変わっても内容は保持
**Why this matters for HANDOFF.md**: コメント付きソースに対する冪等性の定義が明文化されていないと、将来のテスト設計が曖昧になる。
**Draft entry** (lift verbatim if approved):
> `kei_fmt` の冪等条件 `fmt(fmt(x)) == fmt(x)` は M19 以降もコメント付きソースで成立する必要がある。ただし「コメントが元のコラム位置に完全復元される」保証はない。leading コメントはインデントが揃え直され、trailing コメントは 1 スペースに正規化される。コメントの **テキスト内容** は失われないが、**位置** は整形後の正規形に変わる。引数並びや式中間のコメントは v0.4 では「次のアンカーノードの leading」に寄せられる(完全な位置復元は将来の拡張)。これを踏まえて golden test は整形後の位置で expected を書くこと。

## PR #79: chore(hooks): auto-run kei-code-review on gh pr create — 2026-06-28

### Candidate: auto-fix の対象は `CONFIRMED` かつ `kei-invariants` / `correctness` のみ — `pitfalls` / `cleanup` / `altitude` は人間判断に委ねる
**Why this matters for HANDOFF.md**: auto-fix ループがどの findings を自動適用してよいかの判断基準が明文化されていないと、将来 hook を改修した際に境界が曖昧になりリグレッションを招く。
**Draft entry** (lift verbatim if approved):
> `post-pr-create-review` hook の自動修正フィルタは `verdict == "CONFIRMED"` かつ `angle ∈ {kei-invariants, correctness}` の findings のみを対象にする。`PLAUSIBLE` は絶対に自動適用しない。`pitfalls`・`cleanup`・`altitude` の角度は主観・文脈依存が大きく機械判断に向かないため除外。この設計により「確実に壊れている箇所だけを直し、スタイルや改善提案は inline comment のみ」という分離が保たれる。

### Candidate: auto-fix で `git add -A` / `git add .` を禁止する理由はあらかじめ hook 本体に明記
**Why this matters for HANDOFF.md**: PR #71 の dev-notes 教訓「`cargo test` 後に `git add -A` すると e2e lockfile drift を拾う」が hook の hard rule に直結しているが、その因果を知らないと将来の改修者が規則の意図を誤解する。
**Draft entry** (lift verbatim if approved):
> hook の自動コミット処理で `git add -A` / `git add .` / `git add :/` を禁じている理由: `cargo test --workspace` は `tests/e2e/` と `tests/cli/projects/app/` の `package-lock.json` を副作用で変更することがあり、`git add -A` するとそのドリフトをコミットに混入させてしまう。auto-fix hook は `git add <file1> <file2> ...` と **変更した特定ファイルのみ** をステージする。この規則は `pre-commit-ci.sh` の lockfile 復元処理と対になっている(pre-commit 側は working tree を復元するが staged は触らない)。

### Candidate: post-pr-create-review hook のスキップ条件(draft / バージョンバンプ / dependabot)を hook prompt 本体で定義する理由
**Why this matters for HANDOFF.md**: hook が「なぜ発火したのに何もしなかったのか」が外から見えにくいため、skip 判断ロジックの所在を明文化しておく必要がある。
**Draft entry** (lift verbatim if approved):
> `post-pr-create-review` は以下の PR を明示的にスキップする: (1) `isDraft == true`(レビュー準備未完了)、(2) タイトルが `^chore: bump version` で始まるリリースバンプ PR(機械的変更なので review 不要)、(3) タイトルが `chore(deps)` で始まるか `author.login == "dependabot[bot]"`(依存更新 PR)。スキップ判断は hook prompt ファイル(`.claude/hooks/post-pr-create-review.prompt.md`)内に記述され、hook の final reply に `"skipped: <reason>"` が出力される。hook log を確認するときはこの文字列を探す。

### Candidate: 人間レビュー必須サーフェスのリストは hook と HANDOFF.md で共有されるべき
**Why this matters for HANDOFF.md**: `spec/`・`tests/golden/`・`HANDOFF.md`・`.claude/settings.json` など「auto-fix の対象外とする」ファイル群のリストが hook prompt にしか存在せず、HANDOFF.md 読者には見えていない。
**Draft entry** (lift verbatim if approved):
> auto-fix ループが絶対に書き換えない「人間レビュー必須サーフェス」: `spec/`、`tests/golden/`、`.github/`、`.claude/settings.json`、`.claude/workflows/`、`CLAUDE.md`、`ARCHITECTURE.md`、`HANDOFF.md`、`Cargo.lock`。CONFIRMED findings がこれらを指していても inline comment のみ投稿し、Edit は行わない。将来 hook を改修する際もこのリストを hook prompt と HANDOFF.md で同期すること。

## PR #81: feat(skills): kei-dogfood — auto-file next-version Issues from feedback — 2026-06-28

### Candidate: kei-dogfood Step 4 の自動投稿禁止は OS 権限プロンプトを二段目防壁として利用する設計
**Why this matters for HANDOFF.md**: LLM が hard rule を violation しようとした場合の防衛戦略として、`permissions.allow` に意図的にエントリを追加しない設計判断が含まれている。
**Draft entry** (lift verbatim if approved):
> kei-dogfood Step 4 の Issue 化では `gh issue create` / `gh issue comment` を **承認ゲートを通過するまで絶対に実行しない** hard rule がある。SKILL.md にルールを書くだけでなく、`.claude/settings.json` の `permissions.allow` に `Bash(gh issue create:*)` / `Bash(gh issue comment:*)` を **意図的に追加していない** ことで OS レベルの permission prompt を二段目の防壁にしている。承認済みユーザーは prompt に `y` を返すだけで済むが、未承認の violation は OS 段で止まる。将来 kei-dogfood の権限拡張を検討する際はこの二段防衛の意図を壊さないこと。

### Candidate: `gh issue list --milestone` はタイトル文字列を直接受け付ける(number 変換不要)
**Why this matters for HANDOFF.md**: GitHub CLI の `--milestone` が title を受け付けることは公式ドキュメントに目立たない形でしか記載されておらず、将来の dedup 処理実装者が「milestone number を先に取得しなければならない」と誤解するリスクがある。
**Draft entry** (lift verbatim if approved):
> `gh issue list --milestone <X>` は milestone の **title**（例: `v0.5`）を直接受け付ける。`gh api repos/.../milestones` で number を引いてから渡す必要はない。kei-dogfood Step 4-b の dedup 処理はこの挙動を前提にしている。将来 GitHub CLI のバージョンが変わって挙動が変わった場合は Step 4-b を修正する必要がある。

### Candidate: `gh issue create` の `--label` は複数フラグを繰り返す — カンマ区切りは単一ラベル扱い
**Why this matters for HANDOFF.md**: カンマ区切りで複数ラベルを指定できると思い込む実装ミスが起きやすく、Issue が意図したラベルなしで立つ。
**Draft entry** (lift verbatim if approved):
> `gh issue create` でラベルを複数付けるときは `--label <l1> --label <l2>` のように **フラグを繰り返す**。`--label "dogfood,severity:high"` のようにカンマ区切りで 1 フラグに渡すと単一ラベル扱いになり、カンマを含むラベル名で検索されて失敗するか、意図しないラベルが付く。kei-dogfood Step 4-d はこの挙動を前提として設計されている。

### Candidate: Step 4 の gh 呼び出しは non-zero exit で即 halt-and-report する — 部分成功より安全側に倒す
**Why this matters for HANDOFF.md**: "半分だけ立った Issue" を後で整理するコストが高いため、all-or-nothing ではなく halt-at-first-error を選んだ設計判断。
**Draft entry** (lift verbatim if approved):
> kei-dogfood Step 4-d の `gh issue create` / `gh issue comment` は、**いずれかの呼び出しが non-zero exit したら以後の処理を即停止**してユーザーに報告する。milestone / labels 不存在、rate limit、auth 切れ、network、`--label` typo、HTTP 5xx など要因を問わず一律 halt-and-report。「部分的に Issue が立って残りはエラー」の状態はユーザーに後始末を押し付けるため避ける。承認後に失敗したときは全候補を再表示してから再実行してもらう設計を想定。

## PR #82: chore(skills): plan-then-delegate を実装タスクで常時発火に緩める — 2026-06-28

### Candidate: plan-then-delegate のトリガを「明示キーワード」から「コード編集を伴う指示全般」に拡大した理由
**Why this matters for HANDOFF.md**: 「なぜ skill description がこんなに広いのか」と感じた将来の改修者が、誤って以前の narrow trigger に戻す変更を入れないよう意図を明文化しておく必要がある。
**Draft entry** (lift verbatim if approved):
> `plan-then-delegate` の発火トリガは当初「sonnet に任せる」「ハンドオフ」などの明示的な委譲キーワード中心だった。実装タスクでも素通りされることが多く、二段委譲の恩恵(Opus の context 節約・Sonnet の速さ活用)を受けられないケースが頻発したため、「コード編集を伴う指示が来たら原則として常に発火」する設計に変更した。代表トリガ語: 実装 / 修正 / fix / 追加 / リファクタ / レビュー対応。除外は探索的タスクと golden/spec 判断が頻発する作業の 2 ケースのみに絞り、「1 ファイル数行だから委譲オーバーヘッドが大きい」という除外理由は撤廃した。

### Candidate: 「対応」ではなく「レビュー対応」に限定したトリガ絞り込みの経緯
**Why this matters for HANDOFF.md**: 「対応」という動詞は日本語として汎用的すぎ、コード編集を意図しない文脈(「質問に対応して」「エラーに対応した設計案を提示して」)でも skill が誤発火するリスクがあることを知らないと、将来ふたたび「対応」に戻す変更が入りうる。
**Draft entry** (lift verbatim if approved):
> `plan-then-delegate` のトリガ語として「対応」を追加したあと、コードレビュー指摘(PR #82 / pitfalls 角度)により「レビュー対応」に限定した。理由: 「対応」は「質問に対応して」「エラーに対応した設計案を提示して」のようにコード編集を伴わない文脈でも頻出し、skill が誤発火するリスクが高い。複合語「レビュー対応」に絞ることでコードレビューコメントの修正という特定用途だけをカバーし、「実装」「修正」「fix」「追加」「リファクタ」の既存トリガで残りのカバレッジを担保している。

## PR #83 (PostToolUse audit — old_counter correctness): feat: v0.4 remaining — M24 stock e2e + M25 lambdas + M26 Money notice — 2026-06-28

> **Note**: このセクションは kei-code-review verifier セッション内の `PostToolUse`
> (Bash: `grep -n "old_counter\|old\$\|kei\$old\|\"old\"" .../crates/kei_emit/src/emit.rs | head -40`)
> で発火した post-merge-handoff hook によって追記された。PR #83 は現時点で OPEN。
> 最新マージ済み PR は #82(上記セクションに記録)。以下は verifier が `old_counter` /
> `kei$old$N` の実装を確認した際に観察した設計判断の候補。

### Candidate: `forbid_old_capturing_lambda_param` の `refs_any` ガードは lambda param を参照しない `old(...)` を検出できない — emit の `collect_old_exprs` 停止と非対称になる危険

**Why this matters for HANDOFF.md**: `refs_any` が false を返しても `emit_call` は lambda body 内で `old(...)` を見て `old_counter` を進めるため、`kei$old$N` への undeclared 参照が TS に吐き出されて実行時 ReferenceError になる。「check が通った = emit が安全」という前提が崩れるケースの存在を知らないと、将来の実装者が `collect_old_exprs` の lambda 停止を「防御的すぎる」として外してしまいやすい。
**Draft entry** (lift verbatim if approved):
> `check.rs` の `forbid_old_capturing_lambda_param` は `refs_any(expr, lambda_params)` で **lambda param を参照する** `old(e)` だけを KEI-E4002 で弾く。しかし `old(Database.maxLimit())` や `old(42)` のように lambda param を参照しない `old(...)` は `refs_any` が false を返すため check をすり抜ける。一方 `emit.rs` の `collect_old_exprs` は lambda 境界で walk を **無条件停止** するため、これらの式は関数入口の `const kei$old$N = ...` に bind されない。それでも `emit_call`(emit.rs:1028–1032、`old_counter` 0→1 に進める)は lambda body 内で `name == "old"` を見て `kei$old$0` をインライン参照する。結果として TS 実行時に `ReferenceError: kei$old$0 is not defined` が発生する。修正方針: `forbid_old_capturing_lambda_param` の対象を「lambda param を参照するかどうかに依らず、lambda body 内の全 `old(...)` 呼び出し」に広げる。あるいは `collect_old_exprs` の停止を緩めて lambda param を参照しない `old(e)` は lambda body 内でも lift する。いずれの方針でも `err_contract_old_lambda_param.expected.json` golden を更新し E4002 のカバレッジを広げること。

## PR #83 (PostToolUse audit — runtest.mjs ReferenceError 実証): feat: v0.4 remaining — M24 stock e2e + M25 lambdas + M26 Money notice — 2026-06-28

> **Note**: このセクションは同セッション(73569efa)内の別 `PostToolUse`
> (Bash: `node runtest.mjs` → `threw: ReferenceError kei$old$0 is not defined`)
> で発火した post-merge-handoff hook によって追記された。直前のセクション
> (grep audit)と同一 PR #83 レビューセッションの続き。最新マージ済み PR は #82。
> 以下は verifier が実際に kei_emit でトランスパイルした TS 出力と
> Node.js 実行で ReferenceError を確認した際の補足設計判断。

### Candidate: kei_emit が生成する TS の IIFE 本体と ensures チェック部の `old` 参照が分裂する構造的問題

**Why this matters for HANDOFF.md**: 実トランスパイル出力を見ると IIFE 本体(`const kei$result = ...`)では `Database.maxLimit()` を直接呼ぶのに対し、ensures チェック部(`xs.every((p) => p < kei$old$0)`)は存在しない `kei$old$0` を参照するという **二重真実** 状態が生まれる。どちらの半分だけ見ても問題に気付けないため、kei_emit の出力を通して両箇所を並べて見る習慣が必要。
**Draft entry** (lift verbatim if approved):
> `kei_emit` が `func f(xs) ensures xs.all(p => p < old(Database.maxLimit()))` をトランスパイルすると、以下のような TS が生成される:
> ```ts
> export function allBelowLimit(xs: readonly number[]): boolean {
>   const kei$result = ((): boolean => {
>     return xs.every((p) => p < Database.maxLimit());  // ← IIFE 本体: old なし
>   })();
>   if (!(kei$result === xs.every((p) => p < kei$old$0))) {  // ← ensures: kei$old$0 が未宣言!
>     throw new KeiContractViolation({ ... });
>   }
>   return kei$result;
> }
> ```
> IIFE 本体は `collect_old_exprs` が lambda 境界で停止するため `Database.maxLimit()` を lift せず直接呼び出す。ensures チェック部は `emit_call` が `old_counter` を進めて `kei$old$0` を参照するが、`const kei$old$0 = ...` は関数先頭に存在しない。Node.js で実行すると `ReferenceError: kei$old$0 is not defined` が throw される。PBT(`--generative`)はコンビネータ引数の lambda のみ eval するため ensures のこの経路を通らず、generative グリーンのまま本番 ReferenceError になる。

## PR #83 (PostToolUse audit — emit.rs grep / fix design): feat: v0.4 remaining — M24 stock e2e + M25 lambdas + M26 Money notice — 2026-06-28

> **Note**: このセクションは同セッション(73569efa)内の別 `PostToolUse`
> (Bash: `grep -n "old_counter\|name == \"old\"\|kei\\$old" .../emit.rs | head -50`
> → ugrep escape error で失敗)で発火した post-merge-handoff hook によって追記された。
> 直前 2 セクション(old_counter correctness / runtest.mjs 実証)と同一 PR #83 レビュー
> セッションの続き。grep は失敗したが、同セッションで確認した PR #83 の diff から
> 「N3 修正がどの選択肢を採り、なぜか」という設計判断が読み取れるため候補として記録する。

### Candidate: N3 修正は emit 側を緩めず check 側を強化する方針を選択した理由

**Why this matters for HANDOFF.md**: 前 2 セクションで記録した `old(...)` + lambda の ReferenceError バグに対し、修正方針は「`collect_old_exprs` の停止条件を緩める(emit 側)」ではなく「`forbid_old_inside_lambda_body` で check 側を強化する(check 側)」を選んだ。この選択の根拠が明文化されていないと、将来の実装者が emit 側 の停止を「過防衛」と見て外してしまいやすい。
**Draft entry** (lift verbatim if approved):
> PR #83 (N3 / M25) で採用した修正方針: lambda body 内の `old(...)` は **check 側で一律 KEI-E4002 を出す**(`forbid_old_inside_lambda_body` 関数)。emit 側の `collect_old_exprs` lambda 境界停止は **そのまま維持** し、二段防御として残す。emit 側停止を緩める(「lambda param を参照しない `old(e)` は lift を許す」)案も検討されたが却下された。理由: (a) `old(e)` は「関数入口で 1 回評価」・lambda body は「呼び出しごとに評価」という **時相が根本的に噛み合わない** ため、emit 側でどう lift しても契約の意味論が崩れる。(b) check が通った入力で emit が壊れる二重真実状態そのものを排除する方が、将来の emit 実装者の認知負荷が低い。この二段防御(check 禁止 + emit 停止)を外す変更は、どちらの層が何を守っているかを理解してから行うこと。

### Candidate: `lambda_floor` の save/restore パターンはネストラムダ用であり `Option<usize>` は「深さ」ではなく「有無」を表す

**Why this matters for HANDOFF.md**: `lambda_floor: Option<usize>` を見ると「ネスト深度カウンタ」に見えるが実際は「現在ラムダ中かどうかのフラグ兼 scopes のインデックス」であり、深さはスコープスタックの長さで暗黙に表現される。この区別が明文化されていないと、将来の実装者が `lambda_floor` を `usize` のカウンタとして扱い誤ったスコープ境界を生成しやすい。
**Draft entry** (lift verbatim if approved):
> `FnChecker.lambda_floor: Option<usize>` は「現在コンビネータ引数ラムダの body 内にいるか」を示すフラグ兼スコープインデックス。`None` はラムダ外、`Some(i)` は `self.scopes[i..]` だけを `lookup_scope` の参照対象にすることで外側関数スコープのキャプチャを禁止する。**ネストの深さは表現しない**—内側ラムダの `check_combinator_lambda_arg` 呼び出し時に `prev_floor = self.lambda_floor` を退避し、新しい `Some(self.scopes.len() - 1)` をセットして返り際に復元する save/restore パターンを使う。これにより `fold(0, (acc, xs) => xs.fold(0, (a, x) => a + x))` のようなネストラムダでも各層が独立したキャプチャ禁止スコープを持てる。将来このフィールドを深さカウンタに転用しようとしないこと。

## PR #83 (PostToolUse audit — requires_old_lambda test fixture): feat: v0.4 remaining — M24 stock e2e + M25 lambdas + M26 Money notice — 2026-06-28

> **Note**: このセクションは同セッション(73569efa)内の `PostToolUse`
> (Bash: `cat > .../scratchpad/requires_old_lambda.kei` — `requires xs.all(p => old(p.qty) > 0)` の
> テストフィクスチャ作成)で発火した post-merge-handoff hook によって追記された。
> 直前 3 セクション(old_counter correctness / runtest.mjs 実証 / emit.rs grep)と同一
> PR #83 レビューセッションの続き。最新マージ済み PR は #82。
> 以下は `requires` 節での `old(...)` + lambda の二重エラー問題を実証するフィクスチャ作成時に
> 観察した設計判断の候補。

### Candidate: `requires` 節内の `old(...)` は「ensures 外で old を使った」エラーと「lambda 内で old を使った」エラーが同一スパンから二重に発火する

**Why this matters for HANDOFF.md**: `requires xs.all(p => old(p.qty) > 0)` という入力に対して、`forbid_old_inside_lambda_body` (Ensures/Requires 共通発火)と既存の「old は ensures 節のみ合法」チェックが同一スパンを二重に報告するため、ユーザーに重複した KEI-E4002 が届き、fix 提案(`let prev = old(seed)` を ensures 前に記述)が誤解を招く。requires mode では「old を使うな」が正解であり「old をラムダ外に出す」という fix は無意味。
**Draft entry** (lift verbatim if approved):
> `check.rs` の `forbid_old_inside_lambda_body` は `ContractMode::Requires` でも `ContractMode::Ensures` でも発火する設計になっている。しかし `requires` 節で `old(...)` を使うと、さらに既存の「old は ensures 節でのみ使用可能」チェックも同一スパンに KEI-E4002 を出すため、同一入力から **2 つの E4002 エラー** が重複して報告される。`requires xs.all(p => old(p.qty) > 0)` のような入力では: (1) 「old は ensures 節でのみ使用可能」— スパン `old(p.qty)` (2) 「lambda body 内で old は使用不可」— スパン `old(p.qty)` が重複する。fix 提案として表示される `let prev = old(seed); xs.all(...)` は ensures-mode 向けのテンプレートであり requires-mode では無意味な誤誘導になる。修正方針: `forbid_old_inside_lambda_body` は `ContractMode::Ensures` でのみ発火させる(requires では既存の old-in-requires エラーが上位で弾く)か、old-in-requires チェックを早期リターンにして lambda チェックを抑制する。golden `err_contract_old_lambda_nonparam.expected.json` の diagnostics 配列が 1 件か 2 件かでどちらの方針が採られたかを確認できる。

## PR #84: chore: bump version to 0.4.2 — 2026-06-29

(no design-decision candidates for this PR)

<!-- 判断根拠:
     PR #84 はバージョン文字列の機械的置換(Cargo.toml / Cargo.lock / plugin.json /
     marketplace.json / MCP golden 3 件 = 0.4.1 → 0.4.2)と、skills/kei/SKILL.md への
     M25/M26 ドキュメント追記が中心。

     SKILL.md の実質追加内容:
     (a) コンビネータ引数位置限定ラムダ(v0.4 / M25)— lambda body 内 old() 禁止 (KEI-E4002)・
         lambda param 名が TS 予約語衝突 (KEI-E2001)・let f = (lambda) は引き続き KEI-E2001
     (b) M26: spec §2.4 / §2.5 の新設と stock_direct.kei の追加

     これらはすべて PR #83 の post-merge セクション(上記に記録済み)で設計判断として
     詳細に捕捉済みのため、重複登録は不要と判断。

     MCP golden の version 文字列は env!("CARGO_PKG_VERSION") 由来のため
     workspace version 変更時に UPDATE_GOLDEN=1 cargo test -p kei_mcp 再生成が必要だが、
     これも PR #70 の判断根拠コメントで記録済み。

     runtime/ と editors/vscode は Cargo workspace version とは独立して管理される慣例も
     本 PR body に明記されており HANDOFF.md 昇格不要。
-->

## PR #84 (PostToolUse audit — issue state check): chore: bump version to 0.4.2 — 2026-06-29

> **Note**: このセクションはセッション 73569efa 内の `PostToolUse`
> (Bash: `for n in 54 55 56 57 58 59 60 61 62; do echo -n "#$n: "; gh issue view $n --json state --jq '.state'; done`)
> で発火した post-merge-handoff hook によって追記された。最新マージ済み PR は #84(上記に記録済み)。
> 以下はセッションが関連 Issue の状態を確認した際に観察した設計判断の候補。

### Candidate: M24 / M25 / M26 を実装した PR #83 がマージされてもその実装 Issue が自動クローズされない

**Why this matters for HANDOFF.md**: PR body に `#56` / `#59` / `#61` への言及があるにもかかわらず GitHub が Issue を自動クローズしなかった場合、ロードマップ進捗が実態と乖離しているように見える。自動クローズの条件(キーワード `Closes #N` or `Fixes #N`)を満たしていなかった可能性が高く、将来の PR テンプレート設計に影響する。
**Draft entry** (lift verbatim if approved):
> GitHub の Issue 自動クローズは PR body に `Closes #N` / `Fixes #N` / `Resolves #N` キーワードが含まれる場合のみ機能する。PR #83(feat: v0.4 remaining)は `#56` / `#59` / `#61` を参照したが、マージ後の Issue 状態確認(セッション 73569efa)で Issue #56(M24)・#59(M25)・#61(M26)が OPEN のままだった。マージで自動クローズさせたい Issue は PR body に `Closes #<N>` を明記する規律を徹底すること。また Milestone の Issue 完了状態はロードマップ(`docs/kei-roadmap-v0.4.md`)の `✅` 記号と GitHub Issue の状態の両方で確認することが必要(片方だけ見ると不一致が生じる)。

## PR #83: feat: v0.4 remaining — M24 stock e2e + M25 lambdas + M26 Money notice — 2026-06-28

> **Note**: 直前の 4 セクション(old_counter correctness / runtest.mjs 実証 / emit.rs grep / requires_old_lambda fixture)は `gh pr merge 83` **前** の code review セッション内 `PostToolUse` で追記されたもの。本セクションは `gh pr merge 83 --squash --delete-branch --auto` の完了を受けて発火したマージ後フック(セッション 73569efa)が追記した正式な post-merge 候補集。

### Candidate: M25 — lambda は「値」ではなく「コンビネータ引数位置の構文糖」として設計された(案 2 維持)
**Why this matters for HANDOFF.md**: `let f = (p => p.id)` が KEI-E2001 になる理由を知らないと、「型推論が足りないバグ」と誤解されやすい。第一級関数値を将来追加するときも、この決定との整合を意識する必要がある。
**Draft entry** (lift verbatim if approved):
> M25(#59 / v0.4)で追加したラムダ構文 `p => expr` / `(a, b) => expr` は、`List<T>` の `map`/`filter`/`fold`/`all`/`any` **引数位置でのみ** 合法な構文糖であり、値として保存・再利用することはできない。`let f = (p => p.id)` は依然として KEI-E2001(型不一致)になる。これは M9(spec §10)で合意した「案 2: 第一級関数値を導入しない」方針の継続。ラムダの `infer` arm が「コンビネータ引数以外で出現した場合のみ」到達するように設計されており、`check_combinator_fn_arg` がラムダを別経路で先処理してこの arm には降りない。将来第一級関数値を追加する場合は `infer` の Lambda arm を書き換え、`check_combinator_fn_arg` の分岐も見直すこと。

### Candidate: M25 — TS 予約語チェックをラムダパラメータの **check 段階**で弾く理由(emit 段ではなく)
**Why this matters for HANDOFF.md**: 「なぜ Kei 自体は予約していない `class` や `var` をラムダパラメータで弾くのか」が明文化されていないと、将来の実装者がこれを誤ったエラーとして削除しやすい。
**Draft entry** (lift verbatim if approved):
> lambda パラメータ名が TypeScript 予約語(`class`, `var`, `null`, `this`, `function`, `delete`, `typeof`, `let`, `await`, `async` 等)と衝突する場合、`check.rs` が check 段階で KEI-E2001 を出す([4] / M25)。Kei 自体は `class` を予約語と定義していないが、emit 後の `(class) => ...` は `tsc` が parse 不能になる。**emit 段で弾かない**のは、TS コンパイルエラーより明確な Kei レベルの診断を届けるため。検出単位は v0.4 では lambda パラメータのみ(将来 `let` / 関数パラメータ全般への拡張が議論されたが、スコープ外として延期)。予約語リストは ES2022 + TS strict mode を網羅した `is_ts_reserved_word()` ヘルパで管理する。

### Candidate: M25 — 0 引数 `() =>` はパーサが `Expr::Error` sentinel を返し check まで届かない
**Why this matters for HANDOFF.md**: `check_combinator_lambda_arg` に「0 引数ラムダが来ない前提」の `debug_assert` があり、その理由を知らないと将来の変更者が assert を外してしまいやすい。また 0 引数ラムダのエラーが golden にどのレイヤで記録されるかを誤解しやすい。
**Draft entry** (lift verbatim if approved):
> 0 引数ラムダ `() => expr` はパーサ段階で `KEI-E0101` を出し `Expr::Error` sentinel を返す(N0 / M25)。`Expr::Error` は下流の walker が no-op で扱うため、`check_combinator_lambda_arg` には 1 個以上のパラメータを持つ `Expr::Lambda` のみが届く。`check_combinator_lambda_arg` 先頭の `debug_assert!(!lparams.is_empty(), ...)` はこの前提を明示するガード。0 引数ラムダの golden は `tests/golden/syntax/err_lambda_zero_params.*` に記録されており、check レイヤの golden ではなく syntax レイヤに分類される。将来 lambda 引数の arity 検査を拡張するときは、0 引数 case のみ `Expr::Error` 経路であることを忘れないこと。

### Candidate: M24 — `extern query` はスタブで純粋観測子として実装し、`old()` との組み合わせで外部状態事後条件 e2e を可能にする
**Why this matters for HANDOFF.md**: `extern query` がなぜ `uses` エフェクトを持たないのか、またなぜそれが `old()` との組み合わせで機能するかを知らないと、将来の在庫/残高ドメインの e2e 設計が外部観測子を誤って `uses Database.Read` にしてしまう。
**Draft entry** (lift verbatim if approved):
> M24(#56)の在庫ドメイン e2e(`examples/contracts/stock_direct.kei`)は `extern query Database.quantityOf(product: ProductId) -> Int` を使う。`extern query` は副作用(uses)を持たない純粋観測子として定義され、`ensures Database.quantityOf(product) == old(Database.quantityOf(product)) - amount` のように `old(外部状態観測)` として事後条件に書ける。スタブ側(`tests/e2e/stubs/database.ts`)では状態を保持し `quantityOf` が読み取りのみ行う純粋関数として実装される。`extern query` を `uses` 付きで定義すると `ensures` 節内で副作用エラーが出るため、読み取り専用の外部状態にはかならず `extern query`(uses なし)を選ぶこと。反例 3 種(off-by-one / forgot / wrong-id)は `KeiContractViolation(clause: "ensures")` として runtime で検出される設計を確認済み。

### Candidate: M26 — `Money` / `core.money` は spec 上の架空型であり stdlib に実装されていない
**Why this matters for HANDOFF.md**: spec §2.1–§2.3 の例に登場する `Money` や `core.money` が実際には存在しないことを知らないと、実プロジェクトでこれをインポートしようとしてコンパイルエラーに悩む。
**Draft entry** (lift verbatim if approved):
> M26(#61 / v0.4)で `spec/kei-spec-v0.1.md §2.4` に明記: `Money` / `core.money` は spec §2.1–§2.3 の例で登場する **説明用の架空型・架空モジュール** であり、stdlib に実装されていない。実プロジェクトでは (a) `Int`(最小通貨単位)をそのまま使う、または (b) `type Money = Int tagged "Money"` を自前定義する。`Money.zero` のような静的メンバアクセスは Kei 構文にないため `Money(0)` で構築すること。`examples/contracts/withdraw.kei` と `examples/effects/transfer.kei` は架空 Money 例として残り、e2e は `tests/e2e/stubs/core/money.ts` の差し替えで動く — 実装プロジェクトのひな型としては使わないこと。固定小数点(`Decimal`)と `core.money` の実在化は v0.5+ で別途検討予定。

## PR #87: chore(deps): bump lsp-server from 0.7.9 to 0.8.0 — 2026-07-03 merged

(no design-decision candidates for this PR)

## PR #88: chore(deps): bump the npm-minor-patch group across 3 directories with 2 updates — 2026-07-03 merged

(no design-decision candidates for this PR)

## PR #102: docs: v0.5 ロードマップ + v1.0 到達戦略(Workers + Hono API) — 2026-07-03 merged

### Candidate: v1.0 受け入れ基準からの逆算がロードマップの最上位判断軸
**Why this matters for HANDOFF.md**: 今後の機能の取捨選択(何を v0.5 に入れ、何を後回しにするか)の「なぜ」が、この単一の受け入れ基準への逆算で決まっている。
**Draft entry** (lift verbatim if approved):
> v0.5 以降のロードマップは「v1.0 = Cloudflare Workers に Hono を使用した API としてデプロイできること」という単一の受け入れ基準から逆算して構成した(`docs/kei-roadmap-v0.5.md`)。欠落を blocker 層(async / npm import / HTTP 境界 → v0.6〜v0.8)・stdlib 層(v0.5)・検証経路層(v0.5)の3層に分けたのは、言語設計判断(🤝)を含むものを後ろに寄せ、既存機構の自然な拡張(`&&` / `List.contains` / 文字列 stdlib)を先頭に置くため。機能要望の優先度で迷ったら、この受け入れ基準に効くかどうかで判断する。

### Candidate: バージョンタグ運用(0.5.0 はロードマップ完了時、途中は 0.4.x)
**Why this matters for HANDOFF.md**: リリース bump 時に「なぜまだ 0.4.x なのか」を将来のコントリビュータが迷わないための運用上の不変条件。
**Draft entry** (lift verbatim if approved):
> ロードマップ v0.5 の Milestone が全て閉じた時点で初めて 0.5.0 をタグする。途中イテレーションで v0.5 向け機能がマージされても、それは 0.4.x のパッチ/プレリリースとして bump する(`docs/kei-roadmap-v0.5.md` の「バージョン運用」)。版番号はロードマップの完了状態を表す契約であり、機能の所属版とは切り離す。

## PR #103: feat: M28 論理積 && を追加 (#91) — merge 未完了 (2026-07-03 時点)

(no design-decision candidates for this PR — `gh pr merge 103` は base branch policy により失敗し、PR は OPEN のまま。実際のマージ時に hook が再実行され候補を追記する)

## PR #103: feat: M28 論理積 && を追加 (#91) — 2026-07-03 merged

(前回の hook 実行時は base branch policy でマージ失敗と記録したが、今回 `--admin` squash マージが成功。以下が候補)

### Candidate: contract_expr_text の優先順位はパーサ準拠(kei_emit の TS Prec と意図的に別物)
**Why this matters for HANDOFF.md**: 片方に合わせて「統一」すると suggested_contract が再パースでズレて KEI-E2001 になる、という非対称の理由がコードコメントだけでは埋もれやすい。
**Draft entry** (lift verbatim if approved):
> `contract_expr_text`(kei_check)の `bin_prec` は **Kei パーサと同じ単一比較階層**(`==` と `<` 等が同レベル・左結合)を使う。kei_emit の TS 用 `Prec`(JS 準拠で relational > equality)とは**意図的に異なる**。ここをパーサとズラすと `result == b < c` のようなテキストが再パースで `(result == b) < c` に化け、suggested_contract が適用不能(KEI-E2001)になる。「emit と check で優先順位表を統一する」リファクタは禁止。

### Candidate: PBT eval の `&&` / `||` / `implies` は短絡が意味論上の要件
**Why this matters for HANDOFF.md**: 短絡しない実装でもほとんどのテストは通るが、`b != 0 && a / b > 0` で b=0 が偽の trap 反例になる — 壊れ方が静かで気づきにくい。
**Draft entry** (lift verbatim if approved):
> kei_check/pbt の `eval_expr` は `&&` / `||` / `implies` を `eval_short_circuit` で短絡評価する。`eval_binary` にこれらを到達させてはいけない(到達不能アーム)。短絡を外すと `b != 0 && a / b > 0` のようなガード付き契約で b=0 入力が 0 除算 trap の偽反例になる。回帰テスト: `and_short_circuits_avoiding_division_trap`。

### Candidate: 単独 `&` は lexer エラーとして予約
**Why this matters for HANDOFF.md**: 将来 `&`(ビット演算や参照)を導入する余地を残すための意図的なエラーで、単なる未実装ではない。
**Draft entry** (lift verbatim if approved):
> lexer は `&&` のみ受理し、単独の `&` は明示的にエラーにする(黙って無視や `&&` への補正はしない)。将来の `&` 系構文のために表面を予約する意図。

## PR #105: chore: bump version to 0.4.3 — 2026-07-03 merged

(no design-decision candidates for this PR)

## Hook run 2026-07-04 — no new merged PR

PostToolUse hook が非 merge コマンド(M29 PR ブランチ上の `kei check`)で発火。
最新 merged PR は #105 で、本ファイルに記録済み(候補なし)のため新規追記なし。

## Hook run 2026-07-04 (2) — no new merged PR

PostToolUse hook が非 merge コマンド(scratchpad worktree `wt-m29` での
map+contains ワークアラウンド検証 `kei check` — worktree 不在で失敗)で発火。
最新 merged PR は依然 #105(記録済み・候補なし)のため新規候補なし。

## PR #106: feat: M29 List.contains を追加 (#92) — 2026-07-04 merged

### Candidate: contains は「等値比較可能なスカラー」限定(KEI-E2010 の再利用)
**Why this matters for HANDOFF.md**: `contains` の型制約が `==` と同一である理由と、合成型を独自エラーでなく KEI-E2010 で拒否する設計判断はコードからは自明でない。
**Draft entry** (lift verbatim if approved):
> `List<T>.contains(item)` は `T` が等値比較可能なスカラー(Int/String/Bool/tagged スカラー)のときだけ許可する。record/enum/List などの合成型は **`==` と同じコード KEI-E2010** で拒否し、`xs.any(e => ...)` への誘導 fix を出す。新しいエラーコードを作らないのは意図的:`contains` は意味的に「要素との `==`」なので、等値の制約とエラー体系をそのまま共有する。`contains` の制約を緩めるなら `==`(is_equatable)側と必ず同時に見直すこと。

### Candidate: contains → Array.prototype.includes が安全な理由(SameValueZero)
**Why this matters for HANDOFF.md**: emit が `.includes` に写せるのは Kei の等値対象がスカラーに限定されているからで、将来 Float や合成型を等値対象に加えると壊れる landmine。
**Draft entry** (lift verbatim if approved):
> TS emit では `xs.contains(item)` → `xs.includes(item)`。`Array.prototype.includes` は SameValueZero 比較(`NaN === NaN` 扱い、オブジェクトは参照比較)だが、Kei の等値対象が Int/String/Bool のスカラーに限られている現状では `===` と完全に一致するため安全。**等値対象を Float(NaN あり)や構造的等値の合成型に拡張する場合、この `.includes` 写像は成立しなくなる**ので、emit 側をランタイムヘルパー(`keiListGet` 方式)等に切り替える必要がある。

### Candidate: 組み込みメソッド追加時の追従チェックリスト
**Why this matters for HANDOFF.md**: PR #106 で List メソッド追加時に触る箇所が確定した。次にメソッドを足す人が漏らしやすい(did-you-mean 候補 3 箇所・pbt・MCP golden)。
**Draft entry** (lift verbatim if approved):
> List 組み込みメソッドを追加するときの追従箇所: (1) spec の コンビネータ表・§5.1・トランスパイル表、(2) `skills/kei/SKILL.md` のメソッド一覧、(3) `check.rs` の型検査 + **did-you-mean 候補リスト 3 箇所**、(4) `emit.rs` の写像、(5) `pbt.rs` の `eval_list_method`(bounded 検証が新メソッドを評価できないと contract 検証が落ちる)、(6) golden fixture 新規 + 既存 `err_collection_method` 系の候補リスト差分、(7) spec 本文を変えたら `tests/mcp/spec_*.response.json` の embedding golden 再生成。

## Hook run 2026-07-04 (3) — no new merged PR

PostToolUse hook が非 merge コマンド(scratchpad worktree `wt-m30` での
String.map lambda repro に対する `kei check --json` — `fn` 構文が KEI-E0101 で
parse エラーになる repro 実行)で発火。最新 merged PR は #106 で、本ファイルに
記録済み(候補 3 件登録済み)のため新規候補なし。

## Hook run 2026-07-04 (4) — no new merged PR

PostToolUse hook が非 merge コマンド(scratchpad `m30wt` での tagged-string
`+` 連結 repro — `ProductId + ProductId -> ProductId` が check を通過し
`a + b` がそのまま TS に emit される transpile 実行)で発火。
最新 merged PR は依然 #106(候補 3 件記録済み)のため新規候補なし。

---

PostToolUse hook が非 merge コマンド(scratchpad `wt-m30` での String.map
lambda repro — `s.map(c => c)` が KEI-E2002「no method 'map' on 'String'」と
KEI-E2001「lambdas are only allowed as arguments to List combinators」を
出す `kei check --json` 実行)で発火。
最新 merged PR は依然 #106(候補記録済み)のため新規候補なし。

## PR #108: feat: M30 文字列 stdlib 段階1 (#107) — 2026-07-04 merged

### Candidate: tagged String 同士の `+` 連結は意図的に KEI-E2001 で拒否(基底連結 → コンストラクタ再構築へ誘導)
**Why this matters for HANDOFF.md**: `ProductId + ProductId` が「同じ型同士なのになぜエラーか」という疑問に対し、tag の意味論保護という設計判断が根拠であることを明文化しないと、将来「同型なら許す」緩和が入りやすい。
**Draft entry** (lift verbatim if approved):
> `String + String` 連結(M30 / spec §2.6)は **素の String 同士のみ** 合法。tagged String(`type X = String tagged "X"`)同士、または tagged と素の String の連結は `+` では KEI-E2001 で拒否し、「基底 String で連結してから `TagName(base1 + base2)` で再構築せよ」という fix を出す。理由: tag は「この文字列は特定の意味領域に属する」という契約であり、連結結果が同じ tag の不変条件を満たす保証はない(例: `ProductId + ProductId` は有効な ProductId とは限らない)。再構築を強制することで「tag 付与は必ずコンストラクタ経由」という既存不変条件を保つ。判定は `Ty::is_stringy()`(tagged を再帰的に見る)+ `Tagged` matches ガードの組み合わせ。

### Candidate: `is_list_get` を `is_runtime_method(name, arity)` に一般化 — import 収集と emit の判定は必ず 1 か所で共有する
**Why this matters for HANDOFF.md**: ランタイムヘルパー行きメソッド(`get` → `keiListGet`、`toInt` → `keiStringToInt`)が増えるたびに、RuntimeUses(import 収集)と Emitter(呼び出し生成)の判定がズレると「呼び出しはあるが import が無い」TS が生成される。共有ヘルパーの存在理由を知らないと、次のヘルパー追加時に判定を二重実装しやすい。
**Draft entry** (lift verbatim if approved):
> emit のランタイムヘルパー写像(`xs.get(i)` → `keiListGet`、`s.toInt()` → `keiStringToInt`)は `is_runtime_method(callee, args, list_ops, name, arity)` という **単一の判定関数** を import 収集(`RuntimeUses`)と emit 本体の両方が呼ぶ。鍵はメソッド名トークンの位置(Call span は連鎖で衝突するため)+ arity。M30 で `is_list_get` から一般化された。新しいランタイムヘルパー行きメソッドを足すときは、この関数に名前と arity を渡す呼び出しを RuntimeUses と Emitter の **両方** に追加する(片方だけだと import 欠落 TS になる)。

### Candidate: `list_ops` 位置集合は M30 以降「List 専用」ではなく String/Int 組み込みメソッドの権威的位置も運ぶ(名前は歴史的)
**Why this matters for HANDOFF.md**: フィールド名 `list_ops` から「List のみ」と誤解し、String/Int メソッド追加時に別チャネルを新設したり、`list_ops` への insert を漏らして emit が構文判定にフォールバックする事故が起きやすい。
**Draft entry** (lift verbatim if approved):
> `FnChecker.list_ops`(および `collect_list_method_calls` 系)は M9 由来の「検査器が型推論で確定した組み込みメソッド呼び出し位置」の集合であり、M30 以降は List だけでなく **String(`toInt`)/ Int(`toString`)のメソッド位置も同じ集合に入る**。emit はこの権威的位置情報だけを根拠にメソッドを写す — 構文だけでレシーバ型を判別すると同名フィールドや外部呼び出し連鎖を誤写するため。tagged レシーバは `Ty::peel_tagged()` で基底型に落としてから `string_method` / `int_method` に振り分ける。新しい組み込みメソッドを足すときは該当 `*_method` 内で `ops.insert((line, col))` を忘れないこと(忘れると check は通るが emit が写像しない)。

### Candidate: `toInt()` を含む契約は PBT bounded 評価器が Option 未サポートのため `[runtime]` に留まる(意図的なフォールバック、issue #109)
**Why this matters for HANDOFF.md**: `--generative` で `toInt()` 入り ensures が `[generative]` に昇格しないのはバグではなく、bounded 評価器が Option 値を持たない設計上の制約であることを知らないと、誤って eval_expr に不完全な Option サポートを足して偽反例を生みやすい。
**Draft entry** (lift verbatim if approved):
> `kei_check/pbt` の bounded 評価器は Option 値を表現しないため、`s.toInt()`(→ `Option<Int>`)は `EvalError::Unsupported` に **意図的に倒す**(`List.get` と同じ扱い)。結果として `toInt()` を含む契約は `--generative` でも `[generative]` へ昇格せず `[runtime]` のまま留まる(spec §2.6 に明記、issue #109 で追跡)。一方 `n.toString()` は Str 値を返せるので評価可能、`s.length` は `encode_utf16().count()`(TS の `String.prototype.length` と同一の UTF-16 code unit 長)で評価する。評価器に Option を足す場合は #109 の設計判断を先に確認すること。中途半端なサポート(Some だけ扱う等)は偽反例の温床になる。

## PR #110: chore: bump version to 0.4.4 — 2026-07-04 merged

(no design-decision candidates for this PR)

## PR #111: feat: M34 MCP 検証経路強化 — kei_check generative + opaque import 可視化 (#89, #90) — 2026-07-05 merged

### Candidate: `CheckOptions::Default` は手書き実装 — `derive(Default)` に戻すと `generative_max_cases = 0` になり PBT が全関数を黙ってスキップする
**Why this matters for HANDOFF.md**: `#[derive(Default)]` に「リファクタ」で戻すと usize フィールドの既定が 0 になり、生成ケース総数 > 0 の全関数が上限超過扱いで対象外になる(エラーは出ず、契約が `[runtime]` に留まるだけ)ため、気づきにくい退行になる。
**Draft entry** (lift verbatim if approved):
> `kei_check::CheckOptions` は M34 以降 `generative_max_cases: usize` を持ち、`Default` は **手書き実装**(既定 = `pbt::MAX_GENERATIVE_CASES` = 100,000)。`#[derive(Default)]` に戻してはならない — derive だと 0 になり、generative PBT が全関数を「上限超過」として黙ってスキップする(診断は出ない)。新フィールドを足すときも非ゼロ既定が要るなら手書き Default 側に追加する。CLI 側は `..CheckOptions::default()` で追従する構造なので、CLI にフラグを増やさない限り上限は自動で既定値を拾う。

### Candidate: MCP の generative 上限 10,000 は MCP 固有の応答時間ポリシー — 検査ロジックは kei_check::pbt への委譲であり再実装禁止
**Why this matters for HANDOFF.md**: 「MCP と CLI で反例探索の結果が違う」ように見える事象(MCP では対象外だが CLI では反例が出る等)は、`MCP_GENERATIVE_MAX_CASES = 10,000` < CLI 既定 100,000 という意図的な差が原因。これを知らないとバグ扱いして上限を安易に揃えたり、MCP 側に別実装を生やしやすい。
**Draft entry** (lift verbatim if approved):
> MCP `kei_check` の generative は CLI `--generative` と **同一機構**(`kei_check::pbt`。`CheckOptions.generative_max_cases` 経由で上限だけ差し替える)。MCP 側上限は `tools::MCP_GENERATIVE_MAX_CASES = 10,000`(CLI 既定 100,000 より保守的)で、これはインタラクティブなツール呼び出しの応答時間を抑える **MCP 固有ポリシー**。実際の上限は応答の `generative.max_cases` にエコーバックされる契約なので、値を変えるときは golden(`tests/mcp/`)とツール description の両方が追従する。検証ロジックを MCP 側に再実装しないこと — 上限以外の挙動差が生まれたらそれはバグ。

### Candidate: MCP `kei_check` では宣言 import は「常に全て」opaque — `opaque_imports` 非空 = 検査は不完全、が API 契約
**Why this matters for HANDOFF.md**: MCP はソース文字列のみを受け取り M20 の import 解決経路(FS 参照)に乗らないため、import 由来の型は `Ty::Unknown` で照合されない。この制約を知らないと「MCP で clean だったのに CLI で型エラー」を不整合バグと誤診したり、MCP に FS アクセスを安易に足して sandbox 前提を壊しやすい。
**Draft entry** (lift verbatim if approved):
> MCP `kei_check` はソース文字列のみを入力とし、ファイルシステムを参照しない(M20 の import 解決には乗らない)。よって宣言された import は **常に全て opaque**(型は `Ty::Unknown`、照合されない)。M34 以降、応答の `opaque_imports` にドット区切りモジュールパス(sort + dedup 済み)を列挙し、「非空なら clean でも import 由来の型は未検証」をツール description に明記するのが API 契約。import 跨ぎの保証が要る検証は CLI `kei check <dir>` に誘導する。MCP に import 解決用の FS アクセスを足す変更は、この「source-only」前提の設計判断を覆すので先に issue で議論すること。

## PR #112: fix: M34 レビュー対応 — generative スキップ可視化と応答の構造化 — 2026-07-05 merged

### Candidate: generative 上限超過は「部分検査」ではなく関数丸ごと無検査スキップ — 可視化は `run_module_with_limit_reporting` の `skipped` が唯一の経路
**Why this matters for HANDOFF.md**: 「反例なし=充足」という誤読が M34 レビューで実際に問題化した。スキップは outcomes に一切現れないため、`skipped` を見ない呼び出し元(将来の kei_lsp 等)は同じ誤読を再生産する。また「上限内だけ部分検査する」最適化は安全側の哲学(部分検査で generative に格上げしない)に反する。
**Draft entry** (lift verbatim if approved):
> `kei_check::pbt` は候補ケース総数(候補数の積)が上限を超える関数を **丸ごと無検査スキップ**する(部分検査はしない — 部分検査で `[generative]` に格上げしない安全側の哲学)。スキップは `PropertyOutcome` に現れないため、可視化が要る呼び出し元は `run_module_with_limit_reporting` を使い `GenerativeRun.skipped`(関数名 + `required_cases`)を読むこと。既存 `run_module_with_limit` は `outcomes` だけ返す薄いラッパーで CLI 経路は不変。積が usize でオーバーフローする場合 `required_cases` は `usize::MAX`(「途方もなく大きい」の印であり正確な値ではない)。新しい呼び出し元(kei_lsp 等)を作るときは skipped を必ずユーザーに露出する — 出さないと「反例なし=充足」の誤読が再発する。

### Candidate: MCP 応答のキー順は `CheckResponse` 構造体の宣言順が契約 — `json!` 手組みに戻すとアルファベット順に変わり golden が全滅する
**Why this matters for HANDOFF.md**: `json!` マクロはキーをアルファベット順に出す一方、serde Serialize 構造体は宣言順。PR #112 で後者に統一し `tests/mcp/` golden を再生成済みなので、「シンプルだから」と `json!` に戻すリファクタは全 golden を壊す。また `#[serde(flatten)] CheckReport` により CheckReport の将来フィールドが自動でトップレベルに現れる設計。
**Draft entry** (lift verbatim if approved):
> MCP `kei_check` の応答は `tools.rs` の `CheckResponse` 構造体(`#[serde(flatten)] CheckReport` + `opaque_imports` + `generative`)で組み立てる。キー順は **struct 宣言順**(`diagnostics, contracts, opaque_imports, generative`)で `tests/mcp/` golden に固定済み — `json!` 手組みに戻すとアルファベット順になり golden が全滅する。`CheckReport` 本体には手を入れない(CLI `--json` フィクスチャ・golden を巻き込まないため)。flatten 経由なので CheckReport に将来フィールドを足すと MCP 応答にも自動で現れる — その際は `tests/mcp/` golden の再生成が必要。JSON 上のスキップ関数キー名は `function`(kei_check 側の `SkippedFunction.func` ではない)— MCP 応答契約としての意図的な改名。

### Candidate: MCP 引数の暗黙型降格は禁止 — 省略/null は既定値、型不一致は `invalid_arg` エラーが方針
**Why this matters for HANDOFF.md**: 旧実装は `generative: "true"`(文字列)を黙って false に降格しており、ユーザーは generative を頼んだつもりで走っていない状態に気づけなかった。新しい引数を足すときに `and_then(Value::as_*)` + `unwrap_or(default)` パターンを踏襲すると同じ罠を再導入する。
**Draft entry** (lift verbatim if approved):
> MCP ツール引数の扱いは「省略 / null → 既定値、値はあるが型不一致 → `tools::invalid_arg` で明示エラー(`isError: true`)」が方針(M34 レビューで確立)。`args.get(k).and_then(Value::as_bool).unwrap_or(false)` のような暗黙降格パターンは、型を間違えたユーザーが黙って既定値で走らされるため禁止。`server.rs` の `bool_arg` が参照実装(`Result<T, Value>` を返し、Err はそのまま tool_result として応答)。新しい任意引数を足すときは同じ形にする。

## Hook run 2026-07-05 — no new merged PR

PostToolUse hook が非 merge コマンド(scratchpad での lambda capture repro —
`let result = 5` を `xs.map(x => x + result)` で参照する `result_capture.kei` の
`kei check --json`。scratchpad に Cargo.toml が無く `cargo run` 自体は失敗)で発火。
最新 merged PR は #112 で、本ファイルに記録済み(候補 3 件登録済み)のため新規候補なし。

## PR #114: feat: M31 ラムダの読み取り専用キャプチャ (#59 後続 / dogfood critical) — 2026-07-05 merged

### Candidate: `lambda_floor` は M31 以降「隔離壁」ではない — 純粋性・`result` 遮断・`old` 検査の 3 箇所で使うフラグに役割変更
**Why this matters for HANDOFF.md**: M25 時代の「`scopes[floor..]` しか見えない」という理解のまま `lambda_floor` を読むと、キャプチャ許可後のシンボル解決を誤解する。逆に「もう使っていない」と誤って削除すると純粋性検査と `result` 遮断が壊れる。
**Draft entry** (lift verbatim if approved):
> `FnChecker.lambda_floor` は M25 では「ラムダ body から外側ローカルを見えなくする隔離壁」だったが、M31(読み取り専用キャプチャ)以降 `lookup_scope` は `scopes` 全体を innermost-first で探索する(通常のレキシカルスコープ。シャドーイングはラムダパラメータが勝つ)。`lambda_floor` は現在 (a) 純粋性検査 — `check_call_effects` が `is_some()` を見て `uses` 付き呼び出しを拒否、(b) `result` 遮断 — `floor` より外側の `result` だけ見えなくする、(c) `old(...)` 検査 — キャプチャ変数のみ参照する `old` の許可判定、の 3 用途のフラグ。ネストラムダは従来どおり save/restore パターン。「隔離壁」の意味で読み書きしないこと。

### Candidate: emit の `collect_old_exprs` がラムダ内 `old` を入口巻き上げできるのは「E4002 module は emit に到達しない」ゲートが理由
**Why this matters for HANDOFF.md**: emit 単体を見ると「ラムダは要素ごとに評価されるのに `old` を関数入口で一度だけ評価して大丈夫か?」という疑問が生じる。安全性の根拠は check 側(`forbid_old_inside_lambda_body` がキャプチャ変数のみ参照する `old` だけを通す)にあり、check と emit をまたぐ暗黙の契約になっている。
**Draft entry** (lift verbatim if approved):
> ラムダ内 `old(...)` は「キャプチャ変数のみを参照する式」に限り許可(M31)。ラムダパラメータ参照・Call を含む式・キャプチャ変数を参照しない式は従来どおり KEI-E4002。emit の `collect_old_exprs` はラムダ body も走査して `kei$old$N` に関数入口で一度だけ巻き上げるが、これが安全なのは **check が E4002 を出した module は emit に到達しない**ため、emit に届くラムダ内 `old` は必ずキャプチャ変数のみ(= 入口時点の値で固定してよい)だから。check 側の許可条件を緩めるときは emit の巻き上げ前提も同時に見直すこと。

### Candidate: `result` はラムダからキャプチャ不可(#113 保留)— `ScopeLookup` 三値で「診断発火」と「見つからない」を分離
**Why this matters for HANDOFF.md**: `ensures items.all(i => i.qty <= result)` が通らないのはバグではなく保留中の設計判断。また `lookup_scope` が `Option` でなく三値 enum を返すのは `result` 遮断時だけ専用診断を出すためで、`into_option()` に潰すリファクタは診断を消す。
**Draft entry** (lift verbatim if approved):
> ラムダの読み取り専用キャプチャ(M31)は `let` 束縛と関数パラメータが対象。ensures 文脈の特殊束縛 `result` は **意図的にキャプチャ対象外**(#113 で検討中。既存診断を維持)。実装は `lookup_scope` が `ScopeLookup::{Found, ResultBlocked, NotFound}` の三値を返し、`ResultBlocked` のときだけ専用診断を出す。`Option<Ty>` に単純化すると診断が失われるので注意。#113 の結論が出たら golden `err_type_lambda_capture_result` / `ok_lambda_capture_result_name` を見直す。

### Candidate: golden の削除・転用は「言語契約の変更」— 人間承認を経て行う前例(err_type_lambda_capture → ok_lambda_capture)
**Why this matters for HANDOFF.md**: `tests/golden/` は仕様の固定化装置であり、Milestone が意図的に契約を変えるときだけ、承認のうえ削除・転用できる — この手続きの前例として記録価値がある。あわせて PR 本文の「follow-up に回す」記載が最終 diff と食い違う(`err_list_contains_record` の stale fix 文面は結局本 PR 内で修正済み)ことも、PR 本文だけを信じない教訓になる。
**Draft entry** (lift verbatim if approved):
> M31 で golden `err_type_lambda_capture`(関数パラメータのキャプチャを KEI-E2001 で拒否する契約を固定していた)を人間承認のうえ削除し、同 fixture を正常系 `ok_lambda_capture` に転用した。golden の削除・転用は「言語契約の意図的変更」のときだけ、承認を経て行う。また `List.contains` の fix 文面「(lambdas cannot capture outer variables)」は M31 で stale になるため同 PR 内で「Compare a field directly: 'xs.any(e => e.field == value)'」に更新済み(golden `err_list_contains_record` も追従)— 診断・fix 文面がスコープ規則に言及している箇所は、スコープ規則を変える Milestone で grep して巻き取ること。

## PR #116: chore: bump version to 0.4.5 — 2026-07-05 merged

(no design-decision candidates for this PR)

---

2026-07-06: PostToolUse hook が非 merge コマンド(scratchpad での enum variant への
record spread repro — `Shape.Square { ...d, s: 1 }` を `kei check --json` で検査し、
「spread は plain record literal 専用」の診断 + fix を確認するコマンド)で発火。
最新 merged PR は #116 で、本ファイルに記録済み(候補なしセクション登録済み)のため新規候補なし。

<!-- hook note 2026-07-06: PostToolUse hook が非 merge コマンド(review worktree での
`git checkout --detach`)で発火。最新 merged PR は #116(version bump、機械的変更)で
上記に記録済みのため、新規候補なし。 -->

## PR #117: feat: M32 record spread — 差分更新構文 (#97) — 2026-07-06 merged

### Candidate: spread は「最大1個・先頭のみ」の構文制約
**Why this matters for HANDOFF.md**: JS/TS の spread(複数・任意位置)と意図的に異なる制約で、緩めると型検査と TS 出力の前提が壊れる。
**Draft entry** (lift verbatim if approved):
> record spread(`R { ...t, f: v }`)は**最大1個・先頭のみ**(違反は `KEI-E0101`)。この制約により (1) 明示フィールドが常に spread を上書きするという単純な意味論が保て、(2) TS 出力も `({ ...t, f: v })` と位置をそのまま写せる。JS 風に複数/任意位置を許すと上書き順の意味論と E2002(unknown field)の検査が複雑化するため、緩和する場合は両方を再設計すること。

### Candidate: AST 拡張時は serde `skip_serializing_if` で golden JSON を不変に保つ
**Why this matters for HANDOFF.md**: AST にフィールドを足すと syntax golden の expected JSON が全件変わるが、この手法で回避できる。将来の AST 拡張すべてに効くパターン。
**Draft entry** (lift verbatim if approved):
> `Expr::RecordLit` の `spread: Option<Box<Expr>>` は `#[serde(skip_serializing_if = "Option::is_none")]` を付けて追加した。こうすると spread を使わない既存の syntax golden expected JSON が一切変わらない。AST に optional フィールドを足すときは同じパターンを使い、既存 golden の一括更新を避けること。

### Candidate: spread があるときは missing-fields 診断を抑止(冗長 spread に warning は出さない)
**Why this matters for HANDOFF.md**: 診断の抑止条件はコードだけ見ると「バグでは」と誤解されやすく、意図的な判断であることを記録すべき。
**Draft entry** (lift verbatim if approved):
> record リテラルに spread がある場合、missing-fields 診断(不足フィールド)は抑止する — 不足分は spread 元から供給されるため。逆に「全フィールドを明示していて spread が冗長」なケースにも warning は出さない(意図的な判断: リファクタ途中の中間状態を騒がしくしないため)。存在しないフィールド名は spread の有無に関わらず既存 `KEI-E2002` を出す。

<!-- hook note 2026-07-06: PostToolUse hook が非 merge コマンド(scratchpad での
expected_ty leak テストケースの cargo run 検証)で発火。最新 merged PR は #117
(M32 record spread)で上記に記録済みのため、新規候補なし。 -->

<!-- hook note 2026-07-06: PostToolUse hook が非 merge コマンド(kei-invariant-auditor
セッションでの origin/feat/m33-map-stage1 golden expected JSON の git show 検証)で
発火。最新 merged PR は #117(M32 record spread)で上記に記録済みのため、新規候補なし。
M33 Map stage1(KEI-E2011/E2012 の新設)は未マージのため、merge 後の hook 実行時に
候補抽出する。 -->

<!-- hook note 2026-07-06: PostToolUse hook が非 merge コマンド(scratchpad m33 での
Map.empty() ネスト位置ケース検証 — Option<Map<String,Int>> 引数への Some(Map.empty()) と
Map.empty().set(...) メソッドチェーンの `kei check`、両方 exit=0)で発火。
最新 merged PR は #117(M32 record spread)で上記に記録済み(候補 3 件)のため、新規候補なし。
M33 Map stage1 は未マージ — merge 後の hook 実行時に候補抽出する。 -->

<!-- hook note 2026-07-06: PostToolUse hook が非 merge コマンド(scratchpad m33 での
Option<Map<String,Int>> 引数 vs Some(Map<Int,Int>) の KEI-E2001 検出確認と、
未注釈 Map.empty().set(...) チェーンの KEI-E2012 検出確認 — 両ケースとも期待どおり
診断が出ることを `kei check` で検証)で発火。最新 merged PR は #117(M32 record spread)
で上記に記録済み(候補 3 件)のため、新規候補なし。M33 Map stage1 は未マージ —
merge 後の hook 実行時に候補抽出する。 -->

<!-- hook note 2026-07-06: PostToolUse hook が非 merge コマンド(scratchpad m33 での
enum Map シャドー検証 — ユーザー定義 `enum Map` が組み込み Map より優先され exit=0、
および match arm 内 `Map.empty()` の KEI-E2012(型注釈要求)確認、exit=1)で発火。
最新 merged PR は #117(M32 record spread)で上記に記録済み(候補 3 件)のため、新規候補なし。
M33 Map stage1 は未マージ — merge 後の hook 実行時に候補抽出する。 -->

<!-- hook note 2026-07-06: PostToolUse hook が非 merge コマンド(scratchpad m33 での
E2012 抑止確認 — `let n: Int = Map.empty().size` が exit=0 で通ることの検証と、
spec §7.3(Map.empty() と期待型推論)の E2012 文言確認 sed/grep)で発火。
最新 merged PR は #117(M32 record spread)で上記に記録済み(候補 3 件)のため、新規候補なし。
M33 Map stage1(KEI-E2011/E2012)は未マージ — merge 後の hook 実行時に候補抽出する。 -->

<!-- hook note 2026-07-06: PostToolUse hook が非 merge コマンド(scratchpad m33 での
ユーザー定義 `enum Map` vs emit rewrite 検証 — `enum Map { empty full }` が組み込み
Map(2 型引数)としてチェックされ KEI-E2006 が出る、つまり組み込み Map がユーザー定義
enum を型解決でシャドーするケースの確認。check 失敗のため transpile 出力なし)で発火。
最新 merged PR は #117(M32 record spread)で上記に記録済み(候補 3 件)のため、新規候補なし。
M33 Map stage1 は未マージ — merge 後の hook 実行時に候補抽出する。 -->

<!-- hook note 2026-07-06: PostToolUse hook が非 merge コマンド(worktree main の
crates/kei_check/src/check.rs に対する `expected_ty` の grep — ヒットなし)で発火。
最新 merged PR は #117(M32 record spread)で上記に記録済み(候補 3 件)のため、新規候補なし。 -->

<!-- hook note 2026-07-06: PostToolUse hook が非 merge コマンド(scratchpad m33 での
両 arm Map.empty() の match 検証 — `match c { true => Map.empty() false => Map.empty() }`
の `kei check --json`。診断出力に `false` → `false_` の rename suggestion span が含まれる)
で発火。最新 merged PR は #117(M32 record spread)で上記に記録済み(候補 3 件)のため、
新規候補なし。M33 Map stage1 は未マージ — merge 後の hook 実行時に候補抽出する。 -->

<!-- hook note 2026-07-06: PostToolUse hook が非 merge コマンド(scratchpad m33 での
enum Map シャドー + match テスト — `enum Map { empty() full(Int) }` を classify する
テストケースの `kei check --json`。`let m = Map.empty();` の `;` が KEI-E0001
(unexpected character)で exit=1 — Kei にセミコロンはないためテストケース自体の
構文ミス)で発火。最新 merged PR は #117(M32 record spread)で上記に記録済み
(候補 3 件)のため、新規候補なし。M33 Map stage1 は未マージ — merge 後の hook
実行時に候補抽出する。 -->

<!-- hook note 2026-07-06: PostToolUse hook が非 merge コマンド(scratchpad m33 での
nested_leak.kei 検証 — fold 初期値位置の Map.empty()(acc 型注釈あり)と
Map.empty().size レシーバ位置(let 注釈あり)が m33wt(PR ブランチ)worktree の
`kei check --json` で diagnostics 空・exit=0 になることの確認)で発火。
最新 merged PR は #117(M32 record spread)で上記に記録済み(候補 3 件)のため、
新規候補なし。M33 Map stage1(KEI-E2011/E2012)は未マージ — merge 後の hook 実行時に
候補抽出する。 -->

<!-- hook note 2026-07-06: PostToolUse hook が非 merge コマンド(scratchpad leak.kei の
`kei check --json` — `fn main() -> Int { ... }` 形式のテストケースが KEI-E0101(Kei の
関数宣言は `fn` でなく `func`)+ `;` の KEI-E0001 ×2 で exit=1。Rust 風構文の混入による
テストケース自体の構文ミスで、コンパイラ側の問題ではない)で発火。
最新 merged PR は #117(M32 record spread)で上記に記録済み(候補 3 件)のため、
新規候補なし。wt-m33 worktree の HEAD は f02acb2「feat: M33 Map<K, V> 段階1 (#95)」—
M33 Map stage1 の merge 後の hook 実行時に候補抽出する。 -->

<!-- hook note 2026-07-06: PostToolUse hook が非 merge コマンド(scratchpad m33 での
two_empty.kei 検証 — Option<Int> scrutinee の match で両 arm が Map.empty() を返す
関数(戻り型注釈 Map<String, Int> あり)の `kei check --json`。2 番目の arm
(None => Map.empty()、line 6 col 13)に KEI-E2012「型注釈が必要」が 1 件出る —
戻り型注釈からの期待型伝播が match の最初の arm には効くが 2 番目以降の arm には
届かない可能性を示唆する観察)で発火。最新 merged PR は #117(M32 record spread)で
上記に記録済み(候補 3 件)のため、新規候補なし。M33 Map stage1(KEI-E2011/E2012)は
未マージ — merge 後の hook 実行時に候補抽出する。 -->

<!-- hook note 2026-07-06: PostToolUse hook が非 merge コマンド(scratchpad m33 での
shadow_map.kei 検証 — ユーザー定義 `enum Map { empty() full(Int) }` を型注釈なしで
構築・match するケースが m33wt worktree の `kei check --json` で diagnostics 空・
exit=0 になることの確認。ユーザー定義 enum Map がこの経路では組み込み Map に
シャドーされず正常に扱われる観察)で発火。最新 merged PR は #117(M32 record spread)
で上記に記録済み(候補 3 件)のため、新規候補なし。M33 Map stage1(KEI-E2011/E2012)は
未マージ — merge 後の hook 実行時に候補抽出する。 -->

<!-- hook note 2026-07-06: PostToolUse hook が非 merge コマンド(scratchpad での
cross-module テストプロジェクト作成 — lib.types に `Map<Bool, Int>` フィールドを持つ
record Cache を定義し、app.main から import して `.get(true)` する構成。M33 の
Map キー型制約(Bool が有効キーか)のクロスモジュール検証用ファイル生成のみで、
kei check は未実行)で発火。最新 merged PR は #117(M32 record spread)で
上記に記録済み(候補 3 件)のため、新規候補なし。M33 Map stage1(KEI-E2011/E2012)は
未マージ — merge 後の hook 実行時に候補抽出する。 -->

<!-- hook note 2026-07-06: PostToolUse hook が非 merge コマンド(scratchpad m33wt
worktree での `cargo build -p kei_cli --quiet` — PR ブランチの kei CLI ビルド確認、
build-exit:0 で成功)で発火。最新 merged PR は #117(M32 record spread)で
上記に記録済み(候補 3 件)のため、新規候補なし。M33 Map stage1(KEI-E2011/E2012)は
未マージ — merge 後の hook 実行時に候補抽出する。 -->

<!-- hook note 2026-07-06: PostToolUse hook が非 merge コマンド(scratchpad proj での
`kei build` 実行 — lib/types.kei の `Map<Bool, Int>` フィールドに対して KEI-E2011
「Map キー型は Int/String/tagged type のみ」が 1 件出て "no output written"・dist
未生成を確認。ただし build-exit:0 と報告されており、パイプ経由(`| head -8`)のため
exit code は head のものを拾っている可能性あり — kei build がエラー時に非 0 を
返すかどうかは直接検証が必要という観察)で発火。最新 merged PR は #117(M32 record
spread)で上記に記録済み(候補 3 件)のため、新規候補なし。M33 Map stage1
(KEI-E2011/E2012)は未マージ — merge 後の hook 実行時に候補抽出する。 -->

## PR #118: feat: M33 Map<K, V> 段階1 (#95) — 2026-07-06 merged

### Candidate: `expected_ty` の take/restore 規律(偽 E2012 防止)
**Why this matters for HANDOFF.md**: 期待型伝播はコードを読むだけでは「なぜ take() してから限定的に復元するのか」が分からず、次に期待型を使う機能(リテラル推論等)で同じ漏れバグを再発させやすい。
**Draft entry** (lift verbatim if approved):
> `FnChecker.expected_ty` は spec §7.3 の直接 3 位置(let 初期化式・呼び出し引数式・return 式)でのみ子式に伝わるべき値。`infer()` の冒頭で必ず `take()` し、式自身が `Match` か `Map.empty()` 呼び出し形のときだけ復元する。これを守らないと fold の引数や二項演算のオペランドに期待型が漏れて偽 KEI-E2012 が出る(PR #118 レビューで実際に発生)。期待型を使う機能を増やすときは復元ホワイトリストに追加する形で拡張すること。

### Candidate: `op_spans` は 1 パスで list/map をまとめて収集する
**Why this matters for HANDOFF.md**: 旧 API(`list_op_spans_with_resolver`)の形に戻して型別の公開関数を足すと、`Env::build` + 全関数 check が型ごとに再実行される退行になる。
**Draft entry** (lift verbatim if approved):
> emit が使う組み込みメソッド呼び出し位置は `op_spans[_with_resolver]` が `OpSpans { list_ops, map_ops }` として **1 回の FnChecker 実行**でまとめて返す。以前は List/Map 別々の公開関数が検査パスを 2 回走らせていた(#118 レビューで統合)。新しい組み込み型のスパン収集を足すときは `OpSpans` にフィールドを足し、公開関数は増やさない。

### Candidate: import 経由の型は `ty_of` が制約検査を素通しする
**Why this matters for HANDOFF.md**: 型制約(Map キー制約など)をローカル定義側の `resolve_ty` にだけ実装すると、import 経由の型で検出漏れになる — 実際 #118 で漏れ、レビューで `check_imported_map_keys` を追加した。
**Draft entry** (lift verbatim if approved):
> `imports.rs::ty_of` は設計上、型制約検査を持たない(構造変換のみ)。型に制約を付ける機能(例: Map キーは Int/String/tagged 基底限定 = KEI-E2011)を追加したら、`Env::build` の import 取り込み側にも明示的な再帰検査(例: `check_imported_map_keys`、診断 span は import 利用箇所)を必ず対で実装すること。

### Candidate: 組み込み `Map` 名前空間はユーザー定義が優先
**Why this matters for HANDOFF.md**: `Map.empty()` を無条件で組み込みに束ねると既存コードの `record Map` / import alias `Map` を壊す。ガードの置き場所(infer_call 側)も非自明。
**Draft entry** (lift verbatim if approved):
> `Map.empty()` の静的コンストラクタ解決は「`Map` がスコープにも `env.kinds` にも無い」ことを条件に infer_call で行う — ユーザー定義の `Map`(record / import alias)が常に勝つ。ガードは呼び出し元 infer_call に集約してあるので、`call_map_static` 側に到達した時点で組み込み確定として扱ってよい(二重ガード不要)。

### Candidate: generative は Map 引数関数を対象外(runtime 検証のみ)
**Why this matters for HANDOFF.md**: 「なぜ Map で generative が動かないのか」は spec を読まないと分からず、バグ報告と誤修正を招きやすい。
**Draft entry** (lift verbatim if approved):
> generative(M15)は Map 引数を持つ関数を候補ドメイン生成の対象外とする(spec 明記済み、契約は runtime 検証のみ)。Map リテラル(段階2)導入時に再検討する前提の意図的制限であり、生成対応を「修正」として安易に足さないこと。

## PR #120: chore(deps): bump the npm-minor-patch group across 3 directories with 3 updates — 2026-07-07 merged

(no design-decision candidates for this PR)

<!-- 判断根拠: dependabot による純粋な npm minor/patch バンプ。
     変更ファイルは editors/vscode / tests/cli/projects/app / tests/e2e の
     package.json + package-lock.json のみで、設計判断・不変条件・landmine を含まない。 -->

## PR #121: docs: v0.6 ロードマップ — extern package による npm import — 2026-07-07 merged

### Candidate: npm import でも検証境界は extern 署名のまま
**Why this matters for HANDOFF.md**: v0.6 以降 npm パッケージを呼べるようになっても Kei がパッケージの中身を型検査しない、という設計上の一線を将来の貢献者が越えないため。
**Draft entry** (lift verbatim if approved):
> `extern package`(v0.6)は npm bare specifier を extern 署名の名前空間に束縛するだけで、**検証境界は extern 署名(v0.2 M11)から動かさない**。.d.ts の自動取り込みは恒久的にスコープ外の可能性が高い — 「extern 署名が合意書」が Kei の設計であり、パッケージ内部を検査し始めると合意モデルが壊れる。署名なし呼び出しは従来どおり opaque / strict-extern の対象。

### Candidate: パッケージ解決はコンパイラに持ち込まない
**Why this matters for HANDOFF.md**: 「kei build が node_modules を見るべきでは」という自然な発想を明示的に却下した判断で、diff からは理由が読み取れないため。
**Draft entry** (lift verbatim if approved):
> `kei build` は node_modules を**見ない**。`extern package` の specifier は文字列として emit に素通しし、解決は TS エコシステム(tsc / wrangler / bundler)の責務。コンパイラに Node のモジュール解決を持ち込むと保守面の泥沼になる上、検証境界(extern 署名)の外の情報に依存することになる。

### Candidate: import 形は namespace のみから段階導入(実需駆動)
**Why this matters for HANDOFF.md**: named import / default / `new` を「まだ実装していない」のではなく「実需確定まで意図的に設計しない」ことを記録し、先回り実装を防ぐため。
**Draft entry** (lift verbatim if approved):
> v0.6 段階1は `import * as ns from "pkg"` の namespace import **のみ**を生成する。named import・default export の直接束縛は v0.8(Hono アダプタ)で実需が確定してから段階2として設計(🤝 合意事項)。`new Hono()` のようなクラス構築は @kei/hono アダプタ側で吸収する前提であり、言語側に構文を足さない。async 署名は v0.7 の主題。

## PR #122: feat: M35 extern package 宣言 — npm bare specifier 束縛 — 2026-07-07 merged

### Candidate: specifier 検査はホワイトリスト文法(拒否列挙では不十分)
**Why this matters for HANDOFF.md**: KEI-E3006 の検査方針が「不正プレフィックスの列挙」ではなく「受理文法のホワイトリスト」である理由(コード注入・不正 TS 生成の防止)は診断コードからは読めないため。
**Draft entry** (lift verbatim if approved):
> `extern package` の specifier 検査(KEI-E3006, `classify_package_specifier`)は npm bare specifier の**受理文法ホワイトリスト**(spec §2.4: `[a-z0-9][a-z0-9._-]*` セグメント + `@scope/name` + subpath)で行う。相対/URL/空などの拒否プレフィックス列挙だけでは、引用符・改行・空白などを含む specifier がそのまま `import * as x from "<spec>";` に verbatim 出力され、コード注入や不正 TS 生成の余地が残る(M35 レビュー指摘)。specifier 文法を広げるときは必ずホワイトリスト側を拡張すること。

### Candidate: `package` はキーワード化しない(文脈依存識別子)
**Why this matters for HANDOFF.md**: 予約語を増やさず `extern` 直後の先読みで判定するパターンは `extern query` と共通の設計方針で、次に extern 系構文を足す人が踏襲すべき前例のため。
**Draft entry** (lift verbatim if approved):
> `extern package` の `package` はキーワードではなく、`extern` 直後の文脈依存識別子として先読み判定する(`extern query` の `query` と同じパターン)。ユーザーコードで `package` を識別子として使えることを壊さないため、extern 系の新構文は今後もこの方式を踏襲する。

### Candidate: 束縛名は extern 署名の名前空間専用(KEI-E3007 を全使用位置で発火)
**Why this matters for HANDOFF.md**: `NameKind::ExternPackage` を値・型位置・record リテラルなど 7 箇所で個別に弾いている構造は、使用位置チェックを 1 箇所に集約できない check.rs の制約と合わせて知らないと、新しい式形を足したときに検査漏れするため。
**Draft entry** (lift verbatim if approved):
> `extern package` の束縛名(`NameKind::ExternPackage`)は extern 署名の名前空間**専用**。値・型位置・record リテラル・呼び出しなど各使用位置で KEI-E3007 を発火させる分岐が check.rs に 7 箇所あり、message/fix の構築だけ `extern_package_scope_diag` に集約している(`self.push` の引数形が呼び出し元ごとに違うため完全共通化は不可)。新しい式/型の解決パスを足すときは `NameKind::Import` を扱う分岐の隣に `ExternPackage` アームも足すこと。call resolution 自体は既存 extern 解決パス(型確定 / opaque / strict-extern warning)を共有しており、専用パスは無い。

### Candidate: emit は未使用検出なしで verbatim・常時出力
**Why this matters for HANDOFF.md**: 「未使用でも import を出す」「specifier を相対パス化しない」という 2 つの意図的な非対称は、後から最適化したくなる典型ポイントのため先回りで記録する価値がある。
**Draft entry** (lift verbatim if approved):
> `extern package` の TS 出力は宣言ごとに `import * as <name> from "<spec>";` を**常時**出力する(未使用検出なし — 既存 `emit_import` と同じ方針に揃えた)。specifier は相対パス化などの変換をせず verbatim で出す(bare specifier は bundler / runtime の解決に委ねる)。「未使用なら省く」最適化を入れる場合は emit_import と同時に方針を変えること。

## PR #124: fix: M36 レビュー対応 — fixture 統合と契約ドキュメント追従 — 2026-07-07 merged

### Candidate: `tests/cli/projects/app` を共有するテストは `app_project_lock()` で直列化必須
**Why this matters for HANDOFF.md**: cargo test はスレッド並列実行が既定のため、同じ npm プロジェクト fixture を触るテストがロックを取らないと npm install / dist 削除が競合する — コードを読むだけでは「なぜ Mutex があるのか」が分からない landmine。
**Draft entry** (lift verbatim if approved):
> `crates/kei_cli/tests/cli.rs` で `tests/cli/projects/app` を使う `#[test]` は必ず `app_project_lock()`(OnceLock<Mutex>)を取ってから `setup_npm_project` を呼ぶこと。cargo test は既定でスレッド並列のため、npm install / dist 削除が競合してフレーキーになる(greeter_app fixture を app へ統合した際に顕在化)。新たに app fixture を触るテストを足すときも同じロックを取る。なお `setup_npm_project` は意図的に `has_npm()` スキップを含まない — 呼び出し元が先に skip する契約。

### Candidate: npm fixture は複製せず既存 `app/` に統合、パッケージは消費者側の層に置く
**Why this matters for HANDOFF.md**: fixture を丸ごと複製すると CI の npm install(+1321 行 lockfile)が二重化する、という理由は削除された diff からしか読み取れない。
**Draft entry** (lift verbatim if approved):
> npm ツールチェーン(typescript/vitest/@kei/runtime + tsconfig + lockfile)を持つ CLI プロジェクト fixture は `tests/cli/projects/app/` の 1 つに統合する。新機能の e2e が必要でも fixture を複製しない(CI の npm install が二重化するため)。file: 依存のミニパッケージは消費者と同じ層 `tests/cli/packages/` に置く(`tests/e2e/packages/` ではなく)。module 名は fixture 内で衝突しないよう命名する(`app.greet` と衝突するため `app.greeter_hello` に改名した前例)。

### Candidate: MCP `kei_check` の `opaque_imports` は `extern package` 束縛を含まない
**Why this matters for HANDOFF.md**: 「extern package も外部依存なのに opaque_imports に載らない」のは一見バグに見えるが、署名が型・エフェクトを担保するための意図的な設計 — spec §2.4 に明文化済みだが HANDOFF 級の不変条件。
**Draft entry** (lift verbatim if approved):
> MCP `kei_check` の `opaque_imports` はファイル `import` の opaque(型・エフェクト不明)のみを報告し、`extern package` 束縛は**含めない**。extern package は宣言された署名が型とエフェクトを担保するため opaque ではない、という別概念(spec v0.2 §2.4)。ツール description(`crates/kei_mcp/src/server.rs`)と MCP golden(`tests/mcp/tools_list.response.json`)もこの区別に追従させること。

## PR #125: chore: bump version to 0.6.0 — 2026-07-07 merged

(no design-decision candidates for this PR)

## PR #126: docs: v0.7 ロードマップ — async(uses Async エフェクト) — 2026-07-09 merged

### Candidate: Async は「関数の色」ではなくエフェクト(uses モデル統合)
**Why this matters for HANDOFF.md**: Kei の言語アイデンティティを決める根幹の合意で、後続 Milestone で「async キーワードを追加したくなる」誘惑に対する最上位判断軸。ロードマップ本文にしか書かれておらず、コードだけ見ても意図が復元できない。
**Draft entry** (lift verbatim if approved):
> Kei は非同期性を**関数の色ではなくエフェクト**として扱う。`async` キーワードを Kei ソースに追加せず、`func f() -> T uses Network.Read, Async` のように既存の uses モデルに `Async` を追加するだけで表現する。理由: (1) 色システムは呼び出し規約を二分するが、Kei は既にエフェクトで副作用を分類しており重複する、(2) uses モデルの推移伝播機構(KEI-E3001)がそのまま非同期の伝播にも効く、(3) 「色ではなくエフェクト」を保つことで契約純粋性(KEI-E4001)との交差診断が自然に成立する。将来「async fn を復活させたい」提案が来ても、この立場を崩すと契約系の診断設計が総崩れになるため受け入れない。

### Candidate: `Async` は IO 傘下ではなく独立ルート — IO は Async を包含しない
**Why this matters for HANDOFF.md**: エフェクト階層の非自明な形状で、「IO 宣言があれば Async も許可される」という自然な直感を明示的に否定している。互換性破壊(既存の `uses IO` 関数が黙って async 化してしまう)を避けるための意図的な設計。
**Draft entry** (lift verbatim if approved):
> `effects.rs` の `STANDARD_EFFECTS` において、`Async` は IO 傘下ではなく**独立ルート**として置く。`uses IO` は「IO 全包括の雑な許可」だが、Async はそこに含めない。理由: IO を宣言している既存の同期関数(v0.6 以前に書かれた extern net.* など)が v0.7 以降黙って async として扱われる互換性破壊を避けるため。spec §3.2 の「`uses IO` は全 IO の包括許可」記述には**「Async は例外」**の一文を階層図と本文の両方に明記する。実装時は `covers` テーブルで IO → Async の辺が張られていないことをテストで固定する。

### Candidate: Kei ソースに `await` 演算子を持たない — compiler が emit 時に自動挿入
**Why this matters for HANDOFF.md**: 「なぜ await の書き忘れ診断が不要か」の根拠と、v0.8 で並列制御を追加するときに `spawn`/`join` を別プリミティブとして設計する理由が、この判断からしか読めない。
**Draft entry** (lift verbatim if approved):
> Kei ソースには `await` 演算子を追加しない。`let u = fetchUser(id)` と同期的な書き味で書き、生成 TS が `const u = await fetchUser(id);` を出す。根拠: v0.7 は sequential のみで並列制御が言語に不要なため、await の書き忘れが起きえない(uses Async 関数の呼び出しは自動で await される)。並列化が必要になった v0.8 の段階2で `spawn` / `join` 相当を別プリミティブとして 🤝 で設計する — async 自体は v0.7 に閉じる、が Kei の分割方針。「TypeScript ユーザーに合わせて await を出したい」提案は、書き忘れ診断の複雑度と Kei の同期的な書き味の両方を壊すので却下する。

### Candidate: `Promise<T>` 型は Kei に露出しない — emit の責務
**Why this matters for HANDOFF.md**: Kei 型システムを「同期的な値の型」に閉じる設計判断。「非同期性を型に表す」誘惑(Haskell の `IO T` 相当)に対する明確な立場表明。
**Draft entry** (lift verbatim if approved):
> `Promise<T>` は Kei 型システムに露出させない。async 関数の戻り型は `-> T`(第一級関数値と同じく、Promise 化は emit の責務)。生成 TS 側では `uses Async` 関数が `async function f(): Promise<T>` に写る。この判断は「非同期性は型ではなくエフェクトで表す」立場の帰結であり、型システムを同期的な値の代数に閉じる。将来「Promise 型を露出させて `await` を書けるように」という提案があっても、上記の「await 演算子を持たない」と一体の設計なので、両方一緒にでない限り受け入れない。

### Candidate: 契約式は同期・純粋のまま — async 呼び出しは既存の二重診断で拒否
**Why this matters for HANDOFF.md**: 契約系の設計不変条件で、async 追加という大きな言語変更が既存の診断機構(KEI-E4001 + KEI-E3001)にゼロ変更で吸収されるという構造的な整合性を明文化する。
**Draft entry** (lift verbatim if approved):
> `requires` / `ensures` / `old(...)` は v0.7 以降も**同期・純粋**のまま。async 関数(`uses Async` 持ちや extern async)を契約式から呼ぶのは、KEI-E4001(契約純粋性)+ KEI-E3001(uses 越え)の**既存二重診断**で拒否する(新規診断を追加しない)。async 関数の ensures は同期述語で結果を検証し、emit は async wrapper 内で `await → ensures 評価` の順に出す。requires 違反は関数入口(await 前)で throw、ensures 違反は Promise resolve 後に throw(reject ではなく throw で await 側に例外として届く)。「契約で await を使いたい」という将来要望は、契約の同期・純粋性を壊すので受け入れない — 代替は同期の `extern query` 観測子。

### Candidate: `extern query` の Async 化は禁止 — query は純粋観測子のシルエットを守る
**Why this matters for HANDOFF.md**: `extern query` の意味論的な役割(純粋観測子)を守るための境界条件で、実装時に「エフェクト列に Async を足せるようにするだけ」と誤読しないための landmine。
**Draft entry** (lift verbatim if approved):
> `extern query` に `Async` を含む uses を付けるのは M38 で **KEI-E3005 相当の新規診断(または既存 3005 の拡張)で拒否**する。query は「純粋観測子」のシルエットを守るための構文で、非同期化するとその不変条件が崩れる。fix hint は「query は純粋観測子、通常の extern を使え」。extern async 自体は普通の `extern` に `uses ..., Async` を付ける形で表現できるため、query に Async を許す必要はない。

### Candidate: fmt は uses 節をソースのまま素通し — `.sort()` を追加しない
**Why this matters for HANDOFF.md**: レビュー修正コミットで新設された注記で、「fmt は識別子リストをソートするのが自然」という直感に対する明示的な否定。実装エージェントが「既存踏襲」を誤読するのを防ぐための landmine。
**Draft entry** (lift verbatim if approved):
> `kei_fmt/src/lib.rs` は uses 節をソート機構なしで**ソース順に素通し**する(現状 `.sort()` 系のロジックは存在しない)。M37 で `Async` を uses 節に足すとき、fmt 側で識別子をアルファベット順に並べ替えたくなる誘惑があるが、fmt は AST の意味を保存する原則(HANDOFF)に加えて、uses 節の順序も**ユーザーが書いた順を尊重する**規約になっている。もし将来「uses 節を並べ替えたい」提案が来たら、`kei check` の等価判定(順序非依存)を先に固定し、golden 更新のインパクトを見積もってから議論する。fmt 変更は M37 完了条件に含まれない。

## PR #126 (PostToolUse re-fire — cargo check trigger): docs: v0.7 ロードマップ — async(uses Async エフェクト) — 2026-07-09

> **Note**: この hook 発火は `gh pr merge` ではなく `cargo check --workspace` の PostToolUse イベントで
> トリガーされた(`tool_input.command` が `cargo check` のため、prompt のフォールバックで最新 merged PR
> = #126 を選択)。上記の PR #126 セクションで 7 件の設計判断候補を既に記録済みのため、本パスで
> 新規に追加すべき候補はない。

(no new design-decision candidates for this PR — already covered above)

## PR #126 (PostToolUse re-fire #2 — M37 3-command verify trigger): docs: v0.7 ロードマップ — async(uses Async エフェクト) — 2026-07-09

> **Note**: 本 hook 発火も `gh pr merge` ではなく `cargo fmt --all -- --check && cargo clippy ... && cargo test ...`(M37 着手前の 3 コマンド検証)の PostToolUse で
> トリガーされた。prompt のフォールバックで最新 merged PR = #126 を再度選択したが、上記 2 セクション
> (初回発火 + cargo check 再発火)で PR #126 の候補は網羅済み。**新規に追加すべき候補はない**。
>
> 参考メモ: この検証コマンド出力に `error: could not compile kei_syntax (lib test)` を含む clippy stderr
> が乗っていた(`CLIPPY=0` で最終的な exit は 0)が、これはユーザー側の作業ログであって設計判断
> ではなく、PR #126 のマージ内容(docs のみ)には無関係。ハンドオフ候補ではないので記録しない。

(no new design-decision candidates for this PR — already covered above)

## PR #126 (PostToolUse re-fire #3 — M37 emit bug repro trigger): docs: v0.7 ロードマップ — async(uses Async エフェクト) — 2026-07-09

> **Note**: 本 hook 発火も `gh pr merge` ではなく、M37 実装ブランチで発見された emit バグの
> 再現スクリプト(`git stash → git checkout pr-127 → cargo run -p kei_emit --example transpile → git checkout main → git stash pop`)の
> PostToolUse でトリガーされた。prompt のフォールバックで最新 merged PR = #126 を再度選択したが、
> 上記 3 セクション(初回発火 + cargo check 再発火 + 3-command verify 再発火)で PR #126 の候補は
> 網羅済み。**新規に追加すべき候補はない**。
>
> 参考メモ: この検証コマンド出力に `match` 式の分岐内で `await` が生成され `Promise<string>` を
> `string` として返そうとする emit バグの再現(`return (() => { ...; return await fetchName(v); })()` が
> Promise を返してしまう問題)が写っていたが、これは **PR #127 の実装対象バグ**であって PR #126
> (docs のみ)には無関係。M37 の完了条件「async 関数の check + emit(await 自動挿入)」の一環として
> PR #127 側で処理されるべき事項なので、本ハンドオフ候補には記録しない(PR #127 マージ時に別途評価)。

(no new design-decision candidates for this PR — already covered above)

## PR #126 (PostToolUse re-fire #4 — async_match.kei repro creation): docs: v0.7 ロードマップ — async(uses Async エフェクト) — 2026-07-09

> **Note**: 本 hook 発火も `gh pr merge` ではなく、scratchpad に `async_match.kei` を書き出す
> `mkdir + cat > EOF + ls -la` のヒアドキュメントコマンドの PostToolUse でトリガーされた
> (`tool_input.command` が `gh pr merge` を含まないため、prompt のフォールバックで最新 merged PR
> = #126 を再度選択)。上記 4 セクション(初回発火 + cargo check 再発火 + 3-command verify 再発火
> + emit bug repro 再発火)で PR #126 の候補は既に網羅済み。**新規に追加すべき候補はない**。
>
> 参考メモ: 今回書き出された `async_match.kei` は、`match Some(id) { Some(v) => fetchName(v), None => "default" }`
> という「match アーム内から async 関数を呼ぶ」ケースで、M37 の emit バグ(match アームを
> 即時実行 IIFE `(() => { ... })()` に落とすと、IIFE 内の `await` の resolved 値を返す IIFE 自身が
> `Promise<T>` になり、`match` 式全体の型が `T` から `Promise<T>` にズレる)を最小再現するための
> ものと推測される。これは **PR #127(M37 実装)側で処理される emit 責務**であり、PR #126(docs のみ)
> の設計判断ではない。ハンドオフ候補としては PR #127 マージ時に「match アーム内 await の IIFE
> 落とし込みは Promise-of-Promise を作るので、非同期 match は `async` IIFE + outer `await` にする」
> のような設計メモが上がる可能性があるが、それは PR #127 側の担当で、本パスでは記録しない。

(no new design-decision candidates for this PR — already covered above)

## PR #126 (PostToolUse re-fire #5 — async name-ref combinator probe trigger): docs: v0.7 ロードマップ — async(uses Async エフェクト) — 2026-07-09

> **Note**: 本 hook 発火も `gh pr merge` ではなく、M37 実装調査で `ids.map(fetchName)`
> (async 関数を name-ref で高階関数 combinator に渡すパターン)が現行 `kei check --json` を
> 通るかを確認する `cargo run -p kei_cli --bin kei -- check .../async_map.kei --json` の
> PostToolUse でトリガーされた(`tool_input.command` が `gh pr merge` を含まないため、
> prompt のフォールバックで最新 merged PR = #126 を再度選択)。上記 5 セクション
> (初回発火 + cargo check 再発火 + 3-command verify 再発火 + emit bug repro 再発火 +
> async_match.kei repro 再発火)で PR #126 の候補は既に網羅済み。**新規に追加すべき候補はない**。
>
> 参考メモ: 今回の probe 出力は `diagnostics: []` で checker が通過し、`fetchName` の
> `requires id >= 0` 契約が `verification: "runtime"` として拾えていることが確認できた。
> つまり **現行 checker は「async 関数を name-ref で map/filter などに渡す」ケースを型検査で
> 弾いていない**。しかしこれは M37(async 関数の check + emit)実装作業の途中観察であって、
> PR #126(docs のみ)のマージ内容とは無関係。async name-ref を high-order combinator に
> 渡したときに emit をどう成立させるか(呼び出し側 `map` を `Promise.all(xs.map(async ...))`
> に展開するのか、call site での await 自動挿入を諦めて `List<Promise<T>>` を返すのか、
> effect row の Async が map の返り値型に伝播するか)は **PR #127 側の設計判断**であり、
> そちらのマージ時に別途評価する。本ハンドオフ候補には記録しない。
>
> メタ観察(#3 の再掲): PR #126 マージ後、M37 実装フェーズに入ってから同一 PR に対する
> 再発火が 5 回目となった。post-merge-handoff hook は Bash tool のあらゆる呼び出しで
> PostToolUse が発火する現在の設定では、実装作業中の cargo/kei/mkdir コマンドすべてに反応して
> しまう。**hook 側で `tool_input.command` が `gh pr merge` を含むケースにのみ発火するよう
> matcher を絞る** のが本来の設計意図(prompt 冒頭「A `gh pr merge` command just completed」)
> に近い。これは `.claude/settings.json` の hook matcher 設定変更で対処すべき事項であり、
> handoff 候補(HANDOFF.md 昇格対象の設計判断メモ)ではなく hook 運用改善タスクとして
> 別途対応する筋合いの話。ここでは 5 回目の重複としてメタ観察を再記録するに留める。

(no new design-decision candidates for this PR — already covered above)

## PR #126: docs: v0.7 ロードマップ — async(uses Async エフェクト) — 2026-07-09 merged(hook 7 回目・重複発火)

(no new design-decision candidates for this PR — already covered above)

> **Note**: 本 hook 発火は `gh pr merge` ではなく、M37 実装作業の一環で
> `cargo check --workspace --all-targets --quiet` を実行した PostToolUse でトリガーされた
> (`tool_input.command` に `gh pr merge` を含まないため、prompt のフォールバックで
> 最新 merged PR = #126 を再選択)。PR #126(docs のみ、v0.7 async ロードマップ)の
> 設計判断は既に上記 6 セクションで網羅済み(Async を IO 傘下から外す独立ルート化 /
> await 演算子を Kei ソースに持たない / Promise<T> を露出させない / 契約は同期・純粋を
> 保つ / extern query の Async 禁止 / fmt は uses 節を素通し等)。**新規に追加すべき候補はない**。
>
> メタ観察(前セクションの再々掲): PR #126 マージ後 M37 実装フェーズにおける同一 PR への
> hook 再発火は 6 回目(このセクションで 7 回目)。**根本原因は `.claude/settings.json` の
> matcher が Bash tool 全体をキャプチャしていること**であり、handoff-candidates.md 側で
> 対処するものではない。前セクションで指摘した「matcher を `gh pr merge` に絞る」設定変更が
> 未実施のため再発火が続いている。ここでは 6 回目の重複としてメタ観察を保持するに留め、
> hook 運用改善(settings.json の matcher 修正)を別タスクとして明示的に切り出す必要がある
> ことを再度記録する。

## PR #127: feat: M37 uses Async エフェクトと async 関数コア — 2026-07-09 merged

### Candidate: `Async` は `IO` の子ではない独立ルートエフェクト(covers に唯一の例外)
**Why this matters for HANDOFF.md**: 既存 `uses IO` 宣言関数群が M37 の導入で「黙って非同期になる」互換性破壊を起こさないための根本設計。エフェクト階層の直感(「IO が最上位でその下に全部ぶら下がる」)に対する意図的な唯一の例外であり、将来 `Async.X` サブエフェクトを追加する人が最初にぶつかる landmine。
**Draft entry** (lift verbatim if approved):
> `Async` エフェクトは `IO` の子ではなく、独立ルートとして定義する(v0.7 / M37)。`effects::covers` は「declared が自分自身か祖先」の基本則に加え、**`declared == "IO"` かつ `used == "Async"` または `used.starts_with("Async.")` のときは false を返す** という唯一の例外を持つ。理由: この例外がないと、v0.6 以前から存在する `uses IO` 宣言の関数が M37 の導入で黙って `uses Async` を包含したことになり、呼び出し側は既存 `IO` 宣言だけで async 関数を呼べる = ソース側の `await` 記述なしに Promise 汚染が広がる silent breakage が起きる。`Async` 呼び出しは常に明示 `uses Async` を要求する。`Async.X` サブエフェクト(将来)の追加時も、この例外の `starts_with("Async.")` が防御的に効くよう検査済み(`effects::tests::io_does_not_cover_async`)。

### Candidate: async 情報は checker が確定して `OpSpans` で emit へ渡す(構文ヒューリスティック `func_is_async` は廃止)
**Why this matters for HANDOFF.md**: 「emit が構文形から async 判定する」誘惑への恒久的な回答であり、list_ops/map_ops で確立した「checker が権威、emit は span 所属だけを見る」パターンを Async にも徹底したことの記録。PR #127 の初版は `func_is_async` という構文ヒューリスティックを持っていたが、レビューで soundness バグ(高階関数への async 名前渡し・contract 内での判定漏れ等)が発覚し、レビュー対応コミットで廃止された。
**Draft entry** (lift verbatim if approved):
> Async に関する三種の情報は **すべて checker(`kei_check::op_spans_with_resolver`)が確定** し、`OpSpans` の 3 集合として emit に渡す。emit は span 所属テーブル引きだけで判断し、検査ロジックは持たない(list_ops / map_ops と同じ流儀)。
>
> - `async_calls: HashSet<Span>` — `uses Async` 関数への直接呼び出し(Call 式の完全な Span)。**start だけでなく end も含む完全 Span がキー**(`a.f().g()` のように連鎖する Call は開始位置を共有しうるため、start だけでは衝突する — list_ops/map_ops がメソッド名トークンの (line, col) をキーにするのと対照的)。emit はこの位置の Call の直前に `await ` を挿入する。
> - `async_match_spans: HashSet<Span>` — scrutinee か何らかの arm body に `async_calls` に載る呼び出しを含む match 式の Span。emit はこの位置の match IIFE を `await (async () => {...})()` として出す(**そうしないと内側の `await` が非 async な IIFE 内での使用になり TS が SyntaxError**)。実機再現済みのバグとして `kei_emit::tests::match_with_async_arm_body_emits_async_iife` で固定。
> - `async_funcs: HashSet<Span>` — `uses Async` を宣言している **関数宣言そのものの Span** 集合。emit はこれだけを根拠に `async function` + `Promise<T>` として出す。**構文ヒューリスティック(「body に async 呼び出しがあれば async 化」)は廃止済み** — PR #127 初版はそれを持っていたが、レビューで「宣言はしているが body から呼んでいない async 関数」等でズレる soundness 穴が指摘され、checker 権威の宣言集合に一本化した。
>
> 将来 Async 関連の情報が増えるとき(例: `Async.Retry` の retry policy span、Promise.all 対応の並列化位置)も、同じパターン(checker が確定 → OpSpans 経由 → emit は span 所属だけ)を踏襲する。

### Candidate: async 呼び出しは高階コンビネータに渡せない — 専用診断 KEI-E3008(伝播チェックより前に発火)
**Why this matters for HANDOFF.md**: 「呼び出し側が `uses Async` を宣言していれば通してよいのでは?」という直感的な緩和が silent breakage(`Promise<T>[]` を `T[]` として emit)を生む理由の恒久記録。診断コード順(E3008 は E3001 より前)の意味も含めて記録すべき invariant。
**Draft entry** (lift verbatim if approved):
> `xs.map(asyncFn)` のようにコンビネータ引数へ **名前参照で** `uses Async` 関数を渡すケースは、`caller` 側が `uses Async` を宣言していても専用診断 **KEI-E3008(COMBINATOR_ASYNC_FN)で無条件に拒否** する。理由: `Array.prototype.map` などは JS ネイティブ実装でコールバックを await せず、結果は `T[]` ではなく `Promise<T>[]` になる。v0.7 は sequential await のみで `Promise.all` 相当の並列化は未実装のため、combinator 越しに Async を安全に伝播する経路が存在しない。通常のエフェクト伝播チェック(`check_call_effects` → E3001)まで到達すると「caller が Async 宣言済み = 診断ゼロ」で通ってしまうため、**専用診断は伝播チェックより前に発火させて return する** 制御フロー(`check.rs::infer_call_name_arg` 相当の位置)。ラムダ内の async 呼び出し(`xs.filter(id => hasAccess(id))`)は既存 E3001(EFFECT_UNDECLARED)で拒否されるが、fix 文面は Async 特有の 2 択(「同期 for ループに書き換える」「事前に await して値を渡す」)に差し替えている — 非 Async のケースは従来通り「名前関数に置き換える」誘導を維持(名前渡しは非 Async では安全)。

### Candidate: `await` 前置は postfix chain 位置で括弧が必要(Prec::Unary、`Expr::Unary` と同じ needs_paren パターン)
**Why this matters for HANDOFF.md**: `await` の JS 演算子優先順位(UnaryExpression)への対応が抜けると **サイレントに実行時が壊れる**(Promise に `.field` を引く)。PR #127 の 2 番目のコミットで修正されたバグの再発防止として、needs_paren パターンを Async 導入時から明文化しておく価値がある landmine。
**Draft entry** (lift verbatim if approved):
> `await` は JS の UnaryExpression 相当の優先順位(`Prec::Unary`)しか持たない。`fetchUser(id).name` のように async 呼び出しの結果へ postfix chain(フィールドアクセスやメソッドチェーン)を直接続ける文脈で単純に `await ` を前置すると、`await fetchUser(id).name` は `await (fetchUser(id).name)` と解釈され、**Promise オブジェクトに `.name` を引く** サイレントな実行時破壊になる(型検査は通る — Promise<User> と User の構造が偶然合うと `.name` が undefined として返る)。`emit_call` は `emit_expr` の `parent: Prec` を受け取り、`needs_await && parent > Prec::Unary` のとき `(await f(x)).field` に括弧する — 既存 `Expr::Unary`(`-x` / `!x`)と同一の needs_paren パターンを踏襲する。回帰防止テスト `uses_async_call_as_postfix_base_gets_parenthesized_await` は「対照として let 束縛・return 直下(postfix chain なし)は括弧なし」も同時に固定している。

### Candidate: `ensures` を持つ async 関数は契約 IIFE 自体を async 化する(requires は await 前、ensures は resolve 後)
**Why this matters for HANDOFF.md**: 契約(requires / ensures)は「同期・純粋」という強い invariant を保つが、async 関数の body の中では位置関係が重要 — requires は入口(await より前)、ensures は resolve 後(kei$result は同期述語で検証)という定型パターンを固定するための記録。M38 で契約 async の e2e 検証が入るときの前提。
**Draft entry** (lift verbatim if approved):
> `uses Async` かつ `ensures` を持つ関数の emit 形は:
> ```ts
> export async function f(...): Promise<T> {
>   // requires チェック(同期・純粋、await より前で throw)
>   const kei$result = await (async (): Promise<T> => {
>     // body(内部の async 呼び出しは通常通り await)
>   })();
>   // ensures チェック(kei$result は resolve 済みなので同期述語で検証可能)
>   if (!(<ensures 式>)) { throw new KeiContractViolation({...}); }
>   return kei$result;
> }
> ```
> 要点: (a) 契約 IIFE 自体を `async (): Promise<T> => {...}` にして外側で `await` する — こうすると `kei$result` の型は `T`(Promise が剥がれた後)になり、ensures 式は **同期・純粋な述語のまま** で書ける。(b) requires は IIFE の外側(入口)で評価 = await より前で throw する。(c) ensures は resolve 後の値を検証するので、契約式に `await` を持ち込む必要がない — 「契約は同期・純粋」の invariant を破らずに async 関数の contract を検証できる。この形は非 async 版(`ensures_wraps_body_and_captures_old`)と throw ロジック構造は同じで、変わるのは IIFE の宣言・呼び出しに `async`/`await` が付くだけ。M38 の契約 async e2e はこの emit 形の上で拡張する予定。

### Candidate: `check_call_effects` から `is_direct_call` フラグを排除 — 「直接呼び出しかどうか」は呼び出し側で判定する
**Why this matters for HANDOFF.md**: PR #118(list_ops)で確立し、PR #127 で Async にも徹底した「span 集合への挿入は呼び出し側で判定、共通ヘルパは effects 伝播だけを担う」という抽象境界の教訓。次に新エフェクトの emit 支援情報を追加する人が「共通関数にフラグを増やす」誘惑に負けないための記録。
**Draft entry** (lift verbatim if approved):
> `check_call_effects`(エフェクト伝播チェックの共通ヘルパ)には **呼び出し形態のフラグ(`is_direct_call` 等)を持たせない**。理由: 「その Call が直接呼び出しか(そのまま `f(...)` として emit される位置か)」は呼び出し側(`infer_call_name_arg` / `infer_call_field_method` 等)が一番よく知っている情報であり、共通ヘルパに持ち込むと呼び出し側ごとに条件がずれて把握しづらくなる(PR #118 で list_ops の位置判定で同じ問題を踏んだ教訓)。async_calls 集合への挿入は `check_call_effects` を呼んだ **直後の呼び出し側コード** で行う:
> ```rust
> self.check_call_effects(&callee.effects, name, span);
> if callee.effects.iter().any(|e| e == "Async") {
>     if let Some(calls) = self.async_calls.as_deref_mut() {
>         calls.insert(span);
>     }
> }
> ```
> このパターンは Async 呼び出しの複数箇所(名前呼び出し・フィールドメソッド呼び出し等)で明示的に反復するが、各箇所は数行で読めるし、共通ヘルパのシグネチャを膨らませない。将来別のエフェクト(例: `Async.Retry`)で同種の span 集合が必要になっても、同じ「呼び出し側で判定 → span 集合に insert」パターンを繰り返す。

## PR #127: feat: M37 uses Async エフェクトと async 関数コア — 2026-07-09 merged(hook 再発火・重複)

(no new design-decision candidates for this PR — already covered above)

> **Note**: 本 hook 発火は `gh pr merge` ではなく、PR #127 マージ **後** の状態確認
> `gh pr view 127 --json state,mergedAt ...; git checkout main -q; git pull --rebase ...`
> という main 同期 Bash コマンドの PostToolUse でトリガーされた
> (`tool_input.command` に `gh pr merge` を含まないため、prompt のフォールバックで
> 最新 merged PR = #127 を再選択)。PR #127(M37 uses Async エフェクトと async 関数コア)の
> 設計判断は既に直前セクション 6 件で網羅済み(Async の IO 非包含 / OpSpans 経由の
> checker 権威化 / KEI-E3008 combinator 拒否 / await の needs_paren / 契約 IIFE の
> async 化 / `is_direct_call` フラグ排除の抽象境界)。**新規に追加すべき候補はない**。
>
> tool_response からは:
> - `gh pr view 127` が `MERGED 2026-07-09T13:07:06Z` を返し、直前の PR #127 セクションの
>   `merged` 日付と一致していることを確認できる(重複追記していないという証跡)。
> - `git pull --rebase` 前に `docs/dev-notes/handoff-candidates.md` と `lessons-from-reviews.md`
>   がワークツリーで変更されていたため、`git add docs/dev-notes/ && git commit ... && git pull --rebase`
>   というリカバリ経路が回った(コミット `de6c472 chore(dev-notes): record post-merge hook output for PR #127`)。
>   これは post-merge-handoff hook 自身の追記結果を親セッションが手動でコミットしたもので、
>   本 hook prompt の「Do NOT commit, push, or stage anything」規則には抵触しない
>   (**hook 内**での禁止であって、**親セッションのユーザ操作**は自由という切り分け)。
>
> メタ観察(前 PR #126 の再々掲): PR #127 でも `gh pr merge` 以外の Bash PostToolUse で
> hook が発火する現象が継続している。ただし今回は「マージ直後の main 同期」という
> **PR #127 に強く関連する文脈** での発火であり、無関係な cargo/kei 実装コマンドとは
> 質が異なる。それでも `.claude/settings.json` の matcher が `gh pr merge` のみを
> キャプチャするよう絞る運用改善は依然として未実施のまま。hook 運用改善タスクとして
> 明示的に切り出す必要がある事実を再々度記録する(handoff-candidates.md 側で
> 対処するものではないため、hook 側の運用改善は本ファイルの候補には昇格させない)。


## PR #127: feat: M37 uses Async エフェクトと async 関数コア — 2026-07-09 merged(hook 三度目の誤発火)

(no new design-decision candidates for this PR — already covered above, twice)

> **Note**: 本 hook 発火も `gh pr merge` ではなく、`async_greet.kei` フィクスチャ(uses Async +
> ensures 違反)の `cargo run ... check --json` / `... emit` 検証コマンドの PostToolUse で
> トリガーされた(`tool_input.command` に `gh pr merge` を含まないため、prompt のフォールバックで
> 最新 merged PR = #127 を三度目の再選択)。`gh pr list --state merged --limit 3` を確認しても
> #127 より新しい merged PR は存在せず、設計判断は直前 2 セクションで既に網羅済みのため
> 新規追加なし。
>
> メタ観察: `.claude/settings.json` の post-merge-handoff hook matcher が `gh pr merge` 以外の
> Bash 呼び出し(今回は無関係な `cargo run` 検証コマンド)でも発火する問題が三度目も再現。
> 前回・前々回のセクションで記録済みの「matcher を `gh pr merge` に絞る運用改善が未実施」
> という事実を再度確認しただけであり、これ以上同じ指摘を繰り返し追記する実益は薄い。
> hook 側の運用改善タスクとして早めに切り出すことを推奨する(本ファイルの候補には
> 昇格させない)。

## PR #127: feat: M37 uses Async エフェクトと async 関数コア — 2026-07-09 merged(hook 四度目の誤発火)

(no new design-decision candidates for this PR — already covered above, three times)

> **Note**: 本 hook 発火も `gh pr merge` ではなく、SKILL.md 用の async サンプルドラフト
> (`skill_async_draft.kei`: `uses Async` な `fetchName` / `greet` / `greetTwo` を含む)を
> `cargo run ... kei check --json` と `kei fmt --check` で検証する Bash コマンドの
> PostToolUse でトリガーされた(`tool_input.command` に `gh pr merge` を含まないため、
> prompt のフォールバックで最新 merged PR = #127 を四度目の再選択)。現在のブランチは
> `feat/m38-async-boundaries` で、`gh pr list --state merged --limit 3` を確認しても
> #127 より新しい merged PR は存在せず、設計判断は直前 3 セクションで既に網羅済みのため
> 新規追加なし。
>
> メタ観察: `.claude/settings.json` の post-merge-handoff hook matcher が `gh pr merge` 以外の
> Bash 呼び出し(今回は SKILL.md ドラフト例の検証という、M37/M38 の実装作業そのものではなく
> ドキュメント作成中の副産物)でも発火する問題が四度目も再現。過去3回のセクションで
> 記録済みの「matcher を `gh pr merge` に絞る運用改善が未実施」という事実を再度確認した
> だけであり、これ以上同じ指摘を繰り返し追記する実益はほぼゼロに近い。hook 側の運用改善
> タスクとして早急に切り出すことを強く推奨する(本ファイルの候補には昇格させない)。

## PR #127: feat: M37 uses Async エフェクトと async 関数コア — 2026-07-09 merged(hook 五度目の誤発火)

(no new design-decision candidates for this PR — already covered above, four times)

> **Note**: 本 hook 発火も `gh pr merge` ではなく、`fx.async_basic` / `fx.async_extern` /
> `fx.async_sequential` という skill 用 async サンプル3種を `kei check --json` /
> `kei fmt --check` で検証する Bash コマンドの PostToolUse でトリガーされた
> (`tool_input.command` に `gh pr merge` を含まないため、prompt のフォールバックで
> 最新 merged PR = #127 を五度目の再選択)。`gh pr list --state merged --limit 3` でも
> #127 より新しい merged PR は存在せず、設計判断は直前 4 セクションで既に網羅済みのため
> 新規追加なし。同一 PostToolUse hook 発火に対応した実装 subagent 自身のトランスクリプトを
> 確認したが、`pbt.rs` / `kei_cli/tests/cli.rs` / `kei_emit/tests/emit.rs` / `kei_mcp/src/tools.rs` /
> `SKILL.md` / spec / golden fixture の実装編集のみで、`gh pr` 系コマンドや
> `handoff-candidates.md` への追記・git add/commit/push は一切行っていないことも確認済み
> (=hook 自身が誤発火文脈で独立に本ファイルへ追記している)。
>
> メタ観察: matcher 修正が5回連続で未実施のまま継続している。今後は同一事実の再確認を
> 都度長文化せず、本ノートのように短く「五度目」「六度目」の見出しと発火元コマンドの
> 一行要約のみに留める運用が妥当(本ファイルの候補には昇格させない)。

## PR #127: feat: M37 uses Async エフェクトと async 関数コア — 2026-07-09 merged(hook 六度目の誤発火)

(no new design-decision candidates for this PR — already covered above, five times)

> 発火元: `git log --oneline main..HEAD; cargo check --workspace --quiet` の PostToolUse
> (M38 async 境界統合の commit 確認)。`gh pr merge` 非該当のため #127 を六度目に再選択。
> 現ブランチ `feat/m38-async-boundaries` の M38 コミット(`bc49dce`, `0b890f4`)は未マージ。
> matcher 未修正が 6 回連続。設計判断は既出セクションで網羅済み。

## PR #128: feat: M38 async 境界統合 — extern async + 契約 e2e + pbt / MCP / SKILL — 2026-07-09 merged

### Candidate: pbt の skipped 可視化は Async 特別扱いを廃して「エフェクト保有 + ensures」の一般則へ
**Why this matters for HANDOFF.md**: 初版(M38 コミット `0b890f4`)は Async だけを skipped に載せていたが、レビュー対応(`b7dc6601`)で「同期評価器で扱えないのは Async に限らず全 `uses` エフェクト」という一般則へ書き換えた経緯。次に新エフェクトを追加する人が「Async だけ特別」というコードを再導入しない invariant として残す価値がある。
**Draft entry** (lift verbatim if approved):
> `kei_check::pbt::run_function` の generative 検証対象外判定は **エフェクト保有(`!f.uses.is_empty()`)** で一般化する — Async 特別扱いは持たない。同期評価器は Promise の resolve を待てないだけでなく、`Database.Read` 等の一般エフェクトも実環境の観測を伴うため純粋評価器の対象外。**ensures を持つ関数だけ** `skipped[]` に「なぜ検証されなかったか」を `reason: "function has 'uses X' effect"` として載せる(捏造不能性 — 反例なし=充足という誤読を防ぐ、spec v0.2 §5.1)。ensures が無ければ他の対象外理由(Map 引数など)と同じく静かに除外する(可視化のノイズを増やさない)。`reason` は先頭の uses エフェクトだけを短く示す(複数 uses でも冗長にしない)。回帰防止テストは `effect_function_with_ensures_is_reported_as_skipped_with_reason`(`uses Database.Read` を代表として固定)。

### Candidate: `SkippedFunction.required_cases` は `Option<usize>` — 「N/A のときは 0」という数値 sentinel を型に混ぜない(PR #112 教訓の再適用)
**Why this matters for HANDOFF.md**: PR #112 の教訓「N/A を数値 sentinel(0 や `usize::MAX`)で表さない」が M38 初版で再発しかけ、レビューで根本の `pbt::SkippedFunction` から `Option<usize>` 化した。MCP 応答 JSON の後方互換性(既存 golden への影響)も含めて非自明な判断が必要で、次に skipped 情報を拡張する人が同じ穴を踏まないための landmine。
**Draft entry** (lift verbatim if approved):
> `kei_check::pbt::SkippedFunction.required_cases` は `Option<usize>` にする。**上限超過が理由のスキップのときだけ `Some(必要ケース数)`**、エフェクト保有などケース数と無関係な理由のときは `None`(「N/A のときは 0」という数値 sentinel を型に混ぜない — PR #112 の教訓)。MCP 応答 JSON 側(`kei_mcp::tools::SkippedInfo`)は `#[serde(skip_serializing_if = "Option::is_none")]` で JSON からキー自体を省略する(既存の「上限超過だけをスキップ理由に持つ」golden との後方互換を維持)。`reason: Option<String>` も同じ規約で載せる。既存の usize::MAX(積が overflow するほど巨大なケース)は "上限超過の極端な場合" として `Some(usize::MAX)` を維持する(sentinel ではなく「途方もなく大きい」ことを伝える正当な値)。

### Candidate: `EffectRef::dotted()` は AST の唯一の合流点 — 呼び出し側で `path.iter().map(...).join(".")` を書かない
**Why this matters for HANDOFF.md**: レビューで検出された「同じロジックが check.rs 2 箇所と pbt.rs 1 箇所に散らばっていた」問題への恒久ガード。将来 EffectRef の内部表現が変わっても(例: 区切り文字を `::` に変更、あるいは `Vec<Ident>` を `SmallVec` に変更)、変更点が 1 箇所に閉じる。
**Draft entry** (lift verbatim if approved):
> `EffectRef` を dotted 文字列(`Database.Write` 等)に落とすロジックは **`kei_syntax::ast::EffectRef::dotted()` が唯一の場所**。呼び出し側で `u.path.iter().map(|i| i.name.as_str()).collect::<Vec<_>>().join(".")` を書き直さない(check.rs / pbt.rs 全て `u.dotted()` を経由する)。将来 EffectRef の内部表現が変わっても変更点が 1 箇所に閉じるだけでなく、区切り文字を将来 `::` に変えたいような仕様変更も dotted() の実装差し替えだけで済む。同種のヘルパを他の AST ノード(Path, Ident 列など)で書くときも、この「AST 型に impl を生やして呼び出し側の join を根絶する」パターンに揃える。

### Candidate: 内部 skip 状態は `Option<SkipInfo>` に named field 化 — 無名 tuple の field 順ミラー同期は禁止
**Why this matters for HANDOFF.md**: PR #128 レビューで検出された「`Option<(usize, Option<String>)>` の 2-tuple が `SkippedFunction` の 2 field を鏡写しし、write site 3 箇所・destructure 1 箇所で field 順ズレのリスクがあった」問題への恒久ガード。次に skip 情報を拡張する人が「tuple で軽く済ませる」誘惑に負けないための landmine。
**Draft entry** (lift verbatim if approved):
> `run_function` が呼び出し元に渡す skip 内訳は **`SkipInfo` という named field 構造体** に閉じる(`Option<(usize, Option<String>)>` のような無名 tuple を作らない)。理由: 無名 tuple は write site 3 箇所・destructure 1 箇所で `SkippedFunction` の 2 field を鏡写しにするため、片方だけ field 順を入れ替えたときにコンパイラが警告しない(silent bug になる)。`SkipInfo` は crate 内部専用の struct で公開しない(外部 API は `SkippedFunction` のまま)。将来 skip 情報が 3 field 以上に増えたら、`SkipInfo` の field を追加するだけで write site を機械的に追随できる。

### Candidate: async 契約 e2e は npm パッケージも file: 依存の実 Promise で疎通 — M36 greeter パターンの async 版
**Why this matters for HANDOFF.md**: PR #128 が新設した `tests/cli/packages/async-greeter/` は「実 npm パッケージ経由で extern async を tsc --strict + vitest で疎通する」M38 完了条件の最小固定点。M36 で確立した「file: 依存の実 npm パッケージで extern の疎通を固定」パターンを async 版で再適用しており、次に extern の新側面(例: streaming, generators)を追加する人が同じ流儀で fixture を作れるようにするための invariant。
**Draft entry** (lift verbatim if approved):
> extern 境界の e2e 検証は **`tests/cli/packages/<name>/` に file: 依存の実 npm パッケージを配置** して行う(M36 の greeter パターン)。async 版は `async-greeter` として `index.js` が実 `async` 関数(実 Promise を返す)、`index.d.ts` で型を宣言、`tests/cli/projects/app/package.json` に `"async-greeter": "file:../packages/async-greeter"` を追加する。Kei 側からは `extern package "async-greeter" as ...` で束縛し、`extern ...uses Async` の署名を tsc --strict + vitest で検証する。この流儀は (a) tsc --strict の型検査を通ることで extern 署名が真に外部型と合っていることを固定できる、(b) vitest から `await expect(...).rejects.toThrow(KeiContractViolation)` で contract violation の実 Promise 経由伝播を捕捉できる、(c) `build_golden_tree` の `wrote N module(s)` カウントで golden として一体固定できる、という 3 つの効用がある。新しい extern 側面(例: streaming)を追加するときも `tests/cli/packages/<新 fixture>/` を同じ形で作る。

### Candidate: `extern query` は Async 化不可 — 純粋観測子のシルエットを既存 KEI-E3005 で守る(新規診断を作らない)
**Why this matters for HANDOFF.md**: 前 PR #126 で「Async 化禁止」の設計判断は記録済み(候補 4)だが、PR #128 でその**実装形態**が確定した — 新規診断コードを作らず、既存の「query は uses を宣言できない」KEI-E3005 に自然に包含された。診断コードを増やさずに invariant を守る好例で、次に「query に別の新規制約を足したい」場面で参考になる。
**Draft entry** (lift verbatim if approved):
> `extern query ... uses ..., Async` は **既存 KEI-E3005(query cannot declare uses)がそのまま拒否する** — M38 で新規診断コードを追加していない。理由: query は「純粋観測子」のシルエットを守るため、そもそも uses 節を持てない仕様(spec v0.2 §2.5)。Async は uses の要素なので、query に Async を付けようとした瞬間に uses 節の存在自体が E3005 で拒否される。fix hint は既存の「query は純粋観測子、通常の extern を使え」のまま流用できる。golden `err_extern_query_async.expected.json` はこの経路を固定する — 新規診断コードを作らずに新しい invariant を守れる好例として、次に query の制約を足すときは「既存の E3005 が拾ってくれる形にできないか」を先に検討する。

## PR #129: chore: bump version to 0.7.0 — 2026-07-09 merged

(no design-decision candidates for this PR)

## PR #129 (re-fire): chore: bump version to 0.7.0 — 2026-07-09 merged

> **Note**: hook が `gh release edit v0.7.0 ... && mkdir dogfood scratchpad` Bash
> コマンド(PR merge ではない)に対して再発火した。`tool_input.command` に
> `gh pr merge <N>` が含まれないため fallback で「最新 merged PR = #129」を
> 選択したが、実体は release ノート編集 + ドッグフード環境構築であり、内容は
> 上の #129 セクションと同じ(pure version bump)。

(no design-decision candidates for this PR)

