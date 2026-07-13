// M39 e2e: Kei ハンドラ(GET /health, POST /users)を Hono app に登録するヘルパー。
// vitest のテストファイルではなく、app.test.ts から import されるアプリ組み立てのみを行う。

import { Hono } from "hono";
import { mount } from "@kei/hono";
import { KeiContractViolation } from "@kei/runtime";

import { handleHealth } from "../../dist/app/http_health";
import { handleCreateUser } from "../../dist/app/http_users";
import { handleStock } from "../../dist/app/http_stock";
import { brokenHandler } from "../../dist/app/http_ensures_probe";

export const app = new Hono();

mount(app, "get", "/health", handleHealth);
mount(app, "post", "/users", handleCreateUser);
mount(app, "get", "/ensures-probe", brokenHandler);

// closure DI: deps(在庫データ)は Kei 関数の第1引数として渡り、TS 側で部分適用する
// (deps は Kei 側に露出しない Workers bindings 等の代わりに使う定型パターン。
// SKILL.md 「パスパラメータと closure DI」参照)。
const inventory = new Map<string, number>([
  ["ABC-1", 42],
  ["XYZ-9", 0],
]);
const stockDeps = { inventory };

mount(app, "get", "/stock/:sku", (req) => handleStock(stockDeps, req));

// 契約違反(KeiContractViolation)は mount() 内では捕捉せずそのまま投げているので、
// ここで中央処理して 500 に写す。契約違反は「サーバ不変条件の破れ」としてサーバ
// エラー(500)。クライアント都合のエラー(JSON parse 失敗など)はロジック側が
// 直接 400 を返す別経路(こちらは Hono の例外処理経由)。
app.onError((err, c) => {
  if (err instanceof KeiContractViolation) {
    return c.json(
      { error: "contract violation", clause: err.clause, condition: err.condition },
      500,
    );
  }
  throw err;
});
