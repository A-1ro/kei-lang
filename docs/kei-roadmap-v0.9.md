# Kei 開発ロードマップ v0.9 — /goal 契約書集(Workers テンプレート & デプロイ)【叩き台・🤝 未合意】

> 運用ルール: 各 Milestone は「人間が合意する契約」。完了条件は機械検証可能な形で書く。
> 本ファイルは v1.0 戦略(`docs/kei-roadmap-v0.5.md` 冒頭)の **v0.9: Workers テンプレート & デプロイ**
> — v1.0 受け入れ基準への**最後の一歩** — を Milestone 化したもの。
> **着手前に本ファイルの「設計原則」節を熟読すること。**
> **本ファイルは叩き台であり、各 Milestone の 🤝 事前合意事項が人間と確定するまで実装に着手しない。**

## v0.9 のゴール

`examples/workers-api/` に置いた `.kei` ソース群から `kei build` で生成した TS が、
Hono ルーティング + `@kei/hono` アダプタ経由で **wrangler のビルド(esbuild bundling)に通り**、
`wrangler dev` で起動した Workers ランタイムに対して HTTP を叩くと:

1. 正常系がステータス 200/202 で応答し、
2. **契約違反(requires/ensures)が 500(または 🤝 で合意したステータス)として観測できる**

こと。v0.5 の逆算表の定義どおり「`examples/workers-api` 雛形 + wrangler 統合 e2e
(`wrangler dev` 起動 → HTTP 叩き → 契約違反がエラーレスポンスで観測できる)」が受け入れ基準。

**実 Cloudflare へのデプロイ(`wrangler deploy` 本番実行)と、kei-dogfood による初見エージェント実証は
v1.0 の領分**(スコープ外節を参照)。v0.9 はローカルの workerd(wrangler dev)までで閉じる。

バージョン運用: 本ファイルの Milestone が全て閉じた時点で **0.9.0** をタグする(v0.5 と同じ運用)。

## v0.9 の性格 — 「言語機能変更は原則ゼロ、境界の拡張だけ」の版

v0.8 と同じく、**言語コア(syntax / check / emit の意味論)の変更は原則行わない**。
v0.9 の主戦場は:

- **エコシステム側**: Workers テンプレート、wrangler 統合、e2e。
- **アダプタ層(`@kei/hono`)の拡張**: パスパラメータ抽出・依存注入経路(issue #136)。
  拡張は HttpRequest record のフィールド追加やハンドラ wrapper のパターン追加であり、
  Kei 言語仕様には触らない — **ただし issue #136 の選択肢 (c)(String stdlib 段階2)を採る場合のみ
  例外的に言語 stdlib に触る**。どの選択肢を採るかは M41 の 🤝 で確定する(勝手に決めない)。

## 設計原則(HANDOFF 準拠 — 実装エージェントに絶対に破らせない)

v0.8 の原則 1〜6 をすべて引き継いだ上で、v0.9 固有の原則を追加する。

1. **【v0.8 引継ぎ】言語機能追加は最小限、原則ゼロ。複雑さはアダプタ層(`@kei/hono`)で吸収する。**
   Hono の Context / Workers の Request/Response を Kei 型システムに直接引き込まない。
   Kei ハンドラは `(req: HttpRequest) -> HttpResponse`(+ 必要なら `uses Async`)の
   純粋な関数型シグネチャのまま。

2. **【v0.8 引継ぎ】JSON 境界は「record 型ごとの専用 extern」。** 汎用アダプタ(`@kei/hono` の
   `parseAs<T>`)にアプリ固有関数を混ぜない。アプリ固有 wrapper は file: ローカルパッケージ
   (M39 の `parse-app` 方式)で提供する。

3. **【v0.9 新規】Workers bindings / env は Kei に露出させない。TS エントリポイントで受けて record に写す。**
   - `env`(KVNamespace / D1Database / Secrets / vars)・`ExecutionContext` は **TS 側の
     Workers エントリポイント(`src/index.ts` の `fetch(request, env, ctx)`)だけが触る**。
   - Kei ハンドラに渡すものは、env から読み出した値を詰めた **平坦な record**(または
     ハンドラを closure で wrap する TS 側パターン — issue #136 の DI 選択肢 (b))に限る。
   - KV/D1 への読み書きを Kei から行いたくなった場合も、v0.9 では **extern 経由の TS wrapper**
     で表現する(bindings 型そのものを extern 署名に書かない)。Kei のエフェクトシステムが
     把握できない副作用の抜け道を作らない。

4. **【v0.9 新規】デプロイ物の責務分担を固定する。**
   - **Kei の責務**: API のビジネスロジック(ハンドラ関数・record・契約)。`kei build` の出力(dist/)。
   - **TS(テンプレート)の責務**: Workers エントリポイント(`export default { fetch }`)、
     Hono app の組み立て、`mount()` によるルート登録、契約違反の中央処理(`app.onError`)、
     env → record の写像。
   - **wrangler の責務**: bundling(esbuild)と workerd 起動。`wrangler.toml`(または .jsonc)は
     テンプレートの一部として人間がレビューできる形で置く。
   - この 3 層の境界を曖昧にする変更(例: kei_cli に wrangler 呼び出しを組み込む)は v0.9 では行わない。

5. **【v0.9 新規】疎通リスクは最初に潰す(M40 が先頭)。** v0.8 契約書が明記した
   「file: 依存 × wrangler(esbuild) bundling は技術的に妥当だが未検証」のリスクを、
   テンプレート作成や #136 の設計より**先に**回収する。ダメだった場合の分岐(npm 公開前倒し /
   テンプレート内コピー)も M40 の契約に含め、後続 Milestone が手戻りしない順序にする。

6. **【v0.8 引継ぎ】契約と async の意味論を変えない。** 契約式は同期・純粋のまま。
   Workers 上でも `KeiContractViolation` の throw 位置・shape は既存互換。

7. **【v0.9 新規】wrangler 依存のテストは既存 CI(cargo test)を人質に取らない。**
   `wrangler dev` は Node・ネットワーク(初回の workerd バイナリ取得)・ポートに依存する。
   既存の `cargo test --workspace` 全パスという契約を壊さない形(分離ジョブ or ローカル専用)を
   M43 の 🤝 で先に確定してから実装する。

## Milestone 全体像と順序

M 番号は v0.8(M39)からの連番。

| M | テーマ | issue | 優先度 | 状態 | 主な成果物 |
|---|---|---|---|---|---|
| **M40** | file: 依存 × wrangler(esbuild) bundling の疎通確認 | (v0.8 契約書の明記リスク) | high(最優先) | 🤝 未着手 | 疎通レポート + 分岐判断 |
| **M41** | HTTP アダプタ実配線 — パスパラメータ抽出 + 依存注入経路 | #136 | high 🤝 | 🤝 未着手 | `@kei/hono` 拡張 + e2e |
| **M42** | `examples/workers-api` 雛形テンプレート | — | high | 🤝 未着手 | examples/workers-api/ + SKILL 節 |
| **M43** | wrangler 統合 e2e(契約違反の観測) | — | high 🤝 | 🤝 未着手 | e2e + CI 方針 |

順序の論拠:

- **M40 が先頭・単独。** 疎通が NG だった場合、M41〜M43 の成果物の置き場所(file: パッケージのまま
  か、npm 公開か、テンプレート内コピーか)が全部変わる。分岐判断を先に済ませる。
- **M41 は M42 の前。** テンプレート(M42)は #136 の設計(HttpRequest の shape・DI パターン)を
  前提にするため、アダプタの形を先に固める。
- **M43 が最後。** テンプレートと配線が揃ってから、wrangler dev での通し検証と CI 方針を閉じる。

## M40: file: 依存 × wrangler(esbuild) bundling の疎通確認

v0.8 契約書「設計原則 3」が明記した先送りリスクの回収。**これが v0.9 の最初の作業**。

### 事前合意事項(🤝、着手前に確定)

- **疎通確認の判定コマンド**: `wrangler deploy --dry-run --outdir <dir>`(ネットワーク不要の
  bundling のみ)を疎通判定に使う想定。`wrangler dev` 起動まで確認するかは M43 に送るか、
  ここで一緒にやるか(推奨: dry-run + `wrangler dev` への簡易 fetch まで確認して M43 の手戻りを防ぐ)。
- **NG だった場合の分岐(どちらを既定にするか)**:
  - **(a) npm 公開の前倒し**: `@kei/runtime` / `@kei/hono` を npm に publish する。
    影響: v0.8 契約書の「npm 公開はしない(v1.0 以降で検討)」の前倒し変更。公開手順・
    バージョニング・スコープ(`@kei/*` は確保済み — HANDOFF 参照)の運用が発生。
  - **(b) テンプレート内コピー**: `examples/workers-api/` に `@kei/hono`(必要なら
    `@kei/runtime` も)のソースを直接コピーして同梱する。
    影響: 二重管理(tests/cli/packages/kei-hono との乖離リスク)。同期手段(コピースクリプト or
    「テンプレート側が正」の宣言)を決める必要。
  - どちらも一長一短のため、**NG が確定した時点で症状(esbuild のエラー内容)を添えて人間に
    判断を仰ぐ**。実装エージェントが勝手に選ばない。

### 完了条件(機械検証可能)

- 検証用プロジェクト(M42 の前身。`examples/workers-api/` の骨組みでよい)に
  `wrangler.toml` + TS エントリポイント + `@kei/hono` への file: 依存を置き、
  `npm install && npx wrangler deploy --dry-run --outdir dist-wrangler` が **exit 0** で完走する。
  コマンドと出力(bundle 成功ログ)を実行結果として表示する。
- 生成された bundle に Kei ハンドラのシンボル(例: `handleHealth`)と `@kei/hono` の
  `mount` 相当が含まれることを grep 等で確認し、結果を表示する。
- 疎通 OK の場合: 結果を本ファイルの M40 節に「実施記録」として追記し(どの wrangler
  バージョンで確認したかを明記)、M41 に進む。
- 疎通 NG の場合: esbuild のエラー内容・原因分析・分岐 (a)/(b) の推奨案を添えて**停止・報告**する
  (🤝)。人間の判断が出るまで M41 以降に着手しない。

### golden / test 設計方針

- この Milestone は探索的スパイクなので golden は追加しない。恒久的なテスト化は M43 の領分。
- ただし検証用プロジェクトのファイル群は M42 で再利用できる形で置く(使い捨てにしない)。

### スコープ外(M40)

- 実デプロイ(`wrangler deploy` 本番)・Cloudflare アカウント設定。
- minify / tree-shaking の最適化検証。bundle が動くことだけ確認する。

## M41: HTTP アダプタ実配線 — パスパラメータ抽出 + 依存注入経路(#136、🤝)

v0.8.0 ドッグフード(`docs/dogfood/2026-07-10-v0.8.0-stock-api-hono.md`、8/10 の減点理由)で
明確化されたギャップ 2 件。**選択肢はどれも設計判断を含むため、着手前に人間と合意する**。

### 事前合意事項(🤝、着手前に確定 — issue #136 の選択肢を転記)

**1. パスパラメータ抽出**(現状: HttpRequest は `path: String` の生フルパスのみ。Kei 側で
`/stock/42` から `"42"` を切り出せない):

- **(a)** `mount()` の handler シグネチャを拡張: `(req, params: Map<String, String>) => ...`
  - 論点: Kei ハンドラの引数が 2 つになり、v0.8 の「単一引数の純粋な関数型シグネチャ」から変わる。
- **(b)** HttpRequest に `pathParams: Map<String, String>` フィールドを追加(`mount()` 側で
  Hono の `c.req.param()` から埋める)
  - 論点: 境界層で情報を保存する原則(PR #132 レビューと同じ方向)に沿う。既存ハンドラの
    record 構築(golden / e2e の HttpRequest リテラル)に破壊的変更が入るため追従範囲を確認。
- **(c)** Kei 側 String stdlib に `split(delimiter)` / `indexOf` を追加(v0.5 M30 の伏線
  「本格 String API は段階2で」の回収)
  - 論点: 唯一の言語 stdlib 変更。spec 更新 + pbt 追従が必要。(b) と排他ではなく補完。
- issue #136 の推奨は **(b) + 部分的に (c)** だが、**本契約書では決めない**。(c) を v0.9 に
  含めるか v1.x に送るかも含めて合意する。

**2. 依存注入(在庫マスタ・KV・D1 等)**(現状: handler は `(req: HttpRequest) -> HttpResponse`
の単一引数のみで、外部依存を渡す経路が未定義):

- **(a)** `mount()` の handler シグネチャを拡張: `(req, deps: Deps) => ...`(Deps はアプリごとに record 定義)
  - 論点: アダプタが「Deps とは何か」を知る必要が出る(generic 化するか、mount の変種を増やすか)。
- **(b)** TS 側で closure でハンドラを wrap してから mount する(現状の `@kei/hono` で可能な範囲。
  Kei ハンドラを `(deps) => (req) => handler(deps, req)` 的に部分適用する TS パターン)
  - 論点: 言語機能追加ゼロ・アダプタ変更最小で「Kei らしい」。ただし Kei ハンドラ側は
    `handler(deps: Deps, req: HttpRequest)` の 2 引数関数になる(Kei はカリー化を持たないため
    部分適用は TS 側で行う)。SKILL.md に定型を示す必要。
- **(c)** Workers bindings は TS 側で拾って record に写して Kei に渡す(設計原則 3 と同じ。
  (a)/(b) のどちらとも組み合わせ可能)
- issue #136 の推奨は **(b) の TS wrapper パターンを SKILL.md で示す形**だが、これも本契約書では
  決めない。

**3. 契約違反 → HTTP ステータスの写像**(v0.5 逆算表は「契約違反が **500** で観測できる」、
一方 v0.8 M39 の e2e は `app.onError` で **400** に写した。両立の整理が必要):

- **(a)** すべて 500 に統一(契約違反 = サーバ側の不変条件破れ、という立場)
- **(b)** requires 違反 = 400(呼び出し側の入力不正)/ ensures 違反 = 500(実装のバグ)に分離
  (`KeiContractViolation.clause` で判別可能)
- **(c)** v0.8 の 400 を維持し、v0.5 逆算表の「500」の記述を追認修正する
- 推奨は挙げない(intent-align の観点で「契約違反を誰の責任として観測させるか」は人間の判断)。
  合意結果を M43 の e2e 期待値(= 受け入れ基準)に反映する。

### 完了条件(機械検証可能 — 合意結果で確定させる叩き台)

- 合意した設計で `@kei/hono`(+ 必要なら `@kei/runtime` / spec / stdlib)を拡張し、
  パスパラメータを使う Kei ハンドラ(例: `GET /stock/:sku` — ドッグフードの在庫 API 再現)と
  依存注入を使う Kei ハンドラの e2e を `tests/cli/projects/app/` に追加。
  `cargo test --workspace` 全パス(コマンド実行と結果表示を含む)。
- (c) String stdlib を含める合意になった場合のみ: spec 更新(spec-first)+ pbt.rs 追従 +
  syntax/check/fmt/emit golden 追加。
- SKILL.md「HTTP ハンドラを書く」節に、パスパラメータと依存注入の定型を追記
  (SKILL 更新に伴う MCP golden 再生成を含む)。
- issue #136 を PR の `Closes #136` でクローズする(不変条件 6)。

### golden / test 設計方針

- アダプタ層の変更が主なら golden 追加は最小(HttpRequest record のフィールドが増える場合、
  既存 `http_model.kei` 系 golden の expected 変更は人間レビュー必須 — 不変条件 1)。
- e2e は M39 と同じ流儀: `kei build` → `tsc --strict` → vitest で Hono の `app.request(...)`。
  wrangler は使わない(M43 の領分)。

### スコープ外(M41)

- KV / D1 への実接続(v0.9 では DI 経路の設計まで。実バインディング統合は v1.x)。
- Set-Cookie を含む複数値ヘッダー(v0.8 から継続の制限)。
- ミドルウェア(認証・CORS 等)の Kei 表現。TS 側の Hono ミドルウェアで足りる。

## M42: `examples/workers-api` 雛形テンプレート

### 事前合意事項(🤝、着手前に確定)

- **テンプレートに含める API の題材**: 最小(health + users の M39 再演)か、ドッグフード在庫 API
  (パスパラメータ + DI を実演)か。推奨は後者(M41 の成果を雛形で示す)だが分量と相談。
- **`@kei/hono` の参照方法**: M40 の疎通結果に従う(file: / npm / 同梱コピー)。
- **wrangler 設定の形式**: `wrangler.toml` か `wrangler.jsonc` か(現行 wrangler の推奨に合わせる)。

### 完了条件(機械検証可能)

- `examples/workers-api/` に以下を追加:
  - `.kei` ソース群(ハンドラ + record + 契約。`kei fmt --check` クリーン — 不変条件 5)
  - TS エントリポイント(`src/index.ts`: `export default { fetch }` + Hono app 組み立て +
    `mount()` 登録 + 契約違反の中央処理)
  - `wrangler.toml`(または合意した形式)・`package.json`・`tsconfig.json`
  - `README.md`(ビルド → wrangler dev → curl 確認の手順。デプロイ手順は v1.0 送りと明記)
- `kei build examples/workers-api`(相当のディレクトリ指定)が exit 0、生成 TS が `tsc --strict` を
  通り、`wrangler deploy --dry-run` が exit 0。3 コマンドの実行と結果を表示する。
- examples/ は kei_mcp にビルド時埋め込みされる(不変条件 3)ため、MCP golden の再生成と
  `cargo test --workspace` 全パスを確認する。**埋め込み対象に workers-api を含めるかは実装時に
  確認**(TS/toml 混在ディレクトリの扱いが既存の examples 埋め込み機構と整合するか。整合しない
  場合は埋め込み除外の判断を報告 — 勝手に機構を改造しない)。
- SKILL.md に「Workers にデプロイする(v0.9)」節を追加(テンプレートの使い方・責務分担・制限)。

### golden / test 設計方針

- テンプレートの `.kei` は examples/ 配下なので正規形維持(fmt --check)を CI 対象にする
  (既存 examples の検査経路に乗せる)。
- 生成 TS の固定は M39 同様 expected 方式を使うかは実装時判断(テンプレートは動く見本が主目的。
  bundling までは M43 の e2e で担保するため、二重に golden 化しない方向を推奨)。

### スコープ外(M42)

- `kei new` / scaffolding コマンド(CLI へのテンプレート生成機能追加は v1.x で実需確認後)。
- 複数テンプレート(REST 以外の WebSocket / cron trigger 等)。

## M43: wrangler 統合 e2e — 契約違反の観測(🤝)

### 事前合意事項(🤝、着手前に確定)

- **CI での扱い**(設計原則 7。`wrangler dev` は Node・初回の workerd バイナリ取得(ネットワーク)・
  ポート確保に依存するため、既存 `cargo test --workspace` に無条件で混ぜられない):
  - **(a) ローカル専用**: e2e スクリプト(npm script)として置き、CI では実行しない。
    README / SKILL に「リリース前に手元で回す」手順を明記。
    - 論点: 機械検証が人間の運用頼みになり、v0.9 受け入れ基準の担保が弱い。
  - **(b) CI ジョブを分離**: 既存 fmt / clippy / test と並ぶ第 4 ジョブ `workers-e2e` を追加し、
    そこでだけ wrangler dev を回す(GitHub Actions はネットワーク可・Node 22 セットアップ済みの
    流儀がある)。flaky 時の扱い(required にするか)も決める。
    - 論点: CI 時間増・wrangler バージョン固定の保守。ただし受け入れ基準が CI で固定される。
  - **(c) `@cloudflare/vitest-pool-workers`(workerd をプロセス内起動)で vitest に統合**:
    `wrangler dev` のポート待ちが不要になり flaky 要因が減る。
    - 論点: 「`wrangler dev` 起動 → HTTP 叩き」という v0.5 の文言との等価性(workerd 上で
      動く点は同じ)。追加 devDependency の保守。
  - 推奨は **(b) または (c)**(受け入れ基準を CI に固定できるため)だが、コスト判断を含むので合意制。
- **契約違反の期待ステータス**: M41 の 🤝(3)の合意結果を使う。
- **wrangler のバージョン固定方針**: package.json に devDependency として固定(^ か pin か)。

### 完了条件(機械検証可能 — CI 方式の合意で確定させる叩き台)

- `examples/workers-api/`(M42)を対象に、合意した方式で以下を自動検証する e2e を追加:
  1. `kei build` → `wrangler dev`(または vitest-pool-workers)起動。
  2. 正常系: `GET /health` が 200、POST 系が 202、パスパラメータ系(M41 採用時)が正しい値を返す。
  3. **契約違反系: requires を破るリクエストを送り、合意したステータス(500 等)と
     `KeiContractViolation` 由来のエラー body が観測できる。**
  4. JSON parse 失敗(業務ロジックの 400)と契約違反の経路が**別物として**区別して検証される
     (M39 の onError 設計を継承)。
- 合意が (b)/(c) の場合: CI が green になった実行ログ(該当ジョブ)を確認し結果を表示する。
  合意が (a) の場合: ローカル実行コマンドと全ケース pass の出力を表示し、実行手順を README に固定する。
- `cargo test --workspace` は従来どおり wrangler なしで全パスすること(既存 CI を壊さない)。
- 完了時点で v0.9 の受け入れ基準(本ファイル冒頭)を満たしたことを、コマンド出力つきで報告する。

### golden / test 設計方針

- HTTP レスポンスの検証は「ステータス + body の構造(JSON キー)」を assert する。body 全文の
  スナップショット固定は wrangler / Hono のバージョン差で壊れやすいため避ける。
- 契約違反レスポンスの shape(`error` / `clause` / `condition`)は M39 の形式を正とし、
  変える場合は人間レビュー(不変条件 1 の精神)。

### スコープ外(M43)

- 実 Cloudflare へのデプロイと本番 URL での検証(v1.0)。
- 負荷・レイテンシ計測、Workers の制限(CPU 時間等)検証。
- wrangler の secret / KV バインディング実配線(DI 経路の設計は M41、実接続は v1.x)。

## v1.0 受け入れ検証はスコープ外(明示)

以下は **v0.9 に含めない**。v0.9 全 Milestone 完了後、v1.0 の契約書集を別途新設して扱う:

- 実 API を Kei で記述して Cloudflare Workers に**実デプロイ**すること(wrangler deploy 本番)。
- **kei-dogfood による初見実証**(初見エージェントが SKILL.md だけで Workers API を書けることの実証)。
- npm 公開の恒久判断(M40 の分岐で前倒しになった場合を除く)。
- v1.0 タグ付け・リリースノート・README の全面更新。

## 後続 /goal ドラフト(🤝 合意後に確定)

```text
/goal M40: examples/workers-api の骨組み(wrangler.toml + TS エントリポイント + @kei/hono への
file: 依存)を作り、npx wrangler deploy --dry-run --outdir dist-wrangler が exit 0 で完走し、
bundle に Kei ハンドラと mount が含まれることを確認して結果を表示する。NG の場合は esbuild の
エラー内容と分岐案(npm 公開前倒し / テンプレート内コピー)を添えて停止・報告する(🤝)。
```

```text
/goal M41: (🤝 合意結果を反映して確定)issue #136 のパスパラメータ抽出と依存注入経路を実装する。
合意した設計で @kei/hono を拡張し、GET /stock/:sku 相当の e2e を追加、SKILL.md に定型を追記して
MCP golden を再生成する。cargo test --workspace を通して結果を表示し、PR で Closes #136 する。
```

```text
/goal M42: examples/workers-api 雛形を追加する。kei build → tsc --strict → wrangler deploy --dry-run
の 3 コマンドが exit 0 で通ることを実行結果つきで示し、.kei は kei fmt --check クリーン、
SKILL.md に「Workers にデプロイする」節を追加、MCP golden を再生成して cargo test --workspace を通す。
```

```text
/goal M43: (🤝 CI 方式の合意結果を反映して確定)examples/workers-api に対する wrangler 統合 e2e を
追加する。正常系の 200/202 と、requires 違反が合意したステータスで観測できることを自動検証し、
合意した CI 方式(分離ジョブ / vitest-pool-workers / ローカル専用)で固定する。
cargo test --workspace は wrangler なしで従来どおり全パスすること。
```
