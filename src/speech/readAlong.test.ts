import { describe, expect, it } from "vitest";
import { activeSegmentIndex, buildPieces, scrollTarget } from "./readAlong";
import type { Segment } from "./types";

const seg = (start: number, end: number, startMs: number, endMs: number): Segment => ({ start, end, startMs, endMs });

// Three sentences with 250 ms and 600 ms silences between them (as the provider joins them).
const SEGMENTS: Segment[] = [seg(0, 8, 0, 1000), seg(9, 20, 1250, 2500), seg(22, 30, 3100, 4000)];

describe("activeSegmentIndex", () => {
  it("returns the sentence whose span contains the time", () => {
    expect(activeSegmentIndex(SEGMENTS, 0)).toBe(0);
    expect(activeSegmentIndex(SEGMENTS, 500)).toBe(0);
    expect(activeSegmentIndex(SEGMENTS, 1250)).toBe(1);
    expect(activeSegmentIndex(SEGMENTS, 2000)).toBe(1);
    expect(activeSegmentIndex(SEGMENTS, 3100)).toBe(2);
    expect(activeSegmentIndex(SEGMENTS, 3999)).toBe(2);
  });

  it("keeps a sentence active during the silence that follows it", () => {
    expect(activeSegmentIndex(SEGMENTS, 1100)).toBe(0);
    expect(activeSegmentIndex(SEGMENTS, 2800)).toBe(1);
    expect(activeSegmentIndex(SEGMENTS, 99_999)).toBe(2);
  });

  it("is -1 with nothing to show", () => {
    expect(activeSegmentIndex([], 100)).toBe(-1);
    expect(activeSegmentIndex(SEGMENTS, -5)).toBe(-1);
    expect(activeSegmentIndex(SEGMENTS, Number.NaN)).toBe(-1);
    expect(activeSegmentIndex([seg(0, 3, 400, 900)], 100)).toBe(-1);
  });

  it("finds every sentence of a long text", () => {
    const long = Array.from({ length: 500 }, (_, i) => seg(i * 10, i * 10 + 9, i * 1000, i * 1000 + 800));
    for (const i of [0, 1, 137, 499]) {
      expect(activeSegmentIndex(long, i * 1000 + 400)).toBe(i);
    }
  });
});

describe("buildPieces", () => {
  const text = "Un. Deux, trois!\n\nQuatre.";

  it("covers the text exactly, with sentences and the gaps between them", () => {
    const segments = [seg(0, 3, 0, 500), seg(4, 16, 750, 2000), seg(18, 25, 2600, 3200)];
    const pieces = buildPieces(text, segments);
    expect(pieces.map((p) => p.text).join("")).toBe(text);
    expect(pieces.filter((p) => p.segment !== null).map((p) => p.text)).toEqual(["Un.", "Deux, trois!", "Quatre."]);
    expect(pieces.filter((p) => p.segment === null).map((p) => p.text)).toEqual([" ", "\n\n"]);
    expect(pieces.filter((p) => p.segment !== null).map((p) => p.segment)).toEqual([0, 1, 2]);
  });

  it("ignores segments that do not fit the text", () => {
    const segments = [seg(0, 3, 0, 500), seg(2, 8, 600, 900), seg(10, 999, 1000, 2000), seg(5, 5, 0, 0)];
    const pieces = buildPieces(text, segments);
    expect(pieces.map((p) => p.text).join("")).toBe(text);
    expect(pieces.filter((p) => p.segment !== null)).toHaveLength(1);
  });

  it("returns the whole text as one gap when there are no segments", () => {
    expect(buildPieces(text, [])).toEqual([{ text, segment: null }]);
    expect(buildPieces("", [])).toEqual([]);
  });

  it("uses UTF-16 offsets, so accents and emoji line up", () => {
    const t = "Été 😀 chaud. Fin.";
    const start = t.indexOf("Fin");
    const pieces = buildPieces(t, [seg(0, 13, 0, 900), seg(start, start + 4, 1200, 1500)]);
    expect(pieces.map((p) => p.text).join("")).toBe(t);
    expect(pieces[0].text).toBe("Été 😀 chaud.");
    expect(pieces[2].text).toBe("Fin.");
  });
});

describe("scrollTarget", () => {
  it("does nothing when the sentence is comfortably visible", () => {
    expect(scrollTarget(100, 140, 0, 300)).toBeNull();
    expect(scrollTarget(224, 260, 200, 300)).toBeNull();
  });

  it("scrolls when the sentence is below, above or cut by the edges", () => {
    expect(scrollTarget(500, 540, 0, 300)).toBe(400);
    expect(scrollTarget(50, 90, 200, 300)).toBe(0);
    expect(scrollTarget(270, 310, 0, 300)).toBe(170);
  });

  it("never asks for a negative position", () => {
    expect(scrollTarget(10, 30, 400, 300)).toBe(0);
  });
});
