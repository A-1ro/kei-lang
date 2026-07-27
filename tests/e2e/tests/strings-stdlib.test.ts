// examples/strings/stdlib.kei の実行テスト(M45 / #160)。
// Kei が生成した String stdlib 段階2(substring / replace / replaceAll / toLowerCase /
// toUpperCase / trim / startsWith / endsWith / contains)が、JS 参照実装と等価に振る舞うことを
// 固定する。特に substring は **code point 単位**で、絵文字(サロゲートペア)を割らないことを確認する。

import { describe, expect, it } from "vitest";

import {
  hasPrefix,
  hasSuffix,
  includesSub,
  lower,
  normalizeTag,
  replaceEvery,
  replaceFirst,
  slice,
  stripEnds,
  upper,
} from "../generated/strings/stdlib";

// JS 参照実装。native String メソッドに素直に落とす仕様(spec §2.6)を独立に写したもの。
// substring だけは code point 配列を Array.prototype.slice で切る(spec の code point 規定)。
function refSlice(s: string, start: number, end: number): string {
  return Array.from(s).slice(start, end).join("");
}
function refReplaceFirst(s: string, from: string, to: string): string {
  return s.replace(from, to);
}
function refReplaceEvery(s: string, from: string, to: string): string {
  return s.replaceAll(from, to);
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

const pool = ["a", "b", "c", "Z", "0", " ", "あ", "😀", "🎉", "🇯🇵", "é", "文", ".", "_"];

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
  "abc",
  "a😀b",
  "😀😀😀",
  "🇯🇵",
  "café",
  "  spaced  ",
  "\t\n mixed \r\n",
  "Hello World",
  "KEY_value",
  "photo.PNG",
  "a-b--c---d",
  "日本語テスト",
];
const random = randomStrings(0x5eed4560, 200, 10);
const all = [...fixed, ...random];

describe("strings/stdlib — String stdlib 段階2(M45)", () => {
  it("substring は code point 単位で切る(サロゲートを割らない)", () => {
    // "a😀b" は code point で ["a", "😀", "b"]。index 1..2 は "😀"。
    expect(slice("a😀b", 1, 2)).toBe("😀");
    expect(slice("a😀b", 0, 2)).toBe("a😀");
    expect(slice("😀😀😀", 1, 3)).toBe("😀😀");
    // 参考: native String.prototype.substring は UTF-16 index なので "a😀b".substring(1, 2) は
    // サロゲート片("\uD83D")になる。Kei はこれを code point 単位に是正している。
    expect("a😀b".substring(1, 2)).toBe("\uD83D");
    expect(slice("a😀b", 1, 2)).not.toBe("a😀b".substring(1, 2));
  });

  it("substring は Array.from(s).slice(start, end) と一致する(範囲・境界含む)", () => {
    const ranges: Array<[number, number]> = [
      [0, 0],
      [0, 1],
      [0, 100],
      [1, 3],
      [2, 1], // start > end → ""
      [-2, 100], // 負の start(末尾から)
      [0, -1], // 負の end
    ];
    for (const s of all) {
      for (const [a, b] of ranges) {
        expect(slice(s, a, b)).toBe(refSlice(s, a, b));
      }
    }
  });

  it("replace は最初の 1 箇所・replaceAll は全箇所を置換する", () => {
    expect(replaceFirst("a-a-a", "a", "X")).toBe("X-a-a");
    expect(replaceEvery("a-a-a", "a", "X")).toBe("X-X-X");
    for (const s of all) {
      for (const from of ["a", "😀", " ", "b", "文"]) {
        expect(replaceFirst(s, from, "_")).toBe(refReplaceFirst(s, from, "_"));
        expect(replaceEvery(s, from, "_")).toBe(refReplaceEvery(s, from, "_"));
      }
    }
  });

  it("toLowerCase / toUpperCase は native と一致する", () => {
    expect(lower("Hello World")).toBe("hello world");
    expect(upper("Hello World")).toBe("HELLO WORLD");
    for (const s of all) {
      expect(lower(s)).toBe(s.toLowerCase());
      expect(upper(s)).toBe(s.toUpperCase());
    }
  });

  it("trim は前後の空白を除去する", () => {
    expect(stripEnds("  spaced  ")).toBe("spaced");
    for (const s of all) {
      expect(stripEnds(s)).toBe(s.trim());
    }
  });

  it("startsWith / endsWith / contains は native と一致する", () => {
    expect(hasPrefix("photo.PNG", "photo")).toBe(true);
    expect(hasSuffix("photo.PNG", ".PNG")).toBe(true);
    expect(includesSub("photo.PNG", "to.P")).toBe(true);
    expect(hasSuffix("photo.PNG", ".png")).toBe(false);
    for (const s of all) {
      for (const q of ["a", "😀", "", ".png", "KEY", "文"]) {
        expect(hasPrefix(s, q)).toBe(s.startsWith(q));
        expect(hasSuffix(s, q)).toBe(s.endsWith(q));
        expect(includesSub(s, q)).toBe(s.includes(q));
      }
    }
  });

  it("normalizeTag(trim + toLowerCase)の合成も一致する", () => {
    expect(normalizeTag("  Hello World  ")).toBe("hello world");
    for (const s of all) {
      expect(normalizeTag(s)).toBe(s.trim().toLowerCase());
    }
  });
});
