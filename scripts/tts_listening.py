"""Prepare a blind listening session for the TTS voices.

    python -I -X utf8 scripts/tts_listening.py            # generate missing audio, then write the page
    python -I -X utf8 scripts/tts_listening.py --force    # regenerate all audio

Writes benchmark/tts-samples/<voice label>/<lang>-<n>-speed<s>.wav (git-ignored) and
benchmark/tts-samples/index.html: one block per voice with its players and a small form. Voices are
shown under anonymous codes (A, B, C ...) in a fixed pseudo-random order so that the name, the engine
and the licence rating do not influence the notes; the "Copy my notes" button produces a text that
contains the codes AND the real voice names, to paste into the chat. Notes are kept in the browser
(localStorage) while the page is open. The page needs no network.
"""
import json
import random
import re
import shutil
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
TTS = REPO / "src-tauri" / "target" / "release" / "examples" / "tts.exe"
OUT = REPO / "benchmark" / "tts-samples"
TEXTS = REPO / "benchmark" / "tts" / "listening"
MANIFEST = REPO / "src-tauri" / "models-manifest.json"

# (label, voice id, language)
VOICES = [
    ("fr-coqui-css10", "tts-vits-coqui-fr-css10:0", "fr"),
    ("fr-piper-siwis", "tts-vits-piper-fr_FR-siwis-medium:0", "fr"),
    ("fr-piper-gilles-low", "tts-vits-piper-fr_FR-gilles-low:0", "fr"),
    ("fr-kokoro-ff_siwis", "tts-kokoro-int8-multi-lang-v1_0:30", "fr"),
    ("en-piper-ljspeech", "tts-vits-piper-en_US-ljspeech-medium:0", "en"),
    ("en-piper-libritts_r-spk0", "tts-vits-piper-en_US-libritts_r-medium:0", "en"),
    ("en-kokoro-af_heart", "tts-kokoro-int8-multi-lang-v1_0:3", "en"),
    ("en-kokoro-am_adam", "tts-kokoro-int8-multi-lang-v1_0:11", "en"),
]
FILES = [(1, 1.0), (2, 1.0), (3, 1.0), (1, 1.5)]  # (sentence number, speed)
CODES = "ABCDEFGHIJKLMNOP"


def generate(force: bool) -> None:
    tmp = OUT / "_tmp"
    for label, voice, lang in VOICES:
        for n, speed in FILES:
            dest = OUT / label / f"{lang}-{n}-speed{speed}.wav"
            if dest.exists() and not force:
                continue
            txt = TEXTS / f"{lang}-{n}.txt"
            r = subprocess.run([str(TTS), "say", voice, lang, "@" + str(txt), "--speed", str(speed), "--out-dir", str(tmp)],
                               capture_output=True, text=True, encoding="utf-8", errors="replace")
            m = re.search(r"-> (.+\.wav)", r.stdout)
            if not m:
                print("FAILED", label, n, speed, r.stderr[-200:])
                continue
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.move(m.group(1).strip(), dest)
    shutil.rmtree(tmp, ignore_errors=True)


def tiers() -> dict:
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    return {m["id"]: m.get("licenseTier", "unrated") for m in manifest}


def page() -> None:
    tier_of = tiers()
    rng = random.Random(20261008)
    data = []
    for lang in ("fr", "en"):
        group = [v for v in VOICES if v[2] == lang]
        rng.shuffle(group)
        for label, voice, _ in group:
            data.append({"label": label, "voice": voice, "lang": lang, "tier": tier_of.get(voice.rsplit(":", 1)[0], "unrated")})
    # One alphabet over both languages so every code is unique.
    for i, d in enumerate(data):
        d["code"] = CODES[i]
    sentences = {f"{l}-{n}": (TEXTS / f"{l}-{n}.txt").read_text(encoding="utf-8").strip() for l in ("fr", "en") for n in (1, 2, 3)}
    html = TEMPLATE.replace("__DATA__", json.dumps({"voices": data, "sentences": sentences, "files": FILES}, ensure_ascii=False))
    (OUT / "index.html").write_text(html, encoding="utf-8")
    print("page written:", OUT / "index.html")


TEMPLATE = """<!doctype html>
<html lang="fr"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Écoute des voix / Voice listening</title>
<style>
:root { --bg:#fafafa; --fg:#1d1d1f; --card:#fff; --line:#d9d9de; --acc:#2557a7; }
@media (prefers-color-scheme: dark) { :root { --bg:#17181a; --fg:#eceef1; --card:#202226; --line:#3a3d44; --acc:#8ab4f8; } }
body { margin:0; background:var(--bg); color:var(--fg); font:16px/1.5 system-ui,sans-serif; }
main { max-width:860px; margin:0 auto; padding:16px; }
section.voice { background:var(--card); border:1px solid var(--line); border-radius:8px; padding:12px 14px; margin:14px 0; }
h1 { font-size:1.3rem; } h2 { font-size:1.1rem; margin:22px 0 4px; } h3 { margin:0 0 6px; font-size:1.05rem; }
.sent { margin:8px 0 2px; font-size:.92rem; color:inherit; opacity:.85; }
audio { width:100%; max-width:520px; } label { display:block; margin:8px 0 2px; font-size:.92rem; }
select, textarea { font:inherit; max-width:100%; } textarea { width:100%; box-sizing:border-box; }
button { font:inherit; padding:6px 12px; margin:4px 4px 4px 0; cursor:pointer; }
.rate { display:flex; flex-wrap:wrap; gap:12px; } .rate div { min-width:150px; }
.tip { border-left:3px solid var(--acc); padding:4px 10px; margin:10px 0; }
#out { white-space:pre-wrap; background:var(--card); border:1px solid var(--line); padding:10px; border-radius:6px; }
</style></head><body><main>
<h1>Écoute des voix de synthèse / TTS voice listening</h1>
<div class="tip">
<p><strong>Mode d'emploi.</strong> Mettez un casque, même volume pour tout. Pour chaque voix (anonymisée : A, B, C…),
écoutez les 4 extraits (appuyez sur lecture, rien ne démarre seul), puis remplissez la petite fiche.
Ne cherchez pas à deviner la voix : jugez ce que vous entendez. Quand toutes les fiches sont faites, cliquez sur
« Copier mes notes » en bas et collez le texte dans le chat. Si vous ne savez pas, laissez « — ».</p>
<p>Extrait 1 = phrase simple avec un médicament et des nombres ; extrait 2 = date, heure et un autre médicament ;
extrait 3 = trois noms de médicaments ; extrait 4 = l'extrait 1 accéléré (vitesse 1,5).</p>
</div>
<div id="root"></div>
<h2>Fin : vos notes / Your notes</h2>
<label for="global">Remarque générale (facultatif) : quelle voix choisiriez-vous pour lire un compte rendu à un patient ?</label>
<textarea id="global" rows="3"></textarea>
<p><button id="copy">Copier mes notes / Copy my notes</button> <button id="reset">Tout effacer / Clear all</button></p>
<pre id="out"></pre>
</main>
<script>
const D = __DATA__;
const KEY = "speechlab-tts-listening-v1";
let state = {};
try { state = JSON.parse(localStorage.getItem(KEY) || "{}"); } catch (e) { state = {}; }
function save() { try { localStorage.setItem(KEY, JSON.stringify(state)); } catch (e) {} }
const SCALE = [["", "—"], ["1", "1 très mauvais"], ["2", "2 mauvais"], ["3", "3 moyen"], ["4", "4 bon"], ["5", "5 très bon"]];
const DRUGS = [["", "—"], ["ok", "bien prononcés"], ["approx", "approximatifs mais reconnaissables"], ["faux", "faux ou incompréhensibles"]];
const FIELDS = [["clarity", "Clarté (comprend-on chaque mot ?)", SCALE], ["natural", "Naturel (voix humaine ou robot ?)", SCALE],
  ["accent", "Accent / prononciation de la langue", SCALE], ["drugs", "Noms de médicaments", DRUGS], ["speed", "Extrait 4 (vitesse 1,5) reste agréable ?", SCALE]];
const root = document.getElementById("root");
for (const lang of ["fr", "en"]) {
  const h = document.createElement("h2"); h.textContent = lang === "fr" ? "Voix françaises" : "Voix anglaises (English voices)"; root.appendChild(h);
  for (const v of D.voices.filter(x => x.lang === lang)) {
    const s = document.createElement("section"); s.className = "voice";
    let html = "<h3>Voix " + v.code + "</h3>";
    for (const [n, sp] of D.files) {
      const key = lang + "-" + n;
      html += '<div class="sent">Extrait ' + (sp === 1.5 ? 4 : n) + (sp === 1.5 ? " (vitesse 1,5)" : "") + " : " + (D.sentences[key] || "") + "</div>";
      html += '<audio controls preload="none" src="' + v.label + "/" + lang + "-" + n + "-speed" + sp.toFixed(1) + '.wav"></audio>';
    }
    html += '<div class="rate">';
    for (const [f, label, opts] of FIELDS) {
      html += "<div><label>" + label + '</label><select data-v="' + v.code + '" data-f="' + f + '">' + opts.map(o => '<option value="' + o[0] + '">' + o[1] + "</option>").join("") + "</select></div>";
    }
    html += '</div><label>Commentaire libre (ce qui vous gêne, ce que vous aimez)</label><textarea rows="2" data-v="' + v.code + '" data-f="comment"></textarea>';
    s.innerHTML = html; root.appendChild(s);
  }
}
document.querySelectorAll("[data-v]").forEach(el => {
  const v = el.dataset.v, f = el.dataset.f;
  el.value = (state[v] && state[v][f]) || "";
  el.addEventListener("input", () => { state[v] = state[v] || {}; state[v][f] = el.value; save(); render(); });
});
const g = document.getElementById("global"); g.value = state._global || "";
g.addEventListener("input", () => { state._global = g.value; save(); render(); });
function label(opts, val) { const o = opts.find(x => x[0] === val); return o && val ? o[1] : "—"; }
function render() {
  let t = "NOTES D'ÉCOUTE (voix anonymisées pendant l'écoute)\\n";
  for (const v of D.voices) {
    const s = state[v.code] || {};
    t += "\\nVoix " + v.code + " = " + v.label + " [licence: " + v.tier + "]\\n";
    for (const [f, lab, opts] of FIELDS) t += "  - " + lab + " : " + label(opts, s[f]) + "\\n";
    t += "  - Commentaire : " + (s.comment || "—") + "\\n";
  }
  t += "\\nRemarque générale : " + (state._global || "—") + "\\n";
  document.getElementById("out").textContent = t; return t;
}
document.getElementById("copy").onclick = async () => {
  const t = render();
  try { await navigator.clipboard.writeText(t); alert("Notes copiées. Collez-les dans le chat."); }
  catch (e) { alert("Copie automatique impossible : sélectionnez le texte affiché en bas et copiez-le (Ctrl+C)."); }
};
document.getElementById("reset").onclick = () => { if (confirm("Tout effacer ?")) { state = {}; save(); location.reload(); } };
render();
</script></body></html>
"""

if __name__ == "__main__":
    generate("--force" in sys.argv)
    page()
