# mns name quality: phonotactic probability

Model: character-trigram LM on **wordfreq (en, frequency-weighted)**. Higher = smoother / more
English-like. Token = one 4-letter syllable; name = both words.

## Token 4-letter scores

| set | p10 | p25 | p50 | p75 | p90 |
|---|---|--|--|--|--|--|
| status quo | -1.971 | -1.652 | -1.425 | -1.251 | -1.112 |
| new prefixes | -2.033 | -1.822 | -1.559 | -1.318 | -1.144 |
| new suffixes | -1.895 | -1.704 | -1.497 | -1.295 | -1.139 |
| new combined | -1.959 | -1.763 | -1.527 | -1.307 | -1.142 |

## Name scores

| system | p5 | p10 | p25 | p50 | p75 | p90 |
|---|---|--|--|--|--|--|--|
| status quo | -1.916 | -1.838 | -1.695 | -1.569 | -1.451 | -1.349 |
| new | -2.009 | -1.935 | -1.813 | -1.679 | -1.553 | -1.448 |

## What quality costs in name space

Name space is fixed by the slot encoding (12 bits for 4096 tokens), NOT by pool size.
The range only collapses at a hard cliff: pool < 4096 drops to 11-bit slots (2^44),
pool < 2048 to 2^40. Between 4096 and 8192 the real cost of a smaller pool is more
forced overlap (reduplication) + more lookalikes — not fewer names.

| drop worst % by score | pool after join-prune | slot bits | name space |
|---|---|---|---|
| 10% | 5,539 | 12 | 2^48 (281T) |
| 20% | 4,922 | 12 | 2^48 (281T) |
| 30% | 4,308 | 12 | 2^48 (281T) |
| 40% | 3,886 | 11 | 2^44 (17.6T) |
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
| -2.294 | `wumu` | P |
| -2.293 | `wuju` | P |
| -2.287 | `kugi` | P |
| -2.287 | `gupu` | P |
| -2.287 | `buvu` | P |
| -2.286 | `fapu` | P |
| -2.280 | `piwa` | P |
| -2.280 | `muwu` | P |
| -2.278 | `zeju` | P |
| -2.278 | `vunu` | P |
| -2.277 | `lije` | P |
| -2.277 | `nafo` | P |
| -2.277 | `vojo` | P |
| -2.274 | `puhi` | P |
| -2.274 | `kezo` | P |
| -2.274 | `zawu` | P |

## `tuwuwaha` case

word score -2.591 — only 0.0% of 5000 sampled new names score this low (0.0% of status quo) -> a genuine outlier, not 'just new'.

## Watchlist (`scripts/data/quality/watchlist.txt`)

| entry | score | share of new names at/below it |
|---|---|---|
| `tuwuwaha` | -2.591 | 0.0% |

Run `quality.py sweep` for the quality/overlap tradeoff.
