//! `pact` CLI。check / fmt / build / test のサブコマンドは M0 以降で実装する。
//! 言語処理ロジックは持たず、Diagnostic の散文整形と引数解釈・ファイル IO のみを担う。

use pact_check as _;
use pact_emit as _;
use pact_fmt as _;
use pact_syntax as _;

fn main() {
    eprintln!("pact: not yet implemented (see docs/pact-roadmap-goals.md)");
    std::process::exit(1);
}
