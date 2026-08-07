// examples/effects/parallel.kei の実行テスト(M47 / #161 並行結合子)。
// `parallel([...])` が実際に (1) 直列版と結果等価であり、(2) 観測可能な形で並行実行される
// (合計待ち時間が直列和より短い/完了順が入力順と一致しなくても List の順序は入力順のまま)
// ことを、擬似 I/O(setTimeout ベースの infra.remote スタブ)で固定する。

import { describe, expect, it } from "vitest";

import { fetchUsersInParallel, fetchUsersSequential } from "../generated/effects/parallel";
import { completionOrder, resetProbe, setDelay } from "../generated/infra/remote";

describe("effects/parallel — 並行結合子 parallel(xs) -> List<T>(M47)", () => {
  it("直列版と並行版は同じ結果(入力順の List<String>)を返す", async () => {
    resetProbe();
    setDelay(1, 5);
    setDelay(2, 5);
    const sequential = await fetchUsersSequential(1, 2);
    resetProbe();
    setDelay(1, 5);
    setDelay(2, 5);
    const parallel = await fetchUsersInParallel(1, 2);
    expect(parallel).toEqual(sequential);
    expect(parallel).toEqual(["user-1", "user-2"]);
  });

  it("並行版は直列版より速い(合計待ち時間が直列和より短い)", async () => {
    resetProbe();
    setDelay(3, 40);
    setDelay(4, 40);
    const seqStart = performance.now();
    await fetchUsersSequential(3, 4);
    const seqElapsed = performance.now() - seqStart;

    resetProbe();
    setDelay(3, 40);
    setDelay(4, 40);
    const parStart = performance.now();
    await fetchUsersInParallel(3, 4);
    const parElapsed = performance.now() - parStart;

    // 直列は概ね 80ms(40+40)、並行は概ね 40ms(max)。タイマーの揺れを見込んで
    // 「並行版は直列版の 2/3 未満」という緩い閾値で観測可能性を固定する。
    expect(parElapsed).toBeLessThan(seqElapsed * (2 / 3));
  });

  it("並行版は完了順が入力順と異なっても、結果 List は入力順のまま(Promise.all 準拠)", async () => {
    resetProbe();
    // id=6 を id=5 より先に完了させる(逆順で resolve)。
    setDelay(5, 30);
    setDelay(6, 5);
    const result = await fetchUsersInParallel(5, 6);
    // 完了順は 6 が先(観測可能な並行性)。
    expect(completionOrder()).toEqual([6, 5]);
    // にもかかわらず結果は呼び出し引数の順序(5, 6)のまま。
    expect(result).toEqual(["user-5", "user-6"]);
  });
});
