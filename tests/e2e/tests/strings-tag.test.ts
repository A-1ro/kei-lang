// examples/strings/tag.kei の等価テスト(M46 / #160)。
// Kei が生成した normalizeTag(トリム + 小文字化 + 内部空白ランの単一スペース化)が、
// 独立に書いた JS 参照実装(正規表現ベース)と、固定代表ケース + 決定論的ランダム入力
// (絵文字・記号・空文字・長文・タブ/改行を含む)で完全一致することを CI で常設検証する。

import { describe, expect, it } from "vitest";

import { normalizeTag } from "../generated/strings/tag";

// JS 参照実装。空白は Kei 側と同じ 4 種(半角スペース・タブ・改行・復帰)に限定する。
function refNormalizeTag(s: string): string {
  return s
    .trim()
    .toLowerCase()
    .replace(/[ \t\n\r]+/g, " ");
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
  "a", "B", "z", "0", "9", " ", "  ", "\t", "\n", "\r", "-", ".",
  "あ", "😀", "🎉", "🇯🇵", "é", "文", "!", "#",
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
  "  ",
  "tag",
  "  Hello World  ",
  "MULTIPLE   SPACES",
  "\t\n mixed \r\n whitespace \t",
  "CamelCase Tag",
  "日本語 タグ",
  "emoji 😀 tag",
  "with-hyphen and.dot",
  "a".repeat(400) + "   " + "B".repeat(400),
];
const random = randomStrings(0x7a91cd23, 400, 24);
const all = [...fixed, ...random];

describe("strings/tag — タグ正規化の JS 参照等価(M46)", () => {
  it("代表ケースの固定出力(人間が手計算した期待値)", () => {
    expect(normalizeTag("  Hello World  ")).toBe("hello world");
    expect(normalizeTag("MULTIPLE   SPACES")).toBe("multiple spaces");
    expect(normalizeTag("\t\n mixed \r\n whitespace \t")).toBe("mixed whitespace");
    expect(normalizeTag("CamelCase Tag")).toBe("camelcase tag");
    expect(normalizeTag("日本語 タグ")).toBe("日本語 タグ");
    expect(normalizeTag("emoji 😀 tag")).toBe("emoji 😀 tag");
    expect(normalizeTag("")).toBe("");
    expect(normalizeTag("   ")).toBe("");
  });

  it("固定 + ランダム入力すべてで JS 参照実装と一致する", () => {
    for (const s of all) {
      expect(normalizeTag(s)).toBe(refNormalizeTag(s));
    }
  });

  // #171: toLowerCase が code point 数を増やす入力(例: İ(U+0130) → "i" + U+0307 の
  // 2 code point)でも ensures(result.codePointCount() <= s.codePointCount() * 2)を
  // 破らずに完走し、JS 参照実装と一致することを確認する回帰テスト。
  it("小文字化で code point が増える入力(İ)でも契約違反にならず参照実装と一致する", () => {
    const s = "İ";
    expect(() => normalizeTag(s)).not.toThrow();
    expect(normalizeTag(s)).toBe(refNormalizeTag(s));
    expect([...normalizeTag(s)].length).toBeGreaterThan([...s].length);
  });
});
