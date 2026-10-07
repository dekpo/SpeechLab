// Microphone capture inside the WebView (getUserMedia + AudioWorklet).
// Audio never leaves the machine: samples stay in memory until the clip is saved locally.
// Browser audio processing (echo cancellation, noise suppression, AGC) is switched OFF so
// recordings are as raw as possible and comparable between runs.

import { rms } from "./wav";

const WORKLET_SOURCE = `
class CaptureProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.buf = new Float32Array(4096);
    this.n = 0;
    // "flush" sends the partially filled buffer so the end of a recording is not lost.
    this.port.onmessage = () => {
      if (this.n > 0) { this.port.postMessage(this.buf.slice(0, this.n)); this.n = 0; }
    };
  }
  process(inputs) {
    const ch = inputs[0] && inputs[0][0];
    if (ch) {
      for (let i = 0; i < ch.length; i++) {
        this.buf[this.n++] = ch[i];
        if (this.n === this.buf.length) { this.port.postMessage(this.buf.slice(0)); this.n = 0; }
      }
    }
    return true;
  }
}
registerProcessor("capture-processor", CaptureProcessor);
`;

export interface Recording {
  samples: Float32Array;
  sampleRate: number;
  deviceLabel: string;
}

export async function listInputDevices(): Promise<MediaDeviceInfo[]> {
  const all = await navigator.mediaDevices.enumerateDevices();
  return all.filter((d) => d.kind === "audioinput");
}

export class MicRecorder {
  private ctx: AudioContext | null = null;
  private stream: MediaStream | null = null;
  private node: AudioWorkletNode | null = null;
  private chunks: Float32Array[] = [];
  private deviceLabel = "";

  get active(): boolean {
    return this.ctx !== null;
  }

  async start(deviceId: string | undefined, onLevel: (level: number) => void): Promise<void> {
    if (!navigator.mediaDevices?.getUserMedia) {
      throw new Error("Microphone capture is not available in this WebView.");
    }
    this.stream = await navigator.mediaDevices.getUserMedia({
      audio: {
        deviceId: deviceId ? { exact: deviceId } : undefined,
        channelCount: 1,
        echoCancellation: false,
        noiseSuppression: false,
        autoGainControl: false,
      },
    });
    this.deviceLabel = this.stream.getAudioTracks()[0]?.label ?? "";
    this.ctx = new AudioContext();
    const url = URL.createObjectURL(new Blob([WORKLET_SOURCE], { type: "text/javascript" }));
    try {
      await this.ctx.audioWorklet.addModule(url);
    } finally {
      URL.revokeObjectURL(url);
    }
    const source = this.ctx.createMediaStreamSource(this.stream);
    this.node = new AudioWorkletNode(this.ctx, "capture-processor");
    this.chunks = [];
    this.node.port.onmessage = (e: MessageEvent<Float32Array>) => {
      this.chunks.push(e.data);
      onLevel(rms(e.data));
    };
    source.connect(this.node);
    // The worklet node must be connected to the graph to be pulled; it outputs silence.
    this.node.connect(this.ctx.destination);
  }

  async stop(): Promise<Recording> {
    if (!this.ctx) throw new Error("Not recording.");
    const sampleRate = this.ctx.sampleRate;
    // Ask the worklet for its partial buffer and give the messages time to arrive before closing.
    this.node?.port.postMessage("flush");
    await new Promise((resolve) => setTimeout(resolve, 150));
    this.stream?.getTracks().forEach((t) => t.stop());
    this.node?.disconnect();
    await this.ctx.close();
    const total = this.chunks.reduce((n, c) => n + c.length, 0);
    const samples = new Float32Array(total);
    let offset = 0;
    for (const c of this.chunks) {
      samples.set(c, offset);
      offset += c.length;
    }
    const label = this.deviceLabel;
    this.ctx = null;
    this.stream = null;
    this.node = null;
    this.chunks = [];
    return { samples, sampleRate, deviceLabel: label };
  }
}
