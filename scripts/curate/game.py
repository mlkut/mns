#!/usr/bin/env python3
"""Game: guess whether a name comes from the status quo or the new system.

A CSV-style vibe guesser is trained on quality + texture features (phonotactic
LM score, letter/vowel diversity, reduplication texture, and optionally the
`y`-letter tell) — never on the exact characters of a name. Then the player
deals themselves a shuffled hand and guesses each by feel.

Usage:
  game.py                     # play one shuffled round of 8 names
  game.py --rounds 12         # play 12
  game.py --eval-only         # just measure classifier accuracy (no UI)
  game.py --preview 5         # print 5 fresh names labelled (practice)
"""
import argparse
import math
import random
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))
import generate

import quality as Q
import sample as S

NEW_CAP = 10 ** 12


def sq_name(o, sq) -> str:
    return S.status_quo_name(o, sq)


def new_name(o, pre, suf) -> str:
    return S.ordinal_to_name(o, pre, suf)


# ---------- vibe features (no raw-character lookup) ----------

def features(lm, name: str) -> list[float]:
    toks = [name[0:4], name[4:8], name[9:13], name[13:17]]
    n = name.replace("-", "")
    return [
        lm.name_score(name),                            # 0 phonotactic quality
        n.count("y"),                                   # 1 the Urbit y-flavor tell
        len(set(n)) / 22.0,                             # 2 letter diversity
        len(set(c for c in n if c in "aeiou")) / 5.0,   # 3 vowel diversity
        sum(1 for t in toks if t[1] == t[3]),           # 4 same-vowel monotony
        sum(1 for t in toks if t[0] == t[2] and t[1] == t[3]),  # 5 redup texture
    ]


def y_index() -> int:
    return 1


def sigmoid(z):
    return 1.0 / (1.0 + math.exp(-max(min(z, 30.0), -30.0)))


def train(X, y, iters=600, lr=0.5):
    d = len(X[0])
    w = [0.0] * d
    b = 0.0
    n = len(X)
    for _ in range(iters):
        gw = [0.0] * d
        gb = 0.0
        for xi, yi in zip(X, y):
            p = sigmoid(sum(w[j] * xi[j] for j in range(d)) + b)
            err = p - yi
            for j in range(d):
                gw[j] += err * xi[j]
            gb += err
        for j in range(d):
            w[j] -= lr * gw[j] / n
        b -= lr * gb / n
    return w, b


def normalize(Xs, mean=None, std=None):
    if mean is None:
        mean = [sum(x[j] for x in Xs) / len(Xs) for j in range(len(Xs[0]))]
    if std is None:
        std = [max(1e-9, (sum((x[j] - mean[j]) ** 2 for x in Xs) / len(Xs)) ** 0.5)
               for j in range(len(Xs[0]))]
    return [[(x[j] - mean[j]) / std[j] for j in range(len(x))] for x in Xs], mean, std


def build_train(lm, rng, n_each=1500):
    sq = S._sq_lists()
    pre = [l.strip() for l in open(generate.FILTERED_DIR.parent / "final" / "prefixes.txt")]
    suf = [l.strip() for l in open(generate.FILTERED_DIR.parent / "final" / "suffixes.txt")]
    X, y = [], []
    for _ in range(n_each):
        X.append(features(lm, sq_name(rng.randrange(2 ** 40), sq))); y.append(1)
        X.append(features(lm, new_name(rng.randrange(NEW_CAP), pre, suf))); y.append(0)
    return X, y


def acc(X, y, w, b, drop_y=False) -> float:
    ok = 0
    yi_ = y_index()
    for xi, yi in zip(X, y):
        if drop_y:
            xi = [v for j, v in enumerate(xi) if j != yi_]
            ww = [v for j, v in enumerate(w) if j != yi_]
        else:
            ww = w
        p = sigmoid(sum(ww[j] * xi[j] for j in range(len(xi))) + b)
        ok += int((p >= 0.5) == bool(yi))
    return ok / len(y)


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--rounds", type=int, default=8)
    ap.add_argument("--eval-only", action="store_true")
    ap.add_argument("--preview", type=int, default=0, help="print N labelled names, no guesses")
    ap.add_argument("--seed", type=int, default=2026)
    args = ap.parse_args()

    lm = Q.TrigramLM("wordfreq")
    rng = random.Random(args.seed)

    Xtr, ytr = build_train(lm, rng, n_each=2000)
    k = int(len(Xtr) * 0.7)
    Xt, yt = Xtr[:k], ytr[:k]
    Xv, yv = Xtr[k:], ytr[k:]
    Xn, mean, std = normalize(Xt)
    Xv_n, _, _ = normalize(Xv, mean, std)
    w, b = train(Xn, yt)
    acc_full = acc(Xv_n, yv, w, b)
    acc_no_y = acc(Xv_n, yv, w, b, drop_y=True)
    print(f"holdout evaluation ({len(Xv)} names, SCRIPT never sees these):")
    print(f"  classifier accuracy: {100*acc_full:.1f}%  (with 'y' tell)")
    print(f"  classifier accuracy: {100*acc_no_y:.1f}%  (NOT using 'y')")

    sq = S._sq_lists()
    pre = [l.strip() for l in open(generate.FILTERED_DIR.parent / "final" / "prefixes.txt")]
    suf = [l.strip() for l in open(generate.FILTERED_DIR.parent / "final" / "suffixes.txt")]
    idx = 0
    while idx < args.rounds:
        batch = random.Random(args.seed + idx * 7919)
        n_here = min(args.rounds - idx, 8)
        hands = []
        for k in range(n_here):
            if k % 2 == 0:
                hands.append((sq_name(batch.randrange(2 ** 40), sq), 1, "status quo"))
            else:
                hands.append((new_name(batch.randrange(NEW_CAP), pre, suf), 0, "new"))
        random.Random(args.seed * 31 + idx).shuffle(hands)

        if args.preview and idx == 0:
            print("# practice (labelled):")
            for n_, lab, lbl in hands:
                print(f"  {lbl:<10} {n_}")
            break

        if args.eval_only:
            idx += n_here
            continue

        print(f"\n--- round {idx+1}-{idx+n_here} (status quo = 's', new = 'n') ---")
        for i, (n_, lab, lbl) in enumerate(hands, 1):
            print(f"{i:>2}. {n_}")
        guesses = []
        for i in range(1, n_here + 1):
            g = input(f"  guess #{i}? ").strip().lower()
            while g not in ("s", "n"):
                if g == "q":
                    return 0
                g = input("  (s/n) > ").strip().lower()
            guesses.append(g == "s")
        score = 0
        print("\nanswers:")
        for i, (n_, lab, lbl) in enumerate(hands, 1):
            user = guesses[i - 1]
            correct = user == bool(lab)
            score += int(correct)
            x = features(lm, n_)
            xn = normalize([x], mean, std)[0][0]
            d = len(xn)
            p = sigmoid(sum(w[j] * xn[j] for j in range(d)) + b)
            p_no = sigmoid(sum(w[j] * xn[j] for j in range(d) if j != y_index()) + b)
            mark = "OK " if correct else "XX "
            print(f"{i:>2}. {n_:<30} {lbl:<10} you={'✓' if correct else '✗'}  "
                  f"script={lbl if p >= .5 else ('new' if p < .5 else '?')} "
                  f"(conf {p:.2f})  no-y-conf {p_no:.2f}")
        print(f"round score: {score}/{n_here}")
        idx += n_here
    return 0


if __name__ == "__main__":
    raise SystemExit(main())