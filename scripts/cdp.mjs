// Minimal DevTools-protocol client for the UI test (Node 24: built-in fetch and WebSocket).
import fs from "node:fs";

export async function connect(port = 9222, match = "localhost:1430") {
  const list = await (await fetch(`http://127.0.0.1:${port}/json`)).json();
  const target = list.find((t) => t.type === "page" && t.url.includes(match));
  if (!target) throw new Error("no page with " + match + ": " + JSON.stringify(list.map((t) => t.url)));
  const ws = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((res, rej) => {
    ws.onopen = res;
    ws.onerror = rej;
  });
  let id = 0;
  const pending = new Map();
  ws.onmessage = (m) => {
    const msg = JSON.parse(m.data);
    if (msg.id && pending.has(msg.id)) {
      const { res, rej } = pending.get(msg.id);
      pending.delete(msg.id);
      msg.error ? rej(new Error(JSON.stringify(msg.error))) : res(msg.result);
    }
  };
  const send = (method, params = {}) =>
    new Promise((res, rej) => {
      const i = ++id;
      pending.set(i, { res, rej });
      ws.send(JSON.stringify({ id: i, method, params }));
    });
  const ev = async (expression, userGesture = false) => {
    const r = await send("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true, userGesture });
    if (r.exceptionDetails) throw new Error("page error: " + JSON.stringify(r.exceptionDetails.exception?.description ?? r.exceptionDetails));
    return r.result.value;
  };
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  const waitFor = async (expression, timeoutMs = 60000, step = 200) => {
    const t0 = Date.now();
    for (;;) {
      const v = await ev(expression);
      if (v) return v;
      if (Date.now() - t0 > timeoutMs) throw new Error("timeout waiting for: " + expression);
      await sleep(step);
    }
  };
  const shot = async (file) => {
    const r = await send("Page.captureScreenshot", { format: "png" });
    fs.writeFileSync(file, Buffer.from(r.data, "base64"));
  };
  return { send, ev, sleep, waitFor, shot, close: () => ws.close() };
}

/** WCAG contrast ratio of two "rgb(r, g, b)" strings. */
export function contrast(a, b) {
  const lum = (c) => {
    const [r, g, bl] = c.match(/\d+(\.\d+)?/g).slice(0, 3).map(Number).map((v) => {
      const s = v / 255;
      return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
    });
    return 0.2126 * r + 0.7152 * g + 0.0722 * bl;
  };
  const [hi, lo] = [lum(a), lum(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}
