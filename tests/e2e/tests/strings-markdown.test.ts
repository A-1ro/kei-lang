// examples/strings/markdown.kei の等価テスト(M46 / #160)。
// Kei が生成した stripMarkdown(装飾文字の除去 + リンク/画像 `(...)` の除去の状態機械)が、
// 独立に書いた JS 参照実装と、固定代表ケース + 決定論的ランダム入力(絵文字・記号・空文字・長文を含む)で
// 完全一致することを CI で常設検証する。参照側は code point を反復する素朴な命令型で、Kei の
// record アキュムレータ fold が同じ意味論(サロゲートを割らない)を実現していることを確かめる。

import { describe, expect, it } from "vitest";

import { stripMarkdown } from "../generated/strings/markdown";

// JS 参照実装。code point 単位(for-of は code point 反復)で状態機械を回す。
function refStripMarkdown(s: string): string {
  const marks = new Set(["#", "*", "_", "`", "[", "]"]);
  let out = "";
  let afterClose = false;
  let inUrl = false;
  for (const c of s) {
    if (inUrl) {
      if (c === ")") inUrl = false;
      afterClose = false;
      continue;
    }
    if (afterClose && c === "(") {
      afterClose = false;
      inUrl = true;
      continue;
    }
    if (marks.has(c)) {
      afterClose = c === "]";
      continue;
    }
    out += c;
    afterClose = false;
  }
  return out;
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
