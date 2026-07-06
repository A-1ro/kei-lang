//! M33 / #95: `Map<K, V>` 段階1の型検査挙動を固定する(期待型推論・キー型制約・
//! generative 対象外)。

use kei_check::{check_module, check_module_report_with, CheckOptions, Severity};

fn check(src: &str) -> Vec<(String, String)> {
    let parsed = kei_syntax::parse_module(src);
    assert!(
        parsed.errors.is_empty(),
        "test source should parse: {:?}",
        parsed.errors
    );
    check_module("t.kei", &parsed.module)
        .into_iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| (d.code.clone(), d.message.clone()))
        .collect()
}

#[test]
fn map_empty_with_let_annotation_is_clean() {
    let src = "module t\n\
               func mk() -> Map<String, Int> {\n\
                 let m: Map<String, Int> = Map.empty()\n\
                 return m\n\
               }\n";
    let diags = check(src);
    assert!(diags.is_empty(), "expected no errors, got {diags:?}");
}

#[test]
fn map_empty_with_return_position_expected_type_is_clean() {
    let src = "module t\n\
               func mk() -> Map<String, Int> { return Map.empty() }\n";
    let diags = check(src);
    assert!(diags.is_empty(), "expected no errors, got {diags:?}");
}

#[test]
fn map_empty_with_argument_position_expected_type_is_clean() {
    let src = "module t\n\
               func size(m: Map<String, Int>) -> Int { return m.size }\n\
               func mk() -> Int { return size(Map.empty()) }\n";
    let diags = check(src);
    assert!(diags.is_empty(), "expected no errors, got {diags:?}");
}

#[test]
fn map_empty_without_expected_type_reports_e2012() {
    let src = "module t\n\
               func mk() -> Bool {\n\
                 let m = Map.empty()\n\
                 return m.has(\"a\")\n\
               }\n";
    let diags = check(src);
    let codes: Vec<&str> = diags.iter().map(|(c, _)| c.as_str()).collect();
    assert!(
        codes.contains(&"KEI-E2012"),
        "expected KEI-E2012 for unannotated Map.empty(); got {diags:?}"
    );
}

#[test]
fn map_bool_key_reports_e2011() {
    let src = "module t\n\
               func mk() -> Map<Bool, Int> { return Map.empty() }\n";
    let diags = check(src);
    let codes: Vec<&str> = diags.iter().map(|(c, _)| c.as_str()).collect();
    assert!(
        codes.contains(&"KEI-E2011"),
        "expected KEI-E2011 for Bool key; got {diags:?}"
    );
}

#[test]
fn map_record_key_reports_e2011() {
    let src = "module t\n\
               record P { id: String }\n\
               func mk() -> Map<P, Int> { return Map.empty() }\n";
    let diags = check(src);
    let codes: Vec<&str> = diags.iter().map(|(c, _)| c.as_str()).collect();
    assert!(
        codes.contains(&"KEI-E2011"),
        "expected KEI-E2011 for record key; got {diags:?}"
    );
}

#[test]
fn map_int_key_is_clean() {
    let src = "module t\n\
               func mk() -> Map<Int, String> { return Map.empty() }\n";
    let diags = check(src);
    assert!(diags.is_empty(), "expected no errors, got {diags:?}");
}

#[test]
fn map_tagged_string_key_is_clean() {
    let src = "module t\n\
               type ProductId = String tagged \"ProductId\"\n\
               func mk() -> Map<ProductId, Int> { return Map.empty() }\n";
    let diags = check(src);
    assert!(diags.is_empty(), "expected no errors, got {diags:?}");
}

#[test]
fn map_get_set_has_size_roundtrip_is_clean() {
    let src = "module t\n\
               func withDefault(stock: Map<String, Int>, id: String) -> Map<String, Int>\n\
                 requires stock.has(id) == false\n\
                 ensures result.size == stock.size + 1\n\
               {\n\
                 return stock.set(id, 0)\n\
               }\n";
    let diags = check(src);
    assert!(diags.is_empty(), "expected no errors, got {diags:?}");
}

#[test]
fn map_get_wrong_key_type_reports_type_mismatch() {
    let src = "module t\n\
               func bad(stock: Map<String, Int>) -> Option<Int> { return stock.get(1) }\n";
    let diags = check(src);
    let codes: Vec<&str> = diags.iter().map(|(c, _)| c.as_str()).collect();
    assert!(
        codes.contains(&"KEI-E2001"),
        "expected KEI-E2001 for wrong key type; got {diags:?}"
    );
}

#[test]
fn map_unknown_method_reports_e2002_with_did_you_mean() {
    let src = "module t\n\
               func bad(stock: Map<String, Int>) -> Bool { return stock.delete(\"a\") }\n";
    let diags = check(src);
    let codes: Vec<&str> = diags.iter().map(|(c, _)| c.as_str()).collect();
    assert!(
        codes.contains(&"KEI-E2002"),
        "expected KEI-E2002 for unknown Map method; got {diags:?}"
    );
}

/// generative(M15)は Map 引数を持つ関数を対象外にする(候補ドメイン生成非対応)。
/// 反例が生成されない = 検証レベルが `runtime` のまま(格上げされない)。
#[test]
fn generative_skips_functions_with_map_params() {
    let src = "module t\n\
               func hasKey(stock: Map<String, Int>, id: String) -> Bool\n\
                 ensures result == stock.has(id)\n\
               {\n\
                 return stock.has(id)\n\
               }\n";
    let parsed = kei_syntax::parse_module(src);
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    let report = check_module_report_with(
        "t.kei",
        &parsed.module,
        CheckOptions {
            generative: true,
            ..CheckOptions::default()
        },
    );
    assert!(
        report
            .diagnostics
            .iter()
            .all(|d| d.severity != Severity::Error),
        "expected clean check, got {:?}",
        report.diagnostics
    );
    let ensures = report
        .contracts
        .iter()
        .find(|c| c.func == "hasKey" && c.kind == kei_check::ContractKind::Ensures)
        .expect("hasKey ensures contract is reported");
    assert_eq!(
        ensures.verification,
        kei_check::Verification::Runtime,
        "Map-typed params must not be upgraded past runtime verification"
    );
}
