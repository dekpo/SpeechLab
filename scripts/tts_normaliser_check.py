"""Does the text normaliser make a character-based voice speak digits? (rough machine check)

    python -I -X utf8 scripts/tts_normaliser_check.py <stt model id> [repetitions]

For a few French sentences with numbers, dates, times and units it synthesises with the Coqui css10 voice
with the normaliser OFF and ON, transcribes each WAV with the given recogniser and checks whether the key
numbers were heard (as digits or as words). Synthesis is not deterministic (I-056), so each case is
repeated. This is a rough indicator, NOT a measure of naturalness: the recogniser can fail on its own, and
the owner's ear is the judge. Generated WAVs are written to a temporary folder and deleted.
"""
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
BIN = REPO / "src-tauri" / "target" / "release" / "examples"
VOICE = "tts-vits-coqui-fr-css10:0"

# (sentence, [(label, regex of acceptable heard forms)])
CASES = [
    ("Prendre 500 mg trois fois par jour.", [("500", r"\b500\b|cinq[ -]cents?"), ("mg", r"milligramme|\bmg\b")]),
    ("Rendez-vous le 12 mars à 9h00.", [("12", r"\b12\b|douze"), ("9h", r"\b9\s?h|neuf heures?|9 heures")]),
    ("La facture numéro 2045 est payable sous 30 jours.", [("2045", r"2045|2\s?045|deux mille quarante[ -]cinq"), ("30", r"\b30\b|trente")]),
    ("Le patient pèse 72,5 kg et mesure 1,80 m.", [("72,5", r"72[,.]5|soixante[ -]douze virgule cinq"), ("1,80", r"1[,.]80|un virgule quatre[ -]vingt")]),
    ("La tension est de 120/80 mmHg.", [("120", r"\b120\b|cent vingt"), ("80", r"\b80\b|quatre[ -]vingts?")]),
    ("Appelez le 06 12 34 56 78 avant 17 h.", [("06", r"\b06\b|zéro six"), ("17 h", r"17\s?h|dix[ -]sept heures?")]),
]


def say(text_file: Path, flag: str, out: Path) -> Path | None:
    r = subprocess.run([str(BIN / "tts.exe"), "say", VOICE, "fr", "@" + str(text_file), "--normalise", flag, "--out-dir", str(out)],
                       capture_output=True, text=True, encoding="utf-8", errors="replace")
    m = re.search(r"-> (.+\.wav)", r.stdout)
    return Path(m.group(1).strip()) if m else None


def heard(wav: Path, model: str) -> str:
    r = subprocess.run([str(BIN / "transcribe.exe"), "run", model, "fr", str(wav)], capture_output=True, text=True, encoding="utf-8", errors="replace")
    m = re.search(r"^\s*text: (.*)$", r.stdout, re.M)
    return m.group(1).strip() if m else ""


def main(model: str, reps: int) -> None:
    tmp = Path(tempfile.mkdtemp(prefix="speechlab_norm_"))
    totals = {"off": [0, 0], "on": [0, 0]}
    try:
        for i, (sentence, keys) in enumerate(CASES):
            f = tmp / f"case{i}.txt"
            f.write_text(sentence, encoding="utf-8")
            print(f"\n{sentence}")
            for flag in ("off", "on"):
                for rep in range(reps):
                    wav = say(f, flag, tmp / "wav")
                    text = heard(wav, model) if wav else ""
                    found = [label for label, rx in keys if re.search(rx, text, re.I)]
                    totals[flag][0] += len(found)
                    totals[flag][1] += len(keys)
                    print(f"  normaliser {flag:3} #{rep + 1}: {len(found)}/{len(keys)} keys ({', '.join(found) or 'none'})  heard: {text}")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    for flag in ("off", "on"):
        found, total = totals[flag]
        print(f"\nnormaliser {flag}: {found}/{total} key numbers heard ({found / max(1, total) * 100:.0f} %)")


if __name__ == "__main__":
    main(sys.argv[1], int(sys.argv[2]) if len(sys.argv) > 2 else 3)
