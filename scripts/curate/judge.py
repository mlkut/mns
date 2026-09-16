#!/usr/bin/env python3
"""Blind LLM judging of name quality (human-surrogate).

Three modes:
  sample     draw N blind names (half status-quo, half new), labels kept secret
  prompt     emit the locked judging prompt for one sub-batch (20 names)
  aggregate  load ratings JSONs + secret labels -> design/name_judgment.md

Names are drawn with game.py's exact ordinal ranges: status quo over [0, 2^40)
(via name.rs permutation + lists), new over [0, 1e12) (48-bit Feistel + final lists).

Usage:
  judge.py sample --count 60 --seed 7
  judge.py prompt --id 1 --batch 1
  judge.py aggregate --id 1
"""
import argparse
import json
import math
import random
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))
import generate

import sample as S

JUDGE_DIR = generate.FILTERED_DIR.parent / "quality" / "judge"
PROMPT_DIR = JUDGE_DIR / "prompts"
DESIGN_DIR = generate.FILTERED_DIR.parent / "quality" / "reports"

DIMS = ["pron", "mem", "pleas", "safe", "real"]
DIM_LABEL = {"pron": "pronounceable", "mem": "memorable",
             "pleas": "pleasant", "safe": "safe (typo/visual)", "real": "real-name feel"}
NEW_CAP = 10 ** 12
EARLY_CAP = 10 ** 9  # ordinals we'll actually see first (decades/centuries)

PROMPT_SKELETON = """You are judging short two-word names the way a thoughtful person would.

Base every judgment ONLY on the strings listed below. Do NOT use any tools, do
NOT search, and do NOT reason about where the names might have come from or about
any pattern in how they were generated (there is nothing to reverse-engineer).

Rate each name 1 (worst) .. 5 (best) on five dimensions:
  pronounceable - easy to say aloud, flows, unlikely to trip a reader
  memorable     - easy to remember; visually distinctive without being messy
  pleasant      - you'd enjoy having / using / owning this name
  safe          - easy to type, read, and recognize; few lookalike/mistyping risks
  real          - feels like a genuine name rather than a typo or gibberish

Return STRICT JSON: a top-level JSON list of objects, one per name, in the same
order, each with keys "i" (the name number), %s, and a short "note" (one phrase
of your honest gut reaction). Do not output anything besides the JSON list.
Example: [{"i":1,"pron":4,"mem":3,"pleas":4,"safe":5,"real":3,"note":"clean and easy"}]

Names:
%s"""


# ---------- sampling ----------

def sq_name(o, sq) -> str:
    return S.status_quo_name(o, sq)


def new_name(o, pre, suf) -> str:
    return S.ordinal_to_name(o, pre, suf)


def biased_ordinal(rng, cap: int) -> int:
    """Log-uniform ordinal in [1, cap): equal mass per decade, so the small
    ordinals (that real registrations will exhaust first) dominate the sample."""
    return int(10 ** rng.uniform(0.0, math.log10(max(cap, 10)))) % max(cap, 10)


def sample_names(count, seed, cap):
    rng = random.Random(seed)
    sq = S._sq_lists()
    pre = [l.strip() for l in open(generate.FILTERED_DIR.parent / "final" / "prefixes.txt")]
    suf = [l.strip() for l in open(generate.FILTERED_DIR.parent / "final" / "suffixes.txt")]
    half = count // 2
    names = ([sq_name(biased_ordinal(rng, cap), sq) for _ in range(half)] +
             [new_name(biased_ordinal(rng, cap), pre, suf) for _ in range(half)])
    labels = ["sq"] * half + ["new"] * half
    order = list(range(len(names)))
    random.Random(seed + 1).shuffle(order)
    names = [names[i] for i in order]
    labels = [labels[i] for i in order]
    return names, labels


# ---------- modes ----------

def cmd_sample(args):
    cap = args.cap if args.cap else EARLY_CAP
    names, labels = sample_names(args.count, args.seed, cap)
    JUDGE_DIR.mkdir(parents=True, exist_ok=True)
    (JUDGE_DIR / f"names_{args.id}.txt").write_text("\n".join(names) + "\n")
    (JUDGE_DIR / f"labels_{args.id}.json").write_text(
        json.dumps(labels, indent=0) + "\n")
    n_batches = math.ceil(args.count / args.batch_size)
    print(f"wrote {len(names)} names (biased ordinals in [1, {cap:,})) -> "
          f"{JUDGE_DIR / f'names_{args.id}.txt'}  ({n_batches} batches of <= {args.batch_size})")
    return 0


def cmd_prompt(args):
    names = [l.strip() for l in open(JUDGE_DIR / f"names_{args.id}.txt") if l.strip()]
    start = (args.batch - 1) * args.batch_size
    chunk = names[start:start + args.batch_size]
    body = "\n".join(f"{i+1}. {n}" for i, n in enumerate(chunk, start))
    json_keys = ", ".join(f'"{d}"' for d in DIMS)
    prompt = PROMPT_SKELETON % (json_keys, body)
    PROMPT_DIR.mkdir(parents=True, exist_ok=True)
    out = PROMPT_DIR / f"b{args.id}_{args.batch}.txt"
    out.write_text(prompt)
    print(prompt)
    print(f"\n-> {out}")
    return 0


# ---------- aggregation ----------

def spearman(xs, ys):
    n = len(xs)
    if n < 2:
        return float("nan")
    def rank(v):
        s = sorted(range(n), key=lambda i: v[i])
        r = [0.0] * n
        i = 0
        while i < n:
            j = i
            while j + 1 < n and v[s[j + 1]] == v[s[i]]:
                j += 1
            avg = (i + j) / 2 + 1
            for k in range(i, j + 1):
                r[s[k]] = avg
            i = j + 1
        return r
    rx, ry = rank(xs), rank(ys)
    mx, my = sum(rx) / n, sum(ry) / n
    num = sum((a - mx) * (b - my) for a, b in zip(rx, ry))
    den = (sum((a - mx) ** 2 for a in rx) * sum((b - my) ** 2 for b in ry)) ** 0.5
    return num / den if den else float("nan")


def stats(vs):
    s = sorted(vs)
    n = len(s)
    def q(p):
        return s[min(n - 1, int(p * (n - 1)))]
    return {"mean": sum(s) / n, "med": q(.5), "p10": q(.1), "p90": q(.9)}


def load_id(id_):
    names = [l.strip() for l in open(JUDGE_DIR / f"names_{id_}.txt") if l.strip()]
    labels = json.loads((JUDGE_DIR / f"labels_{id_}.json").read_text())
    ratings_files = sorted(JUDGE_DIR.glob(f"ratings_{id_}_*.json"))
    per_name = {i: {d: [] for d in DIMS} for i in range(len(names))}
    notes = {}
    for f in ratings_files:
        try:
            data = json.loads(f.read_text())
        except json.JSONDecodeError as e:
            print(f"BAD rating file {f}: {e}", file=sys.stderr)
            return None
        for row in data:
            i = row["i"] - 1
            for d in DIMS:
                per_name[i][d].append(float(row[d]))
            notes[i] = row.get("note", "")
    return names, labels, per_name, notes, ratings_files


def avg(vals):
    return sum(vals) / len(vals) if vals else float("nan")


def cmd_aggregate(ids, args):
    all_rows = []  # (system, dim means, overall, id)
    L = [f"# Blind LLM judgment: status quo vs new names (multi-round)\n",
         f"Ordinals sampled log-uniform in [1, {EARLY_CAP:,}) — the range real "
         f"registrations will exhaust first. Names: " +
         " / ".join(f"id {i}: 2x{len([l for l in json.loads((JUDGE_DIR/f'labels_{i}.json').read_text()) if l=='sq'])}"
                    for i in ids) +
         " per round.\n"]

    for id_ in ids:
        loaded = load_id(id_)
        if loaded is None:
            return 2
        names, labels, per_name, notes, files = loaded
        grouped = {"sq": [], "new": []}
        for i, lab in enumerate(labels):
            grouped[lab].append(i)

        L.append(f"## Round id {id_} ({len(names)} names, files: "
                 f"{', '.join(f.name for f in files)})\n")
        L.append("| system | " + " | ".join(f"{DIM_LABEL[d]}" for d in DIMS) + " | overall |")
        L.append("|---|---" + "|---" * len(DIMS) + "|")
        for lab in ("sq", "new"):
            cells = []
            for d in DIMS:
                vs = [avg(per_name[i][d]) for i in grouped[lab]]
                cells.append(f"{stats(vs)['mean']:.2f}")
            ov = [avg([avg(per_name[i][d]) for d in DIMS]) for i in grouped[lab]]
            L.append(f"| {lab} | " + " | ".join(cells) + f" | {stats(ov)['mean']:.2f} |")
            for i in grouped[lab]:
                all_rows.append((lab, [avg(per_name[i][d]) for d in DIMS],
                                 avg([avg(per_name[i][d]) for d in DIMS]), id_))

        noise = {}
        for d in DIMS:
            ds = []
            for i in range(len(names)):
                v = per_name[i][d]
                if len(v) >= 2:
                    ds.append(abs(v[0] - v[-1]))
            noise[d] = sum(ds) / len(ds) if ds else float("nan")
        L.append("")
        L.append("| run-to-run Δ | " + " | ".join(f"{noise[d]:.2f}" for d in DIMS) + " |")
        L.append("")
        if id_ == ids[0]:
            names0, labels0, per0, notes0, _ = loaded
    L.append("## Pooled across rounds\n")
    L.append("| system | " + " | ".join(f"{DIM_LABEL[d]}" for d in DIMS) + " | overall | n |")
    L.append("|---|---" + "|---" * len(DIMS) + "|---|")
    for lab in ("sq", "new"):
        rows = [r for r in all_rows if r[0] == lab]
        cells = [f"{stats([r[1][k] for r in rows])['mean']:.2f}" for k in range(len(DIMS))]
        L.append(f"| {lab} | " + " | ".join(cells)
                 + f" | {stats([r[2] for r in rows])['mean']:.2f} | {len(rows)} |")
    w = None
    L.append("")
    wins = sum(1 for r in all_rows if r[0] == "new" and r[2] > 0) if False else None
    # pairwise-style: fraction of better-end names on the extremes (best/worst halves)
    ranked = sorted(all_rows, key=lambda r: r[2], reverse=True)
    top = ranked[:max(10, len(ranked) // 6)]
    bot = ranked[-max(10, len(ranked) // 6):]
    L.append(f"Best {len(top)}: " + ", ".join(f"{'new' if r[0]=='new' else 'sq'}"
             for r in top) + "   " + f"{sum(1 for r in top if r[0]=='new') * 100 // len(top)}% new")
    L.append(f"Worst {len(bot)}: " + ", ".join(f"{'new' if r[0]=='new' else 'sq'}"
             for r in bot) + f"   {sum(1 for r in bot if r[0]=='new') * 100 // len(bot)}% new")
    L.append("")

    L.append("## Agreement with script metrics\n")
    try:
        from quality import TrigramLM
        lm = TrigramLM("wordfreq")
    except BaseException as e:
        lm = None
        L.append(f"(LM skipped: {e})\n")
    if lm is not None:
        obs = []
        for id_ in ids:
            nn, nl, pp, no, _ = load_id(id_)
            for i in range(len(nn)):
                ov = avg([avg(pp[i][d]) for d in DIMS]) if any(pp[i][d] for d in DIMS) else float("nan")
                if ov == ov:
                    obs.append((lm.name_score(nn[i]), ov))
        rho = spearman([o[0] for o in obs], [o[1] for o in obs])
        L.append(f"Spearman ρ between judge overall and the phonotactic LM score "
                 f"(pooled n={len(obs)}): **{rho:.2f}**\n")

    out = "\n".join(L) + "\n"
    print(out)
    DESIGN_DIR.mkdir(parents=True, exist_ok=True)
    (DESIGN_DIR / "name_judgment.md").write_text(out)
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("mode", choices=["sample", "prompt", "aggregate"])
    ap.add_argument("--id", type=int, default=1)
    ap.add_argument("--ids", type=str, default="1", help="aggregate: comma list of round ids")
    ap.add_argument("--count", type=int, default=60)
    ap.add_argument("--seed", type=int, default=7)
    ap.add_argument("--cap", type=int, default=0, help="upper ordinal (default 1e9, log-biased)")
    ap.add_argument("--batch", type=int, default=1)
    ap.add_argument("--batch-size", type=int, default=20)
    args = ap.parse_args()
    if args.mode == "sample":
        return cmd_sample(args)
    if args.mode == "prompt":
        return cmd_prompt(args)
    return cmd_aggregate([int(x) for x in args.ids.split(",")], args)


if __name__ == "__main__":
    raise SystemExit(main())