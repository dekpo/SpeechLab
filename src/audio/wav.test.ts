import { describe, expect, it } from "vitest";
import { encodeWav16, rms, signalStats } from "./wav";

const ascii = (b: Uint8Array, from: number, len: number) => String.fromCharCode(...b.slice(from, from + len));

describe("encodeWav16", () => {
  it("writes a canonical 44-byte PCM header", () => {
    const wav = encodeWav16(new Float32Array(160), 16000);
    const v = new DataView(wav.buffer);
    expect(ascii(wav, 0, 4)).toBe("RIFF");
    expect(ascii(wav, 8, 4)).toBe("WAVE");
    expect(ascii(wav, 12, 4)).toBe("fmt ");
    expect(v.getUint16(20, true)).toBe(1); // PCM
    expect(v.getUint16(22, true)).toBe(1); // mono
    expect(v.getUint32(24, true)).toBe(16000);
    expect(v.getUint16(34, true)).toBe(16);
    expect(ascii(wav, 36, 4)).toBe("data");
    expect(v.getUint32(40, true)).toBe(320);
    expect(wav.length).toBe(44 + 320);
    expect(v.getUint32(4, true)).toBe(wav.length - 8);
  });

  it("clamps out-of-range samples and keeps the sign", () => {
    const wav = encodeWav16(new Float32Array([2, -2, 0.5, -0.5, 0]), 8000);
    const v = new DataView(wav.buffer);
    expect(v.getInt16(44, true)).toBe(32767);
    expect(v.getInt16(46, true)).toBe(-32768);
    expect(v.getInt16(48, true)).toBeGreaterThan(16000);
    expect(v.getInt16(50, true)).toBeLessThan(-16000);
    expect(v.getInt16(52, true)).toBe(0);
  });
});

describe("rms", () => {
  it("is 0 for silence and empty input, and the amplitude for a constant signal", () => {
    expect(rms(new Float32Array(0))).toBe(0);
    expect(rms(new Float32Array(100))).toBe(0);
    expect(rms(new Float32Array(100).fill(0.5))).toBeCloseTo(0.5, 5);
  });
});

describe("signalStats", () => {
  it("reports the peak and the clipped share", () => {
    const s = signalStats(new Float32Array([0, 0.5, -1, 1, 0.2, 0, 0, 0, 0, 0]));
    expect(s.peak).toBe(1);
    expect(s.clippedRatio).toBeCloseTo(0.2, 5);
  });
  it("handles empty input", () => {
    expect(signalStats(new Float32Array(0))).toEqual({ peak: 0, clippedRatio: 0 });
  });
});
