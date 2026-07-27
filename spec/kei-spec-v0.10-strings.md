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
- **M45 実装完了(v0.10)。** String stdlib 段階2(`substring` / `replace` / `replaceAll` /
  `toLowerCase` / `toUpperCase` / `trim` / `startsWith` / `endsWith` / `contains`)を追加(§2.3)。
  `substring` は範囲を **code point 単位**で規定(runtime helper `keiStringSubstring`)、`replaceAll` は
  空 `from` を code point 境界挿入に是正(runtime helper `keiStringReplaceAll`)、他は TS 標準
  String に素直に写る。あわせて**正規表現の態度**を明文化(§5。言語に入れず定石例示 + extern 境界)。
- M46(純ロジック等価テスト実証)・M47(並行 async)は後続。

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
- emit: `keiStringCodePointCount(s)`(runtime helper)。**意味論は `Array.from(s).length` と同一**
  (string iterator = code point 単位の反復)。実装は中間配列を確保しないカウントループ
  (`for...of` で code point を数える。等価性は e2e の JS 参照 `Array.from(s).length` との
  一致テストで固定)。native の `s.length` に落とすとサロゲートで壊れるため helper 経由。
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

### 2.3 String stdlib 段階2(M45 / #160)

実アプリの「面白い純ロジック」(Markdown 除去・slug 生成・タグ正規化・MIME/キーのバリデーション)を
Kei で書くための API 拡充。範囲は #160 🤝(b) 合意の **high tier + `contains`**(`repeat` / `padStart` /
`padEnd` は v1.x 送り)。emit は原則 TS 標準 String に素直に落とし、**code point 単位を規定した
`substring` のみ runtime helper 経由**にする(設計原則 2)。

| メソッド | シグネチャ | 意味論 | emit |
|---|---|---|---|
| `substring` | `(start: Int, end: Int) -> String` | **code point 単位**の半開区間(下記) | `keiStringSubstring`(helper) |
| `replace` | `(from: String, to: String) -> String` | 最初の 1 箇所を置換 | `String.prototype.replace` |
| `replaceAll` | `(from: String, to: String) -> String` | 全箇所を置換(空 `from` は code point 境界挿入・下記) | `keiStringReplaceAll`(helper) |
| `toLowerCase` | `() -> String` | ロケール非依存の小文字化 | `String.prototype.toLowerCase` |
| `toUpperCase` | `() -> String` | ロケール非依存の大文字化 | `String.prototype.toUpperCase` |
| `trim` | `() -> String` | 前後空白の除去 | `String.prototype.trim` |
| `startsWith` | `(prefix: String) -> Bool` | 前方一致 | `String.prototype.startsWith` |
| `endsWith` | `(suffix: String) -> Bool` | 後方一致 | `String.prototype.endsWith` |
| `contains` | `(sub: String) -> Bool` | 部分文字列包含(`indexOf != None` の可読化) | `String.prototype.includes` |

#### `substring` の添字は code point 単位

`substring(start, end)` の `start` / `end` は **UTF-16 code unit index ではなく code point index**
とする(M44 で規定した code point 意味論と整合させる。設計原則 2)。

```text
"a😀b".substring(1, 2)   // "😀"   (code point index 1 の 1 個 = 絵文字まるごと)
"a😀b".substring(0, 2)   // "a😀"
"😀😀😀".substring(1, 3)  // "😀😀"
```

- **範囲の意味論**: code point 列(= `Array.from(s)`)を JS `Array.prototype.slice(start, end)` の index
  意味論で切る。すなわち **負の index は末尾から**(`slice(-2, ...)`)、**範囲外は端にクランプ**、
  **resolve 後の `start >= end` は空文字 `""`** を返す。emit は
  `keiStringSubstring(s, start, end)` = `Array.from(s).slice(start, end).join("")`。
- **なぜ UTF-16(JS の `String.prototype.substring`)にしないか**: native の `substring` は UTF-16
  code unit index で、絵文字を含むと `"a😀b".substring(1, 2)` が `"\uD83D"`(サロゲート片)になり、
  M44 で保証した「code point 単位まで壊さない」路線と食い違う。`length` は互換のため UTF-16 のまま
  温存する(#159 🤝(a))が、**新規に足す範囲 API は code point 単位に揃える**ことで意味論の一貫性を取る。
- **UTF-16 index が必要なとき**: `length` と同じ UTF-16 単位で切りたい特殊ケースは、境界の外として
  extern で TS の `s.substring(...)` / `s.slice(...)` に出す(grapheme / 正規化と同じ extern 誘導)。
- **1 文字ずつの経路との関係**: 多くの用途は `substring` より `s.split("")`(§2.2)+ `List` 畳み込みで
  書ける。`substring` は「前から n code point」「後ろを削る」のような範囲取り出しの糖衣として使う。

#### 大小文字はロケール非依存

`toLowerCase` / `toUpperCase` は TS の `String.prototype.toLowerCase` / `toUpperCase`(**ロケール
非依存**の Unicode デフォルト case マッピング)に写す。`toLocaleLowerCase` / `toLocaleUpperCase` は
使わない。ロケール依存の畳み込み(トルコ語の `i` ↔ `İ`・ギリシャ語末尾シグマ等)は **v0.10 スコープ外**で、
必要なら extern(§3 と同じ発想)。純ロジックを決定的・ロケール非依存に保つための選択。

#### `replace` / `replaceAll` の置換パターン注記と空 `from` の意味論

第 1 引数は文字列(部分文字列一致)で、`replace` は最初の 1 箇所・`replaceAll` は全箇所を置換する。
置換文字列(第 2 引数)は JS の置換パターン(`$&` = マッチ全体、`$$` = 字面の `$` 等)を解釈する
ことに注意する。`$` を字面で入れたいときは `$$` と書く(非空 `from` の場合)。

**空 `from`(`""`)の `replaceAll` は code point 境界ごとの挿入**とする(M45 深層レビュー反映):

```text
"a😀b".replaceAll("", "_")   // "_a_😀_b_"   (先頭・各 code point の間・末尾に挿入)
"".replaceAll("", "_")        // "_"          (native の "".replaceAll("", "_") と同じ形)
```

- native の `s.replaceAll("", to)` は **UTF-16 code unit 境界**に挿入するため、
  `"😀".replaceAll("", "_")` が `"_\uD83D_\uDE00_"` と**孤立サロゲートを作ってしまい**、§1 の
  「code point 単位まで保証」に反する。emit は `keiStringReplaceAll` ヘルパー経由とし、空 `from` のみ
  code point 境界(`Array.from`)への挿入に是正する。非空 `from` はヘルパー内で native
  `String.prototype.replaceAll` に委譲するので振る舞いは従来どおり。
- 空 `from` のとき `to` は**字面どおり**挿入する(マッチが存在しないため `$` パターンは解釈しない。
  native は空マッチにも `$$` 等を適用するが、Kei は挿入の意味論として字面挿入を規定する)。
- `replace("", to)` は**位置 0 に 1 回挿入するだけ**(`"😀".replace("", "_") == "_😀"`)で
  サロゲートを割れないため、native `String.prototype.replace` のままでよい(helper 不要)。

#### 契約と generative

すべて純粋で `requires` / `ensures` 内で使える。`kei check --generative` の bounded 評価器は、
JS と Rust で結果が一致することを保証できる **`substring` / `startsWith` / `endsWith` / `contains` を評価**
する(`ensures result == s.startsWith(p)` 等は `[generative]` へ昇格)。`replace` / `replaceAll` /
`toLowerCase` / `toUpperCase` / `trim` は case マッピング・空白集合・空パターン置換の JS/Rust 差を
避けるため評価対象外(`[runtime]` のまま。誤った counterexample を出さないための保守)。

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

## 4. スコープ(M44 / M45)

- **含む(M44)**: `codePointCount()` の追加、`split("")` の code point 単位への是正、
  grapheme/正規化の境界明文化(本章 §3)。
- **含む(M45)**: String stdlib 段階2(`substring`〔code point 単位〕/ `replace` / `replaceAll` /
  `toLowerCase` / `toUpperCase` / `trim` / `startsWith` / `endsWith` / `contains`。§2.3)、
  正規表現の態度の明文化(§5)。
- **含まない(後続)**: 代表 4 関数の等価テスト実証(M46)、並行 async(M47)。
- **含まない(v1.x 以降)**: grapheme segmentation・正規化の言語内実装(恒久的に extern 誘導)、
  String stdlib medium tier(`repeat` / `padStart` / `padEnd`。#160 🤝(b) で v1.x 送り確定)、
  正規表現エンジンの言語内実装(§5。#160 🤝(c) で不採用確定)。

## 5. 正規表現の態度 — 言語に入れない(定石例示 + extern 境界)

v0.10 は **正規表現エンジンを言語に入れない**(#160 🤝(c) 合意済み)。正規表現は契約検証
(`requires` / `ensures`)・pbt・決定性の観点で重く、Kei の「契約式は同期・純粋・静的に扱える」路線と
相性が悪いため(grapheme = extern と同じ「境界を曖昧にしない」思想)。代わりに、

1. **String プリミティブ + code point イテレーションで「正規表現を使わずに書く定石」を用意する**、
2. どうしても正規表現が要るパターンは **extern で TS 側の `RegExp` に出す境界を明記する**。

### 5.1 定石 — 正規表現を使わずに書く

「小文字化 → `split("")` で 1 code point ずつ → 許可集合で畳み込み(除去 / 置換 / 判定)」が基本形。
slug 生成やタグ正規化はこの形で書ける(代表 4 関数の本格版は M46 で `examples/` に置く)。

```text
// 許可文字(小文字英数)だけ残し、それ以外はハイフンにする 1 文字の写像。
func slugChar(c: String) -> String {
  return match isAllowed(c) {  // isAllowed は c.codePointCount()==1 前提の許可判定(別途定義)
    true => c
    false => "-"
  }
}

// 小文字化 → 1 code point ずつ写像 → 連結。連続ハイフンの畳み込みや前後トリムも
// split("") + fold と substring / trim(§2.3)で書ける(正規表現不要)。
func slugify(s: String) -> String {
  return s.toLowerCase().split("").fold("", (acc, c) => acc + slugChar(c))
}

// 前方 / 後方一致・部分文字列は startsWith / endsWith / contains で直接書ける。
func isImageKey(key: String) -> Bool {
  return key.endsWith(".png") == key.endsWith(".jpg")  // 例。実際は || 相当を match で
}
```

- 文字クラス判定(`[a-z0-9]` 等)は、code point の範囲比較や許可文字列への `contains` で表現する。
- 繰り返し・選択・後方参照など正規表現特有の機能は、`split("")` + 畳み込みの状態機械として書くか、
  下記 extern に出す。

### 5.2 境界 — 正規表現が要るときは extern で TS の `RegExp` に出す

複雑な正規表現(可変長の先読み・込み入った置換)を Kei の定石で書くのが実務的な読みやすさを損なう
場合は、境界を明示して extern で TS 側に出す。数え上げ・変換のロジック自体は Kei の純ロジックに置かない。

```text
// TS 側で RegExp を使う正規化を extern 署名として宣言する(実装はアダプタの TS)。
extern Slug.normalize(s: String) -> String
// TS 側実装(アダプタ):
//   const re = /[^a-z0-9]+/g;
//   return () => s.toLowerCase().replace(re, "-").replace(/^-+|-+$/g, "");
```

- extern の戻り型・エフェクトは v0.2 §「extern 署名」の enforcement-when-declared に従う。
- **運用の指針**: まず §5.1 の定石で書けないかを検討し、書けるならそちらを主経路にする(純ロジックが
  Kei に寄る)。定石が実務的な読みやすさを保てないパターンに限って extern を主経路にする(#160 🤝(c) の運用注記)。
