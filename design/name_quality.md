# mns name quality: phonotactic probability

Model: character-trigram LM on **wordfreq (en, frequency-weighted)**. Higher = smoother / more
English-like. Token = one 4-letter syllable; name = both words.

## Token 4-letter scores

| set | p10 | p25 | p50 | p75 | p90 |
|---|---|--|--|--|--|--|
| status quo | -1.971 | -1.652 | -1.425 | -1.251 | -1.112 |
| new prefixes | -2.029 | -1.821 | -1.558 | -1.318 | -1.151 |
| new suffixes | -1.890 | -1.704 | -1.501 | -1.301 | -1.141 |
| new combined | -1.956 | -1.758 | -1.528 | -1.309 | -1.145 |

## Name scores

| system | p5 | p10 | p25 | p50 | p75 | p90 |
|---|---|--|--|--|--|--|--|
| status quo | -1.916 | -1.838 | -1.695 | -1.569 | -1.451 | -1.349 |
| new | -1.994 | -1.919 | -1.806 | -1.677 | -1.551 | -1.448 |

## What quality costs in name space

Name space is fixed by the slot encoding (12 bits for 4096 tokens), NOT by pool size.
The range only collapses at a hard cliff: pool < 4096 drops to 11-bit slots (2^44),
pool < 2048 to 2^40. Between 4096 and 8192 the real cost of a smaller pool is more
forced overlap (reduplication) + more lookalikes — not fewer names.

| drop worst % by score | pool after join-prune | slot bits | name space |
|---|---|---|---|
| 10% | 5,472 | 12 | 2^48 (281T) |
| 20% | 4,864 | 12 | 2^48 (281T) |
| 30% | 4,256 | 12 | 2^48 (281T) |
| 40% | 3,840 | 11 | 2^44 (17.6T) |
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
| -2.270 | `zefo` | P |
| -2.270 | `vepo` | P |
| -2.267 | `wuge` | P |
| -2.266 | `puje` | P |
| -2.265 | `vumu` | P |
| -2.262 | `buvi` | P |
| -2.262 | `zovo` | P |
| -2.261 | `zomu` | P |
| -2.260 | `wohu` | P |
| -2.259 | `pifo` | P |
| -2.259 | `laje` | P |
| -2.259 | `kuju` | P |
| -2.258 | `jifo` | P |
| -2.257 | `fovu` | P |
| -2.257 | `mupu` | P |

## `tuwuwaha` case

word score -2.591 — only 0.0% of 5000 sampled new names score this low (0.0% of status quo) -> a genuine outlier, not 'just new'.

## Watchlist (`scripts/data/quality/watchlist.txt`)

| entry | score | share of new names at/below it |
|---|---|---|
| `tuwuwaha` | -2.591 | 0.0% |

Run `quality.py sweep` for the quality/overlap tradeoff.
