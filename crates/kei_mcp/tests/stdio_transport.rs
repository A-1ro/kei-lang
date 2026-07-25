//! stdio トランスポートの End-to-End 検証。実バイナリ `kei-mcp` を起動し、
//! 改行区切り JSON-RPC を流し込んで応答行を回収する。「MCP サーバーが stdio で
//! 起動する」ことをプロセス境界越しに確認する。

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

#[test]
fn server_starts_and_answers_over_stdio() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kei-mcp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn kei-mcp");

    // 改行区切りで複数リクエストを送り、stdin を閉じてサーバーを終了させる。
    // initialize は「サポート済みだが最新ではない版」を要求する。プロセス境界越しに
    // バージョン交渉(要求版のエコーバック)まで確認するため。
    let requests = concat!(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"stdio-test","version":"0.0.0"}}}"#,
        "\n",
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        "\n",
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"kei_fmt","arguments":{"source":"module demo\n\nfunc double(x: Int) -> Int {\n  return x + x\n}\n"}}}"#,
        "\n",
    );
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(requests.as_bytes())
        .expect("write requests");

    let stdout = child.stdout.take().expect("stdout");
    let lines: Vec<String> = BufReader::new(stdout)
        .lines()
        .map(|l| l.expect("read line"))
        .collect();
    let status = child.wait().expect("wait");
    assert!(status.success(), "server exited with {status}");

    // 通知には応答しないので、応答は initialize と tools/call の 2 行。
    assert_eq!(lines.len(), 2, "expected 2 responses, got: {lines:?}");

    let init: serde_json::Value = serde_json::from_str(&lines[0]).expect("init response is JSON");
    assert_eq!(init["id"], 1);
    assert_eq!(init["result"]["serverInfo"]["name"], "kei-mcp");
    // 要求した 2024-11-05 はサポート対象なので、そのまま返る。
    assert_eq!(init["result"]["protocolVersion"], "2024-11-05");
    // 固定値を返しているだけでないこと(= 実際に交渉していること)の担保。
    assert_ne!(
        init["result"]["protocolVersion"],
        kei_mcp::server::DEFAULT_PROTOCOL_VERSION,
        "requested version must be echoed back instead of the server default"
    );

    let call: serde_json::Value = serde_json::from_str(&lines[1]).expect("call response is JSON");
    assert_eq!(call["id"], 2);
    assert_eq!(call["result"]["isError"], false);
    let text = call["result"]["content"][0]["text"]
        .as_str()
        .expect("formatted text");
    assert!(text.contains("func double"), "fmt output: {text}");
}

/// `MAX_LINE_BYTES` を超える1行を流し込まれても、無制限にバッファし続けて
/// メモリを枯渇させたりせず、Invalid Request を返して接続を終える。
#[test]
fn oversized_line_gets_invalid_request_and_ends_connection() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kei-mcp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn kei-mcp");

    // 上限超過はパース前(サイズ検査)で弾かれるので、有効な JSON である必要はない。
    let oversized = "a".repeat(kei_mcp::MAX_LINE_BYTES + 1);
    let mut stdin = child.stdin.take().expect("stdin");
    stdin
        .write_all(oversized.as_bytes())
        .expect("write oversized line");
    // 上限超過の行の後にも正当なリクエストを続けて送るが、サーバーは接続を
    // 終了しているのでこれが処理されないこと(応答が1件のみ)を確認する。
    //
    // サーバーは上限超過を検出した時点で応答を書いて終了するので、ここから先の
    // 書き込みは相手側のパイプが閉じた後になり得る(BrokenPipe)。閉じているのは
    // まさにこのテストが期待している状態なので、書き込みの失敗はテストの失敗に
    // しない。検証したい契約は「後続が処理されず応答がちょうど1件」であって
    // 「後続の書き込みが成功すること」ではない。
    let _ = stdin.write_all(b"\n");
    let _ = stdin.write_all(br#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#);
    let _ = stdin.write_all(b"\n");
    drop(stdin);

    let stdout = child.stdout.take().expect("stdout");
    let lines: Vec<String> = BufReader::new(stdout)
        .lines()
        .map(|l| l.expect("read line"))
        .collect();
    let status = child.wait().expect("wait");
    assert!(status.success(), "server exited with {status}");

    assert_eq!(
        lines.len(),
        1,
        "expected exactly one Invalid Request response and no further processing, got: {lines:?}"
    );
    let response: serde_json::Value = serde_json::from_str(&lines[0]).expect("response is JSON");
    // `Value` の `Index` はキー欠落でも Null を返すので、キーの存在ごと固定する
    // (spec は id を検出できないとき null を **含めて** 返すことを要求している)。
    assert_eq!(
        response.get("id"),
        Some(&serde_json::Value::Null),
        "id must be present and null: {response}"
    );
    assert_eq!(response["error"]["code"], -32600);
}

// ---------------------------------------------------------------------------
// 上限の境界。`MAX_LINE_BYTES` は「行末の区切りを除いた JSON 本体」のバイト数に
// 対する上限なので、LF 終端・CRLF 終端・改行なし EOF のどれでも
// 本体 MAX ちょうどは受理・MAX+1 は拒否、で一致していなければならない。
// (区切り文字を含む長さで判定すると、行末の種類ごとに実効上限がずれる)
// ---------------------------------------------------------------------------

/// 指定バイト数ちょうどの JSON-RPC リクエスト本体(行末の区切りは含まない)を作る。
/// `ping` は params を見ないので、パディングは応答内容に影響しない。
fn ping_request_of_len(len: usize) -> Vec<u8> {
    let prefix = br#"{"jsonrpc":"2.0","id":1,"method":"ping","params":{"pad":""#;
    let suffix = br#""}}"#;
    let pad = len
        .checked_sub(prefix.len() + suffix.len())
        .expect("requested length must fit the JSON envelope");
    let mut body = Vec::with_capacity(len);
    body.extend_from_slice(prefix);
    body.resize(prefix.len() + pad, b'a');
    body.extend_from_slice(suffix);
    assert_eq!(body.len(), len, "payload must be exactly {len} bytes");
    body
}

/// 本体 + 行末を実バイナリの stdin に流し込み、応答行を回収する。
fn responses_for(body: &[u8], terminator: &[u8]) -> Vec<String> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kei-mcp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn kei-mcp");

    let mut stdin = child.stdin.take().expect("stdin");
    // 上限超過のときサーバーは残りを読まずに終了するため、書き込みの途中で
    // パイプが閉じ得る(BrokenPipe)。閉じている状態こそ期待どおりなので、
    // 書き込みの失敗はテストの失敗にしない。検証したい契約は応答の内容。
    let _ = stdin.write_all(body);
    let _ = stdin.write_all(terminator);
    drop(stdin);

    let stdout = child.stdout.take().expect("stdout");
    let lines: Vec<String> = BufReader::new(stdout)
        .lines()
        .map(|l| l.expect("read line"))
        .collect();
    let status = child.wait().expect("wait");
    assert!(status.success(), "server exited with {status}");
    lines
}

/// 上限内として処理された(= ping の成功応答が返った)ことを確認する。
fn assert_accepted(lines: &[String]) {
    assert_eq!(
        lines.len(),
        1,
        "expected exactly one response, got: {lines:?}"
    );
    let response: serde_json::Value = serde_json::from_str(&lines[0]).expect("response is JSON");
    assert_eq!(
        response.get("id"),
        Some(&serde_json::Value::from(1)),
        "id must be echoed back: {response}"
    );
    assert_eq!(
        response.get("error"),
        None,
        "body at the limit must not be rejected: {response}"
    );
    assert_eq!(
        response.get("result"),
        Some(&serde_json::json!({})),
        "ping result must be present and empty: {response}"
    );
}

/// 上限超過として弾かれた(= Invalid Request が返った)ことを確認する。
fn assert_rejected(lines: &[String]) {
    assert_eq!(
        lines.len(),
        1,
        "expected exactly one response, got: {lines:?}"
    );
    let response: serde_json::Value = serde_json::from_str(&lines[0]).expect("response is JSON");
    assert_eq!(
        response.get("id"),
        Some(&serde_json::Value::Null),
        "id must be present and null: {response}"
    );
    assert_eq!(
        response.get("result"),
        None,
        "rejected request must not carry a result: {response}"
    );
    assert_eq!(response["error"]["code"], -32600);
}

#[test]
fn body_at_limit_with_lf_is_accepted() {
    let body = ping_request_of_len(kei_mcp::MAX_LINE_BYTES);
    assert_accepted(&responses_for(&body, b"\n"));
}

#[test]
fn body_over_limit_with_lf_is_rejected() {
    let body = ping_request_of_len(kei_mcp::MAX_LINE_BYTES + 1);
    assert_rejected(&responses_for(&body, b"\n"));
}

#[test]
fn body_at_limit_with_crlf_is_accepted() {
    let body = ping_request_of_len(kei_mcp::MAX_LINE_BYTES);
    assert_accepted(&responses_for(&body, b"\r\n"));
}

#[test]
fn body_over_limit_with_crlf_is_rejected() {
    let body = ping_request_of_len(kei_mcp::MAX_LINE_BYTES + 1);
    assert_rejected(&responses_for(&body, b"\r\n"));
}

#[test]
fn body_at_limit_without_trailing_newline_is_accepted() {
    let body = ping_request_of_len(kei_mcp::MAX_LINE_BYTES);
    assert_accepted(&responses_for(&body, b""));
}

#[test]
fn body_over_limit_without_trailing_newline_is_rejected() {
    let body = ping_request_of_len(kei_mcp::MAX_LINE_BYTES + 1);
    assert_rejected(&responses_for(&body, b""));
}
