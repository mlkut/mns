#!/usr/bin/env python3
"""Phase 1: enumerate every 4-letter token for each plausible C/V pattern.

Writes one file per pattern to scripts/data/raw/<pattern>.txt (one token per
line, sorted) and a combined counts.json manifest. Deterministic.
"""
import itertools
import json
from pathlib import Path

CONSONANTS = "bdfghjklmnprstvwz"
VOWELS = "aeiou"

# pattern name -> per-position letter class
PATTERNS = {
    "CVCV": ("C", "V", "C", "V"),
    "CVCC": ("C", "V", "C", "C"),
    "CCVC": ("C", "C", "V", "C"),
    "CVVC": ("C", "V", "V", "C"),
    "VCVC": ("V", "C", "V", "C"),
    "VCCV": ("V", "C", "C", "V"),
    "CCVV": ("C", "C", "V", "V"),
}

CHARS = {"C": CONSONANTS, "V": VOWELS}

DATA_DIR = Path(__file__).resolve().parent.parent / "data"
RAW_DIR = DATA_DIR / "raw"
FILTERED_DIR = DATA_DIR / "filtered"
OUT_MANIFEST = RAW_DIR / "counts.json"


def generate(pattern: str) -> list[str]:
    classes = PATTERNS[pattern]
    pools = [CHARS[k] for k in classes]
    return ["".join(t) for t in itertools.product(*pools)]


def main() -> None:
    RAW_DIR.mkdir(parents=True, exist_ok=True)
    counts = {}
    total = 0
    for name in PATTERNS:
        tokens = generate(name)
        tokens.sort()
        counts[name] = len(tokens)
        total += len(tokens)
        with open(RAW_DIR / f"{name}.txt", "w") as f:
            f.write("\n".join(tokens) + "\n")
        print(f"{name:5s} {len(tokens):>7}")

    manifest = {
        "alphabet": {"consonants": CONSONANTS, "vowels": VOWELS},
        "patterns": {k: PATTERNS[k] for k in PATTERNS},
        "counts": counts,
        "total": total,
    }
    with open(OUT_MANIFEST, "w") as f:
        json.dump(manifest, f, indent=2)
        f.write("\n")
    print(f"total {total:>7} -> {RAW_DIR}")


if __name__ == "__main__":
    main()