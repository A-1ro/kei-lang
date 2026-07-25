//! MCP (JSON-RPC 2.0) ディスパッチ。トランスポート(stdio)からは独立した
//! 純関数 [`Server::handle`] で、リクエスト Value → レスポンス Value を返す
//! (notification は `None`)。これにより tests/mcp/ の golden test が
//! プロセス起動なしでリクエスト→レスポンスを検証できる。

use serde_json::{json, Value};

use crate::tools::{self, ToolOutcome};

/// このサーバーが受け入れる MCP プロトコルバージョン(新しい順)。
///
/// MCP spec 2025-11-25 "Version Negotiation":
/// > If the server supports the requested protocol version, it MUST respond with the
/// > same version. Otherwise, the server MUST respond with another protocol version it
/// > supports. This SHOULD be the latest version supported by the server.
///
/// この規則は `negotiate_protocol_version` が実装する。先頭要素が
/// [`DEFAULT_PROTOCOL_VERSION`] = 自分がサポートする最新版で、交渉が不成立のときに
/// 名乗るバージョンになる。
///
/// **新しいリビジョンへの追従はこの配列の先頭に 1 行足すだけ**でよい。ただし
/// 載せてよいのは実際に準拠しているバージョンだけ(名乗るバージョンに嘘をつかない)。
/// 追加前に、そのリビジョンが「tools のみを stdio で提供するサーバー」に課す要件を
/// 満たしているか確認する。既定版は golden にも焼き込まれているので、
/// `UPDATE_GOLDEN=1 cargo test -p kei_mcp --test golden_mcp` で
/// tests/mcp/initialize_version_*.response.json を再生成すること(不変条件 3 のとおり
/// golden の差分は人間レビュー必須)。
///
/// 2025-03-26 を意図的に外している: このリビジョンの stdio トランスポートは
/// メッセージとして JSON-RPC batch(配列)を許すが、[`crate::run_stdio`] は
/// 1 行 = 1 メッセージしか解釈しない。batch は 2025-06-18 で仕様から削除されたため、
/// 2025-06-18 以降と(batch 導入前の)2024-11-05 には準拠できるが、
/// 2025-03-26 には準拠できない。
pub const SUPPORTED_PROTOCOL_VERSIONS: &[&str] = &["2025-11-25", "2025-06-18", "2024-11-05"];

/// 交渉が成立しなかったときに名乗る既定バージョン(= サポートする最新版)。
pub const DEFAULT_PROTOCOL_VERSION: &str = SUPPORTED_PROTOCOL_VERSIONS[0];

/// サーバー名(serverInfo)。
pub const SERVER_NAME: &str = "kei-mcp";
/// サーバーバージョン(Cargo パッケージ版数)。
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Kei MCP サーバー。状態は埋め込み静的データのみで、インスタンスは空。
#[derive(Debug, Default, Clone, Copy)]
pub struct Server;

impl Server {
    pub fn new() -> Self {
        Server
    }

    /// JSON-RPC リクエストを処理する。`id` を持つ通常リクエストは `Some(response)`、
    /// 通知(`id` なし)は `None` を返す。
    pub fn handle(&self, request: &Value) -> Option<Value> {
        let method = request.get("method").and_then(Value::as_str);

        // 通知(id なし)は応答しない。
        let id = request.get("id").cloned();
        id.as_ref()?;

        let method = match method {
            Some(m) => m,
            None => {
                return Some(error(id, -32600, "Invalid Request: missing 'method'"));
            }
        };
        let params = request.get("params").cloned().unwrap_or(Value::Null);

        let response = match method {
            "initialize" => success(id, initialize_result(&params)),
            "ping" => success(id, json!({})),
            "tools/list" => success(id, tools_list_result()),
            "tools/call" => tools_call(id, &params),
            other => error(id, -32601, &format!("Method not found: {other}")),
        };
        Some(response)
    }
}

fn tools_call(id: Option<Value>, params: &Value) -> Value {
    let name = match params.get("name").and_then(Value::as_str) {
        Some(n) => n,
        None => return error(id, -32602, "Invalid params: missing tool 'name'"),
    };
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let str_arg = |key: &str| args.get(key).and_then(Value::as_str);
    // 省略時(キーなし / null)は従来通り `false`。値はあるが bool でないときは型不一致
    // として明示エラーを返す(暗黙 false 降格を廃止。M34 レビュー対応)。
    let bool_arg = |key: &str| -> Result<bool, Value> {
        match args.get(key) {
            None | Some(Value::Null) => Ok(false),
            Some(Value::Bool(b)) => Ok(*b),
            Some(_) => Err(tool_result(&tools::invalid_arg(key, "boolean"))),
        }
    };

    let outcome_or_result = match name {
        "kei_spec" => Ok(tools::run_spec(str_arg("topic").unwrap_or(""))),
        "kei_check" => match str_arg("source") {
            Some(src) => bool_arg("generative").map(|g| tools::run_check(src, g)),
            None => Ok(tools::missing_arg("source")),
        },
        "kei_fmt" => match str_arg("source") {
            Some(src) => Ok(tools::run_fmt(src)),
            None => Ok(tools::missing_arg("source")),
        },
        "kei_examples" => Ok(tools::run_examples(str_arg("query").unwrap_or(""))),
        other => Ok(tools::unknown_tool(other)),
    };
    match outcome_or_result {
        Ok(outcome) => success(id, tool_result(&outcome)),
        Err(err_result) => success(id, err_result),
    }
}

fn tool_result(outcome: &ToolOutcome) -> Value {
    json!({
        "content": [ { "type": "text", "text": outcome.text } ],
        "isError": outcome.is_error,
    })
}

/// initialize のバージョン交渉。要求版をサポートしていればそれをそのまま返し
/// (spec の MUST)、していなければサポートする最新版 [`DEFAULT_PROTOCOL_VERSION`]
/// を返す(spec の MUST + SHOULD)。
///
/// クライアントは `protocolVersion` を送ることが MUST だが、欠落・非文字列でも
/// 接続を切らずに既定版を名乗る(交渉不成立と同じ扱い)。最終的に版が合わなければ
/// 切断を選ぶのはクライアント側の責務("If the client does not support the version
/// in the server's response, it SHOULD disconnect.")。
fn negotiate_protocol_version(requested: Option<&str>) -> &'static str {
    requested
        .and_then(|want| {
            SUPPORTED_PROTOCOL_VERSIONS
                .iter()
                .copied()
                .find(|&supported| supported == want)
        })
        .unwrap_or(DEFAULT_PROTOCOL_VERSION)
}

fn initialize_result(params: &Value) -> Value {
    let requested = params.get("protocolVersion").and_then(Value::as_str);
    json!({
        "protocolVersion": negotiate_protocol_version(requested),
        "capabilities": { "tools": {} },
        "serverInfo": { "name": SERVER_NAME, "version": SERVER_VERSION },
    })
}

/// tools/list の応答。spec §6.1 のツール定義と入力名(topic/source/query)に一致させる。
fn tools_list_result() -> Value {
    json!({
        "tools": [
            {
                "name": "kei_spec",
                "description": "Look up the Kei language spec. Pass `topic` as a section number (e.g. \"3\"), a heading keyword, or an error code (e.g. \"KEI-E3001\"); omit it for the index.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "topic": {
                            "type": "string",
                            "description": "Section number, heading keyword, or error code. Empty returns the index."
                        }
                    },
                    "required": [],
                    "additionalProperties": false
                }
            },
            {
                "name": "kei_check",
                "description": "Statically check Kei source (syntax + types + effects + contracts). Returns JSON { diagnostics, contracts, opaque_imports, generative }. diagnostics is a Diagnostic[] array, each with span, code, and at least one fix candidate. contracts[].verification reports how strongly each requires/ensures was verified (static/generative/bounded/runtime/trusted/unchecked). opaque_imports lists the dotted module paths of every `import` declaration in the given source (not `extern package` bindings, whose `extern` signatures already carry types/effects): this tool only sees the source text (not the filesystem), so ALL declared imports are always opaque here — their types are Ty::Unknown and are NOT type-checked; a clean check result does not mean imported types were verified. For import-aware checking, use the CLI `kei check <dir>` instead. Set generative: true to additionally run contract-based property-based testing (the same mechanism as CLI `kei check --generative`) and search for counterexamples (reported as KEI-E4005). This tool caps case generation at a conservative limit smaller than the CLI default to bound tool-call latency. Functions whose input space exceeds this limit are skipped entirely (not partially checked) and listed in the response's generative.skipped array with their required case count — a clean check with no counterexamples does NOT mean a skipped function's contract holds. Functions that declare any effect (`uses`) are also always skipped when they have `ensures` (the synchronous evaluator cannot observe effects) and are listed with a `reason` string instead; `required_cases` is present only for skips caused by exceeding the case-generation limit. For exhaustive search, use the CLI `kei check --generative` instead, which defaults to a much larger limit.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "source": { "type": "string", "description": "Kei source text to check." },
                        "generative": { "type": "boolean", "description": "Run contract-based property-based testing for counterexamples (like CLI --generative). Default false. Bounded to a conservative case limit; functions exceeding it are skipped (see response generative.skipped)." }
                    },
                    "required": ["source"],
                    "additionalProperties": false
                }
            },
            {
                "name": "kei_fmt",
                "description": "Format Kei source into canonical form. On a syntax error it does not reformat and returns the Diagnostic[] instead (isError).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "source": { "type": "string", "description": "Kei source text to format." }
                    },
                    "required": ["source"],
                    "additionalProperties": false
                }
            },
            {
                "name": "kei_examples",
                "description": "Search Kei example snippets by keyword (matches path and body). Omit `query` to list all examples.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Keyword to search example paths and bodies. Empty lists all." }
                    },
                    "required": [],
                    "additionalProperties": false
                }
            }
        ]
    })
}

fn success(id: Option<Value>, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id.unwrap_or(Value::Null), "result": result })
}

fn error(id: Option<Value>, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(Value::Null),
        "error": { "code": code, "message": message },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 交渉不成立時に名乗るのは「サポートする最新版」でなければならない
    /// (spec: "This SHOULD be the latest version supported by the server.")。
    /// 配列は新しい順に並べる契約なので、既定版は先頭と一致する。
    #[test]
    fn default_version_is_the_first_supported_version() {
        assert_eq!(
            DEFAULT_PROTOCOL_VERSION, SUPPORTED_PROTOCOL_VERSIONS[0],
            "SUPPORTED_PROTOCOL_VERSIONS must be ordered newest-first"
        );
        let mut sorted = SUPPORTED_PROTOCOL_VERSIONS.to_vec();
        sorted.sort_unstable();
        sorted.reverse();
        assert_eq!(
            sorted, SUPPORTED_PROTOCOL_VERSIONS,
            "SUPPORTED_PROTOCOL_VERSIONS must be ordered newest-first"
        );
    }

    /// 要求版をサポートしていれば、同じ版をそのまま返す(spec の MUST)。
    #[test]
    fn negotiation_echoes_back_every_supported_version() {
        for supported in SUPPORTED_PROTOCOL_VERSIONS {
            assert_eq!(negotiate_protocol_version(Some(supported)), *supported);
        }
    }

    /// 未サポート版を要求されたら、サポートする最新版にフォールバックする。
    #[test]
    fn negotiation_falls_back_when_version_unsupported() {
        for requested in ["2025-03-26", "1.0.0", "", "2099-01-01"] {
            assert_eq!(
                negotiate_protocol_version(Some(requested)),
                DEFAULT_PROTOCOL_VERSION,
                "unsupported version {requested} must fall back to the default"
            );
        }
    }

    /// protocolVersion 欠落(仕様上は MUST 送信)でも切らずに既定版を名乗る。
    #[test]
    fn negotiation_falls_back_when_version_absent() {
        assert_eq!(negotiate_protocol_version(None), DEFAULT_PROTOCOL_VERSION);
    }

    /// initialize 応答は params の protocolVersion を見て決まる。
    /// 非文字列(型違い)は欠落と同じ扱い。
    #[test]
    fn initialize_result_negotiates_from_params() {
        let echoed = initialize_result(&json!({ "protocolVersion": "2024-11-05" }));
        assert_eq!(echoed["protocolVersion"], "2024-11-05");

        let fallback = initialize_result(&json!({ "protocolVersion": "1.0.0" }));
        assert_eq!(fallback["protocolVersion"], DEFAULT_PROTOCOL_VERSION);

        let absent = initialize_result(&json!({}));
        assert_eq!(absent["protocolVersion"], DEFAULT_PROTOCOL_VERSION);

        let wrong_type = initialize_result(&json!({ "protocolVersion": 20251125 }));
        assert_eq!(wrong_type["protocolVersion"], DEFAULT_PROTOCOL_VERSION);

        // params を省いた initialize(Value::Null)でも落ちない。
        let null_params = initialize_result(&Value::Null);
        assert_eq!(null_params["protocolVersion"], DEFAULT_PROTOCOL_VERSION);
    }

    /// tools のみを提供するサーバーとして、tools capability の宣言は MUST。
    #[test]
    fn initialize_result_declares_tools_capability() {
        let result = initialize_result(&json!({}));
        assert!(
            result["capabilities"]["tools"].is_object(),
            "servers that support tools MUST declare the tools capability"
        );
    }
}
