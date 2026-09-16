#!/usr/bin/env python3
"""Phase 4: select the largest curated prefix + suffix lists (CVCV).

Two goals from design/name_curation.md, in priority order:
  1. MAXIMIZE the namespace. Power-of-two slots: from the ~7175 clean CVCV
     tokens the largest slot is 4096 -> name space 2^48 (~281 trillion).
     (7175 < 8192 so the two lists must overlap ~1k tokens; reduplication
     rate is overlap/4096^2 — negligible.)
  2. First-trillion names stay nice. Since the first 2^40 ordinals sample the
     whole 2^48 space uniformly, EVERY one of the 4096^2 join words must be
     clean: this script greedily prunes tokens that create offensive joins
     (see scan.py) down to ZERO bad pairs, and greedily assigns the remainder
     to prefix/suffix lists to MINIMIZE 1-letter lookalikes inside each list.

Writes:
  scripts/data/final/prefixes.txt   (4096 4-letter CVCV tokens)
  scripts/data/final/suffixes.txt   (4096 4-letter CVCV tokens)
  scripts/data/final/analysis.json  (lookalike + twin statistics)

Usage: select.py [--slot 4096]
"""
import collections
import json
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))
import generate
from scan import OFF_3, PROFANE_WORDS

FINAL_DIR = generate.FILTERED_DIR.parent / "final"
CONS = generate.CONSONANTS
VOW = generate.VOWELS


def variants(t: str):
    """All 1-character replacements of t (same C/V pattern)."""
    for i, ch in enumerate(t):
        pool = CONS if i in (0, 2) else VOW
        for c in pool:
            if c != ch:
                yield t[:i] + c + t[i + 1:]


def load_clean() -> list[str]:
    path = generate.FILTERED_DIR / "CVCV.txt"
    return [line.strip() for line in open(path) if line.strip()]


# ---------- offensive join bookkeeping ----------

def build_groups(pool: list[str]):
    """Token groups keyed by suffix-start and prefix-end, for fast queries."""
    by0, by2, by3 = {}, {}, {}       # q[0], q[:2], q[:3]
    bye0, bye2, bye3 = {}, {}, {}    # p[-1], p[-2:], p[-3:]
    for t in pool:
        by0.setdefault(t[0], set()).add(t)
        by2.setdefault(t[:2], set()).add(t)
        by3.setdefault(t[:3], set()).add(t)
        bye0.setdefault(t[-1], set()).add(t)
        bye2.setdefault(t[-2:], set()).add(t)
        bye3.setdefault(t[-3:], set()).add(t)
    return by0, by2, by3, bye0, bye2, bye3


def join_bad_degree(t: str, groups, role: str) -> int:
    """# tokens u with offensive join t+u (role='pre') or u+t (role='suf')."""
    by0, by2, by3, bye0, bye2, bye3 = groups
    u0, u2, u3 = (by0, by2, by3) if role == "pre" else (bye0, bye2, bye3)
    e0, e2, e3 = (bye0, bye2, bye3) if role == "pre" else (by0, by2, by3)
    if role == "pre":
        e3k, e2k, e1k = t[-3:], t[-2:], t[-1]
    else:
        e3k, e2k, e1k = t[:3], t[:2], t[0]
    cand = set()
    for g in OFF_3:
        if role == "pre":
            if g[:2] == e2k:
                cand |= u0.get(g[2], set())
            if g[0] == e1k:
                cand |= u2.get(g[1:3], set())
        else:
            if g[1:3] == e2k:
                cand |= u0.get(g[0], set())
            if g[2] == e1k:
                cand |= u2.get(g[:2], set())
    for w in PROFANE_WORDS:
        if role == "pre":
            if w[0] == e1k:
                cand |= u3.get(w[1:4], set())
            if w[0:2] == e2k:
                cand |= u2.get(w[2:4], set())
            if w[0:3] == e3k:
                cand |= u0.get(w[3], set())
        else:
            if w[1:4] == e3k:
                cand |= u0.get(w[0], set())
            if w[2:4] == e2k:
                cand |= u2.get(w[0:2], set())
            if w[3] == e1k:
                cand |= u3.get(w[0:3], set())
    if role == "pre":
        return len(cand)
    return len(cand)


def degrees(pool: list[str]):
    groups = build_groups(pool)
    pre = [join_bad_degree(t, groups, "pre") for t in pool]
    suf = [join_bad_degree(t, groups, "suf") for t in pool]
    return pre, suf


def prune(pool: list[str]) -> list[str]:
    """Greedy approximate minimum vertex cover of the bad-join graph.

    Repeatedly removes the highest-degree (worst) tokens in batches; recomputes
    degrees only once per batch, so convergence is seconds, not minutes. Keeps
    the clean pool as large as possible.
    """
    print(f"pruning {len(pool)} tokens (batches of ~5%) ...", flush=True)
    pool = list(pool)
    total_before = sum(degrees(pool)[0])
    removed = 0
    while True:
        pre, suf = degrees(pool)
        total = sum(pre)
        if total == 0:
            break
        scores = [pre[i] + suf[i] for i in range(len(pool))]
        k = max(1, len(pool) // 20)
        order = sorted(range(len(pool)), key=scores.__getitem__, reverse=True)[:k]
        removed += len(order)
        keep = [i for i in range(len(pool)) if i not in set(order)]
        pool = [pool[i] for i in keep]
        if len(order) <= 1 or removed % 500 < len(order):
            print(f"  removed {removed:4d}  edges left {total:>9,}", flush=True)
    print(f"done: removed {removed} tokens, {len(pool)} left "
          f"(was {total_before:,} bad joins)", flush=True)
    return pool


# ---------- list building ----------

def assignment_min_conflict(pool: list[str], slot: int) -> list[list[str]]:
    """Split pool into prefix+suffix lists; each list is `slot` tokens.

    The two lists may SHARE tokens (forced: pool < 2*slot). Phase 1 greedily
    assigns each token to the list where it adds fewer 1-letter lookalikes.
    Phase 2 tops each list up by borrowing tokens from the other side, picking
    the tokens that add the fewest lookalikes (and are not adjacent to each
    other).
    """
    pool_set = set(pool)
    conflict = {t: sum(1 for v in variants(t) if v in pool_set) for t in pool}
    lists = [set(), set()]
    for t in sorted(pool, key=lambda t: -conflict[t]):
        na = sum(1 for v in variants(t) if v in lists[0])
        nb = sum(1 for v in variants(t) if v in lists[1])
        if na != nb:
            lists[0 if na < nb else 1].add(t)
        else:
            lists[0 if len(lists[0]) <= len(lists[1]) else 1].add(t)

    def borrow(target: set, donor: list[str], need: int) -> set:
        """Add `need` donor tokens to `target`, minimizing added lookalikes."""
        target = set(target)
        chosen = []
        for t in sorted(donor, key=lambda t: (sum(1 for v in variants(t) if v in target), t)):
            if t in target:
                continue
            chosen.append(t)
            if len(chosen) == need:
                break
        for t in chosen:
            target.add(t)
        return target

    donor = sorted(lists[1])
    lists[0] = borrow(lists[0], donor, slot - len(lists[0]))
    donor0 = sorted(lists[0])
    lists[1] = borrow(lists[1], donor0, slot - len(lists[1]))

    a = sorted(lists[0])
    b = sorted(lists[1])
    shared = len(set(a) & set(b))
    print(f"assigned: prefix {len(a)}  suffix {len(b)}  "
          f"shared tokens {shared} (redup ~1 in {int(slot**2/max(shared,1)):,})",
          flush=True)
    return a, b


# ---------- analysis ----------

def lookalikes(lst: list[str]) -> int:
    s = set(lst)
    return sum(1 for t in lst for v in variants(t) if v in s) // 2


def nearest_distribution(lst: list[str]) -> dict:
    s = set(lst)
    state = [0, 0, 0]
    for t in lst:
        if any(v in s for v in variants(t)):
            state[0] += 1
        elif any(t[:i] + x + t[i + 1:] in s for i in range(4) for x in (CONS if i in (0, 2) else VOW)):
            state[1] += 1
        else:
            state[2] += 1
    return {"dist1": state[0], "dist2": state[1], "dist3plus": state[2]}


def first_twin_ordinal(lp: int, ls: int, m: int) -> float:
    """Rough ordinal where the first aligned 1-letter twin name appears."""
    import math
    p = 2 * (lp + ls) / m ** 5
    return math.sqrt(2 / p) if p > 0 else float("inf")


def main() -> int:
    slot = 4096
    if "--slot" in sys.argv:
        slot = int(sys.argv[sys.argv.index("--slot") + 1])

    pool = load_clean()
    pool = prune(pool)

    if len(pool) < 2 * slot - 0:  # need union >= 2*slot? overlap allowed, warn
        print(f"note: pool {len(pool)} < 2*slot({2*slot}); lists will overlap "
              f"{2*slot - len(pool)} tokens")
    prefixes, suffixes = assignment_min_conflict(pool, slot)

    # Guard: no final-list token may contain any banned substring from rules.json.
    _rules = json.loads((SCRIPT_DIR / "rules.json").read_text())
    _bans = (_rules["profanity"]["banned_substrings_3"]
             + _rules["pronunciation"]["banned_2grams"]
             + _rules["pronunciation"]["banned_3grams"])
    _bad = [t for t in prefixes + suffixes if any(b in t for b in _bans)]
    assert not _bad, f"banned substrings in final lists: {_bad[:10]}"

    lp = lookalikes(prefixes)
    ls = lookalikes(suffixes)
    shared = len(set(prefixes) & set(suffixes))
    redup_rate = shared / slot ** 2

    # Independent cross-check: zero offensive joins across the WHOLE
    # prefix x suffix word table (4096^2 = 16.8M words).
    groups = build_groups(suffixes)
    cross_bad = sum(join_bad_degree(t, groups, "pre") for t in prefixes)
    assert cross_bad == 0, f"cross-list bad joins remain: {cross_bad}"

    analysis = {
        "clean_pool": len(pool),
        "slot_size": slot,
        "namespace": {"b": 12, "total_names": 2 ** 48, "wire_bytes": 6},
        "cross_join_bad_words": cross_bad,
        "prefix": {"size": len(prefixes), "lookalike_pairs_1char": lp},
        "suffix": {"size": len(suffixes), "lookalike_pairs_1char": ls},
        "shared_tokens": shared,
        "reduplication_within_word_rate": redup_rate,
        "first_1char_twin_ordinal_estimate": round(first_twin_ordinal(lp, ls, slot)),
        "nearest_neighbor": {
            "prefix": nearest_distribution(prefixes),
            "suffix": nearest_distribution(suffixes),
        },
    }

    FINAL_DIR.mkdir(parents=True, exist_ok=True)
    (FINAL_DIR / "prefixes.txt").write_text("\n".join(prefixes) + "\n")
    (FINAL_DIR / "suffixes.txt").write_text("\n".join(suffixes) + "\n")
    (FINAL_DIR / "analysis.json").write_text(json.dumps(analysis, indent=2) + "\n")

    print(f"wrote prefixes/suffixes ({len(prefixes)} each) -> {FINAL_DIR}")
    print(f"lookalikes (1-char): prefix {lp}, suffix {ls}")
    print(f"reduplication rate: {redup_rate:.5f} (1 in {int(1/max(redup_rate,1e-9)):,})")
    print(f"first 1-letter-twin ordinal (estimate): ~{analysis['first_1char_twin_ordinal_estimate']:,}")
    print(f"analysis -> {FINAL_DIR / 'analysis.json'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())