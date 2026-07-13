import { SELF } from "cloudflare:test";
import { describe, expect, it } from "vitest";

describe("GET /health", () => {
  it("200 { status: 'ok' }", async () => {
    const res = await SELF.fetch("http://example.com/health");
    expect(res.status).toBe(200);
    expect(await res.json()).toEqual({ status: "ok" });
  });
});

describe("GET /stock/:sku (closure DI)", () => {
  it("在庫あり → 200 と qty(closure DI した inventory の値)", async () => {
    const res = await SELF.fetch("http://example.com/stock/ABC-1");
    expect(res.status).toBe(200);
    expect(await res.json()).toEqual({ qty: 42 });
  });

  it("未知の sku → 業務ロジックが 404 を返す(契約違反ではない)", async () => {
    const res = await SELF.fetch("http://example.com/stock/UNKNOWN");
    expect(res.status).toBe(404);
    expect(await res.json()).toEqual({ error: "not found" });
  });
});

describe("契約違反 → 500(業務エラー 400 系とは別経路)", () => {
  it("requires 違反(/debug/violate)は 500 と clause: 'requires'", async () => {
    const res = await SELF.fetch("http://example.com/debug/violate");
    expect(res.status).toBe(500);
    const body = (await res.json()) as { error: string; clause: string; condition: string };
    expect(body.error).toBe("contract violation");
    expect(body.clause).toBe("requires");
    expect(typeof body.condition).toBe("string");
    expect(body.condition.length).toBeGreaterThan(0);
  });

  it("ensures 違反(/debug/ensures-violate)は 500 と clause: 'ensures'", async () => {
    const res = await SELF.fetch("http://example.com/debug/ensures-violate");
    expect(res.status).toBe(500);
    const body = (await res.json()) as { error: string; clause: string; condition: string };
    expect(body.error).toBe("contract violation");
    expect(body.clause).toBe("ensures");
    expect(typeof body.condition).toBe("string");
    // requires 側と対称: どの契約式が破れたかの情報が空文字列に退化していないことを守る。
    expect(body.condition.length).toBeGreaterThan(0);
  });
});
