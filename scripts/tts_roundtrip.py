"""Machine intelligibility check of generated speech: text -> voice -> speech-to-text -> compare.

    python scripts/tts_roundtrip.py <samples folder> <stt model id>

For each `<voice>/<lang>-<n>-speed1.0.wav` it runs the `transcribe` example and prints what was heard
next to a plain word error rate (lower case, punctuation removed; numbers are NOT folded, so "500" against
"cinq cents" counts as an error and the rate is inflated: read the heard text, not the percentage).
This is NOT a naturalness measure and it inherits the recogniser's own mistakes (drug names are often
misspelled even for human speech); it only shows whether the speech is understandable to a machine.
"""
import re
import subprocess
import sys
import unicodedata
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
TRANSCRIBE = REPO / "src-tauri" / "target" / "release" / "examples" / "transcribe.exe"


def words(text: str) -> list[str]:
    text = unicodedata.normalize("NFC", text).lower()
    return re.findall(r"[\w']+", text.replace("’", "'"))


def wer(ref: list[str], hyp: list[str]) -> float:
    d = list(range(len(hyp) + 1))
    for i, r in enumerate(ref, 1):
        prev, d[0] = d[0], i
        for j, h in enumerate(hyp, 1):
            cur = min(d[j] + 1, d[j - 1] + 1, prev + (r != h))
            prev, d[j] = d[j], cur
    return d[len(hyp)] / max(1, len(ref))


def main(folder: Path, model: str) -> None:
    total_err = total_words = 0.0
    for wav in sorted(folder.glob("*/*-speed1.0.wav")):
        lang, n = wav.name.split("-")[:2]
        ref = (REPO / "benchmark" / "tts" / "listening" / f"{lang}-{n}.txt").read_text(encoding="utf-8").strip()
        proc = subprocess.run([str(TRANSCRIBE), "run", model, lang, str(wav)], capture_output=True, text=True, encoding="utf-8", errors="replace")
        m = re.search(r"^\s*text: (.*)$", proc.stdout, re.M)
        hyp = m.group(1).strip() if m else ""
        r, h = words(ref), words(hyp)
        e = wer(r, h)
        total_err += e * len(r)
        total_words += len(r)
        print(f"{wav.parent.name:24} {wav.name:22} WER {e*100:5.1f} %  heard: {hyp}")
    print(f"\nmicro WER over {int(total_words)} reference words: {total_err / max(1, total_words) * 100:.1f} %")


if __name__ == "__main__":
    main(Path(sys.argv[1]), sys.argv[2])
