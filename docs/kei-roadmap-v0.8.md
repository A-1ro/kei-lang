# Kei 開発ロードマップ v0.8 — /goal 契約書集(HTTP/JSON 境界 + Hono アダプタ)

> 運用ルール: 各 Milestone は「人間が合意する契約」。完了条件は機械検証可能な形で書く。
> 本ファイルは v1.0 戦略(`docs/kei-roadmap-v0.5.md` 冒頭)の **v0.8: HTTP/JSON 境界 + Hono アダプタ**
> — v1.0 blocker 3 つのうちの 3 つ目、最後 — を Milestone 化したもの。
> **着手前に本ファイルの「設計原則」節を熟読すること。**

## v0.8 のゴール

Kei ソースだけで(TS を書かずに)Hono API のハンドラ関数を書け、`kei build` の生成 TS を
Hono の app.get/post に登録するだけで動くこと。**Kei は「API のビジネスロジック層」を書く責務**、
TS 側は「app の組み立て + ルート登録 + Workers エントリポイント」を書く責務、と分担する。
Cloudflare Workers 実デプロイ(wrangler)は v0.9 の領分。

## v0.8 の性格 — 「言語機能変更ゼロ」の版

**v0.8 は言語機能の追加・変更を行わない。** アダプタ層(`@kei/hono`)と e2e パターンだけを作る。
理由:

- Hono は `named` export(`import { Hono } from 'hono'`)なので、当初検討していた
  「default export 対応」の言語機能追加は Hono の受け入れ基準達成には**不要**。
- `@kei/hono` アダプタの TS 側で普通に `import { Hono } from 'hono'` すれば、Kei ソースは
  既存の namespace import(v0.6 M35/M36 で入った `extern package "..." as ...`)経由で
  `@kei/hono` を呼ぶだけで足りる。Kei ソース自体は Hono パッケージを直接参照しない。
- v0.7 で async / extern async / 契約 async / スキップ可視化まで揃っており、**v1.0 受け入れ基準
  の「言語としての機能」は既に完成している**。v0.8 はエコシステム側(アダプタ + パターン)
  の版と位置づける。
- named import / default export の言語追加は、Hono 以外の一般 npm パッケージ利用の実需が
  確定してから独立 Milestone として v1.x で扱う(v1.0 の blocker ではない)。

## 設計原則(HANDOFF 準拠 — Sonnet に絶対に破らせない)

1. **言語機能追加は最小限、原則ゼロ。複雑さはアダプタ層(`@kei/hono`)で吸収する。**
   - Hono の API 表面(Context / Response / Request / ルーティング DSL / c.json / c.req.param)を
     Kei 型システムに直接引き込まない。**Kei には record として扱う HTTP モデル**(HttpRequest /
     HttpResponse record)を `@kei/hono` から extern 経由で提供し、境界の複雑さを TS 側
     (アダプタ)に閉じ込める。
   - Hono の Context 全体を Kei から触らせない。Kei ハンドラは
     `handler(req: HttpRequest) -> HttpResponse uses Async`(または `uses ..., Async`)の
     **純粋な関数型シグネチャ**で書く。TS 側の薄い wrapper が Hono の Context から
     HttpRequest を組み立てて Kei ハンドラを呼び、返った HttpResponse を Hono の response に写す。

2. **JSON 境界は「record 型ごとの専用 extern」で表現する。**
   - Kei にはユーザー定義のジェネリック関数がない(組み込みの Option/Result/List/Map/Async 以外)。
     JSON parse は record 型ごとに専用 extern を書く形にする:
     `extern kh.parseUserRequest(text: String) -> Option<UserRequest>`。
   - TS 側の `@kei/hono` に汎用の `parseAs<T>(text, shapeCheck)` を置き、Kei 側の record ごとの
     extern がそれを呼び出す薄いラッパーになる(SKILL.md で定型を示す)。
   - record → JSON 文字列は `JSON.stringify` を extern で公開(型 Object 相当は record が Kei で
     readonly object に写るのでそのまま使える)。

3. **`@kei/hono` を新規 npm パッケージとして runtime とは別建てする。**
   - `@kei/runtime`(v0.3.0)は Result/Option/契約違反例外の言語ランタイム。
   - `@kei/hono` は Hono アダプタ(HttpRequest/HttpResponse type + Hono Context 変換 helper +
     parseAs helper + Kei ハンドラを Hono に接続する wrapper)。
   - 責務分離。**npm 公開はしない**(v1.0 以降で検討)。e2e は M36 と同じ `file:` 依存で扱う。
   - **v0.9 リスク(明記)**: file: 依存のみで v0.9 の wrangler(esbuild ベース)bundling が
     通ることは技術的に妥当だが未検証。実 Workers デプロイ時に file: 参照解決の破綻が起きた場合、
     v0.9 で「npm 公開の前倒し」あるいは「Workers テンプレート内への直接コピー」への切替を
     検討する。v0.8 では **file: のみでスコープを閉じる**が、v0.9 冒頭でこの経路の疎通を最初に
     確認すること(既に v0.9 の /goal 契約書に反映すべき事項)。

4. **Cloudflare Workers 特有の型は Kei に露出させない。**
   - `env` / `KVNamespace` / `D1Database` / `ExecutionContext` などの Workers bindings 型は
     TS 側の wrapper で受け、Kei ハンドラには扱いやすい record として渡す。**v0.8 では Kei は
     Workers ランタイム API を直接触らない**。
   - 実デプロイ(wrangler)経由の疎通は v0.9。

5. **契約と async の意味論を変えない。**
   - HTTP ハンドラは async(v0.7 の uses Async)。契約(requires/ensures)は同期・純粋のまま
     (Request の shape check は runtime、契約式では不可)。

6. **v1.0 受け入れ検証を意識した最小構成。**
   - v0.9 で wrangler dev / deploy を回すことを見越し、v0.8 の e2e は `tsc --strict` + vitest で
     Hono の `app.request(...)` を叩いて Kei ハンドラの動作を確認する形にする(実 fetch は不要、
     Hono の in-memory テスト API で足りる)。

## Milestone 全体像

v0.8 は Milestone 1 個。言語機能変更なし、成果物はすべて runtime/tests/skill/spec 側。

| M | テーマ | 優先度 | 状態 | 主な成果物 |
|---|---|---|---|---|
| **M39** | @kei/hono アダプタ + HTTP/JSON 境界 + e2e + SKILL 節 | high 🤝 | ✅ 実装済み | `tests/cli/packages/kei-hono/` / tests/cli/projects/app/ / skill(spec 更新なし) |

## M39: @kei/hono アダプタ + HTTP 境界(🤝)

### 事前合意事項(🤝、着手前に確定)

- **HttpRequest / HttpResponse record の最小フィールド**:
  - HttpRequest: `method: String`、`path: String`、`headers: Map<String, String>`、
    `queryParams: Map<String, String>`、`bodyText: Option<String>`。
  - HttpResponse: `status: Int`、`headers: Map<String, String>`、`bodyText: String`。
  - **既知の制約**: `Map<K, V>` は単一値なので `Set-Cookie` 複数指定は素直に表現できない
    (`bodyText` に "\n" 区切りで詰めるハックは NG)。v0.8 では **Set-Cookie 非対応**を明示する
    (cookie 自体スコープ外リストに載っている)。v1.x で `Map<String, List<String>>` か
    独立 `Headers` record を検討。
- **JSON parse extern の型パラメータ問題**: 上記設計原則 2 のとおり、**record 型ごとの専用 extern
  を書く**(Kei のジェネリック無しへの正しい対応)。SKILL.md に定型を示す。
- **parseXxxRequest extern の位置づけ(重要)**: `@kei/hono` は汎用アダプタとして
  `parseAs<T>(text, shapeCheck) -> Option<T>`(TS 側の generic ヘルパー)だけを提供する。
  record 固有の `parseUserRequest` などは **アプリ側のローカル TS wrapper**(例:
  `tests/cli/projects/app/app-extern/parse.ts`)で shapeCheck を定義し、
  Kei 側の `extern kh.parseUserRequest` を **アプリローカル npm パッケージ経由**(または既存の
  `@kei/hono` の再エクスポート機構経由)で解決する。汎用アダプタにアプリ固有関数を混ぜない。
  実装時に「アプリローカル file: パッケージ」か「@kei/hono の module 拡張」のどちらを採るかは
  M39 着手時に確定して報告(🤝 事前合意の一部)。

  **実装時に確定**: 選択肢 A(アプリローカル file: パッケージ)を採用した。既存の
  `tests/cli/packages/greeter/` `tests/cli/packages/async-greeter/` と同じ場所に第三のミニ
  パッケージ `tests/cli/packages/parse-app/` を新設し(`tests/cli/projects/app/` 配下には
  入れ子にしない。file: 依存ミニパッケージを `tests/cli/packages/` に集約する既存規約
  (`ARCHITECTURE.md`)に合わせるため)、`@kei/hono` の `parseAs<T>` を `UserRequest` の
  shape check で特殊化する薄い wrapper とした。依存チェーンは
  `app → parse-app → @kei/hono`(+ `@kei/runtime`)。
- **文字列 stdlib との整合**: v0.5 M30(文字列 stdlib 段階1)のスコープ外に「slice / indexOf /
  split / trim 等の本格 String API は v0.8 の HTTP 境界設計と合わせて段階2で」と予告があった。
  本 v0.8 の設計では **パース処理をアダプタ層(TS 側 shapeCheck)に押し出したため、Kei 側で
  文字列を分解する必要がなく、String API の追加は不要**と判断する。v0.9 以降で実需が出た時点で
  独立 Milestone として扱う。
- **e2e 構成**: `tests/cli/packages/kei-hono/`(file: 依存)+ Kei ハンドラを Kei で書き、TS 側で
  Hono app に登録して `app.request(...)` で in-memory テスト。実 fetch/wrangler は v0.9。

### 完了条件

- `tests/cli/packages/kei-hono/` に @kei/hono のミニ実装を追加:
  - `package.json`(name: `@kei/hono`、`hono` を dependency に持つ)
  - `index.ts`(HttpRequest/HttpResponse type、Hono Context ↔ HttpRequest 変換 helper、
    `parseAs<T>(text, shapeCheck) -> Option<T>` 相当のヘルパー、Kei ハンドラを Hono ルートに
    接続する `mount(app, method, path, handler)` 相当の wrapper)
- Kei 側の使用例: `tests/cli/projects/app/` 配下(または新規サブディレクトリ)に、以下のハンドラを
  Kei で書き、TS 側で Hono app に登録して vitest 疎通確認を CI 固定:
  - **GET /health**: 常に 200 OK JSON。ハンドラは同期的な body 組み立て + async な wrapper。
  - **POST /users**: body を UserRequest record にパースして 202、パース失敗は 400。
    `parseUserRequest` extern が failing case を Option::None として返し、Kei 側 match で 400 に写す。
- **契約が生きた状態でハンドラが動く例**: requires 違反(例: `bodyText.length > 0`)が
  KeiContractViolation として fire し、TS 側で捕捉して 400 に写す例を含める(Hono の onError
  で中央処理する形を SKILL.md で紹介)。
- SKILL.md「HTTP ハンドラを書く(v0.8)」節を追加:
  - HttpRequest/HttpResponse の record 使用
  - `parseXxxRequest` extern の書き方(ジェネリック無しへの対応パターン)
  - Kei ハンドラと Hono の接続の全体像
  - 契約違反の 400 マッピング例
  - Set-Cookie 非対応・v0.8 制限一覧
- spec: HTTP モデル自体は spec 対象外(アダプタなので runtime/skill 領分)。**言語機能変更ゼロ**なので
  spec 更新なし。ただし SKILL.md 更新に伴う MCP golden 再生成が必要。
- ロードマップ M39 → ✅ で **v0.8 全 Milestone 完了、v1.0 blocker 3/3 解消**。
- `cargo test --workspace` 全パス(新 e2e 含む)。

### スコープ外(v0.8 M39)

- 実 wrangler deploy(v0.9)
- KV/D1 バインディング直接アクセス(TS 側で受けるまで、v0.9 で検討)
- WebSocket / streaming response / Server-Sent Events
- cookie(Set-Cookie を含む複数値ヘッダー全般)
- cache API / multi-part form / file upload
- named import / default export の言語機能追加(v1.x で実需確定後、独立 Milestone)
- 動的な JSON(unknown → 任意 record への型なしパース)— record 別 extern パターンで代替
- 汎用 JSON schema validation

## 後続 /goal ドラフト

```text
/goal M39: tests/cli/packages/kei-hono/ に @kei/hono の file: パッケージを追加し、
HttpRequest/HttpResponse record と Hono Context 変換ヘルパー・parseAs helper・ハンドラ mount
wrapper を提供する。tests/cli/projects/app/ 配下(または新規)に Kei ハンドラ 2 本
(GET /health、POST /users)を書き、Hono の app.request(...) を叩く vitest で疎通確認。
契約違反が 400 に写る例と、SKILL.md「HTTP ハンドラを書く(v0.8)」節を追加する。
言語機能変更ゼロ(spec 更新なし)。
```
