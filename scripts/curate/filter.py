#!/usr/bin/env python3
"""Phase 2: filter token pools against scripts/curate/rules.json.

Applies, in order:
  1. exact-match profane word ban
  2. 3-letter profane substring ban (catches embedded profanity)
  3. banned 2-gram substring ban (doubled letters + awkward pairs)
  4. banned 3-gram substring ban
  5. per-pattern onset/coda cluster checks (CCVC/CCVV onsets, CVCC codas,
     VCCV mid-word cluster)

Writes surviving tokens to scripts/data/filtered/<pattern>.txt and a reason
breakdown to scripts/data/filtered/summary.json. Deterministic.

NOTE: boundary rules (token1[3]+token2[0]) are NOT applied here; a token is a
standalone syllable. Pairwise word-level checks live in scan.py.
"""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import generate

RULES_FILE = Path(__file__).resolve().parent / "rules.json"
FILTERED_DIR = generate.DATA_DIR / "filtered"


def load_rules() -> dict:
    with open(RULES_FILE) as f:
        return json.load(f)


def reject_reason(token: str, pattern: str, rules: dict) -> str | None:
    prof = rules["profanity"]
    if token in prof["banned_words"]:
        return "profane word"
    for s in prof["banned_substrings_3"]:
        if s in token:
            return f"profane substring {s!r}"
    pron = rules["pronunciation"]
    for g in pron["banned_2grams"]:
        if g in token:
            return f"banned 2-gram {g!r}"
    for g in pron["banned_3grams"]:
        if g in token:
            return f"banned 3-gram {g!r}"
    if pattern in ("CCVC", "CCVV") and token[:2] not in pron["onset_clusters"]:
        return f"bad onset {token[:2]!r}"
    if pattern == "CVCC" and token[-2:] not in pron["coda_clusters"]:
        return f"bad coda {token[-2:]!r}"
    if pattern == "VCCV" and token[1:3] not in pron["coda_clusters"]:
        return f"bad mid cluster {token[1:3]!r}"
    single = rules["single_letters"]
    if token[0] in single["banned_initial"]:
        return f"banned initial {token[0]!r}"
    if token[-1] in single["banned_final"]:
        return f"banned final {token[-1]!r}"
    return None


def main() -> None:
    rules = load_rules()
    FILTERED_DIR.mkdir(parents=True, exist_ok=True)

    summary = {"alphabet": generate.CHARS, "patterns": {}}
    for name in generate.PATTERNS:
        raw_path = generate.RAW_DIR / f"{name}.txt"
        passed = []
        reasons: dict[str, int] = {}
        with open(raw_path) as f:
            for line in f:
                token = line.strip()
                if not token:
                    continue
                reason = reject_reason(token, name, rules)
                if reason is None:
                    passed.append(token)
                else:
                    reasons[reason] = reasons.get(reason, 0) + 1
        passed.sort()
        with open(FILTERED_DIR / f"{name}.txt", "w") as f:
            if passed:
                f.write("\n".join(passed) + "\n")
        summary["patterns"][name] = {
            "raw": len(generate.generate(name)),
            "filtered": len(passed),
            "rejected": reasons,
        }
        top = ", ".join(f"{k.split()[0]}:{v}" for k, v in sorted(reasons.items(), key=lambda kv: -kv[1])[:4])
        print(f"{name:5s} raw {len(generate.generate(name)):>7}  kept {len(passed):>7}  ({top})")

    with open(FILTERED_DIR / "summary.json", "w") as f:
        json.dump(summary, f, indent=2)
        f.write("\n")
    print(f"summary -> {FILTERED_DIR / 'summary.json'}")


if __name__ == "__main__":
    main()