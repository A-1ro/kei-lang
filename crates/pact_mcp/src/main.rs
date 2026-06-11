//! Pact MCP サーバー。spec/ と examples/ のビルド時埋め込み配信は M5 で実装する。
//! 言語処理ロジックは持たない。

use pact_check as _;
use pact_emit as _;
use pact_fmt as _;
use pact_syntax as _;

fn main() {
    eprintln!("pact-mcp: not yet implemented (see docs/pact-roadmap-goals.md)");
    std::process::exit(1);
}
