#!/usr/bin/env python3
"""Verify the final curated lists are well-formed and duplicate-free.

Checks:
  1. exact size (4096) and uniqueness WITHIN each list
  2. every token 4 chars, lowercase, CVCV, and in the allowed alphabet
  3. no token contains a banned substring from rules.json (e.g. "uw")
  4. no full-reduplicated tokens (kaka / wuwu) per rules.json
  5. cross-slot overlap is reported (shared tokens are allowed by design)

Exit code 0 = pass. Usage: verify.py [prefixes.txt suffixes.txt]
"""
import json
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))
import generate

FINAL_DIR = generate.FILTERED_DIR.parent / "final"
CONS = generate.CONSONANTS
VOW = generate.VOWELS
ALPHA = set(CONS + VOW)


def load(path: Path):
    return [l.strip() for l in path.read_text().splitlines() if l.strip()]


def check_list(name: str, toks: list[str], errors: list[str]) -> None:
    if len(toks) != 4096:
        errors.append(f"{name}: expected 4096 tokens, got {len(toks)}")
    first = {}
    for i, t in enumerate(toks):
        if t in first:
            errors.append(f"{name}: DUPLICATE token '{t}' (indexes {first[t]} and {i})")
        else:
            first[t] = i
        if len(t) != 4:
            errors.append(f"{name}[{i}]: bad length {len(t)}: '{t}'")
        elif not (t[0] in CONS and t[1] in VOW and t[2] in CONS and t[3] in VOW):
            errors.append(f"{name}[{i}]: not CVCV / not in alphabet: '{t}'")
        if t[0] == t[2] and t[1] == t[3]:
            errors.append(f"{name}[{i}]: reduplicated token: '{t}'")


def resolve_arg(name: str) -> Path:
    p = Path(name)
    if p.is_absolute() or (Path.cwd() / p).exists():
        return p
    return FINAL_DIR / p


def main() -> int:
    a, b = sys.argv[1:3] if len(sys.argv) >= 3 else ("prefixes.txt", "suffixes.txt")
    p_path = resolve_arg(a)
    s_path = resolve_arg(b)

    rules = json.loads((SCRIPT_DIR / "rules.json").read_text())
    bans = (rules["profanity"]["banned_substrings_3"]
            + rules["pronunciation"]["banned_2grams"]
            + rules["pronunciation"]["banned_3grams"])

    prefixes = load(p_path)
    suffixes = load(s_path)
    errors: list[str] = []

    check_list(a, prefixes, errors)
    check_list(b, suffixes, errors)

    for i, t in enumerate(prefixes):
        for x in bans:
            if x in t:
                errors.append(f"{a}[{i}]: banned substring '{x}' in '{t}'")
    for i, t in enumerate(suffixes):
        for x in bans:
            if x in t:
                errors.append(f"{b}[{i}]: banned substring '{x}' in '{t}'")

    shared = len(set(prefixes) & set(suffixes))
    print(f"{a}: {len(prefixes):,} tokens, {len(set(prefixes)):,} unique")
    print(f"{b}: {len(suffixes):,} tokens, {len(set(suffixes)):,} unique")
    print(f"cross-slot shared (allowed overlap): {shared:,}")
    print(f"word redup rate: ~1 in {4096**2 // max(shared, 1):,}")

    if errors:
        for e in errors[:30]:
            print("FAIL:", e)
        print(f"verify: {len(errors)} error(s)")
        return 1
    print("verify: PASS (no duplicates, well-formed, no banned/reduplicated tokens)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())