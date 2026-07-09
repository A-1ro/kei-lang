//! 標準エフェクト階層(spec §3.2)と包含判定。
//!
//! v0.1 ではユーザー定義エフェクトはなく、`uses` 節に書けるのはこの階層の
//! ノードのみ。`IO` は階層全体の根で、宣言すると全エフェクトの包括許可になる
//! (spec §3.2 が「雑だが合法」と定めるため warning にもしない — §9 未決事項 3)。

/// 標準エフェクト階層の全ノード(中間ノード含む)。
pub const STANDARD_EFFECTS: &[&str] = &[
    "IO",
    "Network",
    "Network.Read",
    "Network.Write",
    "File",
    "File.Read",
    "File.Write",
    "Database",
    "Database.Read",
    "Database.Write",
    "Clock",
    "Random",
    "Audit",
    "Audit.Log",
    "Async",
];

/// `path` が標準エフェクト階層のノードか。
pub fn is_known(path: &str) -> bool {
    STANDARD_EFFECTS.contains(&path)
}

/// 宣言 `declared` が使用 `used` を包含するか。
/// 自分自身・祖先ノードが包含し、`IO` は Async 以外の全エフェクトを包含する
/// (Async は独立ルート、M37)。
pub fn covers(declared: &str, used: &str) -> bool {
    if declared == used {
        return true;
    }
    if declared == "IO" {
        // Async は IO 傘下ではない独立ルート(v0.7 / M37)。IO 宣言関数が黙って
        // 非同期になる互換性破壊を避けるための唯一の例外。将来 `Async.X` サブ
        // エフェクトが追加されても IO に黙って取り込まれないよう、prefix 判定も
        // 防御的に見る(M37 レビュー対応)。
        return used != "Async" && !used.starts_with("Async.");
    }
    used.len() > declared.len()
        && used.starts_with(declared)
        && used.as_bytes()[declared.len()] == b'.'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hierarchy_containment() {
        assert!(covers("IO", "Database.Write"));
        assert!(covers("IO", "Clock"));
        assert!(covers("Database", "Database.Write"));
        assert!(covers("Database.Write", "Database.Write"));
        assert!(!covers("Database.Write", "Database"));
        assert!(!covers("Database.Read", "Database.Write"));
        assert!(!covers("Database", "DatabaseX"));
        assert!(!covers("Clock", "IO"));
    }

    #[test]
    fn io_does_not_cover_async() {
        assert!(!covers("IO", "Async"));
        assert!(!covers("IO", "Async.Foo"));
        assert!(covers("Async", "Async"));
        assert!(covers("IO", "Database.Write")); // 既存挙動は不変
    }
}
