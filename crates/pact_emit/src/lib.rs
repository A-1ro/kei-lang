//! Pact → TypeScript トランスパイラ + source map。検査の再実装は禁止。
//!
//! 実装は M4 で行う(docs/pact-roadmap-goals.md)。

// pact_check への依存方向(pact_check ← pact_emit)をビルドグラフに固定する。
use pact_check as _;
