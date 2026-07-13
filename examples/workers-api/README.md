# examples/workers-api — Kei v0.9 Workers テンプレート(在庫 API)

Cloudflare Workers 上で Kei が書いた API ハンドラを Hono + `@kei/hono` 経由で動かす最小テンプレート。**初見の人がこのディレクトリをコピーして始められる**ことを目的にした雛形。

題材は在庫 API(`GET /health` + `GET /stock/:sku`)。パスパラメータ抽出と closure DI(依存注入)の定型を実演する。

## クイックスタート

このリポジトリのルートから:

```bash
# 1. @kei/runtime と @kei/hono の dist を作る(file: 依存の解決先)
(cd runtime && npm install --no-audit --no-fund && npm run build)
(cd tests/cli/packages/kei-hono && npm install --no-audit --no-fund && npm run build)

# 2. Kei ソースを TS にトランスパイル(dist/ に配置)
cargo run -q -p kei_cli --bin kei -- build examples/workers-api --out-dir examples/workers-api/dist

# 3. workers-api の依存を入れる
(cd examples/workers-api && npm install --no-audit --no-fund)

# 4. wrangler の bundling(esbuild)が通ることを確認
(cd examples/workers-api && npx wrangler deploy --dry-run --outdir dist-wrangler)

# 5. wrangler dev を起動して叩く
(cd examples/workers-api && npx wrangler dev --port 8787 --local &)
curl -s http://127.0.0.1:8787/health          # -> {"status":"ok"}
curl -s http://127.0.0.1:8787/stock/ABC-1     # -> {"qty":42}
curl -s http://127.0.0.1:8787/stock/UNKNOWN   # -> {"error":"not found"} (404)
```

## 責務分担 (v0.9 設計原則 4)

| 層 | 何を書くか | 場所 |
|---|---|---|
| Kei | API のビジネスロジック(ハンドラ関数・record・契約) | `*.kei` → `dist/` |
| TS | Workers エントリポイント / Hono app 組み立て / `mount()` / 依存注入 / 契約違反の中央処理 | `src/index.ts` |
| wrangler | bundling(esbuild)と workerd 起動 | `wrangler.jsonc` |

`env` / `ExecutionContext` は **TS 側の Workers エントリだけが触る**。Kei ハンドラには平坦な `HttpRequest` レコードと deps レコードだけ渡す(設計原則 3 — bindings を Kei に露出させない)。

## ファイル構成

- `workers_api/api.kei` — Kei 側のビジネスロジック(1 モジュール)。
  - `HttpRequest` / `HttpResponse` record 定義(`pathParams: Map<String, String>` を含む)
  - `handleHealth`: 契約なし・依存なしの最小ハンドラ(`ensures result.status == 200`)
  - `handleStock`: パスパラメータ `:sku` を抽出し、closure DI で受けた `StockDeps.inventory` を lookup する。`lookupStock` は `requires sku.length > 0` の契約付き
- `src/index.ts` — Workers エントリポイント。`export default app` の Hono app に `mount()` で Kei ハンドラを登録し、`app.onError` で `KeiContractViolation` を **500** に写す。
- `wrangler.jsonc` — Workers 設定(name / main / compatibility_date)。
- `package.json` — `@kei/runtime` / `@kei/hono` を `file:` 依存として参照。
- `tsconfig.json` — `strict: true` + `types: ["@cloudflare/workers-types"]`。

モジュール分割は CLAUDE.md 不変条件どおり「モジュールパスとディレクトリパスが 1:1」なので、`module workers_api.api` は `workers_api/api.kei` に置く。テンプレートを分割したい場合はサブモジュール(`workers_api/health.kei` / `workers_api/stock.kei` 等)を追加する形で拡張する。

## 依存注入 (closure DI) の定型

Kei ハンドラの型 `(req: HttpRequest) -> HttpResponse` は変えず、依存は Kei 関数の第1引数として受け取り、TS 側で closure で部分適用してから `mount()` に渡す(SKILL.md「パスパラメータと closure DI」参照):

```ts
// TS 側で env や外部リソースから deps を組み立てる
const stockDeps = { inventory: new Map<string, number>([["ABC-1", 42]]) };
mount(app, "get", "/stock/:sku", (req) => handleStock(stockDeps, req));
```

実プロジェクトでは `stockDeps` を KV / D1 / vars から `fetch(request, env, ctx)` の中で組み立てて渡す。

## 契約違反の HTTP 写像

契約違反(`requires` / `ensures` どちらも)は「サーバ不変条件の破れ」= サーバ異常として **500** に統一する(v0.9 M41)。

- **400** — クライアント都合のロジック判定(JSON parse 失敗など)。Kei ハンドラのロジックが直接返す。
- **500** — サーバ不変条件の破れ(契約違反)。`app.onError` の中央処理が捕捉して写す。

例: `curl -s http://127.0.0.1:8787/stock/` は Hono のルーティングで 404、`GET /stock/x` は `x.length > 0` なので 200/404、契約違反経路は本ディレクトリのハンドラでは通常発火しない(在庫が空でも 404 で正常返却)。契約違反の 500 動作確認は M43 の vitest-pool-workers e2e が担う。

## v0.9 時点での制限

- **実 Cloudflare へのデプロイ(`wrangler deploy` 本番)は v1.0 の領分**。本テンプレートはローカル(`wrangler dev` / workerd)まで。
- Set-Cookie を含む複数値ヘッダー・WebSocket・streaming response は非対応(SKILL.md §9.6 参照)。
- KV / D1 の実接続は v1.x。v0.9 では DI 経路の設計まで(TS 側で record に写して渡す定型)。

## 検証(このテンプレートを CI 的に確かめる 3 コマンド)

```bash
cargo run -q -p kei_cli --bin kei -- fmt --check examples/workers-api    # .kei が正規形か
cargo run -q -p kei_cli --bin kei -- build examples/workers-api --out-dir examples/workers-api/dist
(cd examples/workers-api && npx tsc --noEmit)                             # TS 側の型
(cd examples/workers-api && npx wrangler deploy --dry-run --outdir dist-wrangler)
```
