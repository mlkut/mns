#!/usr/bin/env python3
"""Phase 8: EXPERIMENT (read-only) — texture rules to reduce "samey" names.

Perceived sameness comes from list structure, not per-token phonotactics:
  * full-reduplication tokens (kaka, wuwu, mumu) read baby-talk / meme-y
  * over-concentrated onset-vowel families (the batch of 188 "ka*" prefixes)
  * words that reuse one vowel >=3 times

This script MEASURES the current lists, then replays candidate rules IN MEMORY
(never writing scripts/data/final) and reports what each would cost in pool /
reduplication / lookalikes / phonotactic quality / closeness.

  texture.py                       # baseline metrics of the current lists
  texture.py experiment            # rule sweep (infers caps {50,70,90,120})
  texture.py experiment --caps 60,100,200   # custom family caps
"""
import argparse
import json
import random
import sys
from pathlib import Path

import generate

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))
PROJECT = SCRIPT_DIR.parent.parent
FINAL_DIR = generate.FILTERED_DIR.parent / "final"
DESIGN_DIR = generate.FILTERED_DIR.parent / "quality" / "reports"

import select as S

SEED = 23


def load_final(name):
    return [l.strip() for l in open(FINAL_DIR / f"{name}.txt") if l.strip()]


def family(i): return i[:2]
def is_redup(i): return i[0] == i[2] and i[1] == i[3]
def same_vowel(i): return i[1] == i[3]


def textures(toks):
    fam = {}
    for t in toks:
        fam.setdefault(family(t), 0)
        fam[family(t)] += 1
    top = sorted(fam.values(), reverse=True)
    return {"max_fam": top[0] if top else 0,
            "n_redup": sum(1 for t in toks if is_redup(t)),
            "n_samev": sum(1 for t in toks if same_vowel(t)),
            "top_fams": sorted(fam.items(), key=lambda kv: -kv[1])[:5]}


def name_level(lm, prefixes, suffixes, rng, n=3000):
    v3 = 0
    for _ in range(n):
        w = prefixes[rng.randrange(4096)] + suffixes[rng.randrange(4096)]
        vs = [c for c in w if c in "aeiou"]
        if vs and max(vs.count(v) for v in set(vs)) >= 3:
            v3 += 1
    return v3 / n


def build_under_rule(lm, rng, cap=None, ban_redup=False, ban_samev=False):
    pool = S.load_clean()
    if cap is not None:
        by_fam = {}
        for t in pool:
            by_fam.setdefault(family(t), []).append(t)
        pool = []
        for toks in by_fam.values():
            pool.extend(sorted(toks, key=lm.token_score, reverse=True)[:cap])
    if ban_redup:
        pool = [t for t in pool if not is_redup(t)]
    if ban_samev:
        pool = [t for t in pool if not same_vowel(t)]
    p = S.prune(list(pool))
    if len(p) < 4096:
        return None, None
    return S.assignment_min_conflict(p, 4096)


def measure(lm, prefixes, suffixes, rng):
    m = quality_.list_metrics(lm, prefixes, suffixes, rng)
    m["redup_rate"] = m["redup"]
    t = textures(prefixes), textures(suffixes)
    nlv = name_level(lm, prefixes, suffixes, rng)
    m["redup_tokens"] = t[0]["n_redup"] + t[1]["n_redup"]
    m.update({"max_fam": max(t[0]["max_fam"], t[1]["max_fam"]),
              "samev": t[0]["n_samev"] + t[1]["n_samev"],
              "name_v3": nlv})
    return m


def fmt_row(label, x):
    return (f"| {label:<34} | {x['pool']:>6,} | {x['shared']:>4,} "
            f"| 1-in-{int(1/max(x['redup_rate'],1e-9)):>5,} | {x['look_pairs']:>7,} "
            f"| {x['name_p10']:.3f} | {x['name_p50']:.3f} "
            f"| {x['B_le1']:>5.1f} | {100*x['P_le1_trillion']:>5.1f}% "
            f"| {x['max_fam']:>3} | {x['redup_tokens']:>3} | {x['samev']:>4} | {100*x['name_v3']:.1f}% |")


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("mode", nargs="?", default="report", choices=["report", "experiment"])
    ap.add_argument("--caps", default="50,70,90,120")
    ap.add_argument("--lexicon", default="wordfreq", choices=["wordfreq", "offline"])
    ap.add_argument("--no-redup-ban", action="store_true",
                    help="only family caps, skip the full-reduplication ban")
    args = ap.parse_args()

    global quality_
    import quality as quality_
    lm = quality_.TrigramLM(args.lexicon)
    rng = random.Random(SEED)

    pre = load_final("prefixes")
    suf = load_final("suffixes")
    L = ["# Name texture: why new names feel samey (experiment)\n",
         "Texture = family concentration, reduplicated syllables, repeated vowels. "
         "All rows are measured IN MEMORY — final lists untouched. "
         "p10/p50 = phonotactic (higher better); B(≤1) = closeness ball; "
         "name_v3 = % of names with one vowel ≥3 times in an 8-char word.",
         "redup = tokens like kaka/wuwu (prefix+suffix lists); "
         "samev = tokens with both vowels identical; max_fam = largest first-two-letters family size.\n"]

    if args.mode == "report":
        clean = S.load_clean()
        tc = textures(clean)
        L.append("## Baseline\n")
        L.append("| set | max family | top families | redup tokens | same-vowel tokens |")
        L.append("|---|---|---|---|---|")
        for label, toks in (("clean pool (7,090)", clean),
                            ("final prefixes", pre), ("final suffixes", suf)):
            t = textures(toks)
            tf = ", ".join(f"{k}:{v}" for k, v in t["top_fams"][:4])
            L.append(f"| {label} | {t['max_fam']} | {tf} | {t['n_redup']} | {t['n_samev']} |")
        m = measure(lm, pre, suf, rng)
        L.append("")
        L.append(fmt_row("current final lists (baseline, post-substitution)", m))
        L.append("")
        L.append("Note: clean rebuild rows in `experiment` do NOT get the substitution pass, "
                 "so compare them to each other, not to the post-substitution baseline.")
    else:
        caps = [int(x) for x in args.caps.split(",")]
        L.append("## Experiment matrix (clean pool -> rule -> prune -> assign)\n")
        L.append("| row | pool | shared | redup | lookalikes | p10 | p50 | B(≤1) | P(≤1@1e12) | max_fam | redupT | samevT | name_v3 |")
        L.append("|---|---|---|---|---|---|---|---|---|---|---|---|---|")
        L.append(fmt_row("current final lists (post-substitution)", measure(lm, pre, suf, rng)))
        L.append("")
        rows = [("clean rebuild (no rules)", dict(cap=None, ban_redup=False, ban_samev=False))]
        rows.append(("  + no-redup", dict(cap=None, ban_redup=True, ban_samev=False)))
        if not args.no_redup_ban:
            rows.append(("  + no-redup + no-samevowel", dict(cap=None, ban_redup=True, ban_samev=True)))
        for cap in caps:
            rows.append((f"  + no-redup + family cap {cap}",
                         dict(cap=cap, ban_redup=True, ban_samev=False)))
        for label, kw in rows:
            pl, sl = build_under_rule(lm, rng, **kw)
            if pl is None:
                L.append(f"| {label:<44} | (pool < 4096 -> cliff to 2^44) |")
                continue
            L.append(fmt_row(label, measure(lm, pl, sl, rng)))
        L.append("")
        L.append("(All experiment rows are list-level only; a real adoption would add the "
                 "substitution pass on top, recovering phonotactic quality.)")

    out = "\n".join(L) + "\n"
    print(out)
    DESIGN_DIR.mkdir(parents=True, exist_ok=True)
    (DESIGN_DIR / "name_texture.md").write_text(out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())


quality_ = None  # set in main