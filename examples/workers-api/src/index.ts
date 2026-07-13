// Kei v0.9 M42 テンプレート: Workers エントリポイント。
//
// 設計原則(v0.9 設計原則 3/4):
//   - env / ExecutionContext は TS 側だけが触る。Kei ハンドラには平坦な HttpRequest だけ渡す。
//   - Kei は API のビジネスロジック(dist/)、TS は Hono app 組み立て + mount + 中央例外処理。
//   - wrangler が bundling を担う。
//
// 契約違反(KeiContractViolation)は「サーバ不変条件の破れ」として **500** で写す
// (v0.9 M41 統一)。クライアント都合のエラーは Kei ハンドラ側のロジックで直接 4xx を返す。
import { Hono } from "hono";
import { mount } from "@kei/hono";
import { KeiContractViolation, None } from "@kei/runtime";

import { handleHealth, handleStock, brokenEnsuresHandler } from "../dist/workers_api/api";

const app = new Hono();

// GET /health — 契約なし・依存なしの最小ハンドラ。
mount(app, "get", "/health", handleHealth);

// GET /stock/:sku — 依存(在庫マスタ)を closure DI で注入する定型。
// 実プロジェクトでは以下の inventory を KV / D1 / vars から fetch(request, env, ctx) の中で
// 組み立てて渡す(bindings は Kei に露出させない — 設計原則 3)。
const inventory = new Map<string, number>([
  ["ABC-1", 42],
  ["XYZ-9", 0],
]);
const stockDeps = { inventory };

mount(app, "get", "/stock/:sku", (req) => handleStock(stockDeps, req));

// /debug/* は「契約違反 → 500 集約経路」を実配線で観測するためのテンプレ内実演導線。
// 本番デプロイに紛れ込むと『常時 500 を返す公開エンドポイント』として露出しうるため、
// **`ENABLE_DEBUG_ROUTES` 環境変数が "true" のときだけ配線する**(未設定＝配線なし＝404)。
// e2e (vitest-pool-workers) 側は vitest.config.ts の `miniflare.bindings` で "true" を注入する。
// 本番用 wrangler.jsonc の `vars` にはこの変数を書かない。README のチェックリスト参照。
type DebugEnv = { ENABLE_DEBUG_ROUTES?: string };

app.use("/debug/*", async (c, next) => {
  const env = c.env as DebugEnv | undefined;
  if (env?.ENABLE_DEBUG_ROUTES !== "true") {
    return c.notFound();
  }
  await next();
});

// GET /debug/violate — requires 違反 → 500 集約経路の実演専用エンドポイント。
// pathParams を空にした HttpRequest で handleStock を直接呼び、`requires
// req.pathParams.has("sku")` を確実に破る。app.onError → 500 の中央処理が
// 実配線どおりに動くかを、テンプレート単独(M43 の pool-workers e2e を待たず)で確かめる導線。
app.get("/debug/violate", () => {
  const badReq = {
    method: "GET",
    path: "/debug/violate",
    headers: new Map<string, string>(),
    queryParams: new Map<string, string>(),
    pathParams: new Map<string, string>(),
    bodyText: None(),
  };
  const res = handleStock(stockDeps, badReq);
  // requires が破れているのでここには到達しない。到達したら 500 集約経路が壊れている合図。
  return new Response(res.bodyText, { status: res.status });
});

// GET /debug/ensures-violate — ensures 違反 → 500 集約経路の実演専用エンドポイント。
mount(app, "get", "/debug/ensures-violate", brokenEnsuresHandler);

// 契約違反 → 500 の中央処理。requires / ensures の区別はしない
// (どちらも「サーバ不変条件が破れた」= サーバ異常 として 500)。
app.onError((err, c) => {
  if (err instanceof KeiContractViolation) {
    return c.json(
      { error: "contract violation", clause: err.clause, condition: err.condition },
      500,
    );
  }
  throw err;
});

export default app;
