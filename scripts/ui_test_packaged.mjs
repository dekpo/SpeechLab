// UI test of the PACKAGED app (M7) over the DevTools protocol.
//
//   $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=9222"
//   & "<install dir>\speechlab.exe"                              # the installed app, not `pnpm tauri dev`
//   $env:SPEECHLAB_CDP_MATCH = "tauri.localhost"                 # the release origin of the page
//   node scripts/ui_test_packaged.mjs --install-dir "<install dir>" [--empty] [--no-mic] [shot-dir]
//
// Full mode checks: the model lists, one speech synthesis (this proves that the espeak-ng data folder is
// found from the packaged app), speech-to-text through sherpa-onnx and through whisper.cpp (the sidecar
// next to the executable), the cancellation of both, and a 3 second microphone recording (the permission
// dialog is answered like a person would: Allow). `--empty` is for an app started with
// SPEECHLAB_MODELS_DIR pointing at an empty folder: it checks the first-start experience.
// Everything this script creates (generated speech, the recorded clip) is deleted at the end.
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { connect } from "./cdp.mjs";

const args = process.argv.slice(2);
const flag = (n) => args.includes(n);
const opt = (n) => (args.includes(n) ? args[args.indexOf(n) + 1] : null);
const EMPTY = flag("--empty");
const NO_MIC = flag("--no-mic");
const INSTALL_DIR = opt("--install-dir");
const positional = args.filter((a, i) => !a.startsWith("--") && args[i - 1] !== "--install-dir");
const OUT = positional[0] ?? path.join(os.tmpdir(), "speechlab-ui-packaged");
fs.mkdirSync(OUT, { recursive: true });

const APPDATA = path.join(process.env.APPDATA, "ai.assistantcabinet.speechlab");
const WAV_DIR = path.join(APPDATA, "tts-output");
const REC_DIR = path.join(APPDATA, "recordings");
const list = (d) => (fs.existsSync(d) ? fs.readdirSync(d) : []);
const beforeWav = new Set(list(WAV_DIR));
const beforeRec = new Set(list(REC_DIR));

const results = [];
const check = (name, ok, detail = "") => {
  results.push({ name, ok: !!ok, detail });
  console.log(`${ok ? "PASS" : "FAIL"}  ${name}${detail ? "  (" + detail + ")" : ""}`);
};

/** Paths of the running processes with this image name (empty list if none). */
function processPaths(image) {
  const out = execFileSync(
    "powershell",
    ["-NoProfile", "-Command", `Get-CimInstance Win32_Process -Filter "Name='${image}'" | ForEach-Object { $_.ExecutablePath }`],
    { encoding: "utf8" },
  );
  return out.split(/\r?\n/).map((s) => s.trim()).filter(Boolean);
}

function wavInfo(file) {
  const b = fs.readFileSync(file);
  const ok = b.toString("ascii", 0, 4) === "RIFF" && b.toString("ascii", 8, 12) === "WAVE";
  const rate = b.readUInt32LE(24);
  const bits = b.readUInt16LE(34);
  const channels = b.readUInt16LE(22);
  const dataBytes = b.length - 44;
  return { ok, rate, bits, channels, seconds: dataBytes / (rate * channels * (bits / 8)), bytes: b.length };
}

const cdp = await connect();
const { ev, sleep, waitFor, send } = cdp;

const setValueJs = (id, v, proto = "HTMLInputElement") => `(() => {
  const el = document.getElementById(${JSON.stringify(id)});
  Object.getOwnPropertyDescriptor(window.${proto}.prototype, 'value').set.call(el, ${JSON.stringify(v)});
  el.dispatchEvent(new Event(${JSON.stringify(proto === "HTMLSelectElement" ? "change" : "input")}, { bubbles: true }));
  return el.value; })()`;
const setSelectJs = (id, v) => setValueJs(id, v, "HTMLSelectElement");
const clickButtonJs = (label) => `(() => {
  const b = [...document.querySelectorAll('button')].find((x) => x.textContent.trim() === ${JSON.stringify(label)});
  if (!b) return false; b.click(); return true; })()`;
const errorText = () => ev(`document.querySelector('.error')?.textContent ?? null`);
const optionsOf = (id) => ev(`[...document.getElementById(${JSON.stringify(id)}).options].map((o) => ({ v: o.value, t: o.textContent }))`);

async function generate(text, nSegments, lang = "en", voice = "tts-vits-piper-en_US-ljspeech-medium:0") {
  await ev(setSelectJs("tts-lang", lang));
  await waitFor(`document.getElementById('tts-voice') && [...document.getElementById('tts-voice').options].some((o) => o.value === ${JSON.stringify(voice)})`);
  await ev(setSelectJs("tts-voice", voice));
  if (await ev(`!document.getElementById('tts-text')`)) await ev(clickButtonJs("Edit text"));
  await waitFor(`!!document.getElementById('tts-text')`);
  await ev(setValueJs("tts-text", text, "HTMLTextAreaElement"));
  await sleep(200);
  await ev(`document.getElementById('tts-generate').click()`);
  if (nSegments) {
    await waitFor(`document.querySelectorAll('[data-segment]').length === ${nSegments}`, 180000);
    await ev(`(document.querySelector('audio').muted = true, true)`);
  }
}

async function transcribe(providerPattern, modelPattern, wavPath, lang = "en") {
  const provs = await optionsOf("provider");
  const prov = provs.find((p) => new RegExp(providerPattern, "i").test(p.t));
  if (!prov) throw new Error("no engine matching " + providerPattern + ": " + JSON.stringify(provs));
  await ev(setSelectJs("provider", prov.v));
  await sleep(300);
  const mods = await optionsOf("model");
  const mod = mods.find((m) => new RegExp(modelPattern, "i").test(m.t) && !/not installed/.test(m.t));
  if (!mod) throw new Error("no installed model matching " + modelPattern + ": " + JSON.stringify(mods));
  await ev(setSelectJs("model", mod.v));
  await ev(setSelectJs("lang", lang));
  await ev(setValueJs("audio", wavPath));
  await sleep(200);
  await ev(`document.getElementById('out') && (document.getElementById('out').value = ''); true`);
  const t0 = Date.now();
  await ev(clickButtonJs("Transcribe"));
  return { prov, mod, t0 };
}

try {
  await send("Page.enable");
  await send("Page.reload");
  await sleep(1500);
  await waitFor(`document.querySelectorAll('table tr').length > 3`, 30000);
  await ev(`window.alert = () => {}; true`);

  // ---- 1. the window and the model lists ----
  check("page origin is the release origin", (await ev(`location.origin`)) === "http://tauri.localhost", await ev(`location.origin`));
  check("page title is SpeechLab", (await ev(`document.title`)) === "SpeechLab");
  check("the native bridge is present", (await ev(`typeof window.__TAURI_INTERNALS__`)) === "object");
  const rows = await ev(`[...document.querySelectorAll('table tr')].map((r) => r.innerText.replace(/\\s+/g, ' ').trim())`);
  const modelRows = rows.filter((r) => /\b(installed|not installed)\b/.test(r) && !/Commercial use/.test(r));
  check("the model lists are shown", modelRows.length >= 10, `${modelRows.length} rows`);
  await cdp.shot(path.join(OUT, EMPTY ? "empty-start.png" : "start.png"));

  if (EMPTY) {
    // ---- first start with no model ----
    const installedRows = modelRows.filter((r) => / installed\b/.test(r) && !/not installed/.test(r));
    check("no model is reported as installed", installedRows.length === 0, `${installedRows.length} installed`);
    const installButtons = await ev(`[...document.querySelectorAll('button')].filter((b) => b.textContent.trim() === 'Install').length`);
    check("every model has an Install button", installButtons >= 10, `${installButtons} buttons`);
    check("Transcribe is disabled", await ev(`[...document.querySelectorAll('button')].find((b) => b.textContent.trim() === 'Transcribe').disabled`));
    check("Generate is disabled (no voice)", await ev(`document.getElementById('tts-generate').disabled`));
    check("the engine pickers list the models as not installed", (await optionsOf("model")).every((m) => /not installed/.test(m.t)));
    check("no error banner at start", (await errorText()) === null, String(await errorText()));
    const hint = await ev(`document.body.innerText.includes('Models are downloaded once')`);
    check("the page explains that models are downloaded once", hint);
  } else {
    const installedStt = rows.filter((r) => /Whisper|Canary|Parakeet/.test(r) && / installed\b/.test(r) && !/not installed/.test(r));
    check("at least 6 speech-to-text models are installed", installedStt.length >= 6, `${installedStt.length}`);
    check("no error banner at start", (await errorText()) === null, String(await errorText()));

    // ---- 2. speech synthesis (needs espeak-ng-data from the models folder) ----
    const SENTENCE = "The patient takes two tablets every morning.";
    const t0 = Date.now();
    await generate(SENTENCE, 1);
    const synthMs = Date.now() - t0;
    const made = list(WAV_DIR).filter((f) => !beforeWav.has(f) && f.endsWith(".wav"));
    check("a WAV file was written by the Piper voice", made.length >= 1, made.join(", "));
    let wavPath = null;
    if (made.length) {
      wavPath = path.join(WAV_DIR, made[0]);
      const info = wavInfo(wavPath);
      check("the WAV is valid, 16-bit, longer than 1.5 s", info.ok && info.bits === 16 && info.seconds > 1.5, JSON.stringify(info));
      console.log(`      synthesis of one sentence took ${synthMs} ms wall time (includes voice load)`);
    }
    check("no error after synthesis", (await errorText()) === null, String(await errorText()));
    check("audio is not playing by itself", await ev(`document.querySelector('audio').paused`));

    if (wavPath) {
      // ---- 3. speech-to-text, sherpa-onnx ----
      const words = ["patient", "morning"];
      let r = await transcribe("sherpa", "parakeet", wavPath);
      await waitFor(`document.getElementById('out') && document.getElementById('out').value.length > 0`, 120000);
      let text = (await ev(`document.getElementById('out').value`)).toLowerCase();
      let footer = await ev(`document.querySelector('#out + small')?.textContent ?? ''`);
      check("sherpa-onnx Parakeet transcribes the generated clip", words.every((w) => text.includes(w)), text);
      console.log(`      ${footer}`);

      // ---- 4. speech-to-text, whisper.cpp through the sidecar ----
      r = await transcribe("whisper\\.cpp|whisper cpp", "base", wavPath);
      let sawCli = [];
      const t1 = Date.now();
      while (Date.now() - t1 < 120000) {
        sawCli = sawCli.concat(processPaths("whisper-cli.exe"));
        if (await ev(`!!document.getElementById('out') && document.getElementById('out').value.length > 0`)) break;
        await sleep(100);
      }
      text = (await ev(`document.getElementById('out').value`)).toLowerCase();
      footer = await ev(`document.querySelector('#out + small')?.textContent ?? ''`);
      check("whisper.cpp base transcribes the generated clip", words.every((w) => text.includes(w)), text);
      console.log(`      ${footer}`);
      if (INSTALL_DIR) {
        const norm = (p) => p.toLowerCase().replace(/\//g, "\\");
        const fromInstall = sawCli.length > 0 && sawCli.every((p) => norm(p).startsWith(norm(INSTALL_DIR)));
        check("the whisper-cli process that ran is the one next to the installed executable", fromInstall, [...new Set(sawCli)].join(" | ") || "process never seen");
      }

      // ---- 5. cancel a transcription (a longer clip made of the same sentence) ----
      const long = Array(6).fill(SENTENCE).join(" ");
      await generate(long, 6);
      const longWav = list(WAV_DIR).filter((f) => !beforeWav.has(f) && f.endsWith(".wav")).map((f) => path.join(WAV_DIR, f)).sort((a, b) => fs.statSync(b).mtimeMs - fs.statSync(a).mtimeMs)[0];
      await transcribe("whisper\\.cpp|whisper cpp", "small", longWav);
      let running = [];
      const t2 = Date.now();
      while (Date.now() - t2 < 20000 && running.length === 0) {
        running = processPaths("whisper-cli.exe");
        await sleep(100);
      }
      check("whisper-cli small is running before the Cancel click", running.length > 0);
      await ev(`(() => { const b = [...document.querySelectorAll('button')].filter((x) => x.textContent.trim() === 'Cancel')[0]; b.click(); return true; })()`);
      // While a run is in progress the button reads "Running…": it is the one just before "Cancel".
      const runButton = `[...document.querySelectorAll('button')].find((b) => b.textContent.trim() === 'Cancel').previousElementSibling`;
      await waitFor(`!${runButton}.disabled`, 20000);
      await sleep(1500);
      check("Cancel stops the whisper-cli process", processPaths("whisper-cli.exe").length === 0);
      check("the Transcribe button is usable again after Cancel", await ev(`${runButton}.textContent.trim() === 'Transcribe' && !${runButton}.disabled`));
      check("a cancelled transcription shows no error banner", (await errorText()) === null, String(await errorText()));
    }

    // ---- 6. cancel a synthesis ----
    const twenty = Array.from({ length: 20 }, (_, i) => `This is test sentence number ${i + 1} of the cancellation check.`).join(" ");
    const nWavBefore = list(WAV_DIR).length;
    await generate(twenty, 0);
    await sleep(500);
    await ev(`document.getElementById('tts-generate').nextElementSibling.click()`);
    await waitFor(`!document.getElementById('tts-generate').disabled`, 20000);
    check("Cancel ends a running synthesis", true);
    check("a cancelled synthesis leaves no new audio file", list(WAV_DIR).length === nWavBefore, `${list(WAV_DIR).length - nWavBefore} new`);
    check("a cancelled synthesis shows no error banner", (await errorText()) === null, String(await errorText()));

    // ---- 7. microphone (release origin) ----
    if (!NO_MIC) {
      const state0 = await ev(`navigator.permissions.query({ name: 'microphone' }).then((r) => r.state)`);
      console.log(`      microphone permission before the first click: ${state0}`);
      await ev(clickButtonJs("● Record"));
      let asked = false;
      for (let i = 0; i < 40; i++) {
        const targets = await (await fetch("http://127.0.0.1:9222/json")).json();
        const dlg = targets.find((t) => t.url.includes("permission-request-dialog"));
        if (dlg) {
          asked = true;
          const d = await connect(undefined, "permission-request-dialog");
          await d.ev(`(() => { const w = (root) => { for (const el of root.querySelectorAll('*')) { if (el.id === 'allow-button') { el.click(); return true; } if (el.shadowRoot && w(el.shadowRoot)) return true; } return false; }; return w(document); })()`, true);
          d.close();
          break;
        }
        if (await ev(`[...document.querySelectorAll('button')].some((b) => b.textContent.trim().startsWith('■ Stop'))`)) break;
        await sleep(250);
      }
      console.log(`      permission dialog shown: ${asked}`);
      await waitFor(`[...document.querySelectorAll('button')].some((b) => b.textContent.trim().startsWith('■ Stop'))`, 15000);
      await sleep(3000);
      await ev(`[...document.querySelectorAll('button')].find((b) => b.textContent.trim().startsWith('■ Stop')).click()`);
      await waitFor(`!![...document.querySelectorAll('button')].find((b) => b.textContent.trim() === '● Record') && !document.body.innerText.includes('Processing audio')`, 20000);
      const clips = list(REC_DIR).filter((f) => !beforeRec.has(f) && f.endsWith(".wav"));
      check("a recorded clip was stored", clips.length === 1, clips.join(", "));
      if (clips.length) {
        const info = wavInfo(path.join(REC_DIR, clips[0]));
        check("the clip is 16 kHz mono 16-bit, about 3 s", info.rate === 16000 && info.channels === 1 && info.bits === 16 && info.seconds > 2.4 && info.seconds < 4, JSON.stringify(info));
      }
      check("no error banner after recording", (await errorText()) === null, String(await errorText()));
      const state1 = await ev(`navigator.permissions.query({ name: 'microphone' }).then((r) => r.state)`);
      check("the microphone permission is 'granted' after Allow", state1 === "granted", state1);
    }
    await cdp.shot(path.join(OUT, "after-tests.png"));
  }
} catch (e) {
  check("test script completed without an exception", false, String(e));
} finally {
  const wav = list(WAV_DIR).filter((f) => !beforeWav.has(f));
  for (const f of wav) fs.rmSync(path.join(WAV_DIR, f), { force: true });
  const rec = list(REC_DIR).filter((f) => !beforeRec.has(f));
  for (const f of rec) fs.rmSync(path.join(REC_DIR, f), { force: true });
  console.log(`cleanup: removed ${wav.length} generated and ${rec.length} recorded file(s)`);
  cdp.close();
}
const failed = results.filter((r) => !r.ok);
console.log(`\n${results.length - failed.length}/${results.length} checks passed`);
fs.writeFileSync(path.join(OUT, EMPTY ? "ui-packaged-empty-results.json" : "ui-packaged-results.json"), JSON.stringify(results, null, 2));
process.exit(failed.length ? 1 : 0);
