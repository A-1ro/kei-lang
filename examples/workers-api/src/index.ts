// Kei v0.9 M40 スパイク兼 M42 雛形: Workers エントリポイント。
//
// 設計原則(v0.9 設計原則 3/4):
//   - env / ExecutionContext は TS 側だけが触る。Kei ハンドラには平坦な HttpRequest だけ渡す。
//   - Kei は API のビジネスロジック(dist/)、TS は Hono app 組み立て + mount + 中央例外処理。
//   - wrangler が bundling を担う。
//
// 契約違反(KeiContractViolation)の中央処理は M39 実装踏襲で 400 のまま。
// M41 で 500 に統一予定(v0.9 契約書「契約が破られた時点でサーバ異常」)。
import { Hono } from "hono";
import { mount } from "@kei/hono";
import { KeiContractViolation } from "@kei/runtime";

import { handleHealth } from "../dist/workers_api/http_health";

const app = new Hono();

mount(app, "get", "/health", handleHealth);

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
