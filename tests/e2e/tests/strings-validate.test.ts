// examples/strings/validate.kei の等価テスト(M46 / #160)。
// Kei が生成した isImageMime / isValidObjectKey(startsWith / endsWith / 許可文字判定)が、
// 独立に書いた JS 参照実装(正規表現ベース)と、固定代表ケース + 決定論的ランダム入力
// (絵文字・記号・空文字・長文を含む)で完全一致することを CI で常設検証する。

import { describe, expect, it } from "vitest";

import { isImageMime, isValidObjectKey } from "../generated/strings/validate";

// JS 参照実装(独立戦略 = 正規表現)。
function refIsImageMime(s: string): boolean {
  return /^image\/[a-z0-9.+-]+$/.test(s);
}
function refIsValidObjectKey(s: string): boolean {
  if (s.length === 0) return false;
  if (s.startsWith("/") || s.endsWith("/")) return false;
  return /^[A-Za-z0-9._/-]+$/.test(s);
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

// MIME らしい入力を作りやすいプレフィックス付きプールと、キーらしい文字プール。
const pool = [
  "image/", "text/", "png", "jpeg", "svg+xml", "webp", ".", "+", "-", "/", "_",
  "a", "B", "0", "9", " ", "😀", "🇯🇵", "文", "é", "!", "@",
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
  "image/png",
  "image/jpeg",
  "image/svg+xml",
  "image/webp",
  "image/x-icon",
  "image/",
  "IMAGE/PNG",
  "text/plain",
  "image/png ",
  "image/png\n",
  "image/😀",
  "photos/2024/cover.png",
  "a_b-c.d/e",
  "/leading-slash",
  "trailing-slash/",
  "has space",
  "emoji😀key",
  "a".repeat(1000),
];
const random = randomStrings(0x33ac91fe, 400, 20);
const all = [...fixed, ...random];

describe("strings/validate — MIME / キー検証の JS 参照等価(M46)", () => {
  it("isImageMime の代表ケース(人間が手計算した期待値)", () => {
    expect(isImageMime("image/png")).toBe(true);
    expect(isImageMime("image/svg+xml")).toBe(true);
    expect(isImageMime("image/x-icon")).toBe(true);
    expect(isImageMime("image/")).toBe(false);
    expect(isImageMime("IMAGE/PNG")).toBe(false);
    expect(isImageMime("text/plain")).toBe(false);
    expect(isImageMime("image/png ")).toBe(false);
    expect(isImageMime("image/😀")).toBe(false);
    expect(isImageMime("")).toBe(false);
  });

  it("isValidObjectKey の代表ケース(人間が手計算した期待値)", () => {
    expect(isValidObjectKey("photos/2024/cover.png")).toBe(true);
    expect(isValidObjectKey("a_b-c.d/e")).toBe(true);
    expect(isValidObjectKey("/leading-slash")).toBe(false);
    expect(isValidObjectKey("trailing-slash/")).toBe(false);
    expect(isValidObjectKey("has space")).toBe(false);
    expect(isValidObjectKey("emoji😀key")).toBe(false);
    expect(isValidObjectKey("")).toBe(false);
  });

  it("固定 + ランダム入力すべてで JS 参照実装と一致する", () => {
    for (const s of all) {
      expect(isImageMime(s)).toBe(refIsImageMime(s));
      expect(isValidObjectKey(s)).toBe(refIsValidObjectKey(s));
    }
  });
});
