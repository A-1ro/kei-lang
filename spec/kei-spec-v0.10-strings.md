# Kei 言語仕様書 v0.10 — String の Unicode 境界(code point まで保証)

> v0.1(`kei-spec-v0.1.md` §2.6)への差分章。v0.10 のテーマは
> **「純ロジックは全部 Kei で書ける」— まず String の表現力**。出典は v0.9.0 後の
> 実運用統合で判明した String 意味論のギャップ(`length` の UTF-16 意味論 #159)。
> v0.1 §2.6 と本章が矛盾する箇所は本章を新しい正とする。
> ロードマップは `docs/kei-roadmap-v0.10.md`(M44〜M47)。topic: `"strings"`。

## 0. ステータス

- **M44 実装完了(v0.10)。** `s.codePointCount() -> Int` を追加(加法。既存 `s.length` =
  UTF-16 code unit 長は温存)。「1 文字ずつ」処理する正規経路 `s.split("")` を **code point 単位**に
  是正(runtime helper `keiStringSplit`)。kei_check(型)/ kei_emit(runtime helper)/ pbt
  (境界値 + `codePointCount` の bounded 評価)/ golden / examples に入った。
- M45(String stdlib 段階2)・M47(並行 async)は後続。

## 1. Kei の String がどの単位まで保証するか(境界の明文化)

Kei の `String` は不変の **UTF-16 シーケンス**で、TS の `string` にそのまま写る
(v0.1 §2.6)。文字を数える・区切る操作には次の **3 つの単位**が絡む。Kei が保証するのは
**code point 単位まで**で、それより上(grapheme・正規化)は**言語の外(extern)**に置く。

| 単位 | 例 `😀`(U+1F600) | 例 `🇯🇵`(日本国旗) | 例 `é`(e + U+0301) | Kei の扱い |
|---|---|---|---|---|
| UTF-16 code unit | 2 | 4 | 2 | `s.length`(v0.1 §2.6) |
| Unicode code point | 1 | 2 | 2 | `s.codePointCount()` / `s.split("")`(**v0.10 で保証**) |
| grapheme(書記素クラスタ) | 1 | 1 | 1 | **言語外**(extern / `Intl.Segmenter`) |

- **code point まで保証する**とは: `codePointCount()` は code point を 1 と数え、`split("")` は
  code point 境界で分割する(サロゲートペアを割らない)。絵文字 1 個は 1 code point として扱える。
- **grapheme は保証しない**: `🇯🇵`(地域指示子 2 つ)や ZWJ 連結(`👨‍👩‍👧`)は「人間が 1 文字と感じる」
  が **複数 code point**であり、Kei の `codePointCount()` はそれぞれ 2・複数を返す。「見た目 1 文字」
  = grapheme 単位の数え上げ・分割は Kei の純ロジックでは扱わない(下記 §3)。
- **正規化(NFC/NFD)も保証しない**: `é`(合成済み U+00E9)と `e` + U+0301(結合文字)は
  見た目が同じでも別の code point 列で、`codePointCount()` はそれぞれ 1・2 を返す。正規化して
  比較・数え上げしたい場合は extern(下記 §3)。

「ここまでが Kei の純ロジック、ここからは境界の外」を曖昧にしないのが本章の目的
(HANDOFF: 合意書原則 / 設計原則「String 意味論の変更は境界の明文化とセット」)。

## 2. `codePointCount()` と `split("")` — code point 単位の 2 経路

### 2.1 `s.codePointCount() -> Int`

Unicode code point 数を返す純粋メソッド(引数 0)。

```text
"a😀b".codePointCount()   // 3   ("a" + "😀" + "b")
"a😀b".length             // 4   ("a" + サロゲート 2 + "b")
"🇯🇵".codePointCount()      // 2   (地域指示子 2 つ = 2 code point。grapheme では 1)
```

- **加法変更**: `length`(UTF-16)は意味を変えない。文字数上限バリデーション等で「人間の感覚どおり」に
  数えたいときは `codePointCount()` を使う、を推奨とする(#159 🤝(a))。
- emit: `keiStringCodePointCount(s)`(runtime helper。内部は `Array.from(s).length`)。native の
  `s.length` に落とすとサロゲートで壊れるため helper 経由。
- 契約: 純粋なので `requires` / `ensures` 内で使える。`kei check --generative` の bounded 評価器は
  `s.chars().count()` で評価でき、`length`(UTF-16)との差はサロゲート境界値
  (`str_domain` の `😀` / `🇯🇵`)で PBT に現れる。例: `ensures result <= s.length` は generative で充足を示せる。

### 2.2 `s.split("")` — 「1 文字ずつ」の正規経路

code point イテレーションに**新しい構文(for 文等)は足さない**(v0.10 設計原則 3)。
1 文字ずつ処理する経路は既存の `s.split("")`(空デリミタ)+ `List` 畳み込みで表現する。

```text
"a😀b".split("")                          // ["a", "😀", "b"]  (code point 単位)
"a😀b".split("").fold(0, (acc, c) => acc + 1)   // 3
```

- **M44 での是正**: 従来 emit は `s.split("")` を native `String.prototype.split` にそのまま写しており、
  native は空デリミタ split で**サロゲートペアを UTF-16 code unit に割っていた**
  (`"a😀b".split("")` → `["a", "\uD83D", "\uDE00", "b"]`)。これは v0.1 §2.6 の「空デリミタは
  code point ごとに分割」という記述と食い違っていた(実装が spec に追いついていなかった)。M44 で
  emit を runtime helper `keiStringSplit` 経由に変え、**空デリミタのみ `Array.from(s)`(code point 単位)**に
  落として是正した。非空デリミタは従来どおり native `String.prototype.split` に委譲するので実行時の
  振る舞いは不変。
- JS 参照実装との等価: `s.split("")` の結果は `Array.from(s)` と一致する(`examples/strings/codepoints.kei` +
  `tests/e2e/tests/strings.test.ts` の等価テストで固定)。

## 3. 境界の外 — grapheme と正規化は extern(TS 側)へ誘導

grapheme segmentation(書記素クラスタ・ZWJ 連結・結合文字)と Unicode 正規化(NFC/NFD)は
**言語内で実装しない**(v0.10 設計原則 1)。契約検証(`requires`/`ensures`)・pbt・決定性の観点で
Unicode テーブル依存は重く、Kei の「契約式は同期・純粋・静的に扱える」路線と相性が悪いため。
どうしても必要なパターンは、境界を明示した上で **extern で TS 側の標準機構に出す**。

```text
// grapheme 単位で数える必要があるとき(例: 見た目 1 文字としての 🇯🇵 を 1 と数えたい)は
// extern で TS の Intl.Segmenter に委譲する。数え上げ自体は Kei の純ロジックに置かない。
extern Unicode.graphemeCount(s: String) -> Int
// TS 側実装(アダプタ):
//   const seg = new Intl.Segmenter(undefined, { granularity: "grapheme" });
//   return () => [...seg.segment(s)].length;

// 正規化して比較したいときも extern へ:
extern Unicode.normalizeNfc(s: String) -> String
//   return () => s.normalize("NFC");
```

- extern の戻り型・エフェクトは v0.2 §「extern 署名」の enforcement-when-declared に従う。
- ロケール依存の数え上げ・照合も同様に extern(言語内では持たない)。

## 4. スコープ(M44)

- **含む**: `codePointCount()` の追加、`split("")` の code point 単位への是正、
  grapheme/正規化の境界明文化(本章 §3)。
- **含まない(後続)**: `substring` / `replace` / 大小文字 / `trim` / 前後方一致 等の stdlib 段階2(M45)、
  代表 4 関数の等価テスト実証(M46)、並行 async(M47)。
- **含まない(v1.x 以降)**: grapheme segmentation・正規化の言語内実装(恒久的に extern 誘導)。
