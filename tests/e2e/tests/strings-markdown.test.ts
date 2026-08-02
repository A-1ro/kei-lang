// examples/strings/markdown.kei の等価テスト(M46 / #160)。
// Kei が生成した stripMarkdown(装飾文字の除去 + リンク/画像 `(...)` の除去の状態機械)が、
// 独立に書いた JS 参照実装(正規表現ベース)と、固定代表ケース + 決定論的ランダム入力
// (絵文字・記号・空文字・長文を含む)で完全一致することを CI で常設検証する。参照側は
// strings-tag.test.ts の refNormalizeTag と同様に正規表現ベースの別戦略で書き、Kei の
// 状態機械(record アキュムレータ fold)の命令形写しにしない(#170: 独立オラクル化)。

import { describe, expect, it } from "vitest";

import { stripMarkdown } from "../generated/strings/markdown";

// JS 参照実装。2段階の正規表現置換で状態機械と同じ意味論を実現する:
//  1. `](...)` — リンク/画像の閉じ角括弧に続く丸括弧一式(URL部分)を除去。
//     丸括弧が閉じないまま文字列が終わる場合も `[^)]*` が末尾まで貪欲マッチし、
//     `\)?` で閉じ括弧を必須にしないことで状態機械の「閉じるまで消費し続ける」挙動と一致する。
//  2. 単体の装飾文字(見出し・強調・インラインコード・残った角括弧)を除去。
function refStripMarkdown(s: string): string {
  return s.replace(/\]\([^)]*\)?/g, "").replace(/[#*_`[\]]/g, "");
}

function makeRng(seed: number): () => number {
  let state = seed >>> 0;
  return () => {
    state ^= state << 13;
    state ^= state >>> 17;
    state ^= state << 5;
    state >>>= 0;
    return state / 0xffffffff;
  };
}

const pool = [
  "#", "*", "_", "`", "[", "]", "(", ")", " ", "a", "b", "Z", "0",
  "text", "url", "\n", "😀", "🇯🇵", "文", "é", ".", "!", "-",
];

function randomStrings(seed: number, count: number, maxLen: number): string[] {
  const rng = makeRng(seed);
  const out: string[] = [];
  for (let i = 0; i < count; i++) {
    const len = Math.floor(rng() * (maxLen + 1));
    let s = "";
    for (let j = 0; j < len; j++) {
      s += pool[Math.floor(rng() * pool.length)];
    }
    out.push(s);
  }
  return out;
}

const fixed: string[] = [
  "",
  "plain text",
  "#Heading",
  "**bold**",
  "_italic_",
  "`code`",
  "[text](http://example.com)",
  "![alt](img.png)",
  "a (b) c",
  "[x]y",
  "text with 😀 and `code`",
  "## Multi ### markers **and** _emphasis_",
  "unbalanced [ and ] and ( parens",
  "link [one](a) and [two](b) done",
  "🇯🇵 `flag` 🇯🇵",
  "a".repeat(600) + " **bold** " + "b".repeat(600),
];
const random = randomStrings(0x11deac07, 500, 30);
const all = [...fixed, ...random];

describe("strings/markdown — Markdown 除去の JS 参照等価(M46)", () => {
  it("代表ケースの固定出力(人間が手計算した期待値)", () => {
    expect(stripMarkdown("#Heading")).toBe("Heading");
    expect(stripMarkdown("**bold**")).toBe("bold");
    expect(stripMarkdown("_italic_")).toBe("italic");
    expect(stripMarkdown("`code`")).toBe("code");
    expect(stripMarkdown("[text](http://example.com)")).toBe("text");
    expect(stripMarkdown("![alt](img.png)")).toBe("!alt");
    expect(stripMarkdown("a (b) c")).toBe("a (b) c"); // リンク文脈外の括弧は保持
    expect(stripMarkdown("[x]y")).toBe("xy");
    expect(stripMarkdown("text with 😀 and `code`")).toBe("text with 😀 and code");
    expect(stripMarkdown("link [one](a) and [two](b) done")).toBe("link one and two done");
    expect(stripMarkdown("🇯🇵 `flag` 🇯🇵")).toBe("🇯🇵 flag 🇯🇵");
    expect(stripMarkdown("")).toBe("");
  });

  it("固定 + ランダム入力すべてで JS 参照実装と一致する", () => {
    for (const s of all) {
      expect(stripMarkdown(s)).toBe(refStripMarkdown(s));
    }
  });
});
