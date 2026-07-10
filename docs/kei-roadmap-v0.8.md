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

## 設計原則(HANDOFF 準拠 — Sonnet に絶対に破らせない)

1. **言語機能追加は最小限。アダプタ層(`@kei/hono`)で吸収する。**
   - Hono の API 表面(Context / Response / Request / ルーティング DSL / c.json / c.req.param)を
     Kei 型システムに直接引き込まない。**Kei には record として扱う HTTP モデル**(HttpRequest /
     HttpResponse record)を `@kei/hono` から extern package 経由で提供し、境界の複雑さを
     TS 側(アダプタ)に閉じ込める。
   - Hono の Context 全体を Kei から触らせない。Kei ハンドラは
     `handler(req: HttpRequest) -> HttpResponse uses Async`(または `uses ..., Async`)の
     **純粋な関数型シグネチャ**で書く。TS 側の薄い wrapper が Hono の Context から
     HttpRequest を組み立てて Kei ハンドラを呼び、返った HttpResponse を Hono の response に写す。

2. **JSON 境界は「unknown → Option<record>」で表現する。**
   - `extern json.parseAs(text: String) -> Option<T> uses ...`(段階1)。実体は `@kei/hono`(または
     新設 `@kei/json`)の runtime helper が JSON.parse + record 形状検査を行う。失敗は None。
   - record → JSON 文字列は既存の record emit + `JSON.stringify` を extern で公開。
   - **完全な型付き JSON デシリアライザは v0.8 スコープ外**。ハンドラは HttpRequest.jsonBody:
     Option<Unknown> の unknown 部分を Kei 側では受けない — 実際には `extern hono.jsonBody<T>(req)
     -> Option<T> uses Async` のような形で record 型を context に持たせて runtime で shape check
     する形にする(段階1、詳細は M40 の 🤝 で確定)。

3. **default export と named import を段階的に解禁する。**
   - v0.6 で先送りにした宿題(dogfood v0.6.0 で default export の実需を確認済み)。
   - **v0.8 は namespace + default の 2 形態のみ**(named import は必要性が確定してから段階3で
     🤝、v0.9 以降または恒久的先送り)。default は Hono の `import Hono from "hono"` に必要。

4. **Cloudflare Workers 特有の型は Kei に露出させない。**
   - `env` / `KVNamespace` / `D1Database` / `ExecutionContext` などの Workers bindings 型は
     TS 側の wrapper で受け、Kei ハンドラには扱いやすい record として渡す(または extern package
     経由で opaque な型として)。**v0.8 では Kei は Workers ランタイム API を直接触らない**。
   - 実デプロイ(wrangler)経由の疎通は v0.9。

5. **契約と async の意味論を変えない。**
   - HTTP ハンドラは async(v0.7 の uses Async)。契約(requires/ensures)は同期・純粋のまま
     (Request の shape check は runtime、契約式では不可)。

6. **v1.0 受け入れ検証を意識した最小構成。**
   - v0.9 で wrangler dev / deploy を回すことを見越し、v0.8 の e2e は `tsc --strict` + vitest で
     Hono の `app.request(...)` を叩いて Kei ハンドラの動作を確認する形にする(実 fetch は不要、
     Hono の in-memory テスト API で足りる)。

## Milestone 全体像

| M | テーマ | 優先度 | 状態 | 主な改修クレート/成果物 |
|---|---|---|---|---|
| **M39** | extern package 段階2(default export 対応) | high | ⬜ 未着手 | kei_syntax / kei_check / kei_emit / kei_fmt |
| **M40** | @kei/hono アダプタ + HttpRequest/HttpResponse + JSON 境界 + e2e | high 🤝 | ⬜ 未着手 | runtime(または新 npm パッケージ)/ tests/e2e / spec / skill |

## M39: extern package 段階2 — default export

### 完了条件

- 構文: `extern package "hono" default as Hono`(または `default as` を持つ形。パーサに negotiable)。
  Hono 側の default export と 1:1 対応。namespace 版と同じ束縛名スコープ規則(extern 署名専用、
  値/型位置での使用は KEI-E3007)。
- emit: `import Hono from "hono";`(default import)を出力。既存の `import * as` と並存できる。
- 型の観点: default export の関数呼び出し(`Hono()`)や class インスタンス生成(`new Hono()`)は
  **段階1では extern 署名で明示的に宣言**する形にする(new はしない、コンストラクタは wrapper 側
  = M40 の `@kei/hono` に隠す)。Kei ソースは Hono を直接インスタンス化しない。
- spec: `spec/kei-spec-v0.2.md` §2.4 の extern package 節に default 形の記述を追記。namespace 版
  との差分(default は「複数個 default 宣言は禁止」「サブパスと組み合わせ可」)を明記。
- 拒否: `extern query` に default を組み合わせるのは既存 KEI-E3005 派生で拒否(query は純粋観測子、
  default とは意味論が違う)。
- golden(syntax + check + fmt + emit 単体)+ 既存 e2e で回帰なし。

### スコープ外(v0.8 M39)

- named import(`extern package "hono" as { serve } from "hono"`)— 実需が確定していないため
  🤝 で段階3(v0.9+)へ。
- default + namespace の同一 specifier での同時宣言 — 段階1では片方のみ許す(仕様確定するまで
  診断で拒否)。

## M40: @kei/hono アダプタ + HTTP 境界(🤝)

### 事前合意事項(🤝、着手前に確定)

- **`@kei/hono` を新規 npm パッケージとして runtime とは別建てする**(名前は `@kei/hono` を第一候補、
  ただし発行しない — file: 依存で e2e に留める。npm 公開は v1.0 以降)。
  - runtime(`@kei/runtime`)は Result/Option/契約違反例外の言語ランタイム、`@kei/hono` は
    Hono アダプタ、と責務分離。
- **HttpRequest / HttpResponse record の最小フィールド**:
  - HttpRequest: `method: String`、`path: String`、`headers: Map<String, String>`、
    `queryParams: Map<String, String>`、`bodyText: Option<String>`。
  - HttpResponse: `status: Int`、`headers: Map<String, String>`、`bodyText: String`。
  - JSON パースは別の extern(`extern jsonBody<T>` 相当は段階1で扱わない、段階2で 🤝)。
    段階1は `bodyText` のみ扱い、Kei 側で `parseJsonAs` extern を呼ぶ形。
- **JSON parse extern の型パラメータ問題**: Kei にジェネリック関数が無い(型パラメータは Option/Result/
  List/Map の組み込みのみ)。段階1では **record 型ごとに専用 extern を書く**形にする
  (`extern hono.parseUserRequest(text: String) -> Option<UserRequest>` 等)。TS 側の
  `@kei/hono/parseAs<T>(text, shapeCheck)` を record ごとにラップする定型を SKILL.md で示す。
  ジェネリック extern は v0.9+ の 🤝(検討スコープ外の可能性大)。
- **e2e 構成**: `tests/cli/packages/kei-hono/`(file: 依存)+ ハンドラを Kei で書き、TS 側で
  Hono app に登録して `app.request(...)` で in-memory テスト。実 fetch/wrangler は v0.9。

### 完了条件

- `tests/cli/packages/kei-hono/`(または類似位置)に @kei/hono のミニ実装(package.json、
  index.ts、HttpRequest/HttpResponse type、Kei 側の record 対応 TS 型、Hono Context ↔ HttpRequest
  変換 helper)を追加。
- Kei 側の使用例: `tests/cli/projects/app/` 配下に「GET /health → 200 OK JSON、POST /users → 202」の
  ハンドラ 2 本を Kei で書き、TS 側で Hono app に登録、`app.request(...)` の vitest 疎通確認を CI 固定。
- **契約が生きた状態でハンドラが動く**: requires 違反(例: bodyText が空でない前提)が
  KeiContractViolation として fire し、TS 側で捕捉して 400 に写す例を含める(Hono の onError で
  中央処理する形を SKILL.md でも紹介)。
- SKILL.md「HTTP ハンドラを書く(v0.8)」節を追加: HttpRequest/HttpResponse の record 使用、
  parseXxxRequest extern の書き方、Hono との接続の全体像。
- spec: HTTP モデル自体は spec 対象外(アダプタなので runtime/skill 領分)。ただし SKILL.md
  更新に伴う MCP golden 再生成は必要。
- ロードマップ M40 → ✅ で **v0.8 全 Milestone 完了、v1.0 blocker 3/3 解消**。

### スコープ外(v0.8 M40)

- 実 wrangler deploy(v0.9)/ KV/D1 バインディング直接アクセス(TS 側で受けるまで)/
  WebSocket / streaming response / cookie / cache API / multi-part form / file upload

## 後続 /goal ドラフト

```text
/goal M39: extern package "<spec>" default as <name> 宣言を追加する。spec v0.2 §2.4 を先に更新し、
syntax/check/fmt/emit の golden で固定する。namespace 版との相互排他・同時宣言禁止・query との
非互換を診断で明示する。
```

```text
/goal M40: @kei/hono をローカル file: パッケージとして追加し、HttpRequest/HttpResponse record と
Hono Context 変換ヘルパーを提供する。tests/cli/projects/app/ に Kei ハンドラ 2 本(GET /health、
POST /users)を書き、Hono app.request(...) の vitest で疎通確認。契約違反が 400 に写る例と
SKILL.md「HTTP ハンドラを書く」節を追加。
```
