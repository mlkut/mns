# Name texture: why new names feel samey (experiment)

Texture = family concentration, reduplicated syllables, repeated vowels. All rows are measured IN MEMORY — final lists untouched. p10/p50 = phonotactic (higher better); B(≤1) = closeness ball; name_v3 = % of names with one vowel ≥3 times in an 8-char word.
redup = tokens like kaka/wuwu (prefix+suffix lists); samev = tokens with both vowels identical; max_fam = largest first-two-letters family size.

## Experiment matrix (clean pool -> rule -> prune -> assign)

| row | pool | shared | redup | lookalikes | p10 | p50 | B(≤1) | P(≤1@1e12) | max_fam | redupT | samevT | name_v3 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| current final lists (post-substitution) |  6,297 | 1,895 | 1-in-8,853 |  99,247 | -1.923 | -1.678 |  99.3 |  29.7% |  75 |  96 | 1596 | 13.0% |

| clean rebuild (no rules)           |  6,080 | 2,112 | 1-in-7,943 | 100,199 | -2.039 | -1.752 |  99.2 |  29.7% |  67 |  98 | 1615 | 14.6% |
|   + no-redup                       |  6,010 | 2,182 | 1-in-7,688 |  99,732 | -2.016 | -1.723 |  97.8 |  29.3% |  64 |   0 | 1795 | 15.0% |
|   + no-redup + no-samevowel        |  4,864 | 3,328 | 1-in-5,041 | 120,416 | -2.030 | -1.736 | 118.3 |  34.3% |  59 |   0 |    0 | 0.0% |
|   + no-redup + family cap 50                 | (pool < 4096 -> cliff to 2^44) |
|   + no-redup + family cap 70       |  5,042 | 3,150 | 1-in-5,326 | 104,776 | -1.916 | -1.670 | 103.9 |  30.9% |  59 |   0 | 1719 | 14.8% |
|   + no-redup + family cap 90       |  6,010 | 2,182 | 1-in-7,688 |  99,752 | -2.031 | -1.735 |  96.4 |  29.0% |  64 |   0 | 1792 | 15.3% |
|   + no-redup + family cap 120      |  6,010 | 2,182 | 1-in-7,688 |  99,752 | -2.026 | -1.735 |  98.2 |  29.4% |  64 |   0 | 1792 | 14.1% |

(All experiment rows are list-level only; a real adoption would add the substitution pass on top, recovering phonotactic quality.)
