// app/async_greet の実行テスト。async 関数の ensures 違反が Promise resolve 後に
// throw で伝播し、await 側で例外として捕捉されることを確認する(M38)。

import { describe, expect, it } from "vitest";
import { KeiContractViolation } from "@kei/runtime";

import { fetchName } from "../dist/app/async_greet";

describe("app/async_greet", () => {
  it("ensures 違反は Promise の reject として伝播し、await 側で throw になる", async () => {
    await expect(fetchName(1)).rejects.toThrow(KeiContractViolation);
  });
});
