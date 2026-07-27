// examples/strings/codepoints.kei の実行テスト(M44 / #159)。
// Kei が生成した code point 意味論(codePointCount / split("") イテレーション)が、
// JS 参照実装 Array.from(s)(= code point 単位の反復)と等価に振る舞うことを固定する。

import { describe, expect, it } from "vitest";

import {
  codePointCount,
  codePoints,
  countByFold,
  isWithinLimit,
} from "../generated/strings/codepoints";

// JS 参照実装: Array.from は文字列を code point 単位で反復する(サロゲートを割らない)。
function refCodePoints(s: string): string[] {
  return Array.from(s);
}
function refCodePointCount(s: string): number {
  return Array.from(s).length;
}

// 代表ケース: ASCII / 絵文字(サロゲートペア)/ 旗(地域指示子 2 つ)/ 合成 vs 結合文字 / 空 / 長文。
const fixed: string[] = [
  "",
  "a",
  "abc",
  "a😀b",
  "😀😀😀",
  "🇯🇵",
  "café", // 合成済み é (U+00E9)
  "café", // e + 結合アクセント (U+0301) — code point では 5
  "日本語テスト",
  "mix 混在 😀 text 🎉!",
  "x".repeat(200) + "😀",
];

// 決定論的な擬似乱数(seed 固定)。code point をランダムに選んで文字列を組む。
function makeRng(seed: number): () => number {
  let state = seed >>> 0;
  return () => {
    // xorshift32
    state ^= state << 13;
    state ^= state >>> 17;
    state ^= state << 5;
    state >>>= 0;
    return state / 0xffffffff;
  };
}

const pool = [
  "a",
  "Z",
  "0",
  " ",
  "あ",
  "😀",
  "🎉",
  "🇯🇵",
  "é",
  "é",
  "👍",
  "文",
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

const random = randomStrings(0x5eed1234, 200, 12);
const all = [...fixed, ...random];

describe("strings/codepoints — code point 意味論(M44)", () => {
  it("codePoints(s)(= split(\"\"))は Array.from(s) と一致する", () => {
    for (const s of all) {
      expect(codePoints(s)).toEqual(refCodePoints(s));
    }
  });

  it("codePointCount(s) は Array.from(s).length と一致する", () => {
    for (const s of all) {
      expect(codePointCount(s)).toBe(refCodePointCount(s));
    }
  });

  it("countByFold(s)(split(\"\") + fold)も code point 数と一致する", () => {
    for (const s of all) {
      expect(countByFold(s)).toBe(refCodePointCount(s));
    }
  });

  it("絵文字は length では 2 だが code point では 1(人間の感覚どおり)", () => {
    expect("😀".length).toBe(2); // UTF-16 code unit
    expect(codePointCount("😀")).toBe(1); // code point
    expect(codePointCount("a😀b")).toBe(3);
    expect("a😀b".length).toBe(4);
  });

  it("split(\"\") はサロゲートペアを割らない(native s.split(\"\") とは異なる)", () => {
    expect(codePoints("a😀b")).toEqual(["a", "😀", "b"]);
    // 参考: native の s.split("") は ["a", "\uD83D", "\uDE00", "b"] になってしまう。
    expect("a😀b".split("")).toEqual(["a", "\uD83D", "\uDE00", "b"]);
  });

  it("isWithinLimit は code point 数で判定する", () => {
    // "😀😀😀" は length=6 だが code point 数は 3。
    expect(isWithinLimit("😀😀😀", 3)).toBe(true);
    expect(isWithinLimit("😀😀😀", 2)).toBe(false);
  });

  // 境界値: 孤立サロゲートと ZWJ 列。PBT(kei check --generative)は Rust String =
  // Unicode scalar value のため孤立サロゲートを生成できないので、この境界は
  // runtime e2e 層で固定する。ZWJ 列は「1 grapheme = 複数 code point」の代表。
  it("孤立サロゲート・ZWJ 列も Array.from 意味論と一致する", () => {
    const boundary: string[] = [
      "\uD800", // lone high surrogate
      "\uDC00", // lone low surrogate
      "a\uD800b", // 文中の孤立サロゲート
      "\uD800\uD800", // 連続する孤立 high surrogate
      "👨‍👩‍👧‍👦", // family emoji(ZWJ 列。7 code point / 1 grapheme)
      "👍‍👍", // ZWJ を挟んだ 2 絵文字
    ];
    for (const s of boundary) {
      expect(codePoints(s)).toEqual(refCodePoints(s));
      expect(codePointCount(s)).toBe(refCodePointCount(s));
    }
    // ZWJ 列は grapheme では 1 でも code point では 7(絵文字 4 + ZWJ 3)。
    expect(codePointCount("👨‍👩‍👧‍👦")).toBe(7);
  });
});
