//! MCP (JSON-RPC 2.0) ディスパッチ。トランスポート(stdio)からは独立した
//! 純関数 [`Server::handle`] で、リクエスト Value → レスポンス Value を返す
//! (notification は `None`)。これにより tests/mcp/ の golden test が
//! プロセス起動なしでリクエスト→レスポンスを検証できる。
//!
//! ## Dual-era 対応(MCP spec 2026-07-28 "Versioning and Compatibility")
//!
//! 2026-07-28 で `initialize`/`initialized` ハンドシェイクが廃止され、
//! 各リクエストが `params._meta["io.modelcontextprotocol/protocolVersion"]` で
//! バージョンを申告する方式(spec の呼称で "Modern")に変わった。
//! 2025-11-25 以前(`initialize` ハンドシェイク方式)は "Legacy" と呼ばれ、
//! 両方に応答できる実装は "Dual-era" と呼ばれる。
//!
//! kei_mcp は Dual-era として振る舞う:
//! - `method == "initialize"` は常に Legacy 経路(spec: "An initialize request
//!   selects legacy semantics")。
//! - それ以外で `params._meta["io.modelcontextprotocol/protocolVersion"]` が
//!   付いていれば Modern 経路。バージョンが [`MODERN_SUPPORTED_PROTOCOL_VERSIONS`]
//!   に無ければ [`unsupported_protocol_version_error`](-32022) を返す。
//! - どちらの申告も無い「era 不明」リクエスト(例: `_meta` 無しの `tools/list`)は
//!   これまで通り Legacy と同じ経路で処理する。spec もこの状態
//!   ("era-ambiguous method... processed under legacy semantics")を許容しており、
//!   kei_mcp はもともとセッション状態を持たないためこれで golden test 互換を保てる。

use serde_json::{json, Value};

use crate::tools::{self, ToolOutcome};

/// このサーバーが Legacy(`initialize` ハンドシェイク)経路で受け入れる MCP
/// プロトコルバージョン(新しい順)。
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
/// **新しいリビジョンへの追従はこの配列の先頭に 1 行足すだけ**でよい —
/// ただしこれは 2025-11-25 までの Legacy(`initialize` ハンドシェイク)リビジョン間の
/// 話に限る。2026-07-28 はハンドシェイク自体を廃止する破壊的変更のため Modern 版は
/// 別配列 [`MODERN_SUPPORTED_PROTOCOL_VERSIONS`] で管理する(この配列に混ぜると
/// 「initialize でも 2026-07-28 を名乗れる」という嘘になる)。
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

/// 交渉が成立しなかったときに名乗る既定バージョン(= サポートする Legacy 最新版)。
pub const DEFAULT_PROTOCOL_VERSION: &str = SUPPORTED_PROTOCOL_VERSIONS[0];

/// Modern 経路(`_meta` によるリクエスト単位のバージョン申告)でこのサーバーが
/// 受理するプロトコルバージョン。
///
/// MCP spec 2026-07-28 "Terminology":
/// > Modern: protocol versions that convey version, identity, and capabilities as
/// > per-request metadata (revision 2026-07-28 and later).
///
/// Legacy 専用の [`SUPPORTED_PROTOCOL_VERSIONS`] とは意図的に別配列にしている。
/// 2025-11-25 以前は per-request メタデータの概念自体を持たないため、同じ配列に
/// 混ぜると「Modern としても受理する」と誤読されるおそれがある。
pub const MODERN_SUPPORTED_PROTOCOL_VERSIONS: &[&str] = &["2026-07-28"];

/// Modern リクエストがバージョンを申告する `_meta` キー。
///
/// MCP spec 2026-07-28 "Protocol Version Negotiation":
/// > Every request declares the protocol version it is using in its `_meta` field.
const META_PROTOCOL_VERSION_KEY: &str = "io.modelcontextprotocol/protocolVersion";

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
    /// オブジェクトで `id` を持たない通知は `None` を返す。オブジェクト以外
    /// (配列・文字列・数値・真偽値・null)は通知ではなく不正なリクエストなので、
    /// `id: null` の Invalid Request(`-32600`)を `Some` で返す。
    pub fn handle(&self, request: &Value) -> Option<Value> {
        // 非オブジェクト(配列・文字列・数値・真偽値・null)は `Value::get("...")` が
        // 常に None を返すため、以下の id 取得ロジックだけでは「id なし通知」と
        // 区別できず無応答で握りつぶされてしまう。JSON-RPC 2.0 spec は
        // "If there was an error in detecting the id... it MUST be Null" と定めており、
        // 通知として無視するのではなく id: null で Invalid Request を返す。
        if !request.is_object() {
            return Some(error(None, -32600, "Invalid Request: not a JSON object"));
        }

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

        // Dual-era 判定: `initialize` は常に Legacy(spec: "An initialize request
        // selects legacy semantics")。それ以外で `_meta` にバージョン申告があれば
        // Modern 経路とし、[`MODERN_SUPPORTED_PROTOCOL_VERSIONS`] との一致を見る。
        // 申告が無い era 不明リクエストは(モジュールドキュメント参照)Legacy と
        // 同じ経路にフォールスルーする。
        if method != "initialize" {
            if let Some(requested) = modern_protocol_version(&params) {
                if !MODERN_SUPPORTED_PROTOCOL_VERSIONS.contains(&requested) {
                    return Some(unsupported_protocol_version_error(id, requested));
                }
            }
        }

        let response = match method {
            "initialize" => success(id, initialize_result(&params)),
            "server/discover" => success(id, discover_result()),
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

/// リクエストの `params._meta["io.modelcontextprotocol/protocolVersion"]` を
/// 取り出す。無い、または `_meta`/値が非オブジェクト・非文字列なら `None`
/// (era 不明リクエストとして Legacy 経路にフォールスルーさせる)。
fn modern_protocol_version(params: &Value) -> Option<&str> {
    params
        .get("_meta")?
        .get(META_PROTOCOL_VERSION_KEY)?
        .as_str()
}

/// `server/discover` の応答。MCP spec 2026-07-28 "Discovery":
/// > Servers MUST implement it.
/// > A discovery result includes: `supportedVersions`... `capabilities`...
/// > `_meta['io.modelcontextprotocol/serverInfo']`... Servers SHOULD include this field.
///
/// `instructions`/`ttlMs`/`cacheScope` は任意フィールド(キャッシュ関連の拡張)なので
/// 省略する。`resultType: "complete"` は spec の JSON 例に倣った固定値。
fn discover_result() -> Value {
    json!({
        "resultType": "complete",
        "supportedVersions": MODERN_SUPPORTED_PROTOCOL_VERSIONS,
        "capabilities": { "tools": {} },
        "_meta": {
            "io.modelcontextprotocol/serverInfo": { "name": SERVER_NAME, "version": SERVER_VERSION },
        },
    })
}

/// Modern リクエストが申告したバージョンを本サーバーがサポートしないときの応答。
///
/// MCP spec 2026-07-28 "Protocol Version Negotiation":
/// > it MUST respond with an `UnsupportedProtocolVersionError` listing the versions
/// > it does support:
/// > `{ "error": { "code": -32022, "message": "Unsupported protocol version",
/// >   "data": { "supported": [...], "requested": "..." } } }`
fn unsupported_protocol_version_error(id: Option<Value>, requested: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(Value::Null),
        "error": {
            "code": -32022,
            "message": "Unsupported protocol version",
            "data": {
                "supported": MODERN_SUPPORTED_PROTOCOL_VERSIONS,
                "requested": requested,
            },
        },
    })
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

    /// 非オブジェクトの JSON-RPC メッセージ(配列・文字列・数値・真偽値・null)は、
    /// id なし通知と混同して無応答で捨てるのではなく、id: null で Invalid Request
    /// を返す。
    #[test]
    fn non_object_message_returns_invalid_request() {
        let server = Server::new();
        for non_object in [
            json!([1, 2, 3]),
            json!("just a string"),
            json!(42),
            json!(true),
            Value::Null,
        ] {
            let response = server
                .handle(&non_object)
                .unwrap_or_else(|| panic!("non-object message {non_object} must get a response"));
            assert_eq!(response["jsonrpc"], "2.0");
            // `Value` の `Index` はキー欠落でも Null を返すため、`response["id"]` の
            // 比較だけでは「id フィールドを出していない」応答も素通りしてしまう。
            // spec が要求するのは null を **含めて** 返すことなので、キーの存在ごと固定する。
            assert_eq!(
                response.get("id"),
                Some(&Value::Null),
                "id must be present and null: {response}"
            );
            assert_eq!(response["error"]["code"], -32600);
            assert!(
                response.get("result").is_none(),
                "error response must not carry a result: {response}"
            );
        }
    }

    /// オブジェクトだが id を欠く正当な通知は、これまで通り無応答。
    #[test]
    fn object_notification_without_id_still_gets_no_response() {
        let server = Server::new();
        let notification = json!({ "jsonrpc": "2.0", "method": "notifications/initialized" });
        assert_eq!(server.handle(&notification), None);
    }

    /// `server/discover` は MUST 実装。Modern の申告バージョンが一致すれば
    /// supportedVersions / capabilities / serverInfo を返す。
    #[test]
    fn discover_returns_supported_versions_and_server_info() {
        let server = Server::new();
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "server/discover",
            "params": {
                "_meta": { META_PROTOCOL_VERSION_KEY: "2026-07-28" }
            }
        });
        let response = server.handle(&request).expect("discover responds");
        assert_eq!(
            response["result"]["supportedVersions"],
            json!(MODERN_SUPPORTED_PROTOCOL_VERSIONS)
        );
        assert!(response["result"]["capabilities"]["tools"].is_object());
        assert_eq!(
            response["result"]["_meta"]["io.modelcontextprotocol/serverInfo"]["name"],
            SERVER_NAME
        );
    }

    /// `_meta` にバージョン申告が無い `server/discover` も(era 不明として)
    /// フォールスルーし、通常どおり discover 結果を返す。
    #[test]
    fn discover_without_meta_still_responds() {
        let server = Server::new();
        let request = json!({ "jsonrpc": "2.0", "id": 1, "method": "server/discover" });
        let response = server.handle(&request).expect("discover responds");
        assert_eq!(
            response["result"]["supportedVersions"],
            json!(MODERN_SUPPORTED_PROTOCOL_VERSIONS)
        );
    }

    /// Modern 経路で未サポート版を申告すると `UnsupportedProtocolVersionError`
    /// (-32022, data.supported/data.requested)を返し、メソッドは実行されない。
    #[test]
    fn modern_request_with_unsupported_version_returns_dash_32022() {
        let server = Server::new();
        let request = json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "tools/list",
            "params": {
                "_meta": { META_PROTOCOL_VERSION_KEY: "1900-01-01" }
            }
        });
        let response = server.handle(&request).expect("responds");
        assert_eq!(response["error"]["code"], -32022);
        assert_eq!(response["error"]["data"]["requested"], "1900-01-01");
        assert_eq!(
            response["error"]["data"]["supported"],
            json!(MODERN_SUPPORTED_PROTOCOL_VERSIONS)
        );
        assert!(
            response.get("result").is_none(),
            "unsupported version must not fall through to the method handler"
        );
    }

    /// Modern 経路でサポート済みバージョンを申告すれば、通常どおりメソッドが実行される。
    #[test]
    fn modern_request_with_supported_version_dispatches_normally() {
        let server = Server::new();
        let request = json!({
            "jsonrpc": "2.0",
            "id": 8,
            "method": "tools/list",
            "params": {
                "_meta": { META_PROTOCOL_VERSION_KEY: "2026-07-28" }
            }
        });
        let response = server.handle(&request).expect("responds");
        assert!(
            response.get("error").is_none(),
            "supported version must dispatch without error: {response}"
        );
        assert!(response["result"]["tools"].is_array());
    }

    /// `initialize` は `_meta` にモダン申告が付いていても常に Legacy 経路
    /// (spec: "An initialize request selects legacy semantics")。バージョン
    /// 不一致による -32022 にはならず、既存の Legacy 交渉ロジックが動く。
    #[test]
    fn initialize_always_uses_legacy_negotiation_even_with_modern_meta() {
        let server = Server::new();
        let request = json!({
            "jsonrpc": "2.0",
            "id": 9,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "_meta": { META_PROTOCOL_VERSION_KEY: "1900-01-01" }
            }
        });
        let response = server.handle(&request).expect("responds");
        assert!(
            response.get("error").is_none(),
            "initialize must not be treated as an unsupported modern version: {response}"
        );
        assert_eq!(response["result"]["protocolVersion"], "2025-11-25");
    }

    /// `_meta` の値が非文字列/非オブジェクトなら era 不明として無視し、Legacy と
    /// 同じフォールスルー経路(既存メソッド分岐)へ進む。
    #[test]
    fn modern_protocol_version_ignores_malformed_meta() {
        assert_eq!(modern_protocol_version(&json!({})), None);
        assert_eq!(
            modern_protocol_version(&json!({ "_meta": "not-an-object" })),
            None
        );
        assert_eq!(
            modern_protocol_version(&json!({ "_meta": { META_PROTOCOL_VERSION_KEY: 123 } })),
            None
        );
        assert_eq!(
            modern_protocol_version(
                &json!({ "_meta": { META_PROTOCOL_VERSION_KEY: "2026-07-28" } })
            ),
            Some("2026-07-28")
        );
    }
}
