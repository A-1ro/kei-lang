//! Kei MCP サーバー。spec/ と examples/ をビルド時に埋め込み(ARCHITECTURE.md
//! 不変条件 2)、エージェント向けの取扱説明書として配信する。
//!
//! 言語処理ロジックは持たず、検査・整形は kei_check / kei_fmt / kei_syntax に
//! 委譲する。プロトコル処理は [`Server::handle`] が担い、stdio トランスポートは
//! [`run_stdio`] が包む。起動経路は単一で、`kei-mcp` バイナリ(`src/main.rs`)も
//! `kei mcp` サブコマンド(kei_cli)も同じ [`run_stdio`] を呼ぶ。

use std::io::{self, BufRead, Read, Write};

use serde_json::Value;

pub mod embedded;
pub mod server;
pub mod tools;

pub use server::Server;

/// stdin から読む1行(= JSON-RPCメッセージ1件)の最大バイト数。通常の Kei
/// ソース(kei_check/kei_fmt の `source` 引数)を十分許容しつつ、巨大な1行を
/// 流し込まれて `BufRead::lines()` が無制限にバッファし続ける(メモリ枯渇)のを防ぐ。
/// stdio はローカルプロセス間通信が前提だが、将来 Workers 等ネットワーク越しの
/// 経路にこのトランスポートを再利用する可能性を見て、先に上限を入れておく。
pub const MAX_LINE_BYTES: usize = 1024 * 1024;

/// stdio トランスポートで MCP サーバーを駆動する。改行区切り JSON-RPC を 1 行ずつ
/// 読み、[`Server::handle`] に渡し、応答(通知は無し)を 1 行ずつ書き戻す。stdin が
/// 閉じる(EOF)まで処理を続け、正常終了する。`kei-mcp` バイナリと `kei mcp`
/// サブコマンドの双方がこの単一エントリを共有する。
///
/// 1 行が [`MAX_LINE_BYTES`] を超えたら、その行はパースせず Invalid Request を
/// 返して接続を終了する(以降の入力は読み捨てず、そのままループを抜ける)。
pub fn run_stdio() -> io::Result<()> {
    let server = Server::new();
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut input = stdin.lock();

    loop {
        let mut buf = Vec::new();
        // 上限ちょうどの行は通し、上限超過だけを検出するため +1 バイトまで読む。
        let mut limited = (&mut input).take(MAX_LINE_BYTES as u64 + 1);
        if limited.read_until(b'\n', &mut buf)? == 0 {
            break; // EOF
        }
        if buf.len() > MAX_LINE_BYTES {
            let response = line_too_long_error(MAX_LINE_BYTES);
            serde_json::to_writer(&mut out, &response)?;
            out.write_all(b"\n")?;
            out.flush()?;
            break;
        }
        while matches!(buf.last(), Some(b'\n' | b'\r')) {
            buf.pop();
        }
        let line =
            String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if line.trim().is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<Value>(&line) {
            Ok(request) => server.handle(&request),
            Err(e) => Some(parse_error(&e.to_string())),
        };
        if let Some(response) = response {
            serde_json::to_writer(&mut out, &response)?;
            out.write_all(b"\n")?;
            out.flush()?;
        }
    }
    Ok(())
}

fn parse_error(message: &str) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": null,
        "error": { "code": -32700, "message": format!("Parse error: {message}") },
    })
}

fn line_too_long_error(max_bytes: usize) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": null,
        "error": {
            "code": -32600,
            "message": format!("Invalid Request: line exceeds {max_bytes}-byte limit"),
        },
    })
}
