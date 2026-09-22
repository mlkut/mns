#!/usr/bin/env python3
"""Phase 5: sample names from the curated final pools, biased to an ordinal range.

The deployed system caps ordinals well below 2^48 (contract cap planned for a
trillion, later). Early ordinals are what humans actually register and see, so
this renders the REAL ordinal -> name mapping (same Feistel permutation shape as
name.rs, 48-bit value split into 4 x 12-bit slot indexes into the final lists)
and samples ordinals uniformly from [0, cap).

  ordinal -> 48-bit value (Feistel) -> 4 x 12-bit slots -> final lists

Usage:
  sample.py                              # default: 100 from the first trillion
  sample.py --preset first-million
  sample.py --preset first-billion
  sample.py --preset first-trillion
  sample.py --full                       # whole 2^48 space
  sample.py --cap 5000000 --count 50 --seed 7
  sample.py --ordinals 0,1,2,999999,1000000
  sample.py --compare                    # same ordinals through new vs status quo
"""
import argparse
import random
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))
import generate

FINAL_DIR = generate.FILTERED_DIR.parent / "final"

BITS = 48
MASK_48 = (1 << BITS) - 1
HALF = BITS // 2
MASK_HALF = (1 << HALF) - 1
# Mirrors name.rs's ROUND_KEYS exactly (24-bit odd values; 6 rounds). These
# are derived from successive 24-bit windows of the golden-ratio constant
# 0x9E3779B97F4A7C15, forced odd — see name.rs. Round keys MUST stay odd,
# or the permutation develops a measurable linear bias. Keep in sync with
# mns/src/name.rs.
R = [0x4a7c15, 0x7f4a7d, 0xb97f4b, 0x79b97f, 0x3779b9, 0x9e3779]


def round_f(val: int, r: int) -> int:
    return (val * r ^ (val >> 7) ^ (val << 13)) & MASK_HALF


def permute(ordinal: int) -> int:
    """Bijective 48-bit permutation, mirroring name.rs's 4-round Feistel."""
    x = (ordinal + 1) & MASK_48
    left = (x >> HALF) & MASK_HALF
    right = x & MASK_HALF
    for r in R:
        nxt = left ^ round_f(right, r)
        left, right = right, nxt
    return (left << HALF) | right


def unpermute(x: int) -> int:
    left = (x >> HALF) & MASK_HALF
    right = x & MASK_HALF
    for r in reversed(R):
        prev_left = right ^ round_f(left, r)
        right, left = left, prev_left
    return ((left << HALF) | right - 1) & MASK_48


def load(name: str) -> list[str]:
    return [line.strip() for line in open(FINAL_DIR / f"{name}.txt") if line.strip()]


def ordinal_to_name(ordinal: int, pre: list[str], suf: list[str]) -> str:
    v = permute(ordinal)
    hi = (v >> HALF) & MASK_HALF
    lo = v & MASK_HALF
    p1, s1 = (hi >> 12) & 0xFFF, hi & 0xFFF
    p2, s2 = (lo >> 12) & 0xFFF, lo & 0xFFF
    return f"{pre[p1]}{suf[s1]}-{pre[p2]}{suf[s2]}"


def preset_cap(name: str) -> int:
    return {"first-million": 10**6, "first-billion": 10**9,
            "first-trillion": 10**12}[name]


# ---- status quo (committed name.rs) rendering ----

MASK40 = (1 << 40) - 1
R20 = [0x9E377, 0x6C62D, 0xB5A4B, 0xD2F3E]  # name.rs constants, 20-bit halves


def permute40(ordinal: int) -> int:
    hm = 0xF_FFFF
    x = (ordinal + 1) & MASK40
    left = (x >> 20) & hm
    right = x & hm
    for r in R20:
        nxt = left ^ (((right * r) ^ (right >> 7) ^ (right << 13)) & hm)
        left, right = right, nxt
    return (left << 20) | right


def _sq_lists():
    import closeness
    pref, suff, pv, sv = closeness.parse_name_rs()
    return pref, suff, pv, sv


def status_quo_name(ordinal: int, sq: tuple) -> str:
    pref, suff, pv, sv = sq
    v = permute40(ordinal)

    def word(w: int) -> str:
        hi = (w >> 10) & 0x3FF
        lo = w & 0x3FF
        return (pref[(hi >> 2) & 0xFF] + pv[hi & 3]
                + suff[(lo >> 2) & 0xFF] + sv[lo & 3])
    return word((v >> 20) & 0xF_FFFF) + "-" + word(v & 0xF_FFFF)


def verify_bijection(n: int = 1000) -> bool:
    seen = set()
    for o in range(n):
        v = permute(o)
        if v in seen:
            return False
        seen.add(v)
        if unpermute(v) != o:
            return False
    return True


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--count", type=int, default=100)
    ap.add_argument("--seed", type=int, default=None)
    ap.add_argument("--preset", choices=["first-million", "first-billion", "first-trillion"])
    ap.add_argument("--cap", type=int, default=None, help="sample ordinals in [0, cap)")
    ap.add_argument("--full", action="store_true", help="sample the whole 2^48 space")
    ap.add_argument("--ordinals", type=str, default="", help="exact ordinals, comma list")
    ap.add_argument("--show-ordinals", action="store_true")
    ap.add_argument("--compare", action="store_true",
                    help="print new vs status-quo names for the same ordinals")
    args = ap.parse_args()

    if sum([args.cap is not None, bool(args.preset), args.full]) > 1:
        ap.error("pick at most one of --cap, --preset, --full")

    pre_pool = load("prefixes")
    suf_pool = load("suffixes")
    total = 2 ** 48

    if args.cap is not None:
        cap = args.cap
    elif args.preset:
        cap = preset_cap(args.preset)
    elif args.full:
        cap = total
    else:
        cap = 10 ** 12  # default: the planned contract cap (first trillion)
    cap = min(cap, total)

    label = f"[0, {cap:,})" if cap < total else "[0, 2^48) — full space"
    print(f"prefix {len(pre_pool)}  suffix {len(suf_pool)}  {4 * 12} bits "
          f"-> {total:,} possible names; sampling {label}")

    if args.ordinals:
        print("\n# exact ordinals")
        for tok in args.ordinals.split(","):
            if not tok.strip():
                continue
            o = int(tok.strip())
            print(f"{o:>12}: {ordinal_to_name(o, pre_pool, suf_pool)}")

    if args.compare:
        sq = _sq_lists()
        print(f"\n# {args.count} ordinals:  new (2^48 lists)  vs  status quo (name.rs)")
        print(f"{'ordinal':>12}  {'NEW':28}  STATUS QUO")
        rng = random.Random(args.seed)
        ords = [int(t) for t in args.ordinals.split(",") if t.strip()] if args.ordinals else []
        for o in (ords[: args.count] if ords else [rng.randrange(min(cap, MASK40)) for _ in range(args.count)]):
            new_n = ordinal_to_name(o, pre_pool, suf_pool)
            sq_n = status_quo_name(o, sq)
            print(f"{o:>12,}  {new_n:<28}  {sq_n}")
        return 0

    rng = random.Random(args.seed)
    print(f"\n# {args.count} random ordinals from {label} (seed={args.seed})")
    for _ in range(args.count):
        o = rng.randrange(cap)
        name = ordinal_to_name(o, pre_pool, suf_pool)
        print(f"{o:>12}: {name}" if args.show_ordinals else name)
    return 0


if __name__ == "__main__":
    if not verify_bijection():
        print("Feistel bijection check FAILED", file=sys.stderr)
        raise SystemExit(1)
    raise SystemExit(main())