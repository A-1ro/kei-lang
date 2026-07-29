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

/// stdin から読む JSON-RPC メッセージ1件、すなわち **行末の区切り(LF / CRLF)を
/// 除いた JSON 本体**の最大バイト数。通常の Kei ソース(kei_check/kei_fmt の
/// `source` 引数)を十分許容しつつ、巨大な1行を流し込まれて `BufRead::lines()` が
/// 無制限にバッファし続ける(メモリ枯渇)のを防ぐ。
///
/// 上限は本体のバイト数に課すので、受理される本体の最大バイト数は行末の種類に
/// よらずこの値で一致する(LF 終端・CRLF 終端・改行なし EOF のいずれでも
/// 本体 `MAX_LINE_BYTES` バイトちょうどまで受理し、+1 バイトから拒否する)。
///
/// stdio はローカルプロセス間通信が前提だが、将来 Workers 等ネットワーク越しの
/// 経路にこのトランスポートを再利用する可能性を見て、先に上限を入れておく。
pub const MAX_LINE_BYTES: usize = 1024 * 1024;

/// stdio トランスポートで MCP サーバーを駆動する。改行区切り JSON-RPC を 1 行ずつ
/// 読み、[`Server::handle`] に渡し、応答(通知は無し)を 1 行ずつ書き戻す。stdin が
/// 閉じる(EOF)まで処理を続け、正常終了する。`kei-mcp` バイナリと `kei mcp`
/// サブコマンドの双方がこの単一エントリを共有する。
///
/// 1 行の JSON 本体(行末の LF / CRLF を除いた部分)が [`MAX_LINE_BYTES`] を
/// 超えたら、その行はパースせず Invalid Request を返して接続を終了する
/// (以降の入力は読み捨てず、そのままループを抜ける)。
pub fn run_stdio() -> io::Result<()> {
    let server = Server::new();
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut input = stdin.lock();

    loop {
        let mut buf = Vec::new();
        // 上限ちょうどの本体を LF 終端でも CRLF 終端でも通しつつ超過を検出するため、
        // 本体の上限 + 行末 2 バイトまで読む。1 行あたりの確保量はこの take で
        // 頭打ちになり、どれだけ長い行を流し込まれても MAX_LINE_BYTES + 2 バイトを
        // 超えてバッファすることはない(これが本上限の主目的)。
        let mut limited = (&mut input).take(MAX_LINE_BYTES as u64 + 2);
        if limited.read_until(b'\n', &mut buf)? == 0 {
            break; // EOF
        }
        // 行末の区切り(LF / CRLF)だけを落とし、残りを JSON 本体として扱う。
        if buf.last() == Some(&b'\n') {
            buf.pop();
            if buf.last() == Some(&b'\r') {
                buf.pop();
            }
        }
        // 上限は JSON 本体のバイト数に課す。read_until の戻り値(区切り文字を含む
        // 長さ)で測ると、実効上限が行末の種類でずれてしまうため。
        if buf.len() > MAX_LINE_BYTES {
            let response = body_too_long_error(MAX_LINE_BYTES);
            serde_json::to_writer(&mut out, &response)?;
            out.write_all(b"\n")?;
            out.flush()?;
            break;
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

/// 上限超過の応答。上限は行全体ではなく JSON 本体(行末の LF / CRLF を除いた
/// 部分)のバイト数に対するものなので、メッセージでもそれが分かるようにする。
fn body_too_long_error(max_bytes: usize) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": null,
        "error": {
            "code": -32600,
            "message": format!(
                "Invalid Request: JSON body exceeds {max_bytes}-byte limit \
                 (line terminator excluded)"
            ),
        },
    })
}
