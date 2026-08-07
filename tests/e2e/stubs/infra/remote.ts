// e2e スタブ: examples が import する infra.remote のインメモリ実装(M47 並行 async 用の
// 擬似 I/O)。extern Remote.fetchName(id: Int) -> String uses Network.Read, Async に対応する。
// `setDelay` で id ごとの擬似レイテンシを設定し、`setTimeout` で本物の非同期待ちを再現する。
// これにより (1) 直列/並行の結果一致、(2) 並行実行(合計待ち時間が直列和より短い)、
// (3) 完了順が入力順と異なっても List の順序は入力順のまま、を e2e で観測できる。

const delays = new Map<number, number>();
const completionLog: number[] = [];

export function setDelay(id: number, ms: number): void {
  delays.set(id, ms);
}

export function resetProbe(): void {
  delays.clear();
  completionLog.length = 0;
}

export function completionOrder(): readonly number[] {
  return completionLog;
}

export function fetchName(id: number): Promise<string> {
  const ms = delays.get(id) ?? 0;
  return new Promise((resolve) => {
    setTimeout(() => {
      completionLog.push(id);
      resolve(`user-${id}`);
    }, ms);
  });
}
