import type { Segment } from "./types";

/**
 * Pure helpers of the read-along view (D-040, D-042). No DOM, no React: unit-tested with vitest.
 *
 * Segment offsets are UTF-16 code units of the ORIGINAL text, the same unit as JavaScript string
 * indices, so `text.slice(start, end)` is exactly the sentence.
 */

/**
 * Index of the sentence being read at `timeMs`, or -1 when there is none.
 *
 * A sentence stays active during the silence that follows it (no flicker between sentences), so the
 * answer is the last segment whose start is not after the time. Before the first segment, or with no
 * segment at all, nothing is active.
 */
export function activeSegmentIndex(segments: readonly Segment[], timeMs: number): number {
  if (segments.length === 0 || !Number.isFinite(timeMs) || timeMs < segments[0].startMs) return -1;
  let lo = 0;
  let hi = segments.length - 1;
  while (lo < hi) {
    const mid = (lo + hi + 1) >> 1;
    if (segments[mid].startMs <= timeMs) lo = mid;
    else hi = mid - 1;
  }
  return lo;
}

/** A piece of the displayed text: a sentence (with its index) or the text between sentences. */
export interface TextPiece {
  text: string;
  /** Index into the segments, or null for the white space and punctuation between sentences. */
  segment: number | null;
}

/**
 * Cuts `text` into consecutive pieces that, joined, give back the text exactly. Segments that are out
 * of order, overlapping or outside the text are ignored, so a stale result can never garble the view.
 */
export function buildPieces(text: string, segments: readonly Segment[]): TextPiece[] {
  const pieces: TextPiece[] = [];
  let cursor = 0;
  segments.forEach((s, index) => {
    if (s.start < cursor || s.end <= s.start || s.end > text.length) return;
    if (s.start > cursor) pieces.push({ text: text.slice(cursor, s.start), segment: null });
    pieces.push({ text: text.slice(s.start, s.end), segment: index });
    cursor = s.end;
  });
  if (cursor < text.length) pieces.push({ text: text.slice(cursor), segment: null });
  return pieces;
}

/**
 * Scroll position that brings the active sentence into view, or null when it is already comfortably
 * visible. All values are pixels inside the scroll container: `top`/`bottom` are the sentence's edges
 * relative to the container's content, `viewTop`/`viewHeight` the visible band.
 */
export function scrollTarget(
  top: number,
  bottom: number,
  viewTop: number,
  viewHeight: number,
  margin = 24,
): number | null {
  const visible = top >= viewTop + margin && bottom <= viewTop + viewHeight - margin;
  if (visible) return null;
  // Keep the sentence in the upper third so the next ones are already on screen.
  return Math.max(0, top - viewHeight / 3);
}
