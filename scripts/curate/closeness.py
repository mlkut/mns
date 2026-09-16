#!/usr/bin/env python3
"""Phase 6: how close do registered names get, at scale?

Metric (per the design ask): pick a name uniformly at random; after N ordinals
have been registered, what is the chance someone can find a name within 1 (or 2)
letters of it? Both systems, at N = 1M / 1B / 1T:

  status quo  - current name.rs: 2^40 names, 256 CVC + 4-vowel lists
  new         - curated final pools: 2^48 names, 2 x 4096 CVCV lists

Ball size B(d) = # distinct names within aligned Hamming distance d of a random
name, measured by exact neighbour enumeration over sampled names. Then
P(close after N) = 1 - (1 - B/T)^N, plus the saturation point T/B where a name
expects its first close neighbour.

Writes design/name_closeness.md and prints the table.
"""
import argparse
import random
import re
import sys
from pathlib import Path

import generate

SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT = SCRIPT_DIR.parent.parent
FINAL_DIR = generate.FILTERED_DIR.parent / "final"
DESIGN_DIR = generate.FILTERED_DIR.parent / "quality" / "reports"
NAME_RS = PROJECT / "mns" / "src" / "name.rs"

MILESTONES = [10 ** 6, 10 ** 9, 10 ** 12]
SAMPLES = 400

MASK20 = 0xF_FFFF
MASK24 = 0xFF_FFFF


# ---------- schemes ----------

def parse_name_rs():
    src = NAME_RS.read_text()
    def array(name):
        m = re.search(rf"const {name}: \[&str; 256\] = \[(.*?)\];", src, re.S)
        return re.findall(r'"([a-z]{3})"', m.group(1))
    pref = array("PREFIXES")
    suff = array("SUFFIXES")
    pv = re.search(r'PREFIX_EXTRA_VOWEL: \[u8; 4\] = \*b"([a-z]{4})"', src).group(1)
    sv = re.search(r'SUFFIX_EXTRA_VOWEL: \[u8; 4\] = \*b"([a-z]{4})"', src).group(1)
    return pref, suff, pv, sv


def build_status_quo():
    pref, suff, pv, sv = parse_name_rs()
    pv_set, sv_set = set(pv), set(sv)
    pv_i = {c: i for i, c in enumerate(pv)}
    sv_i = {c: i for i, c in enumerate(sv)}
    pref_lut = {t: i for i, t in enumerate(pref)}
    suff_lut = {t: i for i, t in enumerate(suff)}

    def valid(name: str) -> bool:
        return all((w[0:3] in pref_lut and w[3] in pv_set
                    and w[4:7] in suff_lut and w[7] in sv_set)
                   for w in (name[:8], name[8:]))

    def sample(v: int) -> str:
        def word(w: int) -> str:
            hi = (w >> 10) & 0x3FF
            lo = w & 0x3FF
            return (pref[(hi >> 2) & 0xFF] + pv[hi & 3]
                    + suff[(lo >> 2) & 0xFF] + sv[lo & 3])
        return word((v >> 20) & MASK20) + word(v & MASK20)

    # per-position candidate letters across the whole space (2 identical words)
    sets = ([{t[i] for t in pref} for i in range(3)] + [pv_set]
            + [{t[i] for t in suff} for i in range(3)] + [sv_set]) * 2
    return {"name": "status quo (2^40)", "T": 2 ** 40, "valid": valid,
            "sample": sample, "cand": sets}


def build_new():
    pre = [l.strip() for l in open(FINAL_DIR / "prefixes.txt")]
    suf = [l.strip() for l in open(FINAL_DIR / "suffixes.txt")]
    pre_set, suf_set = set(pre), set(suf)

    def valid(name: str) -> bool:
        return all(w[0:4] in pre_set and w[4:8] in suf_set
                   for w in (name[:8], name[8:]))

    def sample(v: int) -> str:
        hi = (v >> 24) & MASK24
        lo = v & MASK24
        p1, s1 = (hi >> 12) & 0xFFF, hi & 0xFFF
        p2, s2 = (lo >> 12) & 0xFFF, lo & 0xFFF
        return pre[p1] + suf[s1] + pre[p2] + suf[s2]

    sets = ([{t[i] for t in pre} for i in range(4)]
            + [{t[i] for t in suf} for i in range(4)]) * 2
    return {"name": "new (2^48)", "T": 2 ** 48, "valid": valid,
            "sample": sample, "cand": sets}


# ---------- exact neighbour counts ----------

def close_counts(name: str, cand, valid):
    """Return (K1 = exact d=1 neighbours, K2 = exact d=2 neighbours)."""
    n = len(name)
    k1 = 0
    for i in range(n):
        for x in cand[i]:
            if x != name[i] and valid(name[:i] + x + name[i + 1:]):
                k1 += 1
    k2 = 0
    for i in range(n):
        for j in range(i + 1, n):
            for x in cand[i] - {name[i]}:
                for y in cand[j] - {name[j]}:
                    mid = name[:i] + x + name[i + 1:j] + y + name[j + 1:]
                    if valid(mid):
                        k2 += 1
    return k1, k2


def analyze_scheme(scheme, rng, samples):
    T = scheme["T"]
    cand = scheme["cand"]
    k1s, k2s = [], []
    for _ in range(samples):
        name = scheme["sample"](rng.randrange(T))
        assert scheme["valid"](name), "bad sampled name"
        k1, k2 = close_counts(name, cand, scheme["valid"])
        k1s.append(k1)
        k2s.append(k2)
    avg1 = sum(k1s) / samples
    avg2 = sum(k2s) / samples
    b1 = 1 + avg1
    b2 = 1 + avg1 + avg2
    rows = [{"N": N,
             "p_exact1": 1 - (1 - avg1 / T) ** N,
             "p_le1": max(0.0, 1 - (1 - b1 / T) ** N),
             "p_le2": max(0.0, 1 - (1 - b2 / T) ** N)}
            for N in MILESTONES]
    return {"name": scheme["name"], "T": T,
            "avg_K1": avg1, "avg_K2": avg2,
            "min_K1": min(k1s), "max_K1": max(k1s),
            "B_le1": b1, "B_le2": b2,
            "sat_exact1": T / avg1, "sat_le1": T / b1,
            "rows": rows}


# ---------- rendering ----------

def pct(x):
    return f"{100 * x:.4f}%"


def fnum(x, threshold=1e6):
    return f"{x:,.2f}" if x < threshold else f"{x:.3e}"


def render(rs):
    L = ["# Name closeness at scale (phishing exposure)\n",
         "Metric: pick a name uniformly at random; after N ordinals are registered,",
         "what is P(someone can find a name within 1 / ≤2 letters of it)? Measured with",
         f"B = average distinct-neighbour ball (exact enumeration, {SAMPLES} samples),",
         "then P = 1 - (1 - B/T)^N.\n"]
    L.append("| system | T (names) | avg K(d=1) | avg K(d=2) | K1 range | B(≤1) | B(≤2) |")
    L.append("|---|---|---|---|---|---|---|")
    for r in rs:
        L.append(f"| {r['name']} | {r['T']:,} | {fnum(r['avg_K1'])} | {fnum(r['avg_K2'])} "
                 f"| {r['min_K1']}..{r['max_K1']} | {fnum(r['B_le1'])} | {fnum(r['B_le2'])} |")
    L.append("")
    L.append("N where a random name expects its first close (≤1) neighbour: "
             + ", ".join(f"{r['name']}: ~{fnum(r['sat_le1'])}" for r in rs) + ".\n")
    L.append("| system | registered N | P(1 letter off) | P(≤1) | P(≤2) | expected close (≤1) |")
    L.append("|---|---|---|---|---|---|")
    for r in rs:
        for row in r["rows"]:
            L.append(f"| {r['name']} | {row['N']:,} | {pct(row['p_exact1'])} "
                     f"| {pct(row['p_le1'])} | {pct(row['p_le2'])} | "
                     f"{fnum(row['N'] * r['B_le1'] / r['T'])} |")
    L.append("\n> d=0 (identical names) is a separate word-space birthday event, not counted here;")
    L.append("> B(≤1)/B(≤2) implicitly include the name itself. Status-quo numbers use the")
    L.append("> committed name.rs lists; 'new' uses scripts/data/final/*.txt.")
    return "\n".join(L) + "\n"


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--samples", type=int, default=SAMPLES)
    ap.add_argument("--seed", type=int, default=7)
    args = ap.parse_args()

    rng = random.Random(args.seed)
    results = [analyze_scheme(build_status_quo(), rng, args.samples),
               analyze_scheme(build_new(), rng, args.samples)]

    out = render(results)
    DESIGN_DIR.mkdir(parents=True, exist_ok=True)
    (DESIGN_DIR / "name_closeness.md").write_text(out)
    print(out, end="")
    print(f"-> {DESIGN_DIR / 'name_closeness.md'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())