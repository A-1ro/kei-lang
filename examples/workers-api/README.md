# examples/workers-api — Kei v0.9 M42 雛形 & M40 スパイク

Cloudflare Workers 上で Kei が書いた API ハンドラを Hono + `@kei/hono` 経由で動かす最小テンプレート。

- **M40** (このディレクトリの起点): file: 依存 × wrangler(esbuild) bundling + `wrangler dev` の疎通確認スパイク。
- **M42** で在庫 API に肉付けする前提の骨組み。恒久 e2e は **M43** の vitest-pool-workers 側で担う(このディレクトリでは `wrangler dev` は手動確認専用)。

## 責務分担 (v0.9 設計原則 4)

| 層 | 何を書くか | 場所 |
|---|---|---|
| Kei | API のビジネスロジック(ハンドラ関数・record・契約) | `*.kei` → `dist/` |
| TS | Workers エントリポイント / Hono app 組み立て / `mount()` / 契約違反の中央処理 | `src/index.ts` |
| wrangler | bundling(esbuild)と workerd 起動 | `wrangler.jsonc` |

`env` / `ExecutionContext` は **TS 側の Workers エントリだけが触る**。Kei ハンドラには平坦な `HttpRequest` レコードだけ渡す(設計原則 3)。

## 手順(M40 スパイクを再現する場合)

```bash
# 1. @kei/runtime と @kei/hono の dist を作る(file: 依存の解決先)
(cd ../../runtime && npm install --no-audit --no-fund && npm run build)
(cd ../../tests/cli/packages/kei-hono && npm install --no-audit --no-fund && npm run build)

# 2. Kei ソースを TS にトランスパイル
cargo run -q -p kei_cli --bin kei -- build examples/workers-api --out-dir examples/workers-api/dist

# 3. workers-api の依存を入れる
npm install --no-audit --no-fund

# 4. wrangler の bundling が通ることを確認
npx wrangler deploy --dry-run --outdir dist-wrangler

# 5. wrangler dev を起動し GET /health を叩く
npx wrangler dev --port 8787 --local &
curl -s http://127.0.0.1:8787/health   # -> {"status":"ok"}
```

## v0.9 時点での制限

- 実 Cloudflare へのデプロイ(`wrangler deploy` 本番)は **v1.0 の領分**。ここでは行わない。
- 契約違反の中央処理(`src/index.ts` の `app.onError`)は現状 **500** で写している(v0.9 契約書 §M41)。
- M40 スパイクではハンドラは `GET /health` のみ。パスパラメータ・DI・在庫 API は M41/M42 で追加。
