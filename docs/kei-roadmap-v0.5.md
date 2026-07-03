# Kei 開発ロードマップ v0.5 — /goal 契約書集(+ v1.0 到達戦略)

> 運用ルール: 各 Milestone は「人間が合意する契約」。
> 完了条件は必ず機械検証可能な形(テスト・コマンド出力)で書く。
> 本ファイルは v0.5 ラベル/候補の open issue(#85〜#101)と v0.4 ドッグフード総括
> (`docs/dogfood/2026-06-30-v0.4-summary.md`)を、**v1.0 ゴールから逆算**して並べ直したもの。

## v1.0 ゴールと逆算戦略

**v1.0 の受け入れ基準: 「Cloudflare Workers に Hono を使用した API としてデプロイできるトランスパイル言語であること」。**

具体的には、`examples/workers-api/`(または独立サンプルリポジトリ)に置いた `.kei` ソース群から
`kei build` で生成した TS が、Hono ルーティング + Kei 製ハンドラロジックとして
`wrangler deploy` に通り、契約(requires/ensures)が実行時に生きたまま HTTP API として動くこと。

v0.4.2 時点のギャップ分析(2026-07-03、コミット c8db13d 起点)による欠落は3層:

1. **blocker 層(v0.6〜v0.8 で解消)** — 言語コアの欠落
   - async / Promise / `uses Async` が完全に無い(全関数が同期出力。Hono ハンドラ・KV/D1 は全て async)
   - 外部 npm パッケージ(`hono` 等)を import する手段が無い(import は相対パス限定、特別扱いは `@kei/runtime` のみ)
   - Request/Response・HTTP 境界の型/バインディングが無い
2. **stdlib 層(v0.5 で解消)** — API ロジックを書くための表現力
   - 文字列操作・数値⇔文字列変換がゼロ(連結すら不可)
   - Map / 動的キーアクセスが無い(ヘッダ・クエリ・env を表現できない)
   - `&&` 不在・`List.contains` 不在・ラムダキャプチャ禁止・record 差分更新なし(ドッグフードで頻出した躓き)
3. **検証経路層(v0.5 で解消)** — エージェント開発体験
   - MCP `kei_check` に generative モードが無い(#89)、import opaque が不可視(#90)

バージョン運用: ロードマップ v0.5 の Milestone が全て閉じた時点で **0.5.0** をタグする。
途中イテレーションの機能マージは 0.4.x のパッチ/プレリリースとして bump する。

### v0.5 以降の全体ロードマップ(逆算)

| 版 | テーマ | 内容(概要) |
|---|---|---|
| **v0.5** | API ロジックを書ける表現力 | 本ファイルの M28〜M34(下表) |
| **v0.6** | 外部世界への import | extern の npm パッケージ束縛(`extern from "hono"` 相当)、emit の bare-specifier import、strict-extern との整合 |
| **v0.7** | async | `uses Async` エフェクト、async extern 署名、`async function` + `await` の emit、契約(ensures)の async 対応 |
| **v0.8** | HTTP/JSON 境界 | unknown → record の安全な構築(JSON parse + 契約検証)、`@kei/runtime` に HTTP ヘルパ、Hono アダプタ(`@kei/hono` 薄層) |
| **v0.9** | Workers テンプレート & デプロイ | `examples/workers-api` 雛形、wrangler 統合 e2e(`wrangler dev` 起動 → HTTP 叩き → 契約違反が 500 で観測できる) |
| **v1.0** | 受け入れ検証 | 実 API を Kei で記述して Workers にデプロイ、ドッグフードで初見エージェントが API を書けることを実証 |

v0.6〜v0.8 は言語設計判断(🤝)を多く含むため、各版の着手時に本ファイルと同形式の契約書集を新設する。

## v0.5 Milestone 全体像と順序

| M | テーマ | issue | 優先度 | 状態 | 主な改修クレート |
|---|---|---|---|---|---|
| **M28** | 論理積 `&&` | #91 | high | ✅ 実装済み | kei_syntax / kei_check / kei_fmt / kei_emit |
| **M29** | `List.contains(item)` 組み込み | #92 | high | ⬜ 未着手 | kei_check / kei_emit / (pbt) |
| **M30** | 文字列 stdlib 段階1(連結・length・Int⇔String 変換) | (新規) | high | ⬜ 未着手 | kei_syntax / kei_check / kei_emit / runtime |
| **M31** | ラムダキャプチャ緩和(読み取り専用キャプチャ) | (新規, dogfood critical) | high 🤝 | ⬜ 未着手 | kei_syntax / kei_check / kei_emit |
| **M32** | record 差分更新構文(spread) | #97 | medium 🤝 | ⬜ 未着手 | kei_syntax / kei_check / kei_emit / kei_fmt |
| **M33** | `Map<K, V>` 段階1 | #95 | medium 🤝 | ⬜ 未着手 | kei_syntax / kei_check / kei_emit / runtime |
| **M34** | MCP 検証経路強化(generative + opaque 可視化) | #89 / #90 | high | ⬜ 未着手 | kei_mcp |

順序の論拠:

- **M28/M29/M30 が先頭。** いずれも既存機構(二項演算子・List 組み込みメソッド・プリミティブ型)の自然な拡張で
  設計合意事項が少なく、かつドッグフード/実 API の両方で最頻出の欠落。M30 の文字列連結は
  v0.8 以降のレスポンス生成・パス組み立ての前提になる。
- **M31/M32/M33 は言語設計判断を含む(🤝)。** キャプチャ緩和は「純粋・単一式」の制約を保ったまま
  読み取り専用キャプチャだけ許すか、spread は評価順と契約式内での可否、Map はキー型の制約
  (`String` / `Int` 限定か)を先に合意してから着手する。
- **M34 は実装よりも配線。** kei_check 側の generative 機構(M15/M23)は実装済みで、MCP ツールに
  露出させるだけの辺。エージェント開発体験に直結するため v0.5 内で閉じる。
- #98(fix 候補の誤誘導)・#100(TS 予約語チェック拡張)は M28〜M30 の作業でパーサ/チェッカに
  触れる際に同梱できれば拾う。#85(lambda walker 共通化)は M31 の前提リファクタとして扱う。

## M28: 論理積 `&&`(#91)

### 完了条件

- `a >= 0 && a < max` が本体・`requires`・`ensures`・`match` ガードで書け、`kei check` を通る。
- 優先順位は `||` より強く、`implies` より強い(`a || b && c` は `a || (b && c)`)。spec の演算子表に明記。
- TS 出力は `&&` にそのまま写る。短絡評価であることを spec に明記(契約式は副作用禁止なので観測不能だが、0 除算 trap の回避に意味がある)。
- syntax / check / fmt / emit の golden で固定し、`cargo test --workspace` が通る。
- v0.4 M21 の「`&&` はスコープ外」注記を更新する(spec-first: 実装前に spec を更新)。

### スコープ外

- ビット演算。必要性が出た時点で別 issue。

## M29: `List.contains(item)`(#92)

### 完了条件

- `xs.contains(x)` が `List<T>` の `T` が等値比較可能な型(Int / String / Bool / tagged Int/String 基底)のとき `Bool` を返す。
- record / enum 要素の List への `contains` は KEI-E2xxx(構造等価未サポート #93 を参照する診断)で拒否し、fix 候補に `any(e => ...)` を提示する。
- 契約式(`requires` / `ensures`)内でも使える。generative 検証(pbt.rs)の eval にも同時に追従させる(AST/メソッド追加時は pbt.rs の網羅性確認 — PR #83 の教訓)。
- TS 出力は `includes` ではなく Kei の等値定義に沿った実装(`Array.prototype.includes` は SameValueZero なので Int/String/Bool では一致。そのまま `includes` に写してよい根拠を spec に書く)。
- golden + e2e で固定し、`cargo test --workspace` が通る。

### スコープ外

- 構造等価(#93)。record の `contains` はこの Milestone では診断で誘導するだけ。

## M30: 文字列 stdlib 段階1(新規 issue を切る)

### 完了条件

- 文字列連結: `String + String -> String` を型検査で許す(`Int + String` は KEI-E2001 のまま)。TS 出力は `+`。
- `s.length -> Int`(UTF-16 code unit 長。V8 の `String.prototype.length` と同一と spec に明記)。
- 変換: `Int.toString(n) -> String` と `String.toInt(s) -> Option<Int>` を組み込み関数(または メソッド)として追加。
  失敗し得る変換が `Option` を返すことで null 安全の設計原則を保つ。
- 契約式内でも使える(純粋)。pbt.rs の eval も追従。
- spec に「String 組み込み一覧」節を新設し、golden で固定。`cargo test --workspace` が通る。

### スコープ外

- slice / indexOf / split / trim 等の本格 String API(v0.8 の HTTP 境界設計と合わせて段階2で)。
- テンプレート文字列 / 補間。

## M31: ラムダキャプチャ緩和(🤝 設計合意が必要)

### 完了条件(合意のたたき台)

- コンビネータ引数位置のラムダが、囲みスコープの **let 束縛・関数パラメータを読み取り専用で参照**できる。
  例: `items.filter(i => i.sku == target)`(v0.4 ドッグフードで critical 判定の躓き)。
- 純粋性・単一式・引数位置限定の制約は維持する。キャプチャした変数への「代入」は言語に存在しないので健全。
- `old(...)` とラムダの相互作用は PR #83 の教訓(KEI-E4002)を維持: `old(ラムダ内式)` でラムダパラメータ参照は引き続き禁止。
  キャプチャ変数の `old(...)` 内参照の扱いを spec で先に決めてから実装する(spec-first)。
- 着手前に #85(walker 共通化)を前提リファクタとして先行させるか判断する。

## M32: record 差分更新構文 spread(#97, 🤝)

### 完了条件(合意のたたき台)

- `Ticket { ...t, status: Done }` 形式で既存 record から差分構築できる。評価順(spread 先行 → 明示フィールドが上書き)を spec に明記。
- 存在しないフィールドの上書きは KEI-E2002。全フィールドを明示上書きした場合の warning は出さない(正規形は kei_fmt が決める)。
- TS 出力は object spread。契約式内での可否を先に合意する。

## M33: `Map<K, V>` 段階1(#95, 🤝)

### 完了条件(合意のたたき台)

- キー型は `String` / `Int`(+ その tagged)に限定。`m.get(k) -> Option<V>` / `m.set(k, v) -> Map<K,V>`(永続・非破壊)/ `m.has(k) -> Bool` / `m.size -> Int`。
- リテラル構文・fmt 正規形・TS 表現(`ReadonlyMap` か readonly object か)を設計合意してから着手。
- v0.8 の HTTP 境界(ヘッダ・クエリ)がこの型を前提にするため、v0.5 内では最小 API に留める。

## M34: MCP 検証経路強化(#89 / #90)

### 完了条件

- `kei_check` MCP ツールに `generative: true` オプションを追加し、CLI `kei check --generative` と同じ
  反例レポート(JSON)を返す。タイムアウト/ケース数上限を MCP 側で保守的に設定する。
- import が opaque のまま検査された場合、その旨を Diagnostic とは別のメタ情報(`opaque_imports: [...]`)として応答に含める(#90)。
- MCP golden(埋め込み応答のスナップショット)を更新し、`cargo test --workspace` が通る。

## 後続 /goal ドラフト

```text
/goal M28: 論理積 `&&` を追加する。優先順位は `||` より強く、spec の演算子表を先に更新し、
syntax/check/fmt/emit の golden で固定する。cargo fmt --all -- --check、
cargo clippy --workspace --all-targets -- -D warnings、cargo test --workspace を通して結果を表示する。
```

```text
/goal M29: List<T> に contains(item) を追加する。等値比較可能な要素型に限定し、
record/enum 要素は any(...) を fix 候補に持つ診断で拒否する。pbt.rs の eval も追従させ、
golden + e2e で固定する。
```

```text
/goal M30: 文字列 stdlib 段階1(String + String 連結、s.length、Int.toString、String.toInt -> Option<Int>)を
追加する。spec に String 組み込み一覧節を新設してから実装し、pbt.rs も追従、golden で固定する。
```
