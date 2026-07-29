//! kei-lang をブラウザ内(wasm32-unknown-unknown)で実行するための薄いラッパー。
//!
//! コア4クレート(kei_syntax/kei_check/kei_fmt/kei_emit)はOS依存APIを使わない
//! 純粋な文字列変換なので、ここでは wasm-bindgen 境界の型変換だけを担う
//! (本体クレートへの変更は不要)。JSON文字列で受け渡しすることで、
//! JS側の型はTypeScriptの型定義だけで足り、wasm-bindgenのserde連携も不要にしている。

use wasm_bindgen::prelude::*;

const PLAYGROUND_FILE: &str = "playground.kei";

/// 意味検査を実行し、Diagnostic配列をJSON文字列で返す(空配列 = エラーなし)。
#[wasm_bindgen]
pub fn check(source: &str) -> String {
    let parsed = kei_syntax::parse_module(source);
    let mut diagnostics: Vec<kei_check::Diagnostic> =
        kei_check::syntax_diagnostics(PLAYGROUND_FILE, &parsed.errors);
    diagnostics.extend(kei_check::check_module(PLAYGROUND_FILE, &parsed.module));
    serde_json::to_string(&diagnostics).unwrap_or_else(|_| "[]".to_string())
}

/// 正規形フォーマットを実行する。構文エラーがあれば `Err` にDiagnostic配列のJSONを積む
/// (kei_syntax::SyntaxError はserde非対応なので、check()と同じくDiagnosticへ変換する)。
#[wasm_bindgen]
pub fn format(source: &str) -> Result<String, String> {
    kei_fmt::format_source(source).map_err(|errors| {
        let diagnostics = kei_check::syntax_diagnostics(PLAYGROUND_FILE, &errors);
        serde_json::to_string(&diagnostics).unwrap_or_else(|_| "[]".to_string())
    })
}

/// TypeScriptへトランスパイルする。検査エラーがあれば `Err` にDiagnostic配列のJSONを積む。
/// 成功時は `{"ts":..,"map":..,"ts_path":..}` のJSON。
#[wasm_bindgen]
pub fn emit(source: &str) -> Result<String, String> {
    kei_emit::emit_module(PLAYGROUND_FILE, source)
        .map(|out| {
            serde_json::json!({ "ts": out.ts, "map": out.map, "ts_path": out.ts_path }).to_string()
        })
        .map_err(|diagnostics| {
            serde_json::to_string(&diagnostics).unwrap_or_else(|_| "[]".to_string())
        })
}
