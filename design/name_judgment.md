# Blind LLM judgment: status quo vs new names (multi-round)

Ordinals sampled log-uniform in [1, 1,000,000,000) — the range real registrations will exhaust first. Names: id 2: 2x60 / id 3: 2x60 per round.

## Round id 2 (120 names, files: ratings_2_b1A.json, ratings_2_b1B.json, ratings_2_b2A.json, ratings_2_b2B.json, ratings_2_b3A.json, ratings_2_b3B.json, ratings_2_b4A.json, ratings_2_b4B.json, ratings_2_b5A.json, ratings_2_b5B.json, ratings_2_b6A.json, ratings_2_b6B.json)

| system | pronounceable | memorable | pleasant | safe (typo/visual) | real-name feel | overall |
|---|---|---|---|---|---|---|
| sq | 3.36 | 3.14 | 2.89 | 3.13 | 2.58 | 3.02 |
| new | 3.64 | 3.27 | 3.28 | 3.55 | 2.79 | 3.31 |

| run-to-run Δ | 0.43 | 0.53 | 0.54 | 0.47 | 0.64 |

## Round id 3 (120 names, files: ratings_3_b1A.json, ratings_3_b1B.json, ratings_3_b2A.json, ratings_3_b2B.json, ratings_3_b3A.json, ratings_3_b3B.json, ratings_3_b4A.json, ratings_3_b4B.json, ratings_3_b5A.json, ratings_3_b5B.json, ratings_3_b6A.json, ratings_3_b6B.json)

| system | pronounceable | memorable | pleasant | safe (typo/visual) | real-name feel | overall |
|---|---|---|---|---|---|---|
| sq | 3.27 | 2.97 | 2.70 | 3.05 | 2.41 | 2.88 |
| new | 3.77 | 3.08 | 3.00 | 3.51 | 2.58 | 3.19 |

| run-to-run Δ | 0.35 | 0.39 | 0.48 | 0.44 | 0.64 |

## Pooled across rounds

| system | pronounceable | memorable | pleasant | safe (typo/visual) | real-name feel | overall | n |
|---|---|---|---|---|---|---|---|
| sq | 3.31 | 3.05 | 2.80 | 3.09 | 2.50 | 2.95 | 120 |
| new | 3.70 | 3.17 | 3.14 | 3.53 | 2.69 | 3.25 | 120 |

Best 40: sq, new, new, new, sq, sq, sq, sq, new, new, new, new, new, new, new, new, new, new, new, new, new, new, new, new, new, sq, sq, sq, sq, new, new, new, sq, sq, new, new, new, new, sq, new   70% new
Worst 40: new, new, sq, sq, sq, sq, new, new, sq, sq, sq, new, new, sq, sq, sq, sq, new, new, new, sq, sq, sq, sq, sq, sq, sq, sq, new, sq, new, sq, sq, sq, sq, sq, sq, sq, sq, sq   27% new

## Agreement with script metrics

Spearman ρ between judge overall and the phonotactic LM score (pooled n=240): **0.23**

