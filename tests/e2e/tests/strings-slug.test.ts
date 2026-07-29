// examples/strings/slug.kei の等価テスト(M46 / #160)。
// Kei が生成した slugify(小文字化 → code point ごとの許可写像 → 連続ハイフン畳み込み → 前後トリム)が、
// 独立に書いた JS 参照実装(正規表現ベース)と、固定代表ケース + 決定論的ランダム入力
// (絵文字・記号・空文字・長文を含む)で完全一致することを CI で常設検証する。

import { describe, expect, it } from "vitest";

import { slugify } from "../generated/strings/slug";

// JS 参照実装。Kei は正規表現を使わないが、参照側は独立な戦略(正規表現)で書く。
// 1. 小文字化 2. 許可外([^a-z0-9])のランを単一ハイフンに 3. 前後ハイフンを除去。
function refSlugify(s: string): string {
  return s
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

// 決定論的な擬似乱数(seed 固定)。
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
  "a", "b", "Z", "0", "9", " ", "  ", "-", "--", ".", "_", "/",
  "あ", "😀", "🎉", "🇯🇵", "é", "文", "!", "@", "#",
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
  "a",
  "Hello World",
  "  Trim  Me  ",
  "already-a-slug",
  "Multiple   Spaces",
  "dots.and_underscores",
  "--leading--and--trailing--",
  "café au lait",
  "日本語 タイトル",
  "emoji 😀 in 🎉 title",
  "🇯🇵🇯🇵🇯🇵",
  "###heading###",
  "MixedCASE-123",
  "!!!",
  "a".repeat(500) + " " + "B".repeat(500),
];
const randomInputs = randomStrings(0x5a17ce0f, 400, 24);
const all = [...fixed, ...randomInputs];

describe("strings/slug — slug 生成の JS 参照等価(M46)", () => {
  it("代表ケースの固定出力(人間が手計算した期待値)", () => {
    expect(slugify("Hello World")).toBe("hello-world");
    expect(slugify("  Trim  Me  ")).toBe("trim-me");
    expect(slugify("--leading--and--trailing--")).toBe("leading-and-trailing");
    expect(slugify("café au lait")).toBe("caf-au-lait");
    expect(slugify("emoji 😀 in 🎉 title")).toBe("emoji-in-title");
    expect(slugify("###heading###")).toBe("heading");
    expect(slugify("MixedCASE-123")).toBe("mixedcase-123");
    expect(slugify("!!!")).toBe("");
    expect(slugify("")).toBe("");
    // 絵文字(サロゲートペア)を割らずに単一ハイフンとして畳み込む(code point 単位の受け入れ)。
    expect(slugify("🇯🇵🇯🇵🇯🇵")).toBe("");
    expect(slugify("a😀b")).toBe("a-b");
  });

  it("固定 + ランダム入力すべてで JS 参照実装と一致する", () => {
    for (const s of all) {
      expect(slugify(s)).toBe(refSlugify(s));
    }
  });
});
