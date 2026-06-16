// examples/contracts/borrow.kei の実行テスト(#21 短期分):
// 在庫の数量保存(「ちょうど 1 減る」)を純粋ヘルパー decrementAvailable に退避し、
// borrowBook がそれを経由することで、外部状態の数量的不変条件を担保する。

import { beforeEach, describe, expect, it } from "vitest";
import { KeiContractViolation } from "@kei/runtime";

import { BookId, borrowBook, decrementAvailable } from "../generated/contracts/borrow";
import * as Database from "../generated/infra/database";

const dune = BookId("dune");

beforeEach(() => {
  Database.reset();
});

describe("contracts/borrow", () => {
  it("borrowBook は在庫をちょうど 1 減らして Ok(残数) を返す", () => {
    Database.seedAvailable(dune, 3);
    const result = borrowBook(dune);
    expect(result.isOk).toBe(true);
    if (result.isOk) {
      expect(result.value).toBe(2);
    }
    // 外部状態(在庫)もちょうど 1 減っている。
    const after = Database.fetchAvailable(dune);
    expect(after.isSome).toBe(true);
    if (after.isSome) {
      expect(after.value).toBe(2);
    }
  });

  it("在庫ゼロなら Err(OutOfStock) を返し、在庫は変えない", () => {
    Database.seedAvailable(dune, 0);
    const result = borrowBook(dune);
    expect(result.isErr).toBe(true);
    if (result.isErr) {
      expect(result.error.kind).toBe("OutOfStock");
    }
    const after = Database.fetchAvailable(dune);
    if (after.isSome) {
      expect(after.value).toBe(0);
    }
  });

  it("存在しない本は else fail が Err(NotFound) に展開される", () => {
    const result = borrowBook(dune);
    expect(result.isErr).toBe(true);
    if (result.isErr) {
      expect(result.error.kind).toBe("NotFound");
    }
  });

  it("純粋ヘルパー decrementAvailable は requires available > 0 を実行時に強制する", () => {
    expect(decrementAvailable(5)).toBe(4);
    let thrown: unknown;
    try {
      decrementAvailable(0);
    } catch (e) {
      thrown = e;
    }
    expect(thrown).toBeInstanceOf(KeiContractViolation);
    const violation = thrown as KeiContractViolation;
    expect(violation.clause).toBe("requires");
    expect(violation.func).toBe("decrementAvailable");
    expect(violation.condition).toBe("available > 0");
  });
});
