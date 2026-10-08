"""Per-sentence synthesis overhead from a `tts measure --compare 1` run.

    python -I -X utf8 scripts/tts_overhead.py benchmark/results/<run>

Reads every measure-*/tts-measure.jsonl of the run. For each voice and text it compares the two modes that
were measured interleaved inside the same process: one library call per sentence (what the app does since
D-042, with the silences added) and one call for the whole text (the behaviour before). Only warm runs
count (the first run of a voice includes loading it). Prints a markdown table and writes overhead.md.
"""
import json
import statistics
import sys
from collections import defaultdict
from pathlib import Path


def main(run: Path) -> None:
    rows = defaultdict(lambda: defaultdict(list))  # (voice, sentence, chars) -> mode -> [(gen_ms, audio_ms, segments)]
    for f in sorted(run.glob("measure-*/tts-measure.jsonl")):
        for line in f.read_text(encoding="utf-8").splitlines():
            r = json.loads(line)
            if r["cold"]:
                continue
            rows[(r["voice"], r["sentence"], r["chars"])][r["mode"]].append((r["generationMs"], r["audioMs"], r["segments"]))
    out = [
        "| Voice | Text (chars) | Sentences | Gen ms whole | Gen ms per-sentence | Overhead | Audio ms whole | Audio ms per-sentence | RTF whole | RTF per-sentence | Warm runs |",
        "|---|---|---|---|---|---|---|---|---|---|---|",
    ]
    for (voice, sentence, chars), modes in sorted(rows.items()):
        if "whole-text" not in modes or "per-sentence" not in modes:
            continue
        w = statistics.median(g for g, _, _ in modes["whole-text"])
        p = statistics.median(g for g, _, _ in modes["per-sentence"])
        wa = statistics.median(a for _, a, _ in modes["whole-text"])
        pa = statistics.median(a for _, a, _ in modes["per-sentence"])
        n = modes["per-sentence"][0][2]
        out.append(
            f"| {voice} | {sentence} ({chars}) | {n} | {w:.0f} | {p:.0f} | {(p - w) / w * 100:+.0f} % | {wa:.0f} | {pa:.0f} | {w / wa:.3f} | {p / pa:.3f} | {len(modes['per-sentence'])} |"
        )
    text = "\n".join(out)
    print(text)
    (run / "overhead.md").write_text(text + "\n", encoding="utf-8")


if __name__ == "__main__":
    main(Path(sys.argv[1]))
