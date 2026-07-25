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
        // 非オブジェクトメッセージ(配列)は無応答で捨てず Invalid Request を返す。
        r#"[1,2,3]"#,
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

    // 通知には応答しないが、非オブジェクトメッセージには Invalid Request で応答する
    // ので、応答は initialize / tools/call / 配列メッセージの 3 行。
    assert_eq!(lines.len(), 3, "expected 3 responses, got: {lines:?}");

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

    let invalid: serde_json::Value =
        serde_json::from_str(&lines[2]).expect("invalid-request response is JSON");
    assert_eq!(invalid["id"], serde_json::Value::Null);
    assert_eq!(invalid["error"]["code"], -32600);
}
