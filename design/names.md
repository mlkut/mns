# Names

Names are the human-readable identifiers of mns. This document specifies the
name format and the prefix/suffix pools the system uses. A name looks like
`davufezo-hosojise`.

## Format

A name is two words joined by a dash:

```
<prefix><suffix>-<prefix><suffix>
```

Each word is 8 letters long and made of two 4-letter pieces. Every piece is
**CVCV** (consonant–vowel–consonant–vowel) and always ends in a vowel — so
every word, and every name, ends in a vowel and flows cleanly.

## Design goals

Every name produced by these pools has the same properties:

- **Fixed shape** — always 17 characters, always lowercase, always the same rhythm.
- **Easy to say** — every piece is open (vowel-final), there are no consonant
  clusters, and awkward letter pairs or doubled letters never appear.
- **Clean as far as we can tell** — known offensive words and fragments are
  removed both from individual pieces and from every possible joining of a
  prefix and a suffix inside a word; the entire set of word combinations is
  checked, not just the pieces.
- **Unremarkable** — deliberately childish repeats like `kaka` or `wuwu` are
  excluded, and a word that repeats the same piece is extremely rare.
- **Far apart** — registration is capped well below the name space, so a
  name you are given almost never has a registered near-twin; typos and
  phishing lookalikes usually point nowhere.
- **Fair by construction** — every domain is uniform, with nothing that
  incentivizes grabbing or selling any particular one; no early number is
  special and all names are interchangeable, like hashes.

## The registration cap

Registration is capped at **2^40 ≈ 1 trillion names** — far below the full
name space (281 trillion) — so most names stay far apart.

**How likely is it that someone finds a name similar to yours?**

| registered so far | name 1 letter away | name within 2 letters |
|---|---|---|
| 1,000,000,000 | ~0.03% | ~1.7% |
| 1,000,000,000,000 (the cap) | ~29% | ~100% |

Even at the full cap, a name only *expects* its first close (≤1-letter)
neighbour after ~2.9 trillion registrations — beyond the cap.

## Pools

Two fixed, ordered lists of **4,096** four-letter CVCV pieces — a prefix
pool and a suffix pool, using the alphabet:

17 consonants — `b d f g h j k l m n p r s t v w z` — and 5 vowels — `a e i o u`.

With 4,096 choices per slot, the name space is 4096⁴, roughly **281 trillion**
distinct names.

The pools are fixed and shipped: the Rust crate carries them as `mns::PREFIXES`
and `mns::SUFFIXES` (`mns/src/pools.rs`) — nothing is generated at run time.

## How a name is generated from an ordinal

Registration hands out ordinals in order, and each ordinal maps to exactly one
name. The raw ordinal is a number, so it is scrambled and then written as
words:

1. the ordinal is passed through a fixed 48-bit permutation (a bijection, so
   no two ordinals can ever produce the same name, no small ordinal looks
   special, and neighbouring ordinals turn into unrelated names),
2. the permuted value is split into four 12-bit indexes,
3. the first two indexes pick the pieces of the first word (a prefix then a
   suffix), the last two pick the pieces of the second word.

Words are what we show instead of the number, because a word is far easier to
read, type, pronounce, and remember than the twelve hex digits it stands for:

```
ordinal 0  →  0x12728e48358a  →  davufezo-hosojise
ordinal 1  →  0x1164ec8a5e16  →  danuhuni-mojavake
ordinal 42 →  0x747ebaf58d49  →  lehuwadi-zakatari
```

Early ordinals are therefore the first names anyone sees, and every name is
guaranteed to be used eventually.

## Why names at all

Any registry could simply hand out ordinals — an *ordinal* is the cheapest way
to address something. Our goal is that every mns.alt domain is uniform and
nothing incentivizes grabbing or selling any one of them, the way a memorable
number would. Permuting every ordinal keeps all domains interchangeable — like
hashes — while the permutation above still lets anyone verify a name's
ordinal on lookup. And the thing people actually see is a name, not the
permuted number, because a name can be read, typed, pronounced, and remembered
without effort.

## Why these pools

We started from a scheme based on lightly modified Urbit prefixes and
suffixes, with a vowel appended to each, giving a **2^40** namespace — about
a trillion names. It worked, but a trillion names felt small: once that many
are registered, a name close to the one you were given becomes likely. The
pools below exist mainly for a **larger namespace** — 2^48, about 281
trillion — so that even if we ever mint all trillion names, close neighbours
stay rare. (The new names were also compared with the earlier scheme on sound
and closeness; those were checks, not the reason.)

The goals behind the current choices, in plain terms:

- **Why CVCV pieces.** A piece that ends in a vowel — and a name where every
  piece ends in a vowel — is easier to say, spell, and remember across many
  languages. Consonant endings and clusters, such as endings like `-by`, `-y`,
  `-x` or `-gy`, are exactly what make invented names stumble and look like
  typos.
- **Why these letters.** The `c` is dropped because its hard/soft ambiguity is
  unnecessary (the hard sound is written `k`). `q` and `x` are dropped because
  they have no consistent sound across languages. `y` is dropped because it was
  the single biggest source of names that look like typos.
- **Why two pools of 4,096.** This is a pool size that keeps names
  pronounceable while making the space spacious enough that the names people
  actually use stay far apart from each other — a lookalike name is unlikely to
  already exist. Keeping prefixes and suffixes as *separate* lists means a word
  is always built from two different parts.
- **Why the strict bans.** Offense can't be defined perfectly — languages and
  standards differ — so the blocklist is deliberately conservative and open to
  review, and it is checked across joins, not just individual pieces. Banning
  patterns (like `uw`) rather than single words is what keeps the pools easy to
  maintain and hard to slip past.
- **Why we believe this is an improvement.** When a fresh, context-free
  evaluator is handed samples without knowing the source, it consistently
  prefers these pools over the earlier scheme — especially how names sound and
  how unlikely they are to be misread or mistyped.

There is taste in all of this; the pools reflect judgment, not a theorem, and
they are revised whenever that judgment changes.
