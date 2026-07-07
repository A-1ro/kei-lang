# Kei 開発ロードマップ v0.6 — /goal 契約書集(外部世界への import)

> 運用ルール: 各 Milestone は「人間が合意する契約」。完了条件は機械検証可能な形で書く。
> 本ファイルは v1.0 戦略(`docs/kei-roadmap-v0.5.md` 冒頭)の **v0.6: 外部 npm パッケージ import**
> — v1.0 blocker 3 つのうちの 1 つ目 — を Milestone 化したもの。

## v0.6 のゴール

Kei ソースから npm パッケージ(最終目標では `hono`)の関数を **extern 署名経由で**呼び出せ、
`kei build` の生成 TS が bare specifier の ESM import(`import * as h from "hono"`)を含み、
利用側プロジェクトの tsc / bundler / Workers ランタイムでそのまま解決されること。

設計原則(HANDOFF の intent-align に従う):

- **検証境界は変えない。** npm パッケージの中身を Kei は検査しない。extern 署名(v0.2 M11)が
  唯一の型・エフェクト情報源であり、署名のない呼び出しは従来どおり opaque / strict-extern の対象。
- **パッケージ解決をコンパイラに持ち込まない。** `kei build` は node_modules を見ない。
  specifier は文字列として emit に素通しし、解決は TS エコシステム(tsc / wrangler)の責務。
- **段階1は namespace import のみ。** `import * as ns from "pkg"` 形だけを生成する。
  named import(`import { serve } from "pkg"`)や default export の直接束縛は、
  v0.8(Hono アダプタ)で実需が確定してから段階2として設計する(🤝)。

## Milestone 全体像

| M | テーマ | 優先度 | 状態 | 主な改修クレート |
|---|---|---|---|---|
| **M35** | `extern package` 宣言(bare specifier 束縛) | high | ✅ | kei_syntax / kei_check / kei_emit / kei_fmt |
| **M36** | ビルド統合と検証整合(e2e・strict-extern・取説) | high | ⬜ 未着手 | kei_cli / kei_mcp / tests/e2e / spec / skill |

## M35: `extern package` 宣言

### 構文(合意対象)

```kei
extern package "hono" as hono

extern hono.text(body: String) -> String
extern hono.logRequest(path: String) uses Network.Write
```

- `extern package "<specifier>" as <束縛名>` をモジュール先頭(import 群と同じ領域)に置く。
- specifier は npm bare specifier(`hono` / `@scope/pkg` / サブパス `hono/tiny`)。相対パス・URL は
  KEI-E で拒否(相対は既存 import の領分)。
- 束縛名は以降のモジュール内で **extern 署名の名前空間としてのみ**使える(値としての参照・
  re-export は不可)。同名衝突(既存 import / 型名)は既存の重複診断に乗せる。
- 同一 specifier の重複宣言は KEI-E(fix: 統合)。

### 意味論

- `extern <束縛名>.<fn>(...)` 署名(v0.2 M11 の機構そのまま)で型・エフェクトが確定。
  `extern query`(M14)も同様に使える。
- 署名なしの `<束縛名>.<anything>()` 呼び出しは opaque — `--strict-extern` で warning(既存機構)。
- 契約式からは `extern query` 宣言済みのもののみ呼べる(既存規則)。

### emit

- 宣言ごとに `import * as <束縛名> from "<specifier>";` を 1 回だけ出力(使用宣言が 1 つも
  参照されないなら import 自体を出さない — 既存 import の未使用処理に合わせる。未使用検出が
  無いなら常時出力でよい、実装時に既存挙動へ揃えて報告)。
- source map・`@kei/runtime` import との並び順は既存 import 群の直後で安定させ、fmt 正規形を固定。

### 完了条件

- 上記構文が parse / check / fmt / emit を通り、golden(syntax / check / fmt)+ emit 単体テストで固定。
- 相対パス specifier・重複宣言・束縛名の値参照が診断(span / code / fix 付き)で拒否される。
- spec v0.2 の extern 節(§2)に「パッケージ束縛」小節を追記(spec-first)。
- `cargo test --workspace` 全パス。

## M36: ビルド統合と検証整合

### 完了条件

- **e2e**: `tests/e2e`(または cli プロジェクト)に、`file:` 依存のローカル npm ミニパッケージ +
  `extern package` を使う .kei を追加し、`kei build` → `npm test`(tsc + vitest)が通ることを CI で固定。
  (実 `hono` への依存はここでは追加しない — Workers 実物疎通は v0.9 の領分)
- **strict-extern**: パッケージ束縛経由の未宣言呼び出しが `--strict-extern` で従来どおり
  warning になることをテストで固定。
- **MCP**: `kei_check` の `opaque_imports` に extern package 束縛は**含めない**(署名で担保される
  ため。含まれてしまう場合は除外し、応答スナップショットで固定)。
- **取説**: SKILL.md に「npm パッケージを呼ぶ」節(extern package + 署名 + strict-extern の
  ワークフロー)を追加。spec / MCP golden 追従。
- `cargo test --workspace` 全パス。

## スコープ外(v0.6 では入れない)

- named import / default export の直接束縛(段階2、v0.8 で実需確定後 🤝)
- npm パッケージの型自動取り込み(.d.ts 読み取り)— 恒久的にスコープ外の可能性が高い
  (extern 署名が合意書、が Kei の設計)
- async 署名(`uses Async`)— v0.7 の主題
- クラス構築(`new Hono()`)の直接表現 — v0.8 の Hono アダプタ(@kei/hono)で吸収する前提

## 後続 /goal ドラフト

```text
/goal M35: extern package "<spec>" as <name> 宣言を追加する。spec v0.2 extern 節を先に更新し、
syntax/check/fmt/emit の golden で固定する。相対パス・重複・値参照の診断に fix を付ける。
cargo fmt --check / clippy -D warnings / cargo test --workspace を通す。
```

```text
/goal M36: file: ローカルパッケージの e2e で kei build → npm test を CI 固定し、
strict-extern / MCP opaque_imports の整合を取り、SKILL.md に npm 節を追加する。
```
