// M39 e2e: @kei/hono 経由で Hono app に登録された Kei ハンドラの疎通確認。
// app.request(...) による in-memory 呼び出しのみ(実 fetch/wrangler は v0.9 スコープ外)。

import { describe, expect, it } from "vitest";
import { Some } from "@kei/runtime";
import { KeiContractViolation } from "@kei/runtime";

import { app } from "./app";
import type { HttpRequest } from "../../dist/app/http_model";
import { handleCreateUser } from "../../dist/app/http_users";

describe("GET /health", () => {
  it("常に 200 と { status: \"ok\" } を返す", async () => {
    const res = await app.request(new Request("http://localhost/health"));
    expect(res.status).toBe(200);
    expect(await res.json()).toEqual({ status: "ok" });
  });
});

describe("POST /users", () => {
  it("有効な JSON body は 202 と { status: \"accepted\" } を返す", async () => {
    const res = await app.request(
      new Request("http://localhost/users", {
        method: "POST",
        body: JSON.stringify({ name: "Kei" }),
      }),
    );
    expect(res.status).toBe(202);
    expect(await res.json()).toEqual({ status: "accepted" });
  });

  it("パース不能な body は業務ロジック側の判定で 400 を返す(parseUserRequest が None)", async () => {
    // JSON として不正な文字列 → parseApp.parseUserRequest が None を返し、
    // handleCreateUser が badRequestResponse() を直接返す経路(onError は通らない)。
    const res = await app.request(
      new Request("http://localhost/users", {
        method: "POST",
        body: "not json",
      }),
    );
    expect(res.status).toBe(400);
    expect(await res.json()).toEqual({ error: "invalid request" });
  });

  it("name が空文字列は契約違反(requires)経由で 400 を返す(app.onError の中央処理)", async () => {
    // shape check は通る(name は string 型)ので parseUserRequest は Some を返すが、
    // buildCreatedResponse の `requires user.name.length > 0` が発火して
    // KeiContractViolation が投げられ、app.onError が捕捉して 400 に写す。
    // 上のテストとは別経路であることに注意(こちらは onError 経由)。
    const res = await app.request(
      new Request("http://localhost/users", {
        method: "POST",
        body: JSON.stringify({ name: "" }),
      }),
    );
    expect(res.status).toBe(400);
    const body = await res.json();
    expect(body.error).toBe("contract violation");
    expect(body.clause).toBe("requires");
  });

  it("HTTP 層を経由せず直接呼び出しても KeiContractViolation が reject として伝播する", async () => {
    const req: HttpRequest = {
      method: "POST",
      path: "/users",
      headers: new Map(),
      queryParams: new Map(),
      bodyText: Some('{"name":""}'),
    };
    await expect(handleCreateUser(req)).rejects.toThrow(KeiContractViolation);
  });
});
