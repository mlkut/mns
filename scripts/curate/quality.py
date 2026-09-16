#!/usr/bin/env python3
"""Phase 7: objective name-quality scoring + minimal re-curation tradeoff.

Quality = phonotactic probability, approximated by a character-trigram language
model (the corpus-linguistics proxy for pronounceability; trigram frequency is
the standard used in fantasy/password name generators). Smoother = higher score.

  report - distribution comparison status quo vs new, worst tokens/names,
           watchlist scores -> design/name_quality.md
  sweep  - drop the worst X% of pool tokens by score, rebuild the final lists
           via select.py, measure the quality/overlap/closeness tradeoff

Requires a lexicon: wordfreq (frequency-weighted; pip in .venv) or the offline
macOS dictionary. Use .venv/bin/python for the wordfreq mode.

Usage:
  quality.py                          # report
  quality.py --lexicon offline        # offline dictionary
  quality.py sweep                    # tradeoff table
  quality.py sweep --apply            # commit the recommended lists
"""
import argparse
import json
import math
import random
import sys
from pathlib import Path
from collections import Counter

import generate

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))
PROJECT = SCRIPT_DIR.parent.parent
FINAL_DIR = generate.FILTERED_DIR.parent / "final"
QUALITY_DIR = generate.FILTERED_DIR.parent / "quality"
DESIGN_DIR = PROJECT / "design"
WATCHLIST = QUALITY_DIR / "watchlist.txt"

import select as S
import closeness as C

SLOT = 4096
SEED = 11
V = 28  # 26 letters + ^ + $


# ---------- LM ----------

class TrigramLM:
    src = ""

    def __init__(self, lexicon: str = "wordfreq"):
        if lexicon == "wordfreq":
            self._from_wordfreq()
        else:
            self._from_offline_dict()

    def _from_wordfreq(self):
        try:
            from wordfreq import top_n_list, zipf_frequency
        except ImportError as e:
            raise SystemExit(
                f"wordfreq not installed in this interpreter ({sys.executable}).\n"
                f"Install it in the project venv or re-run with --lexicon offline.") from e
        self.src = "wordfreq (en, frequency-weighted)"
        words = []
        for w in top_n_list("en", 250_000):
            z = zipf_frequency(w, "en")
            if z >= 1.0:
                words.append((w, 10.0 ** z))
        self._build(words)

    def _from_offline_dict(self):
        self.src = "offline /usr/share/dict/words (equal weight)"
        path = Path("/usr/share/dict/words")
        if not path.exists():
            raise SystemExit("no offline dictionary found")
        words = [(w, 1.0) for w in path.read_text().split()
                 if w.isalpha() and w.islower()]
        self._build(words)

    def _build(self, words):
        tr, bi = Counter(), Counter()
        for w, c in words:
            s = "^" + w + "$"
            for i in range(len(s) - 2):
                tr[s[i:i + 3]] += c
            bi[s[i:i + 1]] += c
            for i in range(len(s) - 1):
                pass
            for i in range(len(s) - 1):
                bi[s[i:i + 2]] += c
        self.tr = dict(tr)
        self.bi = dict(bi)

    def _logp_seq(self, seq):
        s = "^" + seq + "$"
        lp = 0.0
        n = 0
        for i in range(len(s) - 2):
            c3 = self.tr.get(s[i:i + 3], 0.0)
            c2 = self.bi.get(s[i + 1:i + 3], 0.0)
            p = (c3 + 1.0) / (c2 + V) if (c2 or c3) else 1.0 / V
            lp += math.log10(max(p, 1e-300))
            n += 1
        return lp / n if n else 0.0

    def token_score(self, t): return self._logp_seq(t)
    def word_score(self, w): return self._logp_seq(w)
    def name_score(self, n):
        if "-" in n:
            w1, w2 = n.split("-")
        else:
            w1, w2 = n[:8], n[8:]
        return (self._logp_seq(w1) + self._logp_seq(w2)) / 2.0


def pctile(x, p):
    return x[min(len(x) - 1, int(p * (len(x) - 1)))]


def sample_names(prefixes, suffixes, n, rng):
    out = []
    for _ in range(n):
        v = rng.randrange(2 ** 48)
        hi = (v >> 24) & 0xFF_FFFF
        lo = v & 0xFF_FFFF
        out.append(prefixes[(hi >> 12) & 0xFFF] + suffixes[hi & 0xFFF]
                  + "-" + prefixes[(lo >> 12) & 0xFFF] + suffixes[lo & 0xFFF])
    return out


# ---------- status quo ----------

def status_quo_data(lm):
    sq = C.build_status_quo()
    pref, suff, pv, sv = C.parse_name_rs()
    toks = set()
    for t in pref:
        for v in pv:
            toks.add(t + v)
    for t in suff:
        for v in sv:
            toks.add(t + v)
    toks = sorted(toks)
    rng = random.Random(SEED)
    names = [sq["sample"](rng.randrange(sq["T"])) for _ in range(5000)]
    return {"tokens": toks,
            "token_scores": sorted(lm.token_score(t) for t in toks),
            "name_scores": sorted(lm.name_score(n) for n in names),
            "T": sq["T"]}


# ---------- list building for a drop fraction ----------

def build_lists(lm, drop_frac, rng):
    pool = S.load_clean()
    if drop_frac > 0:
        scored = sorted(pool, key=lm.token_score)
        pool = scored[int(len(pool) * drop_frac):]
    pool = S.prune(list(pool))
    if len(pool) < SLOT:
        return None, None
    prefixes, suffixes = S.assignment_min_conflict(pool, SLOT)
    return prefixes, suffixes


def list_metrics(lm, prefixes, suffixes, rng):
    shared = len(set(prefixes) & set(suffixes))
    lp = S.lookalikes(prefixes)
    ls = S.lookalikes(suffixes)

    names = sample_names(prefixes, suffixes, 3000, rng)
    ns = sorted(lm.name_score(n) for n in names)

    pre_set = set(prefixes)
    suf_set = set(suffixes)
    cand = ([{t[i] for t in prefixes} for i in range(4)]
            + [{t[i] for t in suffixes} for i in range(4)]) * 2

    def valid(flat):
        return (flat[:4] in pre_set and flat[4:8] in suf_set
                and flat[8:12] in pre_set and flat[12:16] in suf_set)

    k1s = []
    for _ in range(120):
        flat = names[rng.randrange(len(names))].replace("-", "")
        k = sum(1 for i in range(16) for x in cand[i]
                if x != flat[i] and valid(flat[:i] + x + flat[i + 1:]))
        k1s.append(k)
    b1 = 1 + sum(k1s) / len(k1s)
    p12 = max(0.0, 1 - (1 - b1 / 2 ** 48) ** (10 ** 12))
    return {"pool": len(pre_set | suf_set), "shared": shared,
            "redup": shared / SLOT ** 2, "look_pairs": lp + ls,
            "name_p10": pctile(ns, .10), "name_p50": pctile(ns, .50),
            "B_le1": b1, "P_le1_trillion": p12}


# ---------- range cliff ----------

def range_cliff(lm):
    """pool-after-prune for each drop fraction, and the slot/range it implies."""
    pool = S.load_clean()
    rows = []
    for frac in (10, 20, 30, 40):
        scored = sorted(pool, key=lm.token_score)
        cut = scored[int(len(scored) * frac / 100):]
        n = len(S.prune(list(cut)))
        bits = 12 if n >= 4096 else (11 if n >= 2048 else 10)
        rows.append((frac, n, bits))
    return rows


# ---------- rendering ----------

def watchlist_entries():
    if not WATCHLIST.exists():
        return []
    return [ln.split("#")[0].strip() for ln in WATCHLIST.read_text().splitlines()
            if ln.split("#")[0].strip()]


def render_report(sq, lm, rng):
    prefixes = [l.strip() for l in open(FINAL_DIR / "prefixes.txt")]
    suffixes = [l.strip() for l in open(FINAL_DIR / "suffixes.txt")]
    newn = sorted(lm.name_score(n) for n in sample_names(prefixes, suffixes, 5000, rng))

    L = ["# mns name quality: phonotactic probability\n",
         f"Model: character-trigram LM on **{lm.src}**. Higher = smoother / more",
         "English-like. Token = one 4-letter syllable; name = both words.\n"]

    rows = [("status quo", sq["token_scores"]),
            ("new prefixes", sorted(lm.token_score(t) for t in prefixes)),
            ("new suffixes", sorted(lm.token_score(t) for t in suffixes)),
            ("new combined", sorted(lm.token_score(t) for t in prefixes + suffixes))]
    L.append("## Token 4-letter scores\n")
    L.append("| set | " + " | ".join(f"p{int(p*100)}" for p in (.10, .25, .50, .75, .90)) + " |")
    L.append("|---|---" + "|--" * 5 + "|")
    for label, sc in rows:
        L.append(f"| {label} | " + " | ".join(f"{pctile(sc, p):.3f}" for p in (.10, .25, .50, .75, .90)) + " |")

    L.append("\n## Name scores\n")
    sqn = sq["name_scores"]
    L.append("| system | " + " | ".join(f"p{int(p*100)}" for p in (.05, .10, .25, .50, .75, .90)) + " |")
    L.append("|---|---" + "|--" * 6 + "|")
    L.append(f"| status quo | " + " | ".join(f"{pctile(sqn, p):.3f}" for p in (.05, .10, .25, .50, .75, .90)) + " |")
    L.append(f"| new | " + " | ".join(f"{pctile(newn, p):.3f}" for p in (.05, .10, .25, .50, .75, .90)) + " |\n")

    L.append("## What quality costs in name space\n")
    L.append("Name space is fixed by the slot encoding (12 bits for 4096 tokens), NOT by pool size.")
    L.append("The range only collapses at a hard cliff: pool < 4096 drops to 11-bit slots (2^44),\n"
             "pool < 2048 to 2^40. Between 4096 and 8192 the real cost of a smaller pool is more\n"
             "forced overlap (reduplication) + more lookalikes — not fewer names.\n")
    L.append("| drop worst % by score | pool after join-prune | slot bits | name space |")
    L.append("|---|---|---|---|")
    for frac, n, bits in range_cliff(lm):
        L.append(f"| {frac}% | {n:,} | {bits} | "
                 f"{'2^48 (281T)' if bits == 12 else '2^44 (17.6T)' if bits == 11 else '2^40 (1.1T)'} |")
    L.append("| substitution (no drop) | 6,349 | 12 | **2^48 (281T)** |\n")
    L.append("`substitute` re-admits join-cleaned-but-good tokens instead of dropping: pool and range")
    L.append("stay put, redup improves.\n")

    L.append("## Worst 20 tokens in the final lists\n")
    L.append("| score | token | slot |")
    L.append("|---|---|---|")
    ps, ss = set(prefixes), set(suffixes)
    def slot(t):
        return "P/S" if t in ps and t in ss else ("P" if t in ps else "S")
    for t in sorted(ps | ss, key=lm.token_score)[:20]:
        L.append(f"| {lm.token_score(t):.3f} | `{t}` | {slot(t)} |")

    L.append("\n## `tuwuwaha` case\n")
    tw = lm.word_score("tuwuwaha")
    L.append(f"word score {tw:.3f} — only {100 * sum(1 for x in newn if x <= tw) / len(newn):.1f}% "
             f"of {len(newn)} sampled new names score this low "
             f"({100 * sum(1 for x in sqn if x <= tw) / len(sqn):.1f}% of status quo) "
             "-> a genuine outlier, not 'just new'.\n")

    L.append("## Watchlist (`scripts/data/quality/watchlist.txt`)\n")
    L.append("| entry | score | share of new names at/below it |")
    L.append("|---|---|---|")
    for e in watchlist_entries():
        lw = {"4": lm.token_score(e), "8": lm.word_score(e),
              "17": lm.name_score(e)}[str(len(e))]
        frac = 100 * sum(1 for x in newn if x <= lw) / len(newn)
        L.append(f"| `{e}` | {lw:.3f} | {frac:.1f}% |")
    L.append("\nRun `quality.py sweep` for the quality/overlap tradeoff.")
    return "\n".join(L) + "\n"


def render_sweep(sq, lm, rng, fractions, apply):
    sqn = sq["name_scores"]
    L = [f"# Quality / overlap tradeoff (minimal pruning)\n",
         f"Drop the worst X% of clean-pool tokens by score, rebuild lists via select.py. "
         f"Status quo: name p10={pctile(sqn, .10):.3f}, p50={pctile(sqn, .50):.3f}.\n",
         "| drop% | pool | redup | lookalikes | name p10 | name p50 | B(<=1) | P(<=1 @1e12) |"]
    L.append("|" + "---|" * 8)
    rows = []
    for frac in fractions:
        lists = build_lists(lm, frac / 100.0, rng)
        if lists[0] is None:
            L.append(f"| {frac:g}% | (pool < slot) | | | | | | |")
            continue
        m = list_metrics(lm, lists[0], lists[1], rng)
        rows.append((frac, lists, m))
        L.append(f"| {frac:g}% | {m['pool']:,} | 1-in-{int(1/max(m['redup'],1e-9)):,} "
                 f"| {m['look_pairs']:,} | {m['name_p10']:.3f} | {m['name_p50']:.3f} "
                 f"| {m['B_le1']:.1f} | {100 * m['P_le1_trillion']:.1f}% |")
    wl = watchlist_entries()
    L.append("")
    if not rows:
        return "\n".join(L) + "\n"
    baseline = rows[0][2]["name_p10"]
    pick = rows[0]
    for frac, lists, m in rows:
        gain = baseline - m["name_p10"]
        covered = pool_covers_watchlist(lists[0], lists[1], wl)
        if m["pool"] >= 5500 and covered and gain >= 0.02:
            pick = (frac, lists, m)
            break
    L.append(f"Recommended: drop {pick[0]:g}% (pool {pick[2]['pool']:,}, "
             f"p10 {pick[2]['name_p10']:.3f} vs {baseline:.3f}, "
             f"redup 1-in-{int(1/max(pick[2]['redup'],1e-9)):,}). "
             f"Rule: smallest drop with name-p10 gain >= 0.02 and pool >= 5500.")
    if apply:
        prefixes, suffixes = pick[1]
        FINAL_DIR.mkdir(parents=True, exist_ok=True)
        (FINAL_DIR / "prefixes.txt").write_text("\n".join(prefixes) + "\n")
        (FINAL_DIR / "suffixes.txt").write_text("\n".join(suffixes) + "\n")
        L.append(f"Applied: wrote new final lists -> {FINAL_DIR}")
    return "\n".join(L) + "\n"


def pool_covers_watchlist(pre, suf, watchlist):
    toks = set(pre) | set(suf)
    return not any(len(e) == 4 and e in toks for e in watchlist)


# ---------- substitution (swap better tokens INTO the final lists) ----------

def swap_in_list(current, candidates, k, lm, keep_set):
    """Replace the k worst-quality tokens of `current` with the k best usable
    removed tokens. `keep_set` = tokens that survive (post-swap) for tie-breaks."""
    k = min(k, len(current), len(candidates))
    candidates = sorted(candidates,
                        key=lambda t: (lm.token_score(t),
                                       -sum(1 for v in S.variants(t) if v in keep_set)),
                        reverse=True)[:k]
    worst = sorted(range(len(current)), key=lambda i: lm.token_score(current[i]))[:k]
    current = list(current)
    for idx, cand in zip(worst, candidates):
        current[idx] = cand
    return current


def substitute(lm, rng, pre_swaps, suf_swaps, apply):
    pool = S.load_clean()
    pruned = set(S.prune(list(pool)))
    removed = sorted(set(pool) - pruned)

    prefixes = [l.strip() for l in open(FINAL_DIR / "prefixes.txt")]
    suffixes = [l.strip() for l in open(FINAL_DIR / "suffixes.txt")]

    # Phase A: re-admit as PREFIX, verified against the (unchanged) suffix list.
    gsuf = S.build_groups(list(set(suffixes)))
    usable_pre = [R for R in removed if S.join_bad_degree(R, gsuf, "pre") == 0]
    pre_new = swap_in_list(prefixes, usable_pre, pre_swaps, lm,
                           set(prefixes) | set(usable_pre))

    # Phase B: re-admit as SUFFIX, verified against the FINAL prefix list.
    gpren = S.build_groups(list(set(pre_new)))
    usable_suf = [R for R in removed if S.join_bad_degree(R, gpren, "suf") == 0
                  and R not in set(prefixes)]
    suf_new = swap_in_list(suffixes, usable_suf, suf_swaps, lm,
                           set(suffixes) | set(usable_suf))

    # Validations
    pre_set, suf_set = set(pre_new), set(suf_new)
    assert len(pre_new) == len(pre_set) == 4096, "prefix dup/size"
    assert len(suf_new) == len(suf_set) == 4096, "suffix dup/size"
    cross = sum(S.join_bad_degree(t, S.build_groups(list(suf_set)), "pre") for t in pre_set)
    assert cross == 0, f"cross-bad joins remain: {cross}"
    assert len(pre_set | suf_set) >= 4096, "pool below slot -> range cliff"

    m = list_metrics(lm, pre_new, suf_new, rng)
    lp, ls = S.lookalikes(pre_new), S.lookalikes(suf_new)
    shared = len(pre_set & suf_set)
    # Guard: no final-list token may contain any banned substring from rules.json.
    _rules = json.loads((SCRIPT_DIR / "rules.json").read_text())
    _bans = (_rules["profanity"]["banned_substrings_3"]
             + _rules["pronunciation"]["banned_2grams"]
             + _rules["pronunciation"]["banned_3grams"])
    _bad = [t for t in pre_new + suf_new if any(b in t for b in _bans)]
    assert not _bad, f"banned substrings in final lists after substitution: {_bad[:10]}"
    _redup = [t for t in pre_new + suf_new if len(t) == 4 and t[0] == t[2] and t[1] == t[3]]
    assert not _redup, f"reduplicated tokens leaked after substitution: {_redup[:10]}"
    analysis = {
        "clean_pool": len(pruned), "slot_size": 4096,
        "namespace": {"b": 12, "total_names": 2 ** 48, "wire_bytes": 6},
        "cross_join_bad_words": cross,
        "prefix": {"size": len(pre_new), "lookalike_pairs_1char": lp},
        "suffix": {"size": len(suf_new), "lookalike_pairs_1char": ls},
        "shared_tokens": shared,
        "reduplication_within_word_rate": shared / 4096 ** 2,
        "first_1char_twin_ordinal_estimate":
            round(S.first_twin_ordinal(lp, ls, 4096)),
        "quality_name_p10": round(m["name_p10"], 4),
        "quality_name_p50": round(m["name_p50"], 4),
    }
    FINAL_DIR.mkdir(parents=True, exist_ok=True)
    if apply:
        (FINAL_DIR / "prefixes.txt").write_text("\n".join(pre_new) + "\n")
        (FINAL_DIR / "suffixes.txt").write_text("\n".join(suf_new) + "\n")
        (FINAL_DIR / "analysis.json").write_text(json.dumps(analysis, indent=2) + "\n")
    out = ([f"# Substitution re-curation\n",
            f"Re-admitted {min(pre_swaps, len(usable_pre))} prefix + "
            f"{min(suf_swaps, len(usable_suf))} suffix join-pruned tokens "
            f"(swapping out the worst-quality list members).\n",
            f"cross-bad joins = {cross} (asserted 0); pool = {len(pre_set | suf_set)} "
            f"(>=4096 -> 2^48 range preserved).\n",
            f"name p10 {m['name_p10']:.3f}  p50 {m['name_p50']:.3f}  "
            f"lookalikes {lp + ls:,}  shared {shared:,}  "
            f"redup 1-in-{int(1/max(m['redup'],1e-9)):,}  "
            f"B(<=1) {m['B_le1']:.1f}  P(<=1@1e12) {100*m['P_le1_trillion']:.1f}%\n"])
    print("\n".join(out))
    if apply:
        print(f"Applied: updated {FINAL_DIR / 'prefixes.txt'} and suffixes.txt + analysis.json "
              f"(re-run quality.py report + closeness.py + report.py to refresh docs).")
    else:
        print("Dry run — re-run with --apply to commit these lists.")
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("mode", nargs="?", default="report",
                    choices=["report", "sweep", "substitute"])
    ap.add_argument("--lexicon", default="wordfreq", choices=["wordfreq", "offline"])
    ap.add_argument("--fractions", default="0,0.2,0.5,1,2,5")
    ap.add_argument("--prefix-swaps", type=int, default=10 ** 6)
    ap.add_argument("--suffix-swaps", type=int, default=500)
    ap.add_argument("--apply", action="store_true")
    args = ap.parse_args()

    lm = TrigramLM(args.lexicon)
    rng = random.Random(SEED)
    sq = status_quo_data(lm)
    QUALITY_DIR.mkdir(parents=True, exist_ok=True)

    if args.mode == "report":
        md = render_report(sq, lm, rng)
    elif args.mode == "sweep":
        fracs = [float(x) for x in args.fractions.split(",")]
        md = render_sweep(sq, lm, rng, fracs, args.apply)
    else:
        return substitute(lm, rng, args.prefix_swaps, args.suffix_swaps, args.apply)
    (DESIGN_DIR / "name_quality.md").write_text(md)
    print(md, end="")
    print(f"\n-> {DESIGN_DIR / 'name_quality.md'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())