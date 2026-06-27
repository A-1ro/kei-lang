//! ファイルシステム経由の `ModuleResolver` 実装(M20 / #55)。
//!
//! - 入力ファイル `<F>` の `module a.b.c` 宣言からプロジェクト root を逆算する。
//!   `<F>` の親を `path.len()` 段遡って root とし、`a/b/c.kei` を一意に解決する。
//! - 循環 import と再解決を避けるため `visiting` セットと `cache` を持つ。
//! - 解決中に対象モジュールがパースエラーを起こした場合は `None`(opaque)に
//!   倒し、consumer の検査をブロックしない(致命的でない健全性ギャップは
//!   既定挙動と同じ "opaque" 段階移行)。
//!
//! 副作用(fs / parse)を kei_check の外に押し出す境界。kei_check は
//! [`kei_check::ModuleResolver`] トレイトだけを知る(ARCHITECTURE.md)。

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use kei_check::imports::{module_type_defs, ModuleResolver, ResolvedModule};
use kei_syntax::ast;

pub struct FsModuleResolver {
    root: PathBuf,
    cache: RefCell<HashMap<Vec<String>, Option<ResolvedModule>>>,
    visiting: RefCell<HashSet<Vec<String>>>,
}

impl FsModuleResolver {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            cache: RefCell::new(HashMap::new()),
            visiting: RefCell::new(HashSet::new()),
        }
    }
}

impl ModuleResolver for FsModuleResolver {
    fn resolve(&self, path: &[String]) -> Option<ResolvedModule> {
        let key: Vec<String> = path.to_vec();
        if let Some(cached) = self.cache.borrow().get(&key) {
            return cached.clone();
        }
        // 循環ガード: 解決中に同じ path を再要求されたら None を返す
        // (深いネストを許す。AST の型定義抽出は今回 transitive を辿らないため、
        // 実害は出ない)。
        if !self.visiting.borrow_mut().insert(key.clone()) {
            return None;
        }

        let mut file = self.root.clone();
        for seg in path {
            file.push(seg);
        }
        file.set_extension("kei");

        let resolved = std::fs::read_to_string(&file)
            .ok()
            .and_then(|src| {
                let parsed = kei_syntax::parse_module(&src);
                if !parsed.errors.is_empty() {
                    return None;
                }
                Some(parsed.module)
            })
            .map(|m| ResolvedModule {
                path: path.to_vec(),
                type_defs: module_type_defs(&m),
            });

        self.visiting.borrow_mut().remove(&key);
        self.cache.borrow_mut().insert(key, resolved.clone());
        resolved
    }
}

/// `module a.b.c` 宣言と入力ファイルパスから project root を逆算する。
/// `<root>/a/b/c.kei` 規約に従わないファイル(`module` 宣言が無い / 段数が合わない /
/// 親が足りない)では `None` を返し、CLI は resolver 無しで従来通り検査する。
pub fn derive_root(file: &Path, module: &ast::Module) -> Option<PathBuf> {
    let decl = module.decl.as_ref()?;
    let n = decl.path.len();
    if n == 0 {
        return None;
    }
    let mut p = file.to_path_buf();
    for _ in 0..n {
        if !p.pop() {
            return None;
        }
    }
    Some(p)
}
