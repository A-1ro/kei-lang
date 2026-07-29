# examples/workers-api — Kei v0.9 Workers テンプレート(在庫 API)

Cloudflare Workers 上で Kei が書いた API ハンドラを Hono + `@kei/hono` 経由で動かす最小テンプレート。**初見の人がこのディレクトリをコピーして始められる**ことを目的にした雛形。

題材は在庫 API(`GET /health` + `GET /stock/:sku`)。パスパラメータ抽出と closure DI(依存注入)の定型を実演する。

## クイックスタート

このリポジトリのルートから:

```bash
# 1. Kei ソースを TS にトランスパイル(dist/ に配置)
cargo run -q -p kei_cli --bin kei -- build examples/workers-api --out-dir examples/workers-api/dist

# 2. workers-api の依存を入れる
(cd examples/workers-api && npm install --no-audit --no-fund)

# 3. wrangler の bundling(esbuild)が通ることを確認
#    (@kei/runtime / @kei/hono の dist が無ければ npm scripts の pre フックが自動で
#    install + build するので、真っさら環境でも事前ビルド手順は不要)
(cd examples/workers-api && npx wrangler deploy --dry-run --outdir dist-wrangler)

# 4. wrangler dev を起動して叩く
(cd examples/workers-api && npx wrangler dev --port 8787 --local &)
curl -s http://127.0.0.1:8787/health          # -> {"status":"ok"}
curl -s http://127.0.0.1:8787/stock/ABC-1     # -> {"qty":42}
curl -s http://127.0.0.1:8787/stock/UNKNOWN   # -> {"error":"not found"} (404)
```

`npm run dry-run` / `npm run dev` / `npm run test` / `npm run typecheck` はいずれも実行前に
`scripts/ensure-deps-built.sh` が走り、`../../runtime` と `../../tests/cli/packages/kei-hono`
の `dist/index.js` が無ければ自動で `npm install && npm run build` する(v0.9 dogfood で
指摘されたギャップ1の対応案(a))。手動で先に事前ビルドしておいた場合はスキップされる。

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

モジュール分割は CLAUDE.md 不変条件どおり「モジュールパスとディレクトリパスが 1:1」なので、`module workers_api.api` は `workers_api/api.kei` に置く。テンプレートを分割したい場合はサブモジュール(`workers_api/health.kei` / `workers_api/stock.kei` 等)を追加する形で拡張する。**ただし v0.9 現在は cross-module で `record.field.Map<K,V>.get()` が正しく `keiMapGet` 化されないコンパイラギャップ(issue #142)があり、`HttpRequest` を別モジュールに切り出すと生成 TS が実行時エラーになる**。#142 解消までは 1 モジュールに集約するのが安全。

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

### 500 を実演する導線 — `GET /debug/violate`

`handleStock` には `requires req.pathParams.has("sku")` を置いてある。通常経路(Hono の `/stock/:sku` ルーティング下)では sku は必ず埋まるので契約違反は発火しない。テンプレート単独で 500 集約経路を確かめられるように、TS エントリポイントに **`GET /debug/violate` を用意し、pathParams を空にした HttpRequest で `handleStock` を直接呼ぶ**。契約違反 → `KeiContractViolation` → `app.onError` → 500 を実機で観測できる。

```bash
curl -s -w "\nHTTP %{http_code}\n" http://127.0.0.1:8787/debug/violate
# -> {"error":"contract violation","clause":"requires","condition":"..."}
# -> HTTP 500
```

自プロジェクトに写すときはこの `/debug/violate` を残しておくと、`app.onError` の中央処理が実配線どおりに動いていることを常時セルフチェックできる(本番配備時の扱いは下記チェックリスト参照)。恒久 e2e(vitest-pool-workers)は M43 で追加。

## 本番デプロイ前チェックリスト(debug 経路の扱い)

`/debug/violate` と `/debug/ensures-violate` は **契約違反 → 500 集約経路** を実配線で観測するための実演専用エンドポイント。素通しで本番に出ると「常時 500 を返す公開エンドポイント」として露出しうるため、以下の安全弁を設けてある:

- `src/index.ts` の `/debug/*` ミドルウェアが **`env.ENABLE_DEBUG_ROUTES === "true"` のときだけ配線する**。未設定なら 404(Hono の `notFound`)を返す。
- 本テンプレの `wrangler.jsonc` には `vars.ENABLE_DEBUG_ROUTES` を **意図的に書いていない**(＝本番デプロイでは自動的に無効)。
- e2e 実行(`npm test`)では `vitest.config.ts` の `miniflare.bindings.ENABLE_DEBUG_ROUTES = "true"` が注入されるので、workerd 上でだけ有効になる。

**本番デプロイ時にチェックすること**:

- [ ] `wrangler.jsonc` の `vars` に `ENABLE_DEBUG_ROUTES` を追加していないか(未設定＝安全)
- [ ] Cloudflare ダッシュボード側で `ENABLE_DEBUG_ROUTES` を設定していないか
- [ ] デプロイ後に `curl -s -o /dev/null -w "%{http_code}\n" https://<your-worker>/debug/violate` が **404** を返すことを確認
- [ ] 実プロダクションで契約違反実演が不要なら、`src/index.ts` の `/debug/*` ミドルウェア＋2 ハンドラごと削除する(ガードがある以上必須ではないが、コード量削減として)

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
