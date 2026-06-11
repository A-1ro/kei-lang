//! Pact の正規形フォーマッタ。AST の意味的変更は禁止。
//!
//! 実装は M2 で行う(docs/pact-roadmap-goals.md)。

// pact_syntax への依存方向(pact_syntax ← pact_fmt)をビルドグラフに固定する。
use pact_syntax as _;
