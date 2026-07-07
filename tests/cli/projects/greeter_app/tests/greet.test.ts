// greeter_app/greet の実行テスト。extern package 束縛(greeter)が file: 依存の
// 実パッケージへ実際に届くこと(トランスパイル境界だけでなく実行結果まで)を確認する。

import { describe, expect, it } from "vitest";

import { hello } from "../dist/greeter_app/greet";

describe("greeter_app/greet", () => {
  it("hello は extern package 経由で greeter.greet を呼び、実パッケージの結果を返す", () => {
    expect(hello("Kei")).toBe("Hello, Kei!");
  });
});
