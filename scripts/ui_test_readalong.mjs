// UI test of the TTS read-along view over the DevTools protocol (M6d, D-042).
//
//   WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222 pnpm tauri dev    (dev port 1430)
//   node scripts/ui_test_readalong.mjs [screenshot-dir]
//
// Drives the page of the app on port 1430 only. The audio is muted and never auto-played; the WAV files
// the test creates are deleted at the end (files that existed before are left alone).
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { connect, contrast } from "./cdp.mjs";

const OUT = process.argv[2] ?? path.join(os.tmpdir(), "speechlab-ui-test");
fs.mkdirSync(OUT, { recursive: true });
const WAV_DIR = path.join(process.env.APPDATA, "ai.assistantcabinet.speechlab", "tts-output");
const before = new Set(fs.existsSync(WAV_DIR) ? fs.readdirSync(WAV_DIR) : []);

const results = [];
const check = (name, ok, detail = "") => {
  results.push({ name, ok: !!ok, detail });
  console.log(`${ok ? "PASS" : "FAIL"}  ${name}${detail ? "  (" + detail + ")" : ""}`);
};

const cdp = await connect();
const { ev, sleep, waitFor, send } = cdp;

const setTextJs = (t) => `(() => {
  const el = document.getElementById('tts-text');
  Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value').set.call(el, ${JSON.stringify(t)});
  el.dispatchEvent(new Event('input', { bubbles: true }));
  return true; })()`;
const setSelectJs = (id, v) => `(() => {
  const el = document.getElementById('${id}');
  Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, 'value').set.call(el, ${JSON.stringify(v)});
  el.dispatchEvent(new Event('change', { bubbles: true }));
  return el.value; })()`;
const clickButtonJs = (label) => `(() => {
  const b = [...document.querySelectorAll('button')].find((x) => x.textContent.trim() === ${JSON.stringify(label)});
  if (!b) return false; b.click(); return true; })()`;
const seekJs = (sec) => `(() => { const a = document.querySelector('audio'); a.muted = true; a.currentTime = ${sec}; return a.currentTime; })()`;
const activeJs = `(() => {
  const spans = [...document.querySelectorAll('[data-segment]')];
  const cur = spans.filter((s) => s.getAttribute('aria-current') === 'true');
  return { count: spans.length, active: cur.map((s) => Number(s.dataset.segment)) };
})()`;
const segmentsJs = `[...document.querySelectorAll('[data-segment]')].map((s) => ({ i: Number(s.dataset.segment), a: Number(s.dataset.startMs), b: Number(s.dataset.endMs) }))`;

// Screenshot with the text block in the middle of the window (the page is long).
async function shotView(file) {
  await ev(`(document.getElementById('tts-readalong') ?? document.getElementById('tts-text')).scrollIntoView({ block: 'center' }); true`);
  await sleep(500);
  await cdp.shot(file);
}

async function generate(lang, voice, text, n) {
  await ev(setSelectJs("tts-lang", lang));
  await waitFor(`document.getElementById('tts-voice') && [...document.getElementById('tts-voice').options].some((o) => o.value === ${JSON.stringify(voice)})`);
  await ev(setSelectJs("tts-voice", voice));
  // The text box is replaced by the read-only view after a generation: go back to editing first.
  if (await ev(`!document.getElementById('tts-text')`)) await ev(clickButtonJs("Edit text"));
  await waitFor(`!!document.getElementById('tts-text')`);
  await ev(setTextJs(text));
  await sleep(200);
  await ev(`document.getElementById('tts-generate').click()`);
  await waitFor(`document.querySelectorAll('[data-segment]').length === ${n}`, 180000);
  await ev(`(document.querySelector('audio').muted = true, true)`);
}

try {
  await send("Page.enable");
  // Start from a fresh page so that no state of an earlier session leaks into the checks.
  await send("Page.reload");
  await sleep(1500);
  await waitFor(`!!document.getElementById('tts-voice') && document.getElementById('tts-voice').options.length > 0`);
  await ev(`window.alert = () => {}; true`);

  // ---- 1. six sentences, Piper English voice ----
  const six = [
    "The first sentence is short.",
    "The second one is a little longer than the first.",
    "Is the third a question?",
    "Dr. Smith takes 500 mg twice a day.",
    "The fifth sentence follows.",
    "And this is the sixth and last sentence.",
  ];
  await generate("en", "tts-vits-piper-en_US-ljspeech-medium:0", six.join(" "), 6);
  check("no sentence is shaded before the first play", (await ev(activeJs)).active.length === 0);
  check("audio is not playing after Generate (no auto-play)", await ev(`document.querySelector('audio').paused`));
  const segs = await ev(segmentsJs);
  check("6 sentence spans, in order", segs.length === 6 && segs.every((s, i) => s.i === i && s.b > s.a));
  const abbreviationKept = await ev(`document.querySelector('[data-segment="3"]').textContent`);
  check("abbreviation 'Dr.' did not split a sentence", abbreviationKept === six[3], abbreviationKept);

  let allMid = true;
  const detail = [];
  for (const s of segs) {
    await ev(seekJs((s.a + s.b) / 2000));
    await sleep(250);
    const st = await ev(activeJs);
    const ok = st.active.length === 1 && st.active[0] === s.i;
    allMid = allMid && ok;
    detail.push(`${s.i}->${st.active}`);
  }
  check("seeking to the middle of each sentence shades exactly that sentence", allMid, detail.join(" "));

  // In the silence after sentence 1 the sentence stays shaded; at the exact start of the next it switches.
  await ev(seekJs((segs[1].b + 100) / 1000));
  await sleep(250);
  check("a sentence stays shaded during the pause after it", (await ev(activeJs)).active[0] === 1);
  await ev(seekJs(segs[2].a / 1000));
  await sleep(250);
  check("the next sentence is shaded from its first millisecond", (await ev(activeJs)).active[0] === 2);

  // Real playback (muted): the shading advances by itself, pause keeps it, Stop clears it.
  await ev(seekJs(0));
  await sleep(200);
  await ev(`(async () => { const a = document.querySelector('audio'); a.muted = true; await a.play(); return !a.paused; })()`, true);
  const seen = new Set();
  const t0 = Date.now();
  while (Date.now() - t0 < 4500) {
    for (const i of (await ev(activeJs)).active) seen.add(i);
    await sleep(60);
  }
  check("during real playback the shading moves through several sentences", seen.size >= 2, [...seen].join(","));
  await ev(`document.querySelector('audio').pause()`);
  await sleep(300);
  const heldA = (await ev(activeJs)).active;
  await sleep(500);
  const heldB = (await ev(activeJs)).active;
  check("pause keeps the shading", heldA.length === 1 && heldB.length === 1 && heldA[0] === heldB[0], `at ${heldA}`);
  await ev(`document.getElementById('tts-stop').click()`);
  await sleep(400);
  check("Stop clears the shading and rewinds", (await ev(activeJs)).active.length === 0 && (await ev(`document.querySelector('audio').currentTime`)) === 0);
  await ev(seekJs((segs[4].a + segs[4].b) / 2000));
  await sleep(250);
  check("seeking after Stop shades the target sentence again", (await ev(activeJs)).active[0] === 4);
  check("aria-current is set on the shaded sentence only", await ev(`document.querySelectorAll('[aria-current]').length === 1`));

  // ---- 2. contrast, light and dark ----
  const colours = `(() => {
    const box = document.getElementById('tts-readalong');
    const cur = document.querySelector('[aria-current="true"]');
    const plain = document.querySelector('[data-segment]:not([aria-current])');
    const cb = getComputedStyle(box), cc = getComputedStyle(cur), cp = getComputedStyle(plain);
    return { boxBg: cb.backgroundColor, boxFg: cb.color, curBg: cc.backgroundColor, curFg: cc.color, plainBg: cp.backgroundColor };
  })()`;
  await shotView(path.join(OUT, "light-6-sentences.png"));
  const light = await ev(colours);
  check("light theme: shaded text contrast >= 7", contrast(light.curFg, light.curBg) >= 7, `${contrast(light.curFg, light.curBg).toFixed(1)} ${JSON.stringify(light)}`);
  check("light theme: the shading differs from the box background", light.curBg !== light.boxBg, `contrast ${contrast(light.curBg, light.boxBg).toFixed(2)}`);
  await send("Emulation.setEmulatedMedia", { features: [{ name: "prefers-color-scheme", value: "dark" }] });
  await sleep(300);
  const dark = await ev(colours);
  await shotView(path.join(OUT, "dark-6-sentences.png"));
  check("dark theme: shaded text contrast >= 7", contrast(dark.curFg, dark.curBg) >= 7, `${contrast(dark.curFg, dark.curBg).toFixed(1)} ${JSON.stringify(dark)}`);
  check("dark theme: plain text contrast >= 7", contrast(dark.boxFg, dark.boxBg) >= 7, contrast(dark.boxFg, dark.boxBg).toFixed(1));
  check("dark theme: the shading differs from the box background", dark.curBg !== dark.boxBg, `contrast ${contrast(dark.curBg, dark.boxBg).toFixed(2)}`);
  await send("Emulation.setEmulatedMedia", { features: [{ name: "prefers-color-scheme", value: "light" }] });

  // ---- 3. edit invalidates the highlight ----
  await ev(clickButtonJs("Edit text"));
  await sleep(300);
  check("'Edit text' brings back the text box and removes the view", await ev(`!!document.getElementById('tts-text') && !document.getElementById('tts-readalong')`));
  await ev(setTextJs(six.join(" ") + " One more."));
  await sleep(200);
  check("a changed text shows the 'generate again' hint", await ev(`document.body.innerText.includes('generate again to get the highlight')`));

  // ---- 4. forty sentences: auto-scroll ----
  const forty = Array.from({ length: 40 }, (_, i) => `This is sentence number ${i + 1} of the long text and it carries a few more words.`).join(" ");
  await generate("en", "tts-vits-piper-en_US-ljspeech-medium:0", forty, 40);
  const long = await ev(segmentsJs);
  const scrollInfo = `(() => {
    const box = document.getElementById('tts-readalong');
    const cur = document.querySelector('[aria-current="true"]');
    const b = box.getBoundingClientRect(), c = cur ? cur.getBoundingClientRect() : null;
    return { top: box.scrollTop, max: box.scrollHeight - box.clientHeight, visible: !!c && c.top >= b.top - 1 && c.bottom <= b.bottom + 1 };
  })()`;
  check("long text: the container is scrollable", (await ev(scrollInfo)).max > 200, JSON.stringify(await ev(scrollInfo)));
  await ev(seekJs((long[0].a + long[0].b) / 2000));
  await sleep(500);
  check("long text: first sentence shaded at scroll position 0", (await ev(scrollInfo)).top === 0);
  // Sentence 5 is already on screen: no scroll needed. Later ones must bring the container down.
  await ev(seekJs((long[5].a + long[5].b) / 2000));
  await sleep(800);
  const near = await ev(scrollInfo);
  check("long text: a sentence already on screen does not move the container", near.top === 0 && near.visible, JSON.stringify(near));
  const marks = [20, 30, 39];
  let lastTop = 0;
  const trace = [];
  let scrollsDown = true;
  for (const m of marks) {
    await ev(seekJs((long[m].a + long[m].b) / 2000));
    await sleep(1200);
    const si = await ev(scrollInfo);
    trace.push(`${m}:top=${Math.round(si.top)}${si.visible ? "" : "(NOT VISIBLE)"}`);
    if (!(si.top > lastTop && si.visible)) scrollsDown = false;
    lastTop = si.top;
  }
  check("long text: the container scrolls down and keeps the shaded sentence visible", scrollsDown, trace.join(" "));
  await shotView(path.join(OUT, "light-40-sentences-scrolled.png"));
  await ev(seekJs((long[2].a + long[2].b) / 2000));
  await sleep(1200);
  const up = await ev(scrollInfo);
  check("long text: seeking back scrolls back up", up.visible && up.top < lastTop / 2, `top=${Math.round(up.top)}`);

  // manual scroll suspends following for a few seconds
  await ev(`document.getElementById('tts-readalong').dispatchEvent(new WheelEvent('wheel', { deltaY: 40, bubbles: true }))`);
  const topBefore = (await ev(scrollInfo)).top;
  await ev(seekJs((long[35].a + long[35].b) / 2000));
  await sleep(1200);
  const topDuring = (await ev(scrollInfo)).top;
  check("after a manual scroll, automatic scrolling is suspended", Math.abs(topDuring - topBefore) < 2, `${Math.round(topBefore)} -> ${Math.round(topDuring)}`);
  await sleep(3500);
  await ev(seekJs((long[37].a + long[37].b) / 2000));
  await sleep(1200);
  const resumed = await ev(scrollInfo);
  check("automatic scrolling resumes after the pause", resumed.visible && resumed.top > topDuring, `top=${Math.round(resumed.top)}`);
  // follow toggle
  await ev(`document.getElementById('tts-follow').click()`);
  await ev(seekJs((long[5].a + long[5].b) / 2000));
  await sleep(1200);
  const off = await ev(scrollInfo);
  check("with 'Follow reading' off, the container does not move", Math.abs(off.top - resumed.top) < 2 && !off.visible, `top=${Math.round(off.top)}`);
  await ev(`document.getElementById('tts-follow').click()`);

  // ---- 5. French character voice: normaliser on by default, toggle, preview ----
  const FR = "tts-vits-coqui-fr-css10:0";
  await ev(clickButtonJs("Edit text"));
  await ev(setSelectJs("tts-lang", "fr"));
  const hasFr = await ev(`!![...document.getElementById('tts-voice').options].some((o) => o.value === ${JSON.stringify(FR)})`);
  if (hasFr) {
    await ev(setSelectJs("tts-voice", FR));
    await sleep(300);
    check("Coqui voice: the rewriting box is checked by default", await ev(`document.getElementById('tts-normalise').checked`));
    await ev(setTextJs("Prendre 500 mg trois fois par jour. Rendez-vous le 12 mars à 9h00."));
    await ev(`document.getElementById('tts-show-spoken').click()`);
    await waitFor(`document.getElementById('tts-spoken') && document.getElementById('tts-spoken').children.length === 2`);
    const sp = await ev(`[...document.getElementById('tts-spoken').children].map((li) => li.textContent)`);
    check("preview shows the rewritten sentences", sp[0] === "Prendre cinq cents milligrammes trois fois par jour." && sp[1] === "Rendez-vous le douze mars à neuf heures.", JSON.stringify(sp));
    await ev(`document.getElementById('tts-normalise').click()`);
    await waitFor(`document.getElementById('tts-spoken').children[0].textContent.includes('500 mg')`);
    check("unchecking shows the original text", true);
    await ev(`document.getElementById('tts-normalise').click()`);
    await ev(`document.getElementById('tts-generate').click()`);
    await waitFor(`document.querySelectorAll('[data-segment]').length === 2`, 120000);
    await ev(`(document.querySelector('audio').muted = true, true)`);
    check("the displayed text keeps the digits", await ev(`document.getElementById('tts-readalong').textContent.includes('500 mg')`));
    check("the result line says the text was rewritten", await ev(`document.body.innerText.includes('text rewritten into words')`));
    await shotView(path.join(OUT, "light-french-normalised.png"));
    const frVoices = await ev(`[...document.getElementById('tts-voice').options].map((o) => o.value)`);
    // A phonemizer voice has the box unchecked by default.
    const piperFr = frVoices.find((v) => v.includes("siwis"));
    if (piperFr) {
      await ev(clickButtonJs("Edit text"));
      await ev(setSelectJs("tts-voice", piperFr));
      await sleep(300);
      check("phonemizer voice: the rewriting box is checked by default too (D-043)", await ev(`document.getElementById('tts-normalise').checked`));
    }
  } else {
    check("Coqui voice installed (needed for the normaliser checks)", false, "voice not installed");
  }
} catch (e) {
  check("test script completed without an exception", false, String(e));
} finally {
  // Clean up: delete only the audio files this run created.
  const created = fs.existsSync(WAV_DIR) ? fs.readdirSync(WAV_DIR).filter((f) => !before.has(f)) : [];
  for (const f of created) fs.rmSync(path.join(WAV_DIR, f), { force: true });
  console.log(`cleanup: removed ${created.length} generated file(s)`);
  cdp.close();
}
const failed = results.filter((r) => !r.ok);
console.log(`\n${results.length - failed.length}/${results.length} checks passed`);
fs.writeFileSync(path.join(OUT, "ui-test-results.json"), JSON.stringify(results, null, 2));
process.exit(failed.length ? 1 : 0);
