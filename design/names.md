# Names

Names are the human labels for mns addresses. Each name is two words joined by
a dash, e.g. `mafojefo-logivada`.

## What a name looks like

Every name is **17 characters** (two 8-letter words, dash between them). A word
is built from two 3-letter pieces plus two fixed vowels:

```
prefix + mid-vowel + suffix + end-vowel
```

- **prefix** and **suffix** — 3-letter pieces from two fixed lists (above),
- **mid-vowel** — one of `PREFIX_EXTRA_VOWEL = "iaou"`,
- **end-vowel** — one of `SUFFIX_EXTRA_VOWEL = "yaou"`.

So a word always reads consonant-vowel-consonant-vowel… and always ends in a
vowel. The vowels are squeezed in deliberately — `e` is left out because a
silent/weak `e` makes the word stumble (`wofe`-style breaks) and every word
closes cleanly.

## The two fixed piece lists

There are two frozen lists of **1,024 pieces each** — one for the prefix slot,
one for the suffix slot. Every client ships and uses the same lists (they live
in `mns::luts`, together with the code that looks pieces up). Nothing is
generated at run time and nothing is generated later: the lists are fixed.

Pieces are 3 letters, built like **CVC** from:

| position | letters |
|---|---|
| first | `b d f g h j k l m n p r s t v w z` |
| middle | `a e i o u` |
| last | `b d f g h j k l m n p r s t v w z` |

Some letters are left out on purpose:

- `c` and `q` — confusing (a hard/soft pair, `c` reads like s/k/q).
- `x` and `y` — look like typos in the middle of a word (`syxy`, `nuxy`).

The lists started life as the old Urbit piece lists, were expanded, and the
expansions were ranked by how naturally the whole word flows in Spanish /
Italian / Portuguese / Catalan. Then the result was checked by blind testers
(name taste, not objective fact) — more on that at the bottom.

## How many names exist

- **Full space:** ~**281 trillion** possible names.
- **Registration cap:** ~**1 trillion** names (regardless of the space).

## What to expect, in numbers

- **A name built from the same piece twice**, like `wapuhiva-zazozaza` (the
  piece `zaz` appears in both halves of one word), happens about **1 in every
  844 names** — roughly **1,184 per million** names, or **~1.3 billion** of the
  trillion names that can ever exist. It reads like a mild rhyme, it's
  harmless, and it's a deliberate trade-off (the two lists share 621 pieces so
  the name space can stay full).
- **A lookalike** — a registered name that's one character off from yours —
  is the real phishing risk, so it's measured in *time* and *distance*, not
  just chances: see the lookalike table below.

## Lookalikes: how long you're safe

Every name that's just a few characters off from yours is known up-front (it's
in the frozen pool tables), so we can reverse each to its registration ordinal
and say exactly when the first lookalike could exist. For `mafojefo-logivada`:

| how close | first such name | first ordinal | ~time at the rate limit |
|---|---:|---:|---|
| 3 characters | `mafojemo-lonivaja` | 61,334,793 | ~3 years |
| 2 characters | `fafojefo-zogivada` | 28,447,665,756 | ~1,200 years |
| 1 character | `majojefo-logivada` | 1,465,657,709,275 | never — beyond the cap |

A 1-character lookalike is expected only after ~2.2 trillion registrations —
more than the 1-trillion cap ever allows — so in practice your name never has
one. Even at the full cap, only ~36% of names sit within one character of another.
(Avatars have the same anti-phishing story — see [avatars.md](./avatars.md).)

## How an ordinal becomes a name

Registration hands out numbers (ordinals) in order. To keep small numbers from
being special and worth squatting (`1`, `42`), the ordinal is scrambled with a
fixed 48-bit permutation and then split into two 24-bit halves; each half picks
a prefix, a vowel, a suffix, and a vowel:

```
ordinal 0   → mafojefo-logivada
ordinal 1   → bugofema-fatagipa
ordinal 42  → wubizuko-mabakasa
```

The scrambling is one-to-one: every ordinal yields a different name, and any
name can be turned back into its ordinal.

## Design goals, in one line each

- **Fair** — every name has the same shape, and no small ordinal looks special;
  permuting makes names interchangeable (like hashes), so there is nothing to
  hoard.
- **Readable** — open syllables, always ends in a vowel, no tricky letters.
- **Safe** — banned 3-letter strings (`fuk`, `sik`, `jod`, …) are excluded, and
  profanity that can *form* inside a word (e.g. `fak` in `lofakin`) was
  measured while picking the lists.
- **Phishing resistant** — see the lookalike numbers above.

## Why the lists overlap (the one compromise)

A repeated-piece word like `zazozaza` exists only because the prefix and
suffix lists are not fully separate: **621 of the 1,024** prefix pieces are
also usable as suffixes. That overlap is the price of keeping the full
~281-trillion space. The only way to make repeats impossible is 512-piece
lists, which gives **2^44 ≈ 17.6 trillion** names — 16× smaller.

## How the lists were picked (for the record)

The script that made them is gone; this is what was done once:

1. generated every possible 3-letter CVC piece from the alphabet;
2. dropped pieces that spell banned English/Romance words (`fuk`, `sik`, …);
3. seeded from the old Urbit lists and scored the rest by how the whole word
   flows in Spanish/Italian/Portuguese/Catalan;
4. kept the best 1,024 for each slot, minimising lookalike pieces and the
   overlap;
5. blind testers confirmed these read better than the old lists, and the lists
   were frozen.

There is taste in all of this — the lists reflect judgment as much as numbers.
They can change, but any change is a breaking change.