const VALID_SRC: &str = "module playground\n\nrecord User {\n  name: String\n}\n";

#[test]
fn check_reports_empty_on_valid_source() {
    let diagnostics = kei_wasm::check(VALID_SRC);
    assert_eq!(diagnostics, "[]");
}

#[test]
fn check_reports_syntax_error() {
    let src = "record User {\n";
    let diagnostics = kei_wasm::check(src);
    assert_ne!(diagnostics, "[]");
}

#[test]
fn format_round_trips_valid_source() {
    let out = kei_wasm::format(VALID_SRC).expect("format should succeed");
    assert!(out.contains("record User"));
}

#[test]
fn emit_produces_typescript() {
    let json = kei_wasm::emit(VALID_SRC).expect("emit should succeed");
    assert!(json.contains("\"ts\""));
}
