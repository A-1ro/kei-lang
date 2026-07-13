# Kei 開発ロードマップ v0.9 — /goal 契約書集(Workers テンプレート & デプロイ)

> 運用ルール: 各 Milestone は「人間が合意する契約」。完了条件は機械検証可能な形で書く。
> 本ファイルは v1.0 戦略(`docs/kei-roadmap-v0.5.md` 冒頭)の **v0.9: Workers テンプレート & デプロイ**
> — v1.0 受け入れ基準への**最後の一歩** — を Milestone 化したもの。
> **着手前に本ファイルの「設計原則」節を熟読すること。**

## 更新履歴

- 2026-07-11: 叩き台を起草(PR #138)。全 Milestone の事前合意事項を 🤝 として列挙。
- 2026-07-13: **🤝 6 点すべて合意済み**(疎通判定は wrangler dev + fetch まで / パスパラメータは
  pathParams 追加 + String stdlib 部分拡張 / 依存注入は TS closure wrap / 契約違反は**全部 500**
  (M39 の 400 は修正対象)/ M42 題材は在庫 API・設定は jsonc / M43 は vitest-pool-workers で
  CI 常設)。各 Milestone の合意節・/goal 文を確定版に更新。

## v0.9 のゴール

`examples/workers-api/` に置いた `.kei` ソース群から `kei build` で生成した TS が、
Hono ルーティング + `@kei/hono` アダプタ経由で **wrangler のビルド(esbuild bundling)に通り**、
`wrangler dev`(および vitest-pool-workers の workerd)で起動した Workers ランタイムに対して
HTTP を叩くと:

1. 正常系がステータス 200/202 で応答し、
2. **契約違反(requires/ensures)が 500 として観測できる**(✅ 合意済み — 「契約が破られた時点で
   サーバ異常」という思想。v0.5 逆算表の記述(500)が正)

こと。v0.5 の逆算表の定義どおり「`examples/workers-api` 雛形 + wrangler 統合 e2e
(`wrangler dev` 起動 → HTTP 叩き → 契約違反が 500 で観測できる)」が受け入れ基準。

**実 Cloudflare へのデプロイ(`wrangler deploy` 本番実行)と、kei-dogfood による初見エージェント実証は
v1.0 の領分**(スコープ外節を参照)。v0.9 はローカルの workerd(wrangler dev / pool-workers)までで閉じる。

バージョン運用: 本ファイルの Milestone が全て閉じた時点で **0.9.0** をタグする(v0.5 と同じ運用)。

## v0.9 の性格 — 「言語機能変更は原則ゼロ、境界の拡張だけ」の版

v0.8 と同じく、**言語コア(syntax / check / emit の意味論)の変更は原則行わない**。
v0.9 の主戦場は:

- **エコシステム側**: Workers テンプレート、wrangler 統合、e2e。
- **アダプタ層(`@kei/hono`)の拡張**: パスパラメータ抽出・依存注入経路(issue #136)。
  拡張は HttpRequest record のフィールド追加やハンドラ wrapper のパターン追加が中心。
- **唯一の言語 stdlib 変更(✅ 合意済み)**: issue #136 の選択肢 (b)+(c) 採用に伴い、
  **String stdlib 段階2の部分拡張(`split(delimiter)` / `indexOf`)を M41 で行う**
  (v0.5 M30 スコープ外の伏線「本格 String API は段階2で」の部分回収)。spec-first で扱う。

## 設計原則(HANDOFF 準拠 — 実装エージェントに絶対に破らせない)

v0.8 の原則 1〜6 をすべて引き継いだ上で、v0.9 固有の原則を追加する。

1. **【v0.8 引継ぎ】言語機能追加は最小限、原則ゼロ。複雑さはアダプタ層(`@kei/hono`)で吸収する。**
   Hono の Context / Workers の Request/Response を Kei 型システムに直接引き込まない。
   `mount()` が受けるハンドラ型は `(req: HttpRequest) -> HttpResponse`(+ 必要なら `uses Async`)の
   純粋な関数型シグネチャのまま(✅ 合意済み — DI のためにこの型を拡張しない)。

2. **【v0.8 引継ぎ】JSON 境界は「record 型ごとの専用 extern」。** 汎用アダプタ(`@kei/hono` の
   `parseAs<T>`)にアプリ固有関数を混ぜない。アプリ固有 wrapper は file: ローカルパッケージ
   (M39 の `parse-app` 方式)で提供する。

3. **【v0.9 新規】Workers bindings / env は Kei に露出させない。TS エントリポイントで受けて record に写す。**
   - `env`(KVNamespace / D1Database / Secrets / vars)・`ExecutionContext` は **TS 側の
     Workers エントリポイント(`src/index.ts` の `fetch(request, env, ctx)`)だけが触る**。
   - Kei ハンドラに渡すものは、env から読み出した値を詰めた **平坦な record** に限る。
     受け渡しは TS 側 closure wrap(✅ 合意済み — M41 参照)で行う。
   - KV/D1 への読み書きを Kei から行いたくなった場合も、v0.9 では **extern 経由の TS wrapper**
     で表現する(bindings 型そのものを extern 署名に書かない)。Kei のエフェクトシステムが
     把握できない副作用の抜け道を作らない。

4. **【v0.9 新規】デプロイ物の責務分担を固定する。**
   - **Kei の責務**: API のビジネスロジック(ハンドラ関数・record・契約)。`kei build` の出力(dist/)。
   - **TS(テンプレート)の責務**: Workers エントリポイント(`export default { fetch }`)、
     Hono app の組み立て、`mount()` によるルート登録、契約違反の中央処理(`app.onError` → 500)、
     env → record の写像と closure wrap。
   - **wrangler の責務**: bundling(esbuild)と workerd 起動。`wrangler.jsonc`(✅ 合意済み)は
     テンプレートの一部として人間がレビューできる形で置く。
   - この 3 層の境界を曖昧にする変更(例: kei_cli に wrangler 呼び出しを組み込む)は v0.9 では行わない。

5. **【v0.9 新規】疎通リスクは最初に潰す(M40 が先頭)。** v0.8 契約書が明記した
   「file: 依存 × wrangler(esbuild) bundling は技術的に妥当だが未検証」のリスクを、
   テンプレート作成や #136 の設計より**先に**回収する。判定は dry-run だけでなく
   **wrangler dev の実挙動(fetch 応答)まで**(✅ 合意済み)。ダメだった場合の分岐(npm 公開前倒し /
   テンプレート内コピー)も M40 の契約に含め、後続 Milestone が手戻りしない順序にする。

6. **【v0.8 引継ぎ】契約と async の意味論を変えない。** 契約式は同期・純粋のまま。
   Workers 上でも `KeiContractViolation` の throw 位置・shape は既存互換。
   HTTP 境界での観測は **requires / ensures を問わず 500**(✅ 合意済み)。

7. **【v0.9 新規】workerd 依存のテストは cargo test を人質に取らない。**
   `cargo test --workspace` は従来どおり wrangler なしで全パスすることを維持する。
   Workers 実行系の e2e は **vitest-pool-workers(workerd のプロセス内起動)で CI に常設**する
   (✅ 合意済み — M43 参照)。`wrangler dev` のポート待ちに依存する形は恒久テストにしない
   (M40 の疎通確認スパイクでのみ使う)。

## Milestone 全体像と順序

M 番号は v0.8(M39)からの連番。

| M | テーマ | issue | 優先度 | 状態 | 主な成果物 |
|---|---|---|---|---|---|
| **M40** | file: 依存 × wrangler bundling + wrangler dev 疎通確認 | (v0.8 契約書の明記リスク) | high(最優先) | ✅ 合意済み・未着手 | 疎通レポート + 分岐判断 |
| **M41** | HTTP アダプタ実配線 — pathParams + String stdlib 部分拡張 + closure DI + 契約違反 500 統一 | #136 | high | ✅ 合意済み・未着手 | `@kei/hono` 拡張 + spec/stdlib + e2e |
| **M42** | `examples/workers-api` 雛形テンプレート(在庫 API) | — | high | ✅ 合意済み・未着手 | examples/workers-api/ + SKILL 節 |
| **M43** | Workers 統合 e2e(vitest-pool-workers・CI 常設) | — | high | ✅ 合意済み・未着手 | e2e + CI ワークフロー更新 |

順序の論拠:

- **M40 が先頭・単独。** 疎通が NG だった場合、M41〜M43 の成果物の置き場所(file: パッケージのまま
  か、npm 公開か、テンプレート内コピーか)が全部変わる。分岐判断を先に済ませる。
- **M41 は M42 の前。** テンプレート(M42)は #136 の設計(HttpRequest の shape・DI パターン)を
  前提にするため、アダプタの形を先に固める。契約違反 500 統一(M39 実装の 400 修正)も
  アダプタ配線を触る M41 で一緒に行う。
- **M43 が最後。** テンプレートと配線が揃ってから、workerd での通し検証と CI 常設を閉じる。

## M40: file: 依存 × wrangler bundling + wrangler dev 疎通確認

v0.8 契約書「設計原則 3」が明記した先送りリスクの回収。**これが v0.9 の最初の作業**。

### ✅ 合意済み(2026-07-13)

- **疎通確認の判定範囲**: `wrangler deploy --dry-run` の bundling 成功だけでは**不可**。
  **`wrangler dev` を起動し、fetch で実応答を確認するところまで**を疎通判定とする(実挙動確認)。
- **NG だった場合の分岐**(NG 確定時に人間判断 — 事前にどちらかへ倒さない):
  - **(a) npm 公開の前倒し**: `@kei/runtime` / `@kei/hono` を npm に publish する。
    影響: v0.8 契約書の「npm 公開はしない(v1.0 以降で検討)」の前倒し変更。公開手順・
    バージョニング・スコープ(`@kei/*` は確保済み — HANDOFF 参照)の運用が発生。
  - **(b) テンプレート内コピー**: `examples/workers-api/` に `@kei/hono`(必要なら
    `@kei/runtime` も)のソースを直接コピーして同梱する。
    影響: 二重管理(tests/cli/packages/kei-hono との乖離リスク)。同期手段(コピースクリプト or
    「テンプレート側が正」の宣言)を決める必要。
  - **NG が確定した時点で症状(esbuild / workerd のエラー内容)を添えて人間に判断を仰ぐ**。
    実装エージェントが勝手に選ばない。

### 完了条件(機械検証可能)

- 検証用プロジェクト(M42 の前身。`examples/workers-api/` の骨組みでよい)に
  `wrangler.jsonc` + TS エントリポイント + `@kei/hono` への file: 依存を置き、以下を順に確認して
  コマンドと出力を実行結果として表示する:
  1. `npm install && npx wrangler deploy --dry-run --outdir dist-wrangler` が **exit 0** で完走する。
  2. 生成された bundle に Kei ハンドラのシンボル(例: `handleHealth`)と `@kei/hono` の
     `mount` 相当が含まれることを grep 等で確認する。
  3. **`wrangler dev` を起動し、`curl`(または fetch)で `GET /health` が 200 と期待 body を
     返すことを確認する**(起動ログとレスポンスを表示)。
- 疎通 OK の場合: 結果を本ファイルの M40 節に「実施記録」として追記し(どの wrangler
  バージョンで確認したかを明記)、M41 に進む。
- 疎通 NG の場合: エラー内容・原因分析・分岐 (a)/(b) の推奨案を添えて**停止・報告**する。
  人間の判断が出るまで M41 以降に着手しない。

### golden / test 設計方針

- この Milestone は探索的スパイクなので golden は追加しない。恒久的なテスト化は M43 の領分
  (恒久テストは wrangler dev ではなく vitest-pool-workers で行う — 設計原則 7)。
- ただし検証用プロジェクトのファイル群は M42 で再利用できる形で置く(使い捨てにしない)。

### スコープ外(M40)

- 実デプロイ(`wrangler deploy` 本番)・Cloudflare アカウント設定。
- minify / tree-shaking の最適化検証。bundle が動くことだけ確認する。
- wrangler dev を使う**恒久**テスト(疎通確認はスパイクのみ。恒久化は M43 の pool-workers)。

## M41: HTTP アダプタ実配線 — pathParams + String stdlib 部分拡張 + closure DI + 契約違反 500 統一(#136)

v0.8.0 ドッグフード(`docs/dogfood/2026-07-10-v0.8.0-stock-api-hono.md`、8/10 の減点理由)で
明確化されたギャップ 2 件の解消と、契約違反 → HTTP ステータス写像の統一。

### ✅ 合意済み(2026-07-13)

**1. パスパラメータ抽出 — 選択肢 (b)+(c) を採用**(issue #136 の推奨案どおり):

- **(b)** HttpRequest に `pathParams: Map<String, String>` フィールドを追加する。`mount()` 側で
  Hono の `c.req.param()` から埋める(境界層で情報を保存する原則 — PR #132 レビューと同じ方向)。
  既存の HttpRequest record 構築箇所(`http_model.kei` / golden / e2e の record リテラル)への
  追従は破壊的変更として扱い、expected の変更は人間レビューに乗せる(不変条件 1)。
- **(c)** String stdlib 段階2の**部分拡張**: `s.split(delimiter) -> List<String>` /
  `s.indexOf(needle) -> Option<Int>` を追加する(見つからない場合を `Option` で返し null 安全を
  保つ。-1 番兵は採らない)。spec-first(String 組み込み一覧節への追記が実装より先)+
  pbt.rs 追従 + syntax/check/fmt/emit golden 追加。trim / slice 等の残りは v1.x 送り。

**2. 依存注入 — 選択肢 (b) を採用**(TS 側 closure wrap。Kei ハンドラと mount() のシグネチャは触らない):

- `mount()` が受けるハンドラ型 `(req: HttpRequest) -> HttpResponse` は**変更しない**。
- 依存を要する Kei 関数は deps を**通常の第1引数**として受ける(例:
  `func handleStock(deps: StockDeps, req: HttpRequest) -> HttpResponse`。Deps はアプリごとに
  record 定義)。TS 側で `mount(app, "get", "/stock/:sku", (req) => handleStock(deps, req))` の形に
  closure で部分適用してから mount する。言語機能追加ゼロ・アダプタ変更ゼロで完結する。
- Workers bindings(KV/D1/vars)は TS エントリポイントで受けて record に写してから deps に詰める
  (設計原則 3)。この定型を SKILL.md に示す。

**3. 契約違反 → HTTP ステータス — 全部 500 に統一**:

- 「**契約が破られた時点でサーバ異常**」という思想で確定。requires / ensures を区別しない。
- **v0.5 逆算表の記述(500)が正であり、M39 実装(`tests/cli/projects/app/tests/http/app.ts` の
  `app.onError` が 400 に写している)は修正対象**。本 Milestone でアダプタ配線を触るのと同時に
  400 → 500 へ修正する(修正は完了条件に機械検証可能な形で含める — 下記)。
- 入力不正を 400 で返したい場合は、契約に到達させる前に **Kei ハンドラの業務ロジックで判定して
  400 を返す**(M39 の JSON parse 失敗 → 400 パターンと同じ)。「クライアント都合のエラーは
  ロジックで、サーバ不変条件はコントラクトで」という責務分担を SKILL.md に明文化する。

### 完了条件(機械検証可能)

- **pathParams**: HttpRequest record(`@kei/hono` の interface と Kei 側 `http_model.kei`)に
  `pathParams: Map<String, String>` を追加し、`mount()` が Hono の `c.req.param()` から埋める。
  パスパラメータを使う Kei ハンドラ(`GET /stock/:sku` — ドッグフードの在庫 API 再現)の e2e を
  `tests/cli/projects/app/` に追加し、`app.request("/stock/ABC-1")` で sku が取れることを固定する。
- **String stdlib**: spec の String 組み込み一覧に `split` / `indexOf` を追記(spec-first)した上で
  check / emit / fmt / pbt を実装。契約式内でも使える(純粋)。syntax/check/fmt/emit golden +
  e2e で固定する。
- **closure DI**: 在庫マスタ(Map ベースの deps record)を closure wrap で注入する e2e を追加し、
  deps の値がレスポンスに反映されることを固定する。
- **契約違反 500 統一**: `tests/cli/projects/app/tests/http/app.ts` の `app.onError` を
  400 → **500** に修正し、既存の契約違反 e2e(vitest)の期待ステータスを 500 に更新する
  (expected 変更は本契約による人間合意済みの変更として PR レビューに明記)。
  requires 違反・ensures 違反の両方が 500 で観測される test case を持つこと。
- SKILL.md「HTTP ハンドラを書く」節に、pathParams・closure DI・「400 はロジック / 500 は契約」の
  定型を追記(SKILL 更新に伴う MCP golden 再生成を含む)。
- `cargo test --workspace` 全パス(コマンド実行と結果表示を含む)。
- issue #136 を PR の `Closes #136` でクローズする(不変条件 6)。

### golden / test 設計方針

- String stdlib(唯一の言語変更)は通常の言語機能と同じ扱い: spec → golden(syntax/check/fmt/emit)
  → pbt 追従 → e2e の順で固定する。
- HttpRequest への pathParams 追加で既存 golden / expected TS が変わる場合、expected の変更は
  人間レビュー必須(不変条件 1)。変更点を PR で明示する。
- e2e は M39 と同じ流儀: `kei build` → `tsc --strict` → vitest で Hono の `app.request(...)`。
  workerd は使わない(M43 の領分)。

### スコープ外(M41)

- KV / D1 への実接続(v0.9 では DI 経路の設計まで。実バインディング統合は v1.x)。
- String stdlib の残り(trim / slice / replace / toUpperCase 等)— v1.x で実需確認後。
- Set-Cookie を含む複数値ヘッダー(v0.8 から継続の制限)。
- ミドルウェア(認証・CORS 等)の Kei 表現。TS 側の Hono ミドルウェアで足りる。

## M42: `examples/workers-api` 雛形テンプレート

### ✅ 合意済み(2026-07-13)

- **テンプレートの題材**: **在庫 API**(ドッグフード
  `docs/dogfood/2026-07-10-v0.8.0-stock-api-hono.md` の再現。`GET /stock/:sku` のパスパラメータと
  closure DI(在庫マスタ)を実演し、M41 の成果を雛形で示す)。
- **wrangler 設定の形式**: **`wrangler.jsonc`**。
- **`@kei/hono` の参照方法**: M40 の疎通結果に従う(file: が通ればそのまま。NG なら分岐の人間判断に従う)。

### 完了条件(機械検証可能)

- `examples/workers-api/` に以下を追加:
  - `.kei` ソース群(在庫 API のハンドラ + record + 契約。`kei fmt --check` クリーン — 不変条件 5)
  - TS エントリポイント(`src/index.ts`: `export default { fetch }` + Hono app 組み立て +
    closure DI + `mount()` 登録 + 契約違反の中央処理(`app.onError` → **500**))
  - `wrangler.jsonc`・`package.json`・`tsconfig.json`
  - `README.md`(ビルド → wrangler dev → curl 確認の手順。デプロイ手順は v1.0 送りと明記)
- `kei build examples/workers-api`(相当のディレクトリ指定)が exit 0、生成 TS が `tsc --strict` を
  通り、`wrangler deploy --dry-run` が exit 0。3 コマンドの実行と結果を表示する。
- examples/ は kei_mcp にビルド時埋め込みされる(不変条件 3)ため、MCP golden の再生成と
  `cargo test --workspace` 全パスを確認する。**埋め込み対象に workers-api を含めるかは実装時に
  確認**(TS/jsonc 混在ディレクトリの扱いが既存の examples 埋め込み機構と整合するか。整合しない
  場合は埋め込み除外の判断を報告 — 勝手に機構を改造しない)。
- SKILL.md に「Workers にデプロイする(v0.9)」節を追加(テンプレートの使い方・責務分担・制限)。

### golden / test 設計方針

- テンプレートの `.kei` は examples/ 配下なので正規形維持(fmt --check)を CI 対象にする
  (既存 examples の検査経路に乗せる)。
- 生成 TS の固定は M39 同様 expected 方式を使うかは実装時判断(テンプレートは動く見本が主目的。
  workerd 上の実挙動は M43 の e2e で担保するため、二重に golden 化しない方向を推奨)。

### スコープ外(M42)

- `kei new` / scaffolding コマンド(CLI へのテンプレート生成機能追加は v1.x で実需確認後)。
- 複数テンプレート(REST 以外の WebSocket / cron trigger 等)。

## M43: Workers 統合 e2e — vitest-pool-workers で CI 常設

### ✅ 合意済み(2026-07-13)

- **CI での扱い**: **`@cloudflare/vitest-pool-workers`(workerd をプロセス内起動)で vitest に統合し、
  CI に常設する**。分離ジョブは作らない(既存 CI の流れの中で常に実行される必須テストとする)。
  **CI ワークフロー(`.github/workflows/ci.yml`)への追加も /goal の完了条件に含める。**
- **`wrangler dev` は恒久テストに使わない**(ポート待ち・外部プロセス依存の flaky 要因を排除。
  wrangler dev での実挙動確認は M40 のスパイクと README の手動手順で担保済み)。
- `cargo test --workspace` は従来どおり wrangler / workerd なしで全パスすることを維持する
  (設計原則 7)。
- **wrangler / vitest-pool-workers のバージョン**: `examples/workers-api/package.json` の
  devDependencies に記載して package-lock.json で固定する(caret 指定 + lockfile。CI は
  lockfile どおりに入れる)。

### 完了条件(機械検証可能)

- `examples/workers-api/`(M42)に vitest-pool-workers の e2e を追加し、workerd 上で以下を自動検証する:
  1. 正常系: `GET /health` が 200。`GET /stock/:sku`(存在する sku)が 200 で在庫値を返し、
     closure DI した在庫マスタの値がレスポンスに反映されている。
  2. 業務エラー系: JSON parse 失敗・存在しない sku 等、**ロジックが返す 400/404** が観測できる。
  3. **契約違反系: requires を破るリクエストと ensures を破らせるパスの両方で、
     ステータス 500 と `KeiContractViolation` 由来のエラー body(`clause` / `condition`)が
     観測できる。**
  4. 業務エラー(400 系)と契約違反(500)の経路が**別物として**区別して検証される。
- **`.github/workflows/ci.yml` を更新**し、上記 e2e が CI で常に実行される(必須・required)こと。
  実装形(既存 test ジョブへのステップ追加か、examples/workers-api の npm test を呼ぶ形か)は
  実装時に選んでよいが、**分離した optional ジョブにはしない**。CI が green になった実行ログ
  (該当ステップ)を確認し結果を表示する。
- `cargo test --workspace` は従来どおり wrangler / workerd なしで全パスすること(既存 CI を壊さない)。
- 完了時点で v0.9 の受け入れ基準(本ファイル冒頭)を満たしたことを、コマンド出力つきで報告する。

### golden / test 設計方針

- HTTP レスポンスの検証は「ステータス + body の構造(JSON キー)」を assert する。body 全文の
  スナップショット固定は wrangler / Hono のバージョン差で壊れやすいため避ける。
- 契約違反レスポンスの shape(`error` / `clause` / `condition`)は M39 の形式を正とし(ステータス
  のみ 500 に変更済み — M41)、変える場合は人間レビュー(不変条件 1 の精神)。
- pool-workers の設定(`vitest.config` の pool 指定・wrangler.jsonc 参照)はテンプレートの一部
  としてレビュー対象に置く。

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

## 後続 /goal ドラフト(✅ 合意反映済み)

```text
/goal M40: examples/workers-api の骨組み(wrangler.jsonc + TS エントリポイント + @kei/hono への
file: 依存)を作り、(1) npx wrangler deploy --dry-run --outdir dist-wrangler が exit 0、
(2) bundle に Kei ハンドラと mount が含まれる、(3) wrangler dev を起動して GET /health が
200 と期待 body を返す、の 3 点を実行結果つきで確認する。NG の場合はエラー内容と分岐案
(npm 公開前倒し / テンプレート内コピー)を添えて停止・報告する(🤝 人間判断)。
```

```text
/goal M41: issue #136 を解消する。(1) HttpRequest に pathParams: Map<String, String> を追加し
mount() が c.req.param() から埋める、(2) String stdlib に split / indexOf(Option 返し)を
spec-first で追加し golden + pbt を固定、(3) 依存注入は TS closure wrap の定型
(mount のハンドラ型は不変)で GET /stock/:sku の e2e を追加、(4) app.onError の契約違反写像を
400 → 500 に修正し requires / ensures 両方の 500 観測を e2e で固定、(5) SKILL.md に
pathParams / closure DI / 「400 はロジック・500 は契約」を追記して MCP golden を再生成する。
cargo test --workspace を通して結果を表示し、PR で Closes #136 する。
```

```text
/goal M42: examples/workers-api 雛形(在庫 API 題材・wrangler.jsonc)を追加する。
kei build → tsc --strict → wrangler deploy --dry-run の 3 コマンドが exit 0 で通ることを
実行結果つきで示し、.kei は kei fmt --check クリーン、契約違反の中央処理は 500、
SKILL.md に「Workers にデプロイする」節を追加、MCP golden を再生成して
cargo test --workspace を通す。
```

```text
/goal M43: examples/workers-api に @cloudflare/vitest-pool-workers の e2e を追加し、workerd 上で
正常系 200 / 業務エラー 400 系 / 契約違反 500(requires・ensures 両方、clause/condition つき body)
を自動検証する。.github/workflows/ci.yml を更新してこの e2e を CI に常設(必須・分離 optional
ジョブにしない)し、CI green のログを確認して表示する。cargo test --workspace は wrangler なしで
従来どおり全パスすること。
```
