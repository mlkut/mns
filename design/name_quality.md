# mns name quality: phonotactic probability

Model: character-trigram LM on **wordfreq (en, frequency-weighted)**. Higher = smoother / more
English-like. Token = one 4-letter syllable; name = both words.

## Token 4-letter scores

| set | p10 | p25 | p50 | p75 | p90 |
|---|---|--|--|--|--|--|
| status quo | -1.971 | -1.652 | -1.425 | -1.251 | -1.112 |
| new prefixes | -2.017 | -1.805 | -1.546 | -1.306 | -1.141 |
| new suffixes | -1.880 | -1.692 | -1.488 | -1.288 | -1.134 |
| new combined | -1.942 | -1.748 | -1.517 | -1.300 | -1.138 |

## Name scores

| system | p5 | p10 | p25 | p50 | p75 | p90 |
|---|---|--|--|--|--|--|--|
| status quo | -1.916 | -1.838 | -1.695 | -1.569 | -1.451 | -1.349 |
| new | -1.988 | -1.916 | -1.796 | -1.668 | -1.542 | -1.437 |

## What quality costs in name space

Name space is fixed by the slot encoding (12 bits for 4096 tokens), NOT by pool size.
The range only collapses at a hard cliff: pool < 4096 drops to 11-bit slots (2^44),
pool < 2048 to 2^40. Between 4096 and 8192 the real cost of a smaller pool is more
forced overlap (reduplication) + more lookalikes — not fewer names.

| drop worst % by score | pool after join-prune | slot bits | name space |
|---|---|---|---|
| 10% | 5,410 | 12 | 2^48 (281T) |
| 20% | 4,808 | 12 | 2^48 (281T) |
| 30% | 4,207 | 12 | 2^48 (281T) |
| 40% | 3,796 | 11 | 2^44 (17.6T) |
| substitution (no drop) | 6,349 | 12 | **2^48 (281T)** |

`substitute` re-admits join-cleaned-but-good tokens instead of dropping: pool and range
stay put, redup improves.

## Worst 20 tokens in the final lists

| score | token | slot |
|---|---|---|
| -2.588 | `kije` | P |
| -2.334 | `kafo` | P |
| -2.308 | `kifo` | P |
| -2.294 | `kijo` | P |
| -2.274 | `kezo` | P |
| -2.262 | `buvi` | P |
| -2.259 | `pifo` | P |
| -2.259 | `laje` | P |
| -2.259 | `kuju` | P |
| -2.258 | `jifo` | P |
| -2.257 | `fovu` | P |
| -2.257 | `mupu` | P |
| -2.256 | `piwu` | P |
| -2.254 | `pohu` | P |
| -2.254 | `jemu` | P |
| -2.253 | `ziwe` | P |
| -2.253 | `zugu` | P |
| -2.252 | `rije` | P |
| -2.252 | `laju` | P |
| -2.250 | `vopo` | P |

## `tuwuwaha` case

word score -2.591 — only 0.0% of 5000 sampled new names score this low (0.0% of status quo) -> a genuine outlier, not 'just new'.

## Watchlist (`scripts/data/quality/watchlist.txt`)

| entry | score | share of new names at/below it |
|---|---|---|
| `tuwuwaha` | -2.591 | 0.0% |

Run `quality.py sweep` for the quality/overlap tradeoff.
