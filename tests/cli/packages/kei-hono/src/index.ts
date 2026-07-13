// @kei/hono — Hono の Context ↔ Kei HttpRequest/HttpResponse の変換アダプタ(M39)。
//
// なぜこの設計か: Hono の Context は req/res の他に env・executionCtx・var など
// Cloudflare Workers 実行時の広い面を持つオブジェクトで、Kei 側に丸ごと渡すと
// (1) Kei のエフェクトシステムが把握できない副作用の抜け道になる、
// (2) Kei の record は構造的な値型なので Context のようなメソッド持ちオブジェクトを
//     表現するには不向き、という2つの理由で相性が悪い。
// そこで Context から Kei の record と 1:1 対応する平坦な HttpRequest を組み立て、
// ハンドラの戻り値 HttpResponse から標準 Response を組み立て直す、という
// 「境界でだけ変換する」設計にしている。Kei ハンドラ本体は Hono にも fetch にも
// 依存しない純粋な record 変換関数のまま書ける。

import type { Context, Hono } from "hono";
import { type Option, Some, None } from "@kei/runtime";

export interface HttpRequest {
  readonly method: string;
  readonly path: string;
  readonly headers: ReadonlyMap<string, string>;
  readonly queryParams: ReadonlyMap<string, string>;
  readonly pathParams: ReadonlyMap<string, string>;
  readonly bodyText: Option<string>;
}

export interface HttpResponse {
  readonly status: number;
  readonly headers: ReadonlyMap<string, string>;
  readonly bodyText: string;
}

/**
 * Hono の Context から Kei 側の HttpRequest を組み立てる。
 * bodyText は常に Some(bodyText) を返す(GET のように body が無いリクエストは
 * Content-Length: 0 で `Some("")` になる)。「body 無し」と「空文字列」を区別せず
 * 畳み込むと境界層で情報が失われるため、判定は呼び出し側の Kei ハンドラに委ねる。
 */
export async function fromContext(c: Context): Promise<HttpRequest> {
  const bodyText = await c.req.text();
  return {
    method: c.req.method,
    path: c.req.path,
    headers: new Map(Object.entries(c.req.header())),
    queryParams: new Map(Object.entries(c.req.query())),
    pathParams: new Map(Object.entries(c.req.param())),
    bodyText: Some(bodyText),
  };
}

/** Kei 側の HttpResponse から標準 Response を組み立てる。 */
export function toResponse(res: HttpResponse): Response {
  const headers = new Headers();
  for (const [key, value] of res.headers) {
    headers.set(key, value);
  }
  return new Response(res.bodyText, { status: res.status, headers });
}

/**
 * JSON パース + shape check を1つの Option に畳み込む汎用ヘルパー。
 * record 固有の shape check(例: UserRequest の name フィールド確認)はアプリ側が
 * shapeCheck として渡す。@kei/hono 自体はどの record にも依存しない。
 */
export function parseAs<T>(
  text: string,
  shapeCheck: (value: unknown) => value is T,
): Option<T> {
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch {
    return None();
  }
  return shapeCheck(parsed) ? Some(parsed) : None();
}

export type HttpMethod = "get" | "post" | "put" | "delete" | "patch";

/** 値そのもの、または Promise で解決されるその値。 */
export type Awaitable<T> = T | Promise<T>;

/**
 * Kei ハンドラ(HttpRequest -> Awaitable<HttpResponse>)を Hono のルートとして登録する。
 * ハンドラの型は同期・非同期どちらも受け付ける — Kei 側で `uses Async` を宣言していない
 * ハンドラは同期関数として生成されるため、ここで `Promise` を強制すると実際には非同期
 * 呼び出しを含まないハンドラにまで `uses Async` を強いてしまい、Kei のエフェクトシステムが
 * 「本当に非同期か」を表す意味が鈍る。呼び出しは `await handler(req)` のままなので、
 * 同期値・Promise のどちらが返っても正しく動く。
 * 契約違反(KeiContractViolation)はここで捕捉せず、そのまま投げる。呼び出し側が
 * `app.onError` で中央処理する設計(SKILL.md 「HTTP ハンドラを書く」節を参照)。
 */
export function mount(
  app: Hono,
  method: HttpMethod,
  path: string,
  handler: (req: HttpRequest) => Awaitable<HttpResponse>,
): void {
  const routeHandler = async (c: Context): Promise<Response> => {
    const req = await fromContext(c);
    const res = await handler(req);
    return toResponse(res);
  };
  switch (method) {
    case "get":
      app.get(path, routeHandler);
      break;
    case "post":
      app.post(path, routeHandler);
      break;
    case "put":
      app.put(path, routeHandler);
      break;
    case "delete":
      app.delete(path, routeHandler);
      break;
    case "patch":
      app.patch(path, routeHandler);
      break;
  }
}
