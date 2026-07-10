// M39 e2e: Kei ハンドラ(GET /health, POST /users)を Hono app に登録するヘルパー。
// vitest のテストファイルではなく、app.test.ts から import されるアプリ組み立てのみを行う。

import { Hono } from "hono";
import { mount } from "@kei/hono";
import { KeiContractViolation } from "@kei/runtime";

import { handleHealth } from "../../dist/app/http_health";
import { handleCreateUser } from "../../dist/app/http_users";

export const app = new Hono();

mount(app, "get", "/health", handleHealth);
mount(app, "post", "/users", handleCreateUser);

// 契約違反(KeiContractViolation)は mount() 内では捕捉せずそのまま投げているので、
// ここで中央処理して 400 に写す。JSON parse 失敗(業務ロジック側が直接 400 を返す
// パス)とは別経路であることに注意 — こちらは Hono の例外処理経由。
app.onError((err, c) => {
  if (err instanceof KeiContractViolation) {
    return c.json(
      { error: "contract violation", clause: err.clause, condition: err.condition },
      400,
    );
  }
  throw err;
});
