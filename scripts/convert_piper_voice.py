"""Turn a Piper voice (model.onnx + model.onnx.json) into a package the sherpa-onnx text-to-speech API reads.

    vendor/py-convert/Scripts/python.exe -I -X utf8 scripts/convert_piper_voice.py convert \
        --onnx vendor/piper-mls-source/fr_FR-mls-medium.onnx --json vendor/piper-mls-source/fr_FR-mls-medium.onnx.json \
        --card vendor/piper-mls-source/MODEL_CARD --espeak-data <models>/vits-piper-en_US-kristin-medium/espeak-ng-data \
        --out-dir <models>/vits-piper-fr_FR-mls-medium

    vendor/py-convert/Scripts/python.exe -I -X utf8 scripts/convert_piper_voice.py check --package <models>/vits-piper-en_US-kristin-medium

What the conversion does (the procedure of the sherpa-onnx documentation for Piper voices): writes
`tokens.txt` from the `phoneme_id_map` of the JSON, adds the metadata keys sherpa-onnx looks for to a COPY
of the .onnx (model_type, comment=piper, language, voice, has_espeak, n_speakers, sample_rate), and copies
the shared espeak-ng data folder. Only the `onnx` package is needed (installed in the git-ignored
`vendor/py-convert` virtual environment). The source file is never modified.

`check` applies the same two steps to the JSON of a package that sherpa-onnx already converted and reports
whether `tokens.txt` and the metadata come out identical: that is how the procedure was verified here.
"""
import argparse
import hashlib
import json
import shutil
import sys
from pathlib import Path

import onnx


def build_tokens(config: dict) -> str:
    # One line per symbol: "<symbol> <first id>" (the symbol may be a space).
    return "".join(f"{symbol} {ids[0]}\n" for symbol, ids in config["phoneme_id_map"].items())


def build_metadata(config: dict) -> dict:
    return {
        "model_type": "vits",
        "comment": "piper",  # must be "piper" for voices that come from Piper
        "language": config["language"]["name_english"],
        "voice": config["espeak"]["voice"],  # e.g. "fr"
        "has_espeak": 1,
        "n_speakers": config["num_speakers"],
        "sample_rate": config["audio"]["sample_rate"],
    }


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def convert(args: argparse.Namespace) -> None:
    src, cfg_path, out = Path(args.onnx), Path(args.json), Path(args.out_dir)
    config = json.loads(cfg_path.read_text(encoding="utf-8"))
    if out.exists():
        sys.exit(f"{out} already exists: remove it first (nothing was changed)")
    out.mkdir(parents=True)
    target = out / src.name
    shutil.copyfile(src, target)
    model = onnx.load(str(target))
    for key, value in build_metadata(config).items():
        prop = model.metadata_props.add()
        prop.key = key
        prop.value = str(value)
    onnx.save(model, str(target))
    (out / "tokens.txt").write_text(build_tokens(config), encoding="utf-8", newline="\n")
    shutil.copyfile(cfg_path, out / cfg_path.name)
    if args.card:
        shutil.copyfile(args.card, out / "MODEL_CARD")
    shutil.copytree(args.espeak_data, out / "espeak-ng-data")
    (out / ".speechlab-installed").write_text(f"sha256={sha256(src)}\nconverted-by=scripts/convert_piper_voice.py\n", encoding="utf-8")
    print(f"converted {src.name}: {target.stat().st_size} bytes, {len(config['phoneme_id_map'])} tokens, "
          f"{config['num_speakers']} speaker(s), {config['audio']['sample_rate']} Hz -> {out}")


def check(args: argparse.Namespace) -> None:
    pkg = Path(args.package)
    cfg_path = next(pkg.glob("*.onnx.json"))
    model_path = cfg_path.with_suffix("")  # drops ".json"
    config = json.loads(cfg_path.read_text(encoding="utf-8"))
    ok = True
    official_tokens = (pkg / "tokens.txt").read_text(encoding="utf-8")
    same_tokens = official_tokens == build_tokens(config)
    print(f"tokens.txt identical to the official one: {same_tokens} ({len(official_tokens.splitlines())} lines)")
    ok &= same_tokens
    model = onnx.load(str(model_path), load_external_data=False)
    official_meta = {p.key: p.value for p in model.metadata_props}
    wanted = {k: str(v) for k, v in build_metadata(config).items()}
    for key, value in wanted.items():
        match = official_meta.get(key) == value
        print(f"  metadata {key}: ours={value!r} official={official_meta.get(key)!r} {'OK' if match else 'DIFFERENT'}")
        ok &= match
    sys.exit(0 if ok else 1)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="cmd", required=True)
    c = sub.add_parser("convert")
    c.add_argument("--onnx", required=True)
    c.add_argument("--json", required=True)
    c.add_argument("--card")
    c.add_argument("--espeak-data", required=True)
    c.add_argument("--out-dir", required=True)
    c.set_defaults(func=convert)
    k = sub.add_parser("check")
    k.add_argument("--package", required=True)
    k.set_defaults(func=check)
    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
