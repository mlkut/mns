#!/usr/bin/env python3
"""Phase 2b: pairwise scan of a prefix pool x a suffix pool.

A name word is P+S (prefix-syllable + suffix-syllable). Offensive substrings and
awkward consonant clusters can form at the join (P[3]+S[0]) even when both
tokens are individually clean, so this scans every pair:

  profanity - banned 3-letter / 2-gram substring anywhere in P+S
  boundary  - P[3]+S[0] in the banned boundary 2-gram set

Usage:
  scan.py <prefix-pool.txt> <suffix-pool.txt>     scan one pair
  scan.py --matrix CVCV,CVCC,CVVC                scan every A x B pairing
"""

import itertools
import json
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))
import generate

RULES = json.loads((SCRIPT_DIR / "rules.json").read_text())
SCAN_DIR = generate.FILTERED_DIR.parent / "scans"

AWRK_2 = set(RULES["pronunciation"]["banned_2grams"]) | set(RULES["pronunciation"]["banned_boundary_2grams"])
OFF_3 = set(RULES["profanity"]["banned_substrings_3"])
AWRK_3 = set(RULES["pronunciation"]["banned_3grams"])
PROFANE_WORDS = set(RULES["profanity"]["banned_words"])


def load_pool(path: Path) -> list[str]:
    with open(path) as f:
        return [line.strip() for line in f if line.strip()]


def join_bad(p: str, q: str) -> bool:
    """True if joining token p (prefix) + token q (suffix) is offensive."""
    for g in OFF_3:
        if g[:2] == p[-2:] and q[0] == g[2]:
            return True
        if g[0] == p[-1] and q[:2] == g[1:3]:
            return True
    w = p[-1] + q[:3]
    if w in PROFANE_WORDS:
        return True
    if (p[-2:] + q[:2]) in PROFANE_WORDS:
        return True
    if (p[-3:] + q[0]) in PROFANE_WORDS:
        return True
    return False


def scan_pair(pre_name: str, suf_name: str) -> dict:
    pre_pool = load_pool(generate.FILTERED_DIR / f"{pre_name}.txt")
    suf_pool = load_pool(generate.FILTERED_DIR / f"{suf_name}.txt")
    tag = f"{pre_name}-x-{suf_name}"

    # Per-boundary lookup tables.
    awk2_by_last = {}     # L -> set of suffix first letters F with "LF" in AWRK_2
    off3_by_pre2 = {}     # pre[-2:] -> set of F with "pre2F" in OFF_3
    off_start2_by_last = {}  # L -> set of suffix starts s[:2] with "L"+"s[:2]" in OFF_3
    awk3_by_pre2 = {}     # pre[-2:] -> set of F with "pre2F" in AWRK_3
    awk_start2_by_last = {}  # L -> set of s[:2] with "L"+"s[:2]" in AWRK_3
    for g in OFF_3:
        off3_by_pre2.setdefault(g[:2], set()).add(g[2])
        off_start2_by_last.setdefault(g[0], set()).add(g[1:3])
    for g in AWRK_3:
        awk3_by_pre2.setdefault(g[:2], set()).add(g[2])
        awk_start2_by_last.setdefault(g[0], set()).add(g[1:3])
    for g in AWRK_2:
        awk2_by_last.setdefault(g[0], set()).add(g[1])

    offensive = 0
    awkward = 0
    word_count = 0
    examples = {"offensive": [], "awkward": []}
    for p in pre_pool:
        L = p[-1]
        a2 = awk2_by_last.get(L, ())
        o3 = off3_by_pre2.get(p[-2:], ())
        o3b = off_start2_by_last.get(L, ())
        w3 = awk3_by_pre2.get(p[-2:], ())
        w3b = awk_start2_by_last.get(L, ())
        for s in suf_pool:
            word_count += 1
            pan = p[-1] + s[:3]
            if (s[0] in o3 or s[:2] in o3b or any(w in PROFANE_WORDS for w in (pan, p[-2:] + s[:2], p[-3:] + s[0]))):
                offensive += 1
                if len(examples["offensive"]) < 5:
                    examples["offensive"].append(p + s)
            elif s[0] in a2 or s[0] in w3 or s[:2] in w3b:
                awkward += 1
                if len(examples["awkward"]) < 5:
                    examples["awkward"].append(f"{p}{s}  ({p[-1]}+{s[0]})")

    result = {
        "prefix_pool": pre_name,
        "suffix_pool": suf_name,
        "prefix_count": len(pre_pool),
        "suffix_count": len(suf_pool),
        "pairs_total": word_count,
        "pairs_clean": word_count - offensive - awkward,
        "pairs_offensive": offensive,
        "pairs_awkward": awkward,
        "offense_rate": round(offensive / word_count, 5),
        "examples_offensive": examples["offensive"],
        "examples_awkward": examples["awkward"],
    }
    SCAN_DIR.mkdir(parents=True, exist_ok=True)
    (SCAN_DIR / f"{tag}.json").write_text(json.dumps({tag: result}, indent=2) + "\n")
    return result


def main() -> int:
    if len(sys.argv) >= 3 and sys.argv[1] == "--matrix":
        names = sys.argv[2].split(",")
        total_pairs = 0
        for a, b in itertools.product(names, names):
            r = scan_pair(a, b)
            total_pairs += r["pairs_total"]
            print(f"{a}-x-{b}: clean {r['pairs_clean']:,} / {r['pairs_total']:,}  "
                  f"offensive {r['pairs_offensive']:,}  awkward {r['pairs_awkward']:,}",
                  flush=True)
        print(f"total pairs scanned: {total_pairs:,}")
        return 0
    if len(sys.argv) == 3:
        pre = Path(sys.argv[1])
        suf = Path(sys.argv[2])
        r = scan_pair(pre.with_suffix("").name, suf.with_suffix("").name)
        print(f"{pre.stem}-x-{suf.stem}: clean {r['pairs_clean']:,} / {r['pairs_total']:,}  "
              f"offensive {r['pairs_offensive']:,}  awkward {r['pairs_awkward']:,}")
        return 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    raise SystemExit(main())