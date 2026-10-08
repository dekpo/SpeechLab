"""Clarity scan of the speakers of a multi-speaker voice (machine proxy, to shortlist what a human then hears).

    python -I -X utf8 scripts/tts_speaker_scan.py <voice package id> <lang> <speaker count> <stt model id> <out.csv>

Synthesises two fixed sentences with every speaker, transcribes them with the given recogniser and records the
word error rate. A low rate means the recogniser understood the speaker; it says NOTHING about naturalness,
accent or pleasantness, and it inherits the recogniser's own mistakes. Use it to drop unintelligible speakers
and to pick a short list for a blind listening session. Generated audio is deleted.
"""
import csv
import re
import shutil
import subprocess
import sys
import tempfile
import unicodedata
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
BIN = REPO / "src-tauri" / "target" / "release" / "examples"
TEXTS = {
    "fr": "Le dossier est incomplet, il manque le justificatif de revenus. Quel jour sommes-nous aujourd'hui ?",
    "en": "The file is incomplete, the proof of income is missing. Which day is it today?",
}


def words(text: str) -> list[str]:
    return re.findall(r"[\w']+", unicodedata.normalize("NFC", text).lower().replace("’", "'"))


def wer(ref: list[str], hyp: list[str]) -> float:
    d = list(range(len(hyp) + 1))
    for i, r in enumerate(ref, 1):
        prev, d[0] = d[0], i
        for j, h in enumerate(hyp, 1):
            cur = min(d[j] + 1, d[j - 1] + 1, prev + (r != h))
            prev, d[j] = d[j], cur
    return d[len(hyp)] / max(1, len(ref))


def main(voice: str, lang: str, count: int, model: str, out_csv: Path) -> None:
    tmp = Path(tempfile.mkdtemp(prefix="speechlab_scan_"))
    text = TEXTS[lang]
    ref = words(text)
    rows = []
    try:
        for sid in range(count):
            r = subprocess.run([str(BIN / "tts.exe"), "say", f"{voice}:{sid}", lang, text, "--out-dir", str(tmp)],
                               capture_output=True, text=True, encoding="utf-8", errors="replace")
            m = re.search(r"-> (.+\.wav)", r.stdout)
            rtf = re.search(r"RTF ([0-9.]+)", r.stdout)
            if not m:
                rows.append([sid, "", "", "", "synthesis failed"])
                continue
            wav = Path(m.group(1).strip())
            t = subprocess.run([str(BIN / "transcribe.exe"), "run", model, lang, str(wav)], capture_output=True, text=True, encoding="utf-8", errors="replace")
            hm = re.search(r"^\s*text: (.*)$", t.stdout, re.M)
            heard = hm.group(1).strip() if hm else ""
            rows.append([sid, f"{wer(ref, words(heard)) * 100:.1f}", rtf.group(1) if rtf else "", heard, ""])
            wav.unlink(missing_ok=True)
            print(sid, rows[-1][1], heard[:70], flush=True)
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    with out_csv.open("w", encoding="utf-8", newline="") as f:
        w = csv.writer(f)
        w.writerow(["speaker", "wer_percent", "rtf_cold_or_warm", "heard", "note"])
        w.writerows(rows)
    scored = sorted((float(r[1]), r[0]) for r in rows if r[1] != "")
    print("\nbest 12 (wer %, speaker):", scored[:12])
    print("worst 5:", scored[-5:])
    print("median WER %:", scored[len(scored) // 2][0])


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2], int(sys.argv[3]), sys.argv[4], Path(sys.argv[5]))
