#!/usr/bin/env python3
"""Whole-clip versus chunked (VAD) transcription of long audio.

Usage:
  python scripts/chunking_study.py runs <whole results folder> <vad results folder> [--min-ref-words N]
  python scripts/chunking_study.py same <results folder A> <results folder B> [--category c1,c2]
  python scripts/chunking_study.py clips <folder of transcribe --out files>

runs   For every configuration (model + decoding) and every sample present in both folders:
       WER, output words versus reference words, repetition loops, a missing ending, inference
       time, segments. Only samples with at least --min-ref-words reference words are listed
       (default 40, which selects the dictations); totals are over those samples.
same   Checks that two result folders have IDENTICAL transcripts (used to show that chunking
       leaves short audio unchanged); prints the number of differing transcripts.
clips  Private clips without a reference: only counts are printed (words, loops, segments,
       time), never the text. Files are named <clip>__<model>__<mode>.json.

Definitions (heuristics, stated so that they can be argued with):
- loop: a group of 1 to 8 words that repeats in a row at least 3 times (4 times for a single word)
  in the output and fewer times in the reference.
- ending missing: fewer than half of the last 8 reference words appear among the last 16 output
  words (checked as a multiset, punctuation and case ignored).
- length ratio: output words / reference words. Below 0.85 suggests dropped speech, above 1.15
  suggests hallucinated or repeated text.
"""
import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path


def words(text):
    return re.findall(r"\w+", text.lower())


def max_repeat(ws):
    """Longest run of an immediately repeated group: (repeats, group length)."""
    best = (1, 0)
    n = len(ws)
    for size in range(1, 9):
        i = 0
        while i + size <= n:
            k = 1
            while ws[i + k * size:i + (k + 1) * size] == ws[i:i + size] and i + (k + 1) * size <= n:
                k += 1
            need = 4 if size == 1 else 3
            if k >= need and k > best[0]:
                best = (k, size)
            i += 1
    return best


def has_loop(out, ref):
    k, size = max_repeat(words(out))
    if size == 0:
        return False
    kr, _ = max_repeat(words(ref))
    return k > kr


def ending_missing(out, ref, tail=8):
    r = words(ref)[-tail:]
    pool = Counter(words(out)[-2 * tail:])
    if not r:
        return False
    found = 0
    for w in r:
        if pool[w] > 0:
            pool[w] -= 1
            found += 1
    return found < len(r) / 2


def load(folder):
    path = Path(folder) / "runs.jsonl"
    rows = [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]
    return [r for r in rows if r["repetition"] == 1]


def key(r):
    return (r["modelId"], r["decoding"], r["sampleId"])


def fmt(v, digits=1):
    return "n/a" if v is None else f"{v:.{digits}f}"


def runs(args):
    min_words = 40
    if "--min-ref-words" in args:
        i = args.index("--min-ref-words")
        min_words = int(args[i + 1])
        del args[i:i + 2]
    whole = {key(r): r for r in load(args[0])}
    vad = {key(r): r for r in load(args[1])}
    keys = sorted(k for k in whole if k in vad and whole[k]["referenceWords"] >= min_words and not whole[k]["error"] and not vad[k]["error"])
    configs = sorted({(k[0], k[1]) for k in keys})
    print(f"samples with >= {min_words} reference words, repetition 1, whole folder {args[0]} / chunked folder {args[1]}\n")
    print("| Configuration | Mode | Samples | WER micro | Runaway (WER>=100%) | Loops | Ending missing | Length ratio (min-max) | Inference s (sum) | Chunking s (sum) | Segments (per sample) |")
    print("|---|---|---|---|---|---|---|---|---|---|---|")
    for model, dec in configs:
        for mode, table in (("whole", whole), ("vad", vad)):
            rs = [table[k] for k in keys if k[0] == model and k[1] == dec]
            errs = sum(r["substitutions"] + r["deletions"] + r["insertions"] for r in rs)
            refw = sum(r["referenceWords"] for r in rs)
            ratios = [len(words(r["text"])) / max(1, r["referenceWords"]) for r in rs]
            print("| {} {} | {} | {} | {} | {} | {} | {} | {}-{} | {} | {} | {} |".format(
                model, dec, mode, len(rs), fmt(100 * errs / refw), sum(1 for r in rs if (r["wer"] or 0) >= 1.0),
                sum(1 for r in rs if has_loop(r["text"], r["reference"])),
                sum(1 for r in rs if ending_missing(r["text"], r["reference"])),
                fmt(min(ratios), 2), fmt(max(ratios), 2),
                fmt(sum(r["inferenceMs"] for r in rs) / 1000), fmt(sum(r.get("chunkingMs", 0) for r in rs) / 1000),
                "/".join(str(r.get("segments", 1)) for r in rs)))
    print("\nPer sample (WER %, words out / reference, loop L, missing ending E):\n")
    for sample in sorted({k[2] for k in keys}):
        print(f"{sample}:")
        for model, dec in configs:
            k = (model, dec, sample)
            cells = []
            for table in (whole, vad):
                r = table[k]
                flags = ("L" if has_loop(r["text"], r["reference"]) else "") + ("E" if ending_missing(r["text"], r["reference"]) else "")
                cells.append("{} % {}/{} {}".format(fmt(100 * (r["wer"] or 0)), len(words(r["text"])), r["referenceWords"], flags).rstrip())
            print(f"  {model} {dec}: whole {cells[0]}  ->  vad {cells[1]}")


def same(args):
    cats = None
    if "--category" in args:
        i = args.index("--category")
        cats = set(args[i + 1].split(","))
        del args[i:i + 2]
    a = {key(r): r for r in load(args[0])}
    b = {key(r): r for r in load(args[1])}
    keys = sorted(k for k in a if k in b and (cats is None or a[k]["category"] in cats))
    both = [k for k in keys if not a[k]["error"] and not b[k]["error"]]
    diff = [k for k in both if a[k]["text"] != b[k]["text"]]
    print(f"compared {len(both)} transcripts present in both folders ({len(keys) - len(both)} skipped for errors); different: {len(diff)}")
    print("segments per record in the second folder:", dict(Counter(b[k].get("segments", 1) for k in both)))
    print("chunking field in the second folder:", dict(Counter(b[k].get("chunking", "off") for k in both)))
    for k in diff[:20]:
        print("  differs:", k)


def clips(args):
    folder = Path(args[0])
    table = defaultdict(dict)
    for f in sorted(folder.glob("*.json")):
        clip, model, mode = f.stem.split("__")
        table[(clip, model)][mode] = json.loads(f.read_text(encoding="utf-8"))
    print("| Clip | Model | Mode | Words | Loop | Segments | Inference s | Chunking s | Audio s |")
    print("|---|---|---|---|---|---|---|---|---|")
    for (clip, model), modes in sorted(table.items()):
        for mode in ("off", "vad"):
            r = modes.get(mode)
            if not r:
                continue
            text = r["text"]
            k, size = max_repeat(words(text))
            loop = f"{k}x{size}w" if size and k >= (4 if size == 1 else 3) else "no"
            print("| {} | {} {} | {} | {} | {} | {} | {} | {} | {} |".format(
                clip, model, r.get("decoding", ""), mode, len(words(text)), loop, r.get("segments", 1),
                fmt(r["processingMs"] / 1000), fmt(r.get("chunkingMs", 0) / 1000), fmt(r["audioMs"] / 1000)))


def main():
    if len(sys.argv) < 3 or sys.argv[1] not in ("runs", "same", "clips"):
        sys.exit(__doc__)
    {"runs": runs, "same": same, "clips": clips}[sys.argv[1]](sys.argv[2:])


if __name__ == "__main__":
    main()
