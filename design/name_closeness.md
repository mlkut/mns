# Name closeness at scale (phishing exposure)

Metric: pick a name uniformly at random; after N ordinals are registered,
what is P(someone can find a name within 1 / ≤2 letters of it)? Measured with
B = average distinct-neighbour ball (exact enumeration, 400 samples),
then P = 1 - (1 - B/T)^N.

| system | T (names) | avg K(d=1) | avg K(d=2) | K1 range | B(≤1) | B(≤2) |
|---|---|---|---|---|---|---|
| status quo (2^40) | 1,099,511,627,776 | 73.84 | 2,634.53 | 46..96 | 74.84 | 2,709.37 |
| new (2^48) | 281,474,976,710,656 | 97.57 | 4,799.65 | 68..126 | 98.57 | 4,898.22 |

N where a random name expects its first close (≤1) neighbour: status quo (2^40): ~1.469e+10, new (2^48): ~2.856e+12.

| system | registered N | P(1 letter off) | P(≤1) | P(≤2) | expected close (≤1) |
|---|---|---|---|---|---|
| status quo (2^40) | 1,000,000 | 0.0067% | 0.0068% | 0.2461% | 0.00 |
| status quo (2^40) | 1,000,000,000 | 6.4956% | 6.5806% | 91.4920% | 0.07 |
| status quo (2^40) | 1,000,000,000,000 | 100.0000% | 100.0000% | 100.0000% | 68.07 |
| new (2^48) | 1,000,000 | 0.0000% | 0.0000% | 0.0017% | 0.00 |
| new (2^48) | 1,000,000,000 | 0.0347% | 0.0350% | 1.7251% | 0.00 |
| new (2^48) | 1,000,000,000,000 | 29.2920% | 29.5428% | 100.0000% | 0.35 |

> d=0 (identical names) is a separate word-space birthday event, not counted here;
> B(≤1)/B(≤2) implicitly include the name itself. Status-quo numbers use the
> committed name.rs lists; 'new' uses scripts/data/final/*.txt.
