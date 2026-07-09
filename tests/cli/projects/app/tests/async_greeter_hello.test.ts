// app/async_greeter_hello の実行テスト。extern package 束縛(async-greeter)が file: 依存の
// 実 async パッケージへ実際に届き、await が正しく効くことを確認する(M38)。

import { describe, expect, it } from "vitest";

import { helloAsync } from "../dist/app/async_greeter_hello";

describe("app/async_greeter_hello", () => {
  it("helloAsync は extern package 経由で async-greeter.greetAsync を呼び、実 async パッケージの結果を返す", async () => {
    await expect(helloAsync("Kei")).resolves.toBe("Hello, Kei! (async)");
  });
});
