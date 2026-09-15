# Name similarity: how close can registered names get?

This document answers a design question: as ordinals are registered in order,
how similar can the resulting *displayed names* become? It records the exact
measurements for three schemes: the original 256-entry Urbit-derived lists,
the intermediate 1024-entry mixed-list experiment, and the current CVCV-only
curated lists.

## Model

A name is `<word1>-<word2>`, each word is 8 characters. Ordinal → 40-bit value
is a bijective Feistel permutation (`permute_ordinal`); the 40-bit value is
split `p1 s1 p2 s2` into four 10-bit syllable indexes. Word1 = `P[p1]+S[s1]`,
word2 = `P[p2]+S[s2]`.

Two names are "close" by how many aligned (same-position) letters differ
across the whole 16-character string. A **1-letter twin** is the classic
phishing vector (`mukuhuvi-lyjudimi` vs `mukuhuvi-lyjudimi`-with-one-letter-
changed); identical words (`ladupehu-powapuve` vs `ladupehu-sorogevu`) are
*not* phishing-relevant — they still differ in 8 of 8 letters of the other
word.

Because closeness requires values to collide on the *word* level (identical
`word1`, or identical `word2`), and each word is a 20-bit value, the identical-
word class is governed by the birthday bound of the 2^20 word space regardless
of the syllable lists.

## Scheme A — original 256-entry Urbit lists (vowel bit)

| class | first pair (ordinals) |
|---|---|
| identical word (benign) | **235** (`linobudu-midiruto` vs `linobudu-dasodyny`) |
| near-identical word (same prefix+suffix, vowels differ) | **~158** (`malotuda-…` vs `malutudy-…`) |
| whole-name 1-letter twin | **200,704** |
| whole-name ≤2-letter twin | **~47,250** |

## Scheme B — intermediate 1024-entry mixed CVCV+VCVC lists

Decisive structure: 0-char twin impossible (no two syllables differ by one
aligned letter), but half the syllables were VCVC (vowel-initial,
consonant-final), which made names unpronounceable (`ekogjuru-ojonyhiw`).
Abandoned for pronunciation reasons.

| class | first pair (ordinals) |
|---|---|
| whole-name 1-letter twin | **never** |
| whole-name ≤2-letter twin | **145,879** |

## Scheme C — current CVCV-only curated lists (committed)

Every syllable is consonant-vowel-consonant-vowel, so every syllable begins
and ends with a vowel (`mukuhuvi-lyjudimi`). `scripts/syllables.py`:

1. minimises "lookalike" pairs within each list (syllables differing by one
   aligned letter): ~830/524k pairs, ~0.16% — the CVCV-only alphabet makes a
   zero-conflict 1024-code impossible (~590 would be the max),
2. guarantees no two 10-bit labels differing in one bit map to syllables
   sharing an aligned 2-gram (verified 0/5120 violating Hamming edges).

Measured exactly (same scan method):

| class | first pair (ordinals) |
|---|---|
| identical word (benign) | **235** (inherent 2^20-word birthday, unchanged) |
| whole-name 1-letter twin | **265,572** (`nisenyme-kowopimo` vs `nisenyte-kowopimo`) |
| whole-name ≤2-letter twin | **109,648** (`ladupehu-powapuve` vs `ladutahu-powapuve`) |

| property | Scheme A | Scheme B | Scheme C (current) |
|---|---|---|---|
| pronounceable (vowel-final) | yes | **no** | **yes** |
| 1-letter twin | from 200,704 | never | from ~265,572 |
| ≤2-letter twin | from ~47,250 | from 145,879 | from ~109,648 |
| single-bit flip → syllable | often still looks alike | zero aligned letters | zero aligned letters |

## Why the numbers are what they are

- **Identical words arrive at ~235 in every scheme.** The Feistel distributes
  ordinals uniformly across the 2^20 `word1` values; the first repeated `word1`
  is a 2^20 birthday event (~1024 expected, 235 observed). This cannot be
  changed by the syllable lists — it is a property of the 2×20-bit encoding.
- **1-letter twins are bounded by the list's lookalike pairs.** A 1-letter
  twin needs two registered names that share 3 syllables and use a lookalike
  pair in the 4th. With ~830 lookalike pairs per list the first such name
  arrives at ~265k; making it *impossible* would require B's zero-lookalike
  code, which the CVCV-only alphabet can't provide at 1024 entries.
- **The single-bit-flip guarantee is orthogonal and kept.** The Hamming-label
  guarantee (verified 0/5120) is what protects Hamming-adjacent registrations
  and single-bit wire corruption — the situations closest in ordinal — and
  does not depend on the lookalike count.

## Metric caveat

All figures use **aligned Hamming distance** over the 16-character string.
Levenshtein distance (insert/delete allowed) produces very similar rankings
for these lists and does not change the headline conclusions.

## How to regenerate the exact numbers

The scan keys off the two syllable arrays plus the Feistel constants. The
working script (simplified) is:

```
for ordinal in 0..:
    v = permute_ordinal(ordinal)
    word1vals[word1(v)].append(ordinal)
    for each prior ordinal sharing word1: d = aligned(word2(a), word2(b))
        record the first d<=1 and d<=2 occurrences
    (repeat bucketing on word2)
```

Update this document (and `rate_limit.md` tables) any time the syllable lists
or permutation change.