"""Summarise a `tts measure` result folder (one sub-folder per voice, each with tts-measure.jsonl).

    python scripts/tts_summary.py benchmark/results/<stamp>-tts-m6

Warm figures exclude the first repetition of the first sentence (cold start: it includes nothing
but the voice is loaded before it, so it is reported separately as `load ms`). RTF = generation time
divided by audio duration (below 1 is faster than real time). Prints a Markdown table and writes it
to summary.md in the folder.
"""
import json
import statistics
import sys
from pathlib import Path


def main(folder: Path) -> None:
    rows = []
    for sub in sorted(p for p in folder.iterdir() if p.is_dir()):
        file = sub / "tts-measure.jsonl"
        if not file.exists():
            continue
        records = [json.loads(line) for line in file.read_text(encoding="utf-8").splitlines() if line.strip()]
        voice = records[0]["voice"]
        lang = records[0]["lang"]
        load = max(r["loadMs"] for r in records)
        peak = max((r["peakMemoryMb"] or 0) for r in records)
        sample_rate = records[0]["sampleRate"]
        by_sentence = {}
        for r in records:
            if r["cold"]:
                continue  # the very first generation after loading is excluded from warm figures
            by_sentence.setdefault(r["sentence"], []).append(r)
        for n, rs in sorted(by_sentence.items()):
            gens = [r["generationMs"] for r in rs]
            rtfs = [r["rtf"] for r in rs]
            rows.append({
                "voice": voice, "lang": lang, "sentence": n, "chars": rs[0]["chars"],
                "audio": rs[0]["audioMs"], "gen": statistics.median(gens),
                "spread": (max(gens) - min(gens)) / statistics.median(gens) * 100 if len(gens) > 1 else 0.0,
                "rtf": statistics.median(rtfs), "runs": len(rs), "load": load, "peak": peak, "rate": sample_rate,
            })
    md = ["| Voice | Lang | Sentence | Chars | Audio ms | Gen ms (median) | Spread % | RTF (median) | Warm runs |",
          "|---|---|---|---|---|---|---|---|---|"]
    for r in rows:
        md.append(f"| {r['voice']} | {r['lang']} | {r['sentence']} | {r['chars']} | {r['audio']} | {r['gen']:.0f} | {r['spread']:.0f} | {r['rtf']:.3f} | {r['runs']} |")
    md += ["", "| Voice | Cold load ms | Peak memory MB (process) | Sample rate Hz |", "|---|---|---|---|"]
    seen = set()
    for r in rows:
        if r["voice"] in seen:
            continue
        seen.add(r["voice"])
        md.append(f"| {r['voice']} | {r['load']} | {r['peak']:.0f} | {r['rate']} |")
    text = "\n".join(md) + "\n"
    print(text)
    (folder / "summary.md").write_text(text, encoding="utf-8")


if __name__ == "__main__":
    main(Path(sys.argv[1]))
