// Audio helpers: any decodable audio -> mono 16 kHz 16-bit PCM WAV bytes.
// Pure functions are unit-tested; the Web Audio parts run only in the WebView.

export const TARGET_RATE = 16000;

/** Encodes mono float samples in [-1, 1] as a 16-bit PCM WAV file. */
export function encodeWav16(samples: Float32Array, sampleRate: number): Uint8Array {
  const dataBytes = samples.length * 2;
  const out = new Uint8Array(44 + dataBytes);
  const view = new DataView(out.buffer);
  const text = (offset: number, s: string) => {
    for (let i = 0; i < s.length; i++) view.setUint8(offset + i, s.charCodeAt(i));
  };
  text(0, "RIFF");
  view.setUint32(4, 36 + dataBytes, true);
  text(8, "WAVE");
  text(12, "fmt ");
  view.setUint32(16, 16, true); // PCM chunk size
  view.setUint16(20, 1, true); // PCM
  view.setUint16(22, 1, true); // mono
  view.setUint32(24, sampleRate, true);
  view.setUint32(28, sampleRate * 2, true); // byte rate
  view.setUint16(32, 2, true); // block align
  view.setUint16(34, 16, true); // bits per sample
  text(36, "data");
  view.setUint32(40, dataBytes, true);
  for (let i = 0; i < samples.length; i++) {
    const s = Math.max(-1, Math.min(1, samples[i]));
    view.setInt16(44 + i * 2, s < 0 ? s * 0x8000 : s * 0x7fff, true);
  }
  return out;
}

/** Root-mean-square level of a block of samples, 0..1. */
export function rms(samples: Float32Array): number {
  if (samples.length === 0) return 0;
  let sum = 0;
  for (let i = 0; i < samples.length; i++) sum += samples[i] * samples[i];
  return Math.sqrt(sum / samples.length);
}

export interface SignalStats {
  /** Largest absolute sample, 0..1. */
  peak: number;
  /** Share of samples at or above 99 % of full scale (a sign the input is clipping). */
  clippedRatio: number;
}

export function signalStats(samples: Float32Array): SignalStats {
  let peak = 0;
  let clipped = 0;
  for (let i = 0; i < samples.length; i++) {
    const a = Math.abs(samples[i]);
    if (a > peak) peak = a;
    if (a >= 0.99) clipped++;
  }
  return { peak, clippedRatio: samples.length ? clipped / samples.length : 0 };
}

/** Resamples an AudioBuffer (any channel count) to mono at `rate` using Web Audio. */
export async function resampleToMono(buffer: AudioBuffer, rate = TARGET_RATE): Promise<Float32Array> {
  const length = Math.max(1, Math.ceil(buffer.duration * rate));
  const ctx = new OfflineAudioContext(1, length, rate);
  const src = ctx.createBufferSource();
  src.buffer = buffer;
  src.connect(ctx.destination);
  src.start();
  const rendered = await ctx.startRendering();
  return rendered.getChannelData(0);
}

export interface EncodedClip {
  wav: Uint8Array;
  durationMs: number;
}

/** Raw mono samples recorded at `sampleRate` -> 16 kHz WAV. */
export async function samplesToWav16k(samples: Float32Array, sampleRate: number): Promise<EncodedClip> {
  const buffer = new AudioBuffer({ length: samples.length, sampleRate, numberOfChannels: 1 });
  buffer.copyToChannel(new Float32Array(samples), 0);
  const mono = await resampleToMono(buffer);
  return { wav: encodeWav16(mono, TARGET_RATE), durationMs: Math.round((mono.length / TARGET_RATE) * 1000) };
}

/** Any audio file the WebView can decode (WAV, MP3, M4A/AAC, Ogg, WebM...) -> 16 kHz WAV. */
export async function fileToWav16k(file: File): Promise<EncodedClip> {
  const ctx = new AudioContext();
  try {
    const decoded = await ctx.decodeAudioData(await file.arrayBuffer());
    const mono = await resampleToMono(decoded);
    return { wav: encodeWav16(mono, TARGET_RATE), durationMs: Math.round((mono.length / TARGET_RATE) * 1000) };
  } catch (e) {
    throw new Error(
      `This WebView cannot decode "${file.name}" (${file.type || "unknown type"}): ${String(e)}. ` +
        "Convert it to WAV (for example with Audacity or VLC).",
    );
  } finally {
    void ctx.close();
  }
}
