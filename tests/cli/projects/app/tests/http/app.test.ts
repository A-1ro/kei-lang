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
    // handleHealth は uses Async を宣言しない同期ハンドラ(kei_emit は Promise を
    // 返さない plain function を生成する)。mount() の handler 型が
    // Awaitable<HttpResponse> に緩和されたことで、この同期ハンドラも
    // `await handler(req)` 経由でそのまま疎通することを確認する(M39 レビュー対応)。
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

  it("name が空文字列は契約違反(requires)経由で 500 を返す(app.onError の中央処理)", async () => {
    // shape check は通る(name は string 型)ので parseUserRequest は Some を返すが、
    // buildCreatedResponse の `requires user.name.length > 0` が発火して
    // KeiContractViolation が投げられ、app.onError が捕捉して 500 に写す
    // (契約違反 = サーバ不変条件の破れとして 500 に統一。M41)。
    // 上のテストとは別経路であることに注意(こちらは onError 経由)。
    const res = await app.request(
      new Request("http://localhost/users", {
        method: "POST",
        body: JSON.stringify({ name: "" }),
      }),
    );
    expect(res.status).toBe(500);
    const body = await res.json();
    expect(body.error).toBe("contract violation");
    expect(body.clause).toBe("requires");
  });

  it("HTTP 層を経由せず直接呼び出しても KeiContractViolation が同期的に伝播する", () => {
    // handleCreateUser は uses Async を宣言していないため kei_emit は同期関数を生成する
    // (mount() の handler 型が Awaitable<HttpResponse> に緩和されたことで、これは
    // 契約違反 = reject ではなく同期 throw になる — M39 レビュー対応)。
    const req: HttpRequest = {
      method: "POST",
      path: "/users",
      headers: new Map(),
      queryParams: new Map(),
      pathParams: new Map(),
      bodyText: Some('{"name":""}'),
    };
    expect(() => handleCreateUser(req)).toThrow(KeiContractViolation);
  });
});

describe("GET /ensures-probe", () => {
  it("ensures 違反は契約違反経由で 500 を返す(clause: \"ensures\")", async () => {
    const res = await app.request(new Request("http://localhost/ensures-probe"));
    expect(res.status).toBe(500);
    const body = await res.json();
    expect(body.error).toBe("contract violation");
    expect(body.clause).toBe("ensures");
  });
});

describe("GET /stock/:sku", () => {
  it("在庫があれば 200 と qty を返す", async () => {
    const res = await app.request(new Request("http://localhost/stock/ABC-1"));
    expect(res.status).toBe(200);
    expect(await res.json()).toEqual({ qty: 42 });
  });

  it("未知の sku は 404 を返す", async () => {
    const res = await app.request(new Request("http://localhost/stock/UNKNOWN"));
    expect(res.status).toBe(404);
    expect(await res.json()).toEqual({ error: "not found" });
  });
});
