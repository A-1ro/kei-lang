# Kei 開発ロードマップ v0.10 — /goal 契約書集(純ロジックは全部 Kei で書ける)

> 運用ルール: 各 Milestone は「人間が合意する契約」。完了条件は機械検証可能な形で書く。
> 本ファイルは `docs/kei-roadmap-v0.5.md` の v1.0 逆算戦略を差し替えるもの。
> v0.9.0 の後、**v1.0(実 `wrangler deploy`)を後ろ倒しし、その手前に v0.10 を挿入する**(オーナー決定)。
> **🤝(人間との設計合意)5 点はすべて合意済み(2026-07-27)。** 決定と経緯は末尾「🤝 着手前合意事項」節と各 Milestone の「✅ 合意済み」節を参照。
> **着手前に本ファイルの「設計原則」節を熟読すること。**

## 更新履歴

- 2026-07-27: 起草。v0.10 のテーマ・傘下 issue・受け入れ基準を確定。詳細 /goal は 🤝 合意後に追記。
- 2026-07-27: **/goal 契約書化(M44〜M47)を起草。** 設計原則・Milestone 分解・完了条件・🤝 着手前合意事項(5 点)を追記。
- 2026-07-27: **🤝 5 点すべて合意済み**(length は加法 — UTF-16 の `length` 温存 + `codePointCount()` 追加 / stdlib 段階2は high tier + `contains`(pad・repeat は v1.x 送り)/ 正規表現は定石例示 + extern 境界の併用(言語内エンジンなし)/ 並行結合子は `parallel` — 同種 List を並行実行し emit は `Promise.all` / 順序は String 先行 M44→M45→M46→M47)。各 Milestone の合意節・/goal 文を確定版に更新。

## なぜ v0.10 を挿入するか

v0.9.0 の後、Kei が初めて実運用の Cloudflare Worker に統合された(文字数上限バリデーションを、契約付き純関数として `kei build` → TS → ベンダリングして本番 Worker に組み込んだ事例)。ここで、より広い採用を阻んだのは wrangler 側ではなく **言語の String 表現力**だった:

1. **String の意味論**: `s.length` が UTF-16 code unit 長(v0.5 M30 / #107)で、絵文字(`😀` = 2 code unit)を含む文字数判定が実仕様とズレる。この 1 点で**文字数カウント本体を Kei 化できなかった**。
2. **String stdlib が薄い**: `split` / `indexOf`(v0.9 M41 / #136)止まりで、Markdown 除去・slug 生成・タグ正規化・MIME/キーのバリデーションといった「典型的なアプリの面白い純ロジック」を書けず TS に残った。

実デプロイ(v1.0)に進む前に、「純ロジックが本当に全部 Kei に寄るか」を言語表現力の側で満たす、というのが v0.10 の狙い。

## v0.10 のゴール

**Markdown 除去 / slug 生成 / タグ正規化 / MIME・キーのバリデーション**という class の純関数群が、Kei で表現でき、**JS 参照実装との等価テスト(同一入力 → 同一出力)に通る**こと。加えて、実運用ステージ2で必要になる独立 I/O の**並行 async** を最小形で言語に導入する(#161・オーナー決定で v0.10 に正式編入)。

バージョン運用: 本ファイルの Milestone が全て閉じた時点で **0.10.0** をタグする(v0.5 / v0.9 と同じ運用)。

## 傘下 issue

| # | テーマ | 状態 |
|---|---|---|
| #159 | String の code point 意味論 — `length`/イテレーションを code point 単位に(grapheme は境界明示 + extern 誘導) | 提案 |
| #160 | String stdlib 段階2拡張 — replace / substring / 大小文字 / trim / 前後方一致 等 + 正規表現の態度明示 | 提案 |
| #161 | 並行 async の緩和 — v0.10 に正式に含める(オーナー決定 2026-07-27) | 提案 |
| #162 | (トラッキング)本テーマの傘 issue | — |

## 受け入れ基準

- 絵文字を含む文字数判定が、UTF-16 ではなく code point 単位で人間の感覚どおりに動く(#159)。
- Markdown 除去 / slug 生成 / タグ正規化 / MIME・キーのバリデーションの代表 4 関数が `examples/` に Kei で置かれ、**JS 参照実装との等価テストが CI で通る**(#160)。
- 独立 I/O 2 本を並行実行する例が `examples/` に入り、直列版との結果等価と並行性(観測可能な順序非依存)がテストで示される(#161)。
- grapheme と正規表現について、v0.10 が「どこまで約束し、どこからは境界の外(extern 誘導)か」を spec に明文化する。

## v0.10 の性格 — 「String に限って言語を意図的に拡張する + 並行 async を最小形で足す」版

v0.8/v0.9 の「言語コア意味論は原則据え置き」路線に対し、v0.10 は **String まわりに限って言語 stdlib / 意味論を意図的に拡張する**。拡張範囲は String に閉じる:

- HTTP/Workers 境界(v0.8/v0.9 の領分)には踏み込まない。
- 実デプロイ(v1.0)には踏み込まない。
- 並行 async(#161)は effect / 実行モデルの領分で、テーマ本丸(String)からは外れるが、**v0.10 に正式に含める**(オーナー決定 2026-07-27)。**最小形に限る**(下記 M47・設計原則 6)。v1.0 は機能拡張を持たず、**なるべくリリース判断だけ**(実 `wrangler deploy` の実施と初見エージェント実証)の薄いマイルストーンにする。

## 設計原則(HANDOFF 準拠 — 実装エージェントに絶対に破らせない)

v0.7〜v0.9 の原則(Async は色ではなくエフェクト / `await` 演算子を持たない / `Promise<T>` を Kei に露出させない / 契約式は同期・純粋のまま / 言語コアの変更は最小限)をすべて引き継いだ上で、v0.10 固有の原則を追加する。

1. **【v0.10 新規】String 意味論の変更は「境界を明文化する」こととセット。** Kei の String が **どの単位まで保証するか**を spec に一義的に書く。v0.10 が約束するのは **code point 単位まで**。grapheme(書記素クラスタ・ZWJ 連結・結合文字)segmentation と Unicode 正規化(NFC/NFD)は **言語内で実装せず、境界を明記した上で extern(TS 側 `Intl.Segmenter` 等)へ誘導**する。「ここまでが Kei の純ロジック、ここからは境界の外」を曖昧にしない(#159)。

2. **【v0.10 新規】String stdlib の追加は spec-first・emit は TS 標準 String に素直に落とす。** 新 API はまず `spec/kei-spec-v0.1.md` §2.6(String / Int 組み込み一覧)に意味論を書き、その後に check(`STRING_BUILTIN_MEMBERS` / `string_method`)/ emit / pbt / golden を実装する(v0.9 M41 の `split` / `indexOf` と同じ流儀)。emit は原則 TS の `String.prototype.*` へ写す。**ただし code point 単位を規定した API(`substring` など)は、UTF-16 前提の TS メソッドに素直に落とすとサロゲートを壊すため、runtime helper 経由で code point 尊重の実装にする**(#159 との整合を emit で守る)。

3. **【v0.10 新規】code point イテレーションに新構文を足さない。** 「1 文字ずつ処理」の経路は、v0.9 M41 で spec 済みの **`s.split("")`(空デリミタは code point 単位で分割)→ `List<String>`** を土台にする。畳み込み(既存 `List` 段階1)と組み合わせれば Markdown を 1 文字ずつ舐める用途は表現できる。新しいイテレーション構文・for 文などは v0.10 では導入しない。

4. **【v0.10 新規】正規表現は言語に入れない(✅ 🤝(c) 合意済み)。** 正規表現は契約検証(`requires`/`ensures`)・pbt・決定性の観点で重く、Kei の「契約式は同期・純粋・静的に扱える」路線と相性が悪い。v0.10 は **String プリミティブ + code point イテレーションで「正規表現を使わずに書く定石」を spec / SKILL に例示**し、どうしても必要なパターンは **extern で TS 側の `RegExp` に出す境界**を明記する(grapheme = extern と同じ発想)。

5. **【v0.10 新規】既存契約本文(golden / #107 の `length` 意味論)を壊さない。** `tests/golden/` は契約本文(不変条件 1)。`length` の意味を変えると既存 golden と実運用でベンダリング済みの TS の前提が同時に崩れる。v0.10 は **既存 `length`(UTF-16)を温存し、code point 用の新 API を追加**する加法的変更とする(✅ 🤝(a) 合意済み)。expected の変更が避けられない場合は人間レビュー必須。

6. **【v0.10 新規】並行 async は「最小の並行結合子」に閉じる。契約と effect の相互作用を増やさない。**
   - 並行の対象は **独立した副作用(複数 I/O)の同時実行**のみ。導入するのは並行結合子 1 つ(✅ 🤝(d) 合意済み: 同種リスト結合子 `parallel`)で、emit は `await Promise.all([...])` に落とす。
   - **契約の意味論は増やさない**: 各 async 関数の `requires` は自分の入口で、`ensures` は自分の resolve 後に(v0.7 の既存挙動そのまま)評価される。**並行結合子そのものは契約を持たない**。「並行実行時の事後条件」という新しい契約意味論を v0.10 では作らない。
   - 失敗時は **fail-fast**(いずれかが reject/throw したら全体が伝播。`Promise.all` 準拠)。race / キャンセル / タイムアウト / 構造化並行スコープ / 異種型タプル結合は **v0.10 スコープ外**(v1.x 以降で実需確認後)。
   - `uses Async` の推移伝播は既存機構そのまま。並行結合子を使う関数も `uses Async` を宣言する。

## Milestone 全体像と順序

M 番号は v0.9(M40〜M43)からの連番。**🤝 5 点合意済み(2026-07-27)— 全 Milestone 着手可**。

| M | テーマ | issue | 優先度 | 状態 | 主な改修クレート |
|---|---|---|---|---|---|
| **M44** | String の code point 意味論(`codePointCount` + code point イテレーション + grapheme 境界明文化) | #159 | high | ✅ 合意済み・未着手 | kei_check / kei_emit / pbt / spec / skill / examples |
| **M45** | String stdlib 段階2(substring / replace / 大小文字 / trim / 前後方一致 等)+ 正規表現の態度明示 | #160 | high | ✅ 合意済み・未着手 | kei_check / kei_emit / pbt / spec / skill |
| **M46** | 純ロジック等価テスト実証(Markdown 除去 / slug / タグ正規化 / MIME・キーバリデーションの代表 4 関数)| #160 | high(テーマ合否ゲート) | ✅ 合意済み・未着手 | examples / tests(等価テスト)/ CI / kei_mcp(埋め込み) |
| **M47** | 並行 async — 最小の並行結合子(独立 I/O の同時実行) | #161 | medium | ✅ 合意済み・未着手 | kei_syntax / kei_check / kei_emit / spec / skill / tests |

順序の論拠:

- **M44 → M45 → M46 が String の本丸。** M46(等価テスト実証)は v0.10 テーマの**合否ゲート**で、M44(code point 意味論)と M45(stdlib)の両方の完了に依存する。この 3 本を先に閉じてテーマ受け入れを確定させる。
- **M44 は M45 の前。** M45 の `substring` などは「範囲を code point 単位で規定」(#160 の high tier)するため、M44 の code point 意味論(単位の定義・emit の runtime helper 方針)を先に固める。
- **M47(並行 async)は最後・独立トラック。** テーマ本丸(String)からは外れ(#161 も明記)、effect × 契約という別レイヤに触れる。テーマ受け入れ(M46)をブロックしないよう最後に置く。String 系(M44〜M46)と実装が独立なので並走も技術的には可能だが、String 完了後に着手する(✅ 🤝(e) 合意済み — String 先行)。

## M44: String の code point 意味論(#159)

実運用統合で「文字数カウント本体を Kei 化できなかった」直接の障害物(`length` の UTF-16 意味論)の解消。**v0.10 の最初の言語変更**。

### ✅ 合意済み(2026-07-27)

- **🤝(a) `length` の意味論 — 加法を採用**: 既存 `s.length`(UTF-16 code unit 長)は**温存**し、`s.codePointCount() -> Int`(`😀` = 1)を**追加**する。spec / SKILL で「文字数判定は `codePointCount` を使う」を推奨として明示する。経緯・不採用案は末尾 🤝(a) 参照。

### 完了条件(機械検証可能)

- 合意した API(`s.codePointCount() -> Int` を**追加**。既存 `length`(UTF-16)は温存)を `spec/kei-spec-v0.1.md` §2.6 の String 組み込み一覧に spec-first で追記する。
- 新 API が `😀`(サロゲートペア)・合字を含む文字列で **code point 数を返す** golden test(check / emit)が通る(`"a😀b".codePointCount() == 3`)。
- emit は code point を尊重する TS 機構(`Array.from(s).length` / `String.fromCodePoint` / `for...of`)に落とし、サロゲートを壊さない(runtime helper 経由。`crates/kei_emit/src/emit.rs` の runtime-method 登録に追加)。
- **code point イテレーション**は新構文を足さず、既存 `s.split("")`(空デリミタ = code point 単位。v0.9 M41 で spec 済み)+ `List` 畳み込みで表現する。1 文字ずつ処理する例が `examples/` に入り、**JS 参照実装(`Array.from(s)`)との等価テストが通る**。
- 契約式内でも新 API を使用可(純粋)。pbt(`crates/kei_check/src/pbt.rs` の `str_domain`)に **サロゲートペア / 合字を含む境界値**を追加し、`length`(UTF-16)と `codePointCount`(code point)の差が pbt で観測できる。
- spec / `skills/kei/SKILL.md` に「**Kei の String は code point 単位まで保証・grapheme と正規化は境界外(extern 誘導)**」の節を新設する(設計原則 1 の明文化)。SKILL 更新に伴う MCP golden 再生成を含む。
- `cargo fmt --all -- --check` / `cargo clippy --workspace --all-targets -- -D warnings` / `cargo test --workspace` 全パス。
- issue #159 を PR の `Closes #159` でクローズする(不変条件 6)。

### golden / test 設計方針

- 新 API は通常の言語機能と同じ扱い: spec → golden(check / fmt / emit)→ pbt 追従 → e2e(等価テスト)の順で固定する。
- `length` を温存する加法路線(✅ 合意済み)なので既存 golden は無変更のはず。万一 expected の変更が必要になったら人間レビューに乗せる(不変条件 1)。
- grapheme は**約束しない**ので golden を持たない。spec / SKILL の文章と、extern 誘導のコード例(fmt --check クリーン)で境界を示す。

### スコープ外(M44)

- grapheme(書記素クラスタ)segmentation の言語内実装 — extern(`Intl.Segmenter` 等)へ誘導。
- Unicode 正規化(NFC/NFD)・大小文字畳み込み以外の照合 — 言語内で持たない。
- ロケール依存の文字数え上げ。

## M45: String stdlib 段階2 + 正規表現の態度明示(#160)

実アプリの「面白い純ロジック」を Kei で書くための String API 拡充。#159 の code point 意味論と対になる両輪。

### ✅ 合意済み(2026-07-27)

- **🤝(b) stdlib 段階2 の API 範囲 — high tier + `contains` を採用**(`repeat` / `padStart` / `padEnd` の medium tier は v1.x 送り)。経緯は末尾 🤝(b) 参照。
- **🤝(c) 正規表現の態度 — 定石例示 + extern 境界の併用を採用**(言語内エンジンは入れない)。経緯は末尾 🤝(c) 参照。

### 完了条件(機械検証可能)

- 合意した API(high tier + `contains`)を `spec/kei-spec-v0.1.md` §2.6 に spec-first で追記する:
  - `s.substring(start: Int, end: Int) -> String`(範囲は **code point 単位**で規定。#159 / M44 と整合。runtime helper で実装)
  - `s.replace(from: String, to: String) -> String` / `s.replaceAll(from: String, to: String) -> String`
  - `s.toLowerCase() -> String` / `s.toUpperCase() -> String`
  - `s.trim() -> String`
  - `s.startsWith(prefix: String) -> Bool` / `s.endsWith(suffix: String) -> Bool`
  - `s.contains(sub: String) -> Bool`(`indexOf(...) != None` の可読化)
- 各 API を check(`STRING_BUILTIN_MEMBERS` への追加 + `string_method` の arity/型付け)/ emit(TS `String.prototype.*` へ写す。`substring` は code point 尊重 runtime helper)/ fmt(識別子扱いで変更不要見込み)/ pbt に実装し、syntax / check / fmt / emit の golden で固定する。
- **正規表現の態度**(✅ 合意済み: 言語に入れない)を spec に **1 節**として明文化する。slug 生成・タグ正規化を**正規表現を使わずに書く定石**(小文字化 → 許可外 code point を畳み込みで除去/置換)を spec / SKILL に例示し、どうしても必要なパターンは extern で TS `RegExp` に出す境界を書く。
- 契約式内でも新 API を使用可(純粋)。
- `skills/kei/SKILL.md` の String 節を更新(段階2 API 一覧 + 正規表現の方針)。MCP golden 再生成を含む。
- `cargo fmt --check` / `clippy -D warnings` / `cargo test --workspace` 全パス。
- issue #160 の stdlib 部分を PR で対応(等価テスト実証は M46。#160 のクローズは M46 の PR で行う — 下記)。

### golden / test 設計方針

- 段階2 API は M41 の `split` / `indexOf` と同格: spec → golden(check/fmt/emit)→ pbt 追従 → e2e。
- `substring` の code point 単位規定は、UTF-16 index との差が出る入力(絵文字を含む)を golden / pbt に必ず含める(#159 との整合の回帰防止)。
- 正規表現は**言語機能として実装しない**(✅ 合意済み)ので golden を持たない。方針文と定石コード例(fmt --check クリーン)で担保する。

### スコープ外(M45)

- medium tier(`repeat` / `padStart` / `padEnd`)— ✅ 🤝(b) で v1.x 送り確定(実需確認後)。
- 正規表現エンジンの言語内実装 — ✅ 🤝(c) で不採用確定。
- ロケール依存の大小文字変換・照合。

## M46: 純ロジック等価テスト実証 — テーマ合否ゲート(#160)

v0.10 テーマ「純ロジックは全部 Kei で書ける」の**合否そのもの**。M44 / M45 の成果で、代表 4 関数が JS 参照と等価に振る舞うことを CI で示す。

### ✅ 合意済み(2026-07-27)

- 本 Milestone 固有の 🤝 はない(M44 / M45 の 🤝 が確定済みなので着手可)。ただし 4 関数の**仕様(入出力の正確な定義)**は実装前の明示が必要(例: slug の許可文字集合、Markdown 除去の対象記法)。実装エージェントは 4 関数の JS 参照仕様を PR 冒頭に明記してからコードを書く。

### 完了条件(機械検証可能)

- `examples/` に代表 4 関数を Kei で追加する(いずれも純関数・契約付き・`kei fmt --check` クリーン):
  1. **Markdown 除去**(見出し `#` / 強調 `*_` / インラインコード `` ` `` / リンク記法などを剥がしてプレーンテキスト化)
  2. **slug 生成**(小文字化 → 許可外文字をハイフンに → 連続ハイフン畳み込み → 前後トリム)
  3. **タグ正規化**(トリム + 小文字化 + 内部空白の単一化など)
  4. **MIME / キーのバリデーション**(`startsWith` / `endsWith` / 許可文字判定で真偽を返す)
- 各関数に **JS 参照実装**を対に置き、**等価テスト(同一入力 → 同一出力)**を追加する。入力は固定代表ケース + ランダム/プロパティ生成(絵文字・記号・空文字・長文を含む)。テストは `tests/` 配下に置き、`kei build` → `tsc --strict` → vitest の既存 e2e 流儀に乗せる。
- **絵文字を含む文字数判定が code point 単位で人間の感覚どおり**に動くことを、上記のいずれか(またはバリデーション例)で観測できる受け入れをここで確定する。
- **`.github/workflows/ci.yml`** でこの等価テストが CI で常に実行される(必須・分離 optional ジョブにしない)。CI green のログを確認して表示する。
- examples/ は kei_mcp にビルド時埋め込み(不変条件 3)されるため、MCP golden の再生成と `cargo test --workspace` 全パスを確認する。
- issue #160 を PR の `Closes #160`(および #159 が未クローズなら併せて)でクローズする。
- 完了時点で v0.10 の受け入れ基準(本ファイル冒頭)のうち String 系(#159 / #160)を満たしたことを、コマンド出力つきで報告する。

### golden / test 設計方針

- 等価テストは「Kei 生成 TS の出力 == JS 参照実装の出力」を assert する(スナップショット固定ではなく参照実装との一致)。ランダム入力の seed を固定して再現性を持たせる。
- 4 関数は examples/ 配下なので正規形維持(fmt --check)を CI 対象にする(既存 examples 検査経路に乗せる)。
- 言語コアの変更はここでは行わない(M44 / M45 で完了済みが前提)。不足 API が見つかったら M45 に差し戻す(勝手に stdlib を足さない)。

### スコープ外(M46)

- HTTP / Workers 境界への組み込み(v0.9 で完了・実デプロイは v1.0)。
- 正規表現前提の実装(✅ 🤝(c) の方針 — 言語に入れない — に従う)。
- 4 関数以外のアプリ全体の Kei 化。

## M47: 並行 async — 最小の並行結合子(#161)

v0.7 で入った `uses Async` は逐次のみ。独立 I/O(複数 KV/D1 読み取り・複数 fetch)を並行実行する手段を最小形で足す。**テーマ本丸(String)からは外れる独立トラック**(オーナー決定で v0.10 に正式編入)。

### ✅ 合意済み(2026-07-27)

- **🤝(d) 並行結合子の形 — 同種リスト結合子 `parallel` を採用**: `parallel(xs) -> List<T>`(同じ型 `T` を返す独立 async を並行実行し `List<T>` で受ける。emit は `await Promise.all([...])`)。異種タプル結合子・構造化並行ブロックは不採用(経緯は末尾 🤝(d) 参照)。

### 完了条件(機械検証可能)

- 合意した並行結合子(**同種リスト結合子** `parallel(xs) -> List<T>`。`xs` は `uses Async` を伴う要素の並行実行を表す)を実装する。
  - **check**: 結合子を使う関数は `uses Async` を宣言していること(既存推移伝播機構。宣言漏れは KEI-E3001)。結合子自体は契約を持たない(設計原則 6)。契約式内での結合子使用は既存の純粋性診断(KEI-E4001 + KEI-E3001)で拒否。
  - **emit**: `await Promise.all([...])` に落とす(要素の各 async 呼び出しは並行に開始され、全 resolve を待つ)。fail-fast(`Promise.all` 準拠)。
  - **spec**: `spec/kei-spec-v0.1.md` §5(非同期の扱い)に、並行の**意味論(独立実行・実行順非依存・失敗時 fail-fast・ensures との相互作用 = 各要素の ensures はそのまま、結合子は契約を持たない)**を明文化する。v0.7 の「sequential のみ」注記を「v0.10 で最小の並行結合子を追加」に更新する。
- **例と等価/並行テスト**: `examples/` に独立 I/O 2 本を並行実行する例を置き、(1) 直列版との**結果等価**、(2) **並行性**(観測可能な順序非依存 — 例: 開始ログの順序が入力順に固定されない/合計待ち時間が直列和より短いことを擬似 I/O で観測)をテストで示す。`tests/` の e2e に乗せ、Node の Promise で足りる(Workers 実行は v0.9 済 / 実デプロイは v1.0)。
- **pbt / const_eval**: 並行結合子を含む契約は generative 検証で `skipped` として応答に列挙(v0.7 M38 の可視化機構に相乗り)。
- **MCP**: `kei_check` の応答に並行結合子を含む関数のスキップ理由が出ることをスナップショットで固定。
- `skills/kei/SKILL.md` に「**並行 async を書く**」節を新設(結合子の使い方 / 逐次との使い分け / 制限 = race・キャンセルは未サポート)。MCP golden 再生成を含む。
- `cargo fmt --check` / `clippy -D warnings` / `cargo test --workspace` 全パス。
- issue #161 を PR の `Closes #161` でクローズする。

### golden / test 設計方針

- 並行結合子は言語機能なので spec → golden(syntax / check / fmt / emit)→ pbt 追従 → e2e の順で固定する。
- emit の `Promise.all` 展開は golden(emit)で形を固定する。
- 契約意味論を増やさない(設計原則 6)ので、契約 × 並行の新規 golden は「結合子は契約を持たない・各要素の ensures は既存どおり」を確認する最小限に留める。

### スコープ外(M47)

- race(`Promise.race` 相当)・最初に成功したものを取る合流。
- キャンセル / タイムアウト / `AbortController`。
- 構造化並行スコープ(nursery / task group)。
- 異種型タプル結合 — ✅ 🤝(d) で不採用確定(需要が強ければ v1.x で再検討)。
- 並行実行時の新しい契約意味論(「並行事後条件」等)。

## 🤝 着手前合意事項(✅ 全 5 点合意済み 2026-07-27)

以下 5 点は着手前にオーナー判断を仰いだ設計合意事項。**2026-07-27 にすべて決定済み**。各項は決定と、検討したが採らなかった案の理由(経緯)を残す。

### 🤝(a) `String.length` の意味論 — ✅ 加法を採用

**決定: 既存 `s.length`(UTF-16 code unit 長。v0.5 M30 / #107)を温存し、`s.codePointCount() -> Int`(`😀` = 1)を追加する。** spec / SKILL で「文字数判定は `codePointCount` を使う」を推奨として明示する。

経緯(不採用案): `length` を code point に切り替える破壊的変更は、`tests/golden/`(契約本文・不変条件 1)と実運用でベンダリング済みの TS の前提を同時に壊し、「Kei の `length` = JS の `String.prototype.length`」という spec 明記の対応関係も崩すため不採用。code point 保証の新 String 型は型と stdlib が二重化しコスト過大で不採用。実統合で必要だったのは「code point 数を数える手段」であり加法で過不足なく満たせる。**残る懸念**: 「素朴に `length` を使うと絵文字でズレる」罠は、spec / SKILL の推奨明示と lint 的な誘導(将来検討)で緩和する。

### 🤝(b) String stdlib 段階2 の API 範囲 — ✅ high tier + `contains` を採用

**決定: `substring` / `replace` / `replaceAll` / `toLowerCase` / `toUpperCase` / `trim` / `startsWith` / `endsWith` + `s.contains(sub) -> Bool`。** medium tier(`repeat` / `padStart` / `padEnd`)は v1.x 送り(実需確認後)。

経緯: 受け入れ基準は「代表 4 関数の等価テスト(M46)」で、high tier がほぼ直結する。`contains` は `indexOf(...) != None` の薄い糖衣だが可読性の効果が高く実装コストがほぼゼロ。pad/repeat は 4 関数に不要で、「言語変更は最小限」に従い送り。**分岐**: M46 の実装中に本範囲で足りないと判明したら、勝手に足さず M45 に差し戻して人間判断を仰ぐ。

### 🤝(c) 正規表現に対する v0.10 の態度 — ✅ 定石例示 + extern 境界の併用を採用

**決定: 正規表現エンジンは言語に入れない。** String プリミティブ + code point イテレーションで「正規表現なしで書く定石」を spec / SKILL に例示し、どうしても必要なパターンは extern で TS 側 `RegExp` に出す境界を spec に明記する。

経緯: grapheme = extern(設計原則 1)と同じ「境界を曖昧にしない」思想に一貫させた。小型エンジン内蔵案は、正規表現が契約式で使えると `requires`/`ensures` の静的扱い・pbt・決定性がすべて重くなり、v0.10 の String に閉じたスコープを大きく超えるため不採用。**運用注記**: 代替定石だけで 4 関数(特に slug)が実務的な読みやすさで書けない場合は、extern 誘導を主経路として spec に強めに書く。

### 🤝(d) 並行 async の結合子の形 — ✅ 同種リスト結合子 `parallel` を採用

**決定: `parallel(xs) -> List<T>`。** 同じ型 `T` を返す複数の独立 async を並行実行し `List<T>` で受ける。emit は `await Promise.all([...])`(fail-fast も `Promise.all` 準拠)。

経緯: 既存 `List<T>` に載るので新しい型(タプル)を導入せずに済み、emit が `Promise.all` に一対一で落ちて意味論が明快。異種タプル結合子(`join(a, b) -> (A, B)`)はタプル型という言語コアの拡張を要し「最小形」を超えるため不採用。構造化並行ブロック(`parallel { ... }`)は「どの呼び出しを並行と見なすか」の解析と `await` 自動挿入(v0.7)との相互作用が複雑になるため不採用。**制約の明示**: 型の違う 2 本(例: KV から String・D1 から record)を 1 回で並行にしたい需要が強ければ、異種タプル結合を v1.x で再検討する、と spec に注記する。

### 🤝(e) Milestone の順序 — ✅ String 先行を採用

**決定: M44 → M45 → M46(テーマ受け入れ確定)→ M47(async)。**

経緯: v0.10 のテーマは「純ロジック(String)は全部 Kei で書ける」で、合否ゲートは M46。テーマの受け入れを最短で確定させるため String 3 本を先に閉じる。並行 async(#161)は「テーマ本丸から外れる」と issue 自身が認めており、effect × 契約という別レイヤに触れるため最後に置く。M47 は String 系と実装が独立なので並走しても技術的衝突はないが、「テーマ受け入れ(M46)を async の遅延で止めない」を担保する。

## スコープ外(v1.0 以降)

- 実 `wrangler deploy`(本番デプロイ)と初見エージェントによる受け入れ実証 — v1.0。
- grapheme(書記素クラスタ)segmentation の言語内実装 — v0.10 は code point 単位までを約束し、grapheme は extern(TS 側 `Intl.Segmenter` 等)へ誘導する境界を spec に書くに留める。
- 正規表現エンジンの言語内実装(✅ 🤝(c) で不採用確定)。
- 並行 async の race / キャンセル / タイムアウト / 構造化並行スコープ / 異種型タプル結合(M47 スコープ外)。
- String stdlib medium tier(`repeat` / `padStart` / `padEnd`。✅ 🤝(b) で v1.x 送り確定)。

## 後続 /goal ドラフト(✅ 合意反映済み)

🤝(a)〜(e)の決定(2026-07-27)を反映済み。M44 から順に実行する。

```text
/goal M44: String の code point 意味論を追加する。🤝(a) 合意(length は温存し
codePointCount() を追加)に従い、(1) spec §2.6 に新 API を spec-first で追記、
(2) "a😀b".codePointCount() == 3 の golden(check/emit)、(3) emit は Array.from/
String.fromCodePoint で code point 尊重(runtime helper)、(4) code point イテレーションは
既存 split("") + 畳み込みで表現し JS 参照(Array.from)との等価テストを examples に追加、
(5) pbt の str_domain にサロゲート/合字の境界値を追加、(6) spec/SKILL に「code point まで
保証・grapheme と正規化は extern 誘導」節を新設して MCP golden 再生成。cargo fmt/clippy/test
を通し、Closes #159。
```

```text
/goal M45: String stdlib 段階2 を追加する。🤝(b) 合意の範囲(high + contains)を spec §2.6 に
spec-first で追記し、substring(code point 単位・runtime helper)/ replace / replaceAll /
toLowerCase / toUpperCase / trim / startsWith / endsWith / contains を check(STRING_BUILTIN_MEMBERS
+ string_method)/ emit / pbt / golden(check/fmt/emit)で固定する。🤝(c) 合意の正規表現の態度
(言語に入れない + 定石例示 + extern 境界明記)を spec に 1 節書く。SKILL の String 節を
更新して MCP golden 再生成。cargo fmt/clippy/test を通す(#160 のクローズは M46)。
```

```text
/goal M46: 純ロジック等価テストを実証する。examples に Markdown 除去 / slug 生成 / タグ正規化 /
MIME・キーバリデーションの 4 関数を Kei(純関数・契約付き・fmt --check クリーン)で書き、
各関数に JS 参照実装を対に置いて等価テスト(固定 + ランダム入力・絵文字含む)を tests に追加する。
kei build → tsc --strict → vitest の流儀に乗せ、.github/workflows/ci.yml で常設(必須)。
絵文字を含む文字数判定が code point 単位で動く受け入れをここで確定。MCP 埋め込み golden を
再生成して cargo test --workspace を通し、CI green のログを表示して Closes #160(必要なら #159 も)。
```

```text
/goal M47: 並行 async の最小結合子を追加する。🤝(d) 合意の形(parallel(xs) -> List<T>)を
check(uses Async 伝播・結合子は契約を持たない・契約式内は既存純粋性診断で拒否)/ emit
(await Promise.all([...])・fail-fast)/ golden(syntax/check/fmt/emit)で実装する。spec §5 に
並行の意味論(独立実行・順序非依存・fail-fast・ensures は各要素のまま)を明文化し「sequential のみ」
注記を更新。examples に独立 I/O 2 本の並行例を置き、直列版との結果等価と並行性(順序非依存)を
tests で示す。generative は skipped 列挙、SKILL に「並行 async を書く」節を追加して MCP golden
再生成。cargo fmt/clippy/test を通し、Closes #161。
```

## v1.0 の位置づけ(オーナー決定 2026-07-27・再掲)

機能拡張はすべて v0.10 側に寄せ、v1.0 は**なるべくリリース判断だけ**の薄いマイルストーンにする:

- 実 API を Kei で記述して Cloudflare Workers に**実デプロイ**すること(`wrangler deploy` 本番実行)。
- **kei-dogfood による初見実証**(初見エージェントが SKILL.md だけで Workers API を書けることの実証)。
- v1.0 タグ付け・リリースノート・README の全面更新。

v1.0 契約書集は v0.10 全 Milestone 完了後に別途新設して扱う。
