# Kei 開発ロードマップ v0.7 — /goal 契約書集(async)

> 運用ルール: 各 Milestone は「人間が合意する契約」。完了条件は機械検証可能な形で書く。
> 本ファイルは v1.0 戦略(`docs/kei-roadmap-v0.5.md` 冒頭)の **v0.7: async** — v1.0 blocker 3 つのうちの
> 2 つ目 — を Milestone 化したもの。着手前に本ファイルの「設計原則」を熟読すること。

## v0.7 のゴール

Kei ソースから非同期関数(Cloudflare Workers の fetch handler、KV/D1 の bindings、npm の
async ライブラリ等)を型・エフェクト安全に呼び出せ、`kei build` の生成 TS が
`async function` / `await` / `Promise<T>` を正しく含むこと。**v0.7 は sequential async のみ。**
Promise.all 相当の並列化・レース・キャンセルは v0.8 以降(段階2 🤝)。

## 設計原則(HANDOFF 準拠 — Sonnet に絶対に破らせない)

これらは「Kei らしさ」を決める根幹の合意。実装エージェントは契約書のこの節を必ず読み、疑問が
出たら停止して報告する。

1. **Async は uses モデルに統合する。関数の「色」ではない。**
   - `func fetchUser(id: String) -> User uses Network.Read, Async` の形。既存のエフェクト階層に
     `Async` を独立ルート(IO 傘下ではない — IO 宣言関数が黙って async になる互換性破壊を避ける)
     として追加する。
   - 推移伝播は既存機構そのまま(async 関数を呼ぶ関数は `uses Async` を宣言しなければ KEI-E3001)。
   - `async` キーワードは Kei ソースに登場しない(色ではなくエフェクト、が Kei の立場)。

2. **呼び出し側に `await` 演算子を持たない。コンパイラが emit 時に自動挿入する。**
   - Kei ソース: `let u = fetchUser(id)`(同期的な書き味)。
   - 生成 TS: `const u = await fetchUser(id);`。
   - 根拠: v0.7 は sequential のみで並列制御が言語に要らず、await の書き忘れが起きない。並列化が
     必要になった v0.8 の段階2で `spawn` / `join` 相当を 🤝 で設計する(async そのものは v0.7 に閉じる)。

3. **`Promise<T>` 型は Kei に露出しない。**
   - 戻り型は `-> T`(Promise 化は emit の責務)。第一級関数値と同じく、Kei 型システムには現れない。
   - 生成 TS 側では `uses Async` 関数は `async function f(): Promise<T>` に写る。

4. **契約式は同期・純粋のまま。**
   - `requires` / `ensures` / `old(...)` から async 関数(`uses Async` 持ちや extern async)を呼ぶのは
     KEI-E4001(契約純粋性)+ KEI-E3001(uses 越え)の既存二重診断で拒否。fix は「同期の
     `extern query` 観測子を使う」誘導。
   - async 関数の ensures は**同期述語**で結果を検証(Promise resolve 後、result 値に対して既存
     ヘルパーで評価)。emit は async wrapper 内で await → ensures 評価の順に出す。
   - `old(...)` は関数入口の同期スナップショット(既存挙動そのまま)。

5. **extern async は独立キーワードではなく `uses Async` で表現する。**
   - `extern net.fetch(url: String) -> Response uses Network.Read, Async` の形(既存 extern 署名機構の
     ままエフェクト列に Async が入るだけ)。
   - `extern query` の Async 化は禁止(query は純粋観測子、そのシルエットを守る)。

6. **生成 TS は Kei ソースの意味論を保存する。**
   - async 関数の requires 違反は関数入口(await 前)で `KeiContractViolation` を throw。
   - ensures 違反は Promise resolve 後に throw(reject ではなく throw で、await 側に例外として届く)。
   - source map は既存挙動を維持。

## Milestone 全体像

| M | テーマ | 優先度 | 状態 | 主な改修クレート |
|---|---|---|---|---|
| **M37** | `uses Async` エフェクト + async 関数コア(check + emit) | high | ⬜ 未着手 | kei_check / kei_emit / kei_syntax |
| **M38** | async 境界と検証整合(extern / 契約 / pbt / e2e / 取説) | high | ⬜ 未着手 | kei_check / kei_emit / kei_cli / tests/e2e / spec / skill |

## M37: `uses Async` エフェクトと async 関数コア

### 完了条件

- `effects.rs` の `STANDARD_EFFECTS` に `Async` を独立ノードとして追加(`covers` の IO 包括からは
  除外する。IO 宣言が Async を包含しないことをテストで固定)。
- `func f() -> T uses Async` が check を通り、生成 TS が `async function f(): Promise<T>` になる。
- 内部から `uses Async` 関数を呼ぶ関数は `uses Async` を持つこと(推移伝播、既存機構そのまま)。
- `let x = f()`(f が uses Async)の emit が `const x = await f();`。文式 `f();` は `await f();`。
- ラムダ内での uses Async 関数呼び出しは既存 KEI-E3001(ラムダは純粋スコープ、F6 同型)で拒否。
- 契約式内での uses Async 関数呼び出しは既存 KEI-E4001 + KEI-E3001 の二重診断で拒否。
- spec `spec/kei-spec-v0.1.md` §5 の「非同期の扱い」注記を「v0.7 で導入」に更新し、§3.2 の
  エフェクト階層に Async を独立ルートとして追記。§9 の「未決事項 1」を決着として消化。
- spec §3.2 付近の「`uses IO` は全 IO の包括許可(雑だが合法)」相当の記述に、
  **Async は例外(IO は Async を包含しない)** の一文を明記する(本文と階層図の両方で整合)。
- syntax は変更なしのはず(uses 節のパースは既存の識別子リスト)。fmt は uses 節の順序を
  ソースのまま素通しする既存挙動(`kei_fmt/src/lib.rs` に .sort() 系のロジックなし)を維持
  し、Async も通常の識別子として扱う(**fmt 側の変更は不要**)。
- 生成 TS の async 関数が既存の `KeiContractViolation` throw を保つこと(requires: 入口・await 前、
  ensures: await 後の同期評価)。
- golden(check / fmt / emit 単体)+ 既存 CLI e2e で回帰なし。`cargo test --workspace` 全パス。

### 明示的にスコープ外(M37)

- extern async 署名(M38)/ 並列(Promise.all 相当、v0.8+ 🤝)/ Async を含む extern query(禁止仕様)。
- await 明示演算子(spec で「不採用」と明記)。
- キャンセル / タイムアウト / AbortController(v0.8 の HTTP 境界で必要なら再検討)。

## M38: async 境界と検証整合

### 完了条件

- **extern async**: `extern net.fetch(url: String) -> Response uses Network.Read, Async` の形が
  check / emit を通り、呼び出し側が uses Async を宣言していれば `await net.fetch(url)` が emit される。
- **extern query の Async 排除**: `extern query` に `Async` を含む uses が付いたら KEI-E3005 相当の
  新規診断(または既存 3005 の拡張)で拒否。fix「query は純粋観測子、通常の extern を使え」。
- **strict-extern 整合**: パッケージ束縛 + async 未宣言呼び出しが `--strict-extern` で warning になる
  ことをテストで固定。
- **契約 async**: async 関数の ensures 違反が Promise resolve 後に throw で伝播することを e2e で固定
  (KeiContractViolation の名前・shape は既存互換)。
- **pbt / const_eval**: `uses Async` 関数を含む契約は generative 検証で `skipped` として応答に列挙
  (M34 の可視化機構に相乗り)。eval は Unsupported に倒す。
- **e2e**: `tests/cli/packages/` に async な関数を返す小さな npm パッケージを追加し(M36 の greeter と
  同じ流儀、`file:` 依存)、`kei build` → `tsc --strict` → vitest で await が正しく効いていることを
  CI で固定。Cloudflare Workers そのものは v0.9 の領分なので、ここでは Node の Promise で足りる。
- **MCP**: `kei_check` の応答(generative の skipped)に uses Async 関数がスキップ理由付きで出ることを
  スナップショットで固定。
- **取説**: `skills/kei/SKILL.md` に「async を書く」節を新設(uses Async / 呼び出し規約 / 契約の
  書き方 / extern async / 制限)。コード例は `kei check` / `kei fmt --check` クリーンに。
- `cargo test --workspace` 全パス。

## 後続 /goal ドラフト

```text
/goal M37: uses Async エフェクトを追加し、async 関数の check + emit(async function/await 自動挿入)を
実装する。IO は Async を包含しない。呼び出し側に await 演算子は追加しない。spec §3.2/§5 を先に
更新し、check/fmt/emit の golden で固定する。契約式内の async 呼び出しは既存の純粋性診断で拒否する。
cargo fmt --check / clippy -D warnings / cargo test --workspace を通す。
```

```text
/goal M38: extern async 署名(uses Async)を受理し、extern query の Async を拒否する。file: ローカル
async パッケージの e2e を CI 固定し、契約 ensures が Promise resolve 後に throw で伝播することを
e2e で確認。generative は Async 関数を skipped に列挙、SKILL.md に「async を書く」節を追加する。
```
