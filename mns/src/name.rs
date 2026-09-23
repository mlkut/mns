use std::{fmt::Display, str::FromStr};

// The curated 3-letter CVC piece pools, the extra-vowel sets and the decode
// LUTs live in crate::luts (frozen — see design/names.md for how they were
// chosen; there is no generator). A word is <prefix><mid-vowel><suffix><end-vowel>.
use crate::luts::{
    PREFIXES, SUFFIXES, PREFIX_EXTRA_VOWEL, SUFFIX_EXTRA_VOWEL, SLOT_MASK,
    PREFIX_LUT, SUFFIX_LUT,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name(u64);

impl Name {
    /// Maximum number of names that will ever exist (2^40 ≈ 1 trillion).
    /// Registration is capped here — in the code, not just the contract.
    pub const MAX_ORDINALS: u64 = 1 << 40;

    /// Creates a unique Name from the ordinal of on-chain registration.
    ///
    /// Ordinals are capped at [`MAX_ORDINALS`](Self::MAX_ORDINALS): a name
    /// cannot be produced for an ordinal at or beyond the cap — this returns
    /// an error rather than silently mapping onto another ordinal's name.
    pub fn from_ordinal(ordinal: u64) -> Result<Self, &'static str> {
        if ordinal >= Self::MAX_ORDINALS {
            return Err("ordinal at or beyond the 2^40 name cap");
        }
        Ok(Self(permute_ordinal(ordinal)))
    }

    /// Recovers the ordinal used to create this Name (inverse of `from_ordinal`).
    pub fn to_ordinal(&self) -> u64 {
        unpermute_ordinal(self.0)
    }

    /// The internal 48-bit permuted value used for wire encoding and name display.
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    /// Reconstruct a [`Name`] from its internal 48-bit value (the same value
    /// returned by [`Name::as_u64`]).
    pub fn from_raw(val: u64) -> Self {
        Name(val & MASK_48)
    }

    /// Encode the internal 48-bit value as 6 bytes (big-endian) for wire format.
    pub fn to_wire_bytes(&self) -> [u8; 6] {
        let be = self.0.to_be_bytes();
        [be[2], be[3], be[4], be[5], be[6], be[7]]
    }

    /// Decode a [`Name`] from 6 wire-format bytes (big-endian, 48 bits).
    pub fn from_wire_bytes(wire: &[u8; 6]) -> Self {
        let mut buf = [0u8; 8];
        buf[2..8].copy_from_slice(wire);
        Name(u64::from_be_bytes(buf))
    }

    /// Returns the canonical zone domain for this Name.
    pub fn canonical_domain(&self) -> String {
        format!("{}.mns.alt", self.encode())
    }

    pub(crate) fn encode(&self) -> String {
        let val = self.0;
        let mut result = String::with_capacity(17); // 8 + '-' + 8
        encode_word((val >> 24) & MASK_24, &mut result);
        result.push('-');
        encode_word(val & MASK_24, &mut result);
        result
    }

}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.encode())
    }
}

impl FromStr for Name {
    type Err = &'static str;

    /// Parse a [`Name`] from a string.
    ///
    /// Accepts a bare name (`mafojefo-logivada`), a DNS label
    /// (`mafojefo-logivada.mns.alt`), or any string containing the
    /// `XXXXXXXX-XXXXXXXX` pattern. When multiple patterns are present the
    /// **rightmost** match (the domain apex) is returned.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.is_ascii() {
            return Err("name must be ASCII");
        }

        if s.len() == 17 && s.as_bytes()[8] == b'-' {
            return Ok(Name(decode(s)?));
        }

        let bytes = s.as_bytes();
        for start in (0..bytes.len().saturating_sub(16)).rev() {
            if bytes[start + 8] == b'-' {
                let candidate =
                    std::str::from_utf8(&bytes[start..start + 17]).map_err(|_| "invalid UTF-8")?;
                if let Ok(val) = decode(candidate) {
                    return Ok(Name(val));
                }
            }
        }
        Err("could not find a valid mns name in the string")
    }
}

const MASK_24: u64 = 0xFF_FFFF;
const MASK_48: u64 = 0xFFFF_FFFF_FFFF;

/// 24-bit odd round keys for the permutation's Feistel network.
///
/// Derived from 24-bit windows of the golden-ratio constant `0x9E3779B97F4A7C15`,
/// forced odd. Fixed forever — every client must agree. Keys must stay **odd**
/// (see
/// [`round_f`]); changing them, the round count, or the round function is a
/// breaking change (regenerate the golden vectors and `design/names.md`).
const ROUND_KEYS: [u64; 6] = [0x4a7c15, 0x7f4a7d, 0xb97f4b, 0x79b97f, 0x3779b9, 0x9e3779];

/// Mix a 24-bit half: multiply by the round key, XOR two shifted copies.
///
/// Need not be a permutation — Feistel injectivity is structural. Invariant:
/// `r` must be **odd**; an even multiplier zeroes the product's LSB, leaving
/// `round_f`'s low bit a copy of `val`'s bit 7 — a linear leak that made the
/// permutation measurably non-uniform (old even keys measured ~2.2% bias; see
/// `test_permutation_linear_bias_is_noise`).
#[inline(always)]
fn round_f(val: u64, r: u64) -> u64 {
    (val.wrapping_mul(r) ^ (val >> 7) ^ (val << 13)) & MASK_24
}

/// Scrambles an ordinal into a 48-bit value (a bijection) so sequential
/// registrations get unrelated names and no small ordinal looks special.
///
/// A 6-round Feistel network on 24-bit halves ([`round_f`] with
/// [`ROUND_KEYS`]); Feistel is bijective by construction, so no two ordinals
/// collide and the inverse ([`unpermute_ordinal`]) is trivial. Uniformity of
/// the early ordinals is checked by `test_permutation_linear_bias_is_noise`
/// and `test_permutation_slot_uniformity`. Keep the mapping stable once it
/// ships. Not a security primitive: [`Name::to_ordinal`] and these keys are
/// public, so every name is reversible to its ordinal by design.
pub(crate) fn permute_ordinal(x: u64) -> u64 {
    // Offset by one to avoid permuting 0 to 0.
    let x = x.wrapping_add(1) & MASK_48;
    let mut left = (x >> 24) & MASK_24;
    let mut right = x & MASK_24;
    for &r in &ROUND_KEYS {
        let next_right = left ^ round_f(right, r);
        left = right;
        right = next_right;
    }
    (left << 24) | right
}

/// Inverse of [`permute_ordinal`]: the same Feistel rounds run backwards.
pub(crate) fn unpermute_ordinal(x: u64) -> u64 {
    let mut left = (x >> 24) & MASK_24;
    let mut right = x & MASK_24;
    for &r in ROUND_KEYS.iter().rev() {
        let prev_left = right ^ round_f(left, r);
        right = left;
        left = prev_left;
    }
    ((left << 24) | right).wrapping_sub(1) & MASK_48
}

/// Encode a 24-bit value into 8 characters: P(10) + mid vowel(2) + S(10) + end vowel(2).
fn encode_word(bits: u64, out: &mut String) {
    let p = ((bits >> 14) & SLOT_MASK) as usize; // prefix index
    let m = ((bits >> 12) & 3) as usize; // mid-word vowel
    let s = ((bits >> 2) & SLOT_MASK) as usize; // suffix index
    let e = (bits & 3) as usize; // word-end vowel

    out.push_str(PREFIXES[p]);
    out.push(PREFIX_EXTRA_VOWEL[m] as char);
    out.push_str(SUFFIXES[s]);
    out.push(SUFFIX_EXTRA_VOWEL[e] as char);
}

fn decode(encoded: &str) -> Result<u64, &'static str> {
    if !encoded.is_ascii() {
        return Err("name must be ASCII");
    }
    let bytes = encoded.as_bytes();
    if bytes.len() != 17 || bytes[8] != b'-' {
        return Err("name must be 17 characters in the form XXXXXXXX-XXXXXXXX");
    }
    let hi = decode_word(&bytes[0..8])?;
    let lo = decode_word(&bytes[9..17])?;
    Ok((hi << 24) | lo)
}

/// Decode an 8-byte slice (P + mid-vowel + S + end-vowel) into 24 bits.
fn decode_word(word: &[u8]) -> Result<u64, &'static str> {
    if word.len() != 8 {
        return Err("word must be 8 bytes");
    }

    let prefix_str = std::str::from_utf8(&word[0..3]).map_err(|_| "invalid UTF-8 in prefix")?;
    let suffix_str = std::str::from_utf8(&word[4..7]).map_err(|_| "invalid UTF-8 in suffix")?;

    let m = PREFIX_EXTRA_VOWEL
        .iter()
        .position(|&c| c == word[3])
        .ok_or("unknown mid vowel")? as u64;
    let e = SUFFIX_EXTRA_VOWEL
        .iter()
        .position(|&c| c == word[7])
        .ok_or("unknown end vowel")? as u64;

    let prefix_idx = PREFIX_LUT.lookup(prefix_str).ok_or("unknown prefix")? as u64;
    let suffix_idx = SUFFIX_LUT.lookup(suffix_str).ok_or("unknown suffix")? as u64;

    Ok((prefix_idx << 14) | (m << 12) | (suffix_idx << 2) | e)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_lut_uniqueness_and_integrity() {
        // 1. Verify Prefix List Uniqueness
        let mut prefix_set = HashSet::new();
        for &p in &PREFIXES {
            assert!(prefix_set.insert(p), "DUPLICATE PREFIX DETECTED: {}", p);
            assert_eq!(p.len(), 3, "Prefix {} is not 3 chars", p);
        }

        // 2. Verify Suffix List Uniqueness
        let mut suffix_set = HashSet::new();
        for &s in &SUFFIXES {
            assert!(suffix_set.insert(s), "DUPLICATE SUFFIX DETECTED: {}", s);
            assert_eq!(s.len(), 3, "Suffix {} is not 3 chars", s);
        }
    }

    #[test]
    fn test_lut_round_trip_all_prefixes() {
        for (i, &p) in PREFIXES.iter().enumerate() {
            assert_eq!(
                PREFIX_LUT.lookup(p),
                Some(i as u16),
                "prefix '{p}' at index {i}"
            );
        }
    }

    #[test]
    fn test_lut_round_trip_all_suffixes() {
        for (i, &s) in SUFFIXES.iter().enumerate() {
            assert_eq!(
                SUFFIX_LUT.lookup(s),
                Some(i as u16),
                "suffix '{s}' at index {i}"
            );
        }
    }

    #[test]
    fn test_no_forbidden_letters_in_lists() {
        for &p in &PREFIXES {
            for bad in ['c', 'q', 'x', 'y'] {
                assert!(!p.contains(bad), "prefix '{p}' contains forbidden {bad}");
            }
        }
        for &s in &SUFFIXES {
            for bad in ['c', 'q', 'x', 'y'] {
                assert!(!s.contains(bad), "suffix '{s}' contains forbidden {bad}");
            }
        }
    }

    #[test]
    fn test_name_length_and_format() {
        for ordinal in [0u64, 1, 42, 1_000_000, 1_099_511_627_775] {
            let name = Name::from_ordinal(ordinal).unwrap().to_string();
            assert_eq!(name.len(), 17, "name '{name}' should be 17 chars");
            assert_eq!(&name[8..9], "-", "dash should be at position 8 in '{name}'");
        }
    }

    #[test]
    fn test_ordinal_roundtrip() {
        for ordinal in [0u64, 1, 2, 42, 255, 1_000, 1_000_000, 1_099_511_627_775] {
            let name = Name::from_ordinal(ordinal).unwrap();
            assert_eq!(
                name.to_ordinal(),
                ordinal,
                "ordinal roundtrip failed for {ordinal}"
            );
        }
    }

    #[test]
    fn test_permutation_linear_bias_is_noise() {
        // The early ordinals (what actually gets registered) must be spread as
        // if by a random permutation. A cheap, strong sieve is the maximum
        // single-bit linear bias between any output bit and any input bit over
        // the first 2^16 ordinals — ideal (pure noise across the 48x48 tests)
        // is ~0.008 max. The 0.010 bound is ~1.25x that, and the current odd
        // keys measure ~0.006, right at the ideal.
        //
        // This guards the invariant documented on `round_f`: round keys must
        // be odd. The previous even-keyed permutation measured 0.0225 (~3x
        // the ideal and ~11 sigma — real, reproducible bias). Never go back
        // to even keys.
        const N: usize = 1 << 16;
        const WORDS: usize = N / 64;
        let outs: Vec<u64> = (0..N as u64).map(permute_ordinal).collect();

        // Bit-planes: plane[b] has bit i set iff the b-th bit of output i
        // (resp. input i) is 1, packed 64 per word.
        let mut out_planes: [Vec<u64>; 48] = std::array::from_fn(|_| vec![0u64; WORDS]);
        let mut in_planes: [Vec<u64>; 48] = std::array::from_fn(|_| vec![0u64; WORDS]);
        for (i, &o) in outs.iter().enumerate() {
            let w = i >> 6;
            let bit = 1u64 << (i & 63);
            for b in 0..48 {
                if (o >> b) & 1 == 1 {
                    out_planes[b][w] |= bit;
                }
                if (i >> b) & 1 == 1 {
                    in_planes[b][w] |= bit;
                }
            }
        }

        let mut max_bias: f64 = 0.0;
        for ob in 0..48 {
            for ib in 0..48 {
                let mut mism: u32 = 0;
                for w in 0..WORDS {
                    mism += (out_planes[ob][w] ^ in_planes[ib][w]).count_ones();
                }
                let bias = (mism as f64 / N as f64 - 0.5).abs();
                max_bias = max_bias.max(bias);
            }
        }
        assert!(
            max_bias < 0.007,
            "max single-bit linear bias {max_bias:.4} is too high — \
             permutation is measurably non-uniform (are the round keys odd?)"
        );
    }

    #[test]
    fn test_permutation_slot_uniformity() {
        // A name is four 10-bit indexes into the prefix/suffix pools. The first
        // 2^18 ordinals visit each of the 1024 slot values ~256 times, so the
        // chi-square across the 4x1024 buckets should sit at its expectation
        // df = 4095 (sd ~90). The 3800..4450 window is deliberately
        // loose — it only flags gross skew, where some words
        // dominate early names. The current permutation measures ~4100, i.e.
        // at the ideal.
        const N: usize = 1 << 18;
        const SLOTS: usize = 1024;
        const NBUCKETS: usize = 4 * SLOTS;
        let mut counts = [0u64; NBUCKETS];
        for ordinal in 0..N as u64 {
            let v = permute_ordinal(ordinal);
            counts[((v >> 36) & 0x3FF) as usize] += 1; // word 1 prefix
            counts[SLOTS + ((v >> 24) & 0x3FF) as usize] += 1; // word 1 suffix
            counts[2 * SLOTS + ((v >> 12) & 0x3FF) as usize] += 1; // word 2 prefix
            counts[3 * SLOTS + (v & 0x3FF) as usize] += 1; // word 2 suffix
        }
        let expected = N as f64 / SLOTS as f64;
        let chi: f64 = counts
            .iter()
            .map(|&c| {
                let d = c as f64 - expected;
                d * d / expected
            })
            .sum();
        // Chi-square over 4095 df: mean 4095, sd ~90.
        // a lot of slack; both pathological clustering AND suspicious
        // "too-perfect" balancing fail here.
        assert!(
            (3800.0..4450.0).contains(&chi),
            "slot chi-square {chi:.1} outside the expected range for a \
             uniform permutation"
        );
    }

    #[test]
    fn test_permutation_adjacent_diffusion() {
        // Consecutive ordinals must land far apart: roughly half of the 48
        // bits should flip between neighbours, or adjacent registrations would
        // be visibly related. The bounds are wide because the batch is fixed:
        // this only guards against regressions, while
        // test_global_uniqueness_and_diffusion does the tight statistical pass.
        const N: u64 = 1 << 16;
        let mut sum: u64 = 0;
        let mut prev = permute_ordinal(0);
        for ordinal in 1..N {
            let v = permute_ordinal(ordinal);
            sum += (v ^ prev).count_ones() as u64;
            prev = v;
        }
        let mean = sum as f64 / (N - 1) as f64;
        assert!(
            (mean - 24.0).abs() < 0.25,
            "mean adjacent bit change {mean:.3} too far from the ideal 24"
        );
    }

    #[test]
    fn test_string_roundtrip() {
        for ordinal in [0u64, 1, 42, 1_000_000, 1_099_511_627_775] {
            let name = Name::from_ordinal(ordinal).unwrap();
            let encoded = name.to_string();
            let decoded: Name = encoded.parse().expect("parse failed");
            assert_eq!(
                name, decoded,
                "string roundtrip failed for ordinal {ordinal}: '{encoded}'"
            );
        }
    }

    #[test]
    fn test_name_regression_golden_values() {
        // Update these by running the test once with `-- --nocapture` and
        // recording the output, then paste the values here.
        let cases: &[(u64, &str)] = &[
            (0, "mafojefo-logivada"),
            (1, "bugofema-fatagipa"),
            (42, "wubizuko-mabakasa"),
            (1_000_000, "lufovasu-newafopy"),
            (1_099_511_627_775, "kadijiko-sazolija"),
        ];
        for &(ordinal, expected) in cases {
            let actual = Name::from_ordinal(ordinal).unwrap().to_string();
            assert_eq!(
                actual, expected,
                "regression failure for ordinal {ordinal}: got '{actual}', expected '{expected}'"
            );
        }
    }

    #[test]
    #[ignore]
    fn check_names_for_ordinals() {
        for ordinal in 0..=6_u64 {
            let name = Name::from_ordinal(ordinal).unwrap();
            println!("{ordinal:>4}: {name}");
            // println!("{}", name.render_avatar_svg());
        }
    }

    #[test]
    #[ignore]
    fn test_global_uniqueness_and_diffusion() {
        use rayon::prelude::*;

        let count: u64 = 10_000_000;
        let chunk_size: u64 = 100_000;
        let num_chunks = count.div_ceil(chunk_size);

        let chunk_heads: Vec<u64> = (0..num_chunks)
            .map(|c| Name::from_ordinal(c * chunk_size).unwrap().as_u64())
            .collect();

        let results: Vec<(Vec<u64>, u64)> = (0..num_chunks)
            .into_par_iter()
            .map(|chunk| {
                let start = chunk * chunk_size;
                let end = (start + chunk_size).min(count);
                let mut values = Vec::with_capacity((end - start) as usize);
                let mut bit_diff: u64 = 0;
                let mut prev = Name::from_ordinal(start).unwrap().as_u64();
                values.push(prev);
                for ordinal in (start + 1)..end {
                    let v = Name::from_ordinal(ordinal).unwrap().as_u64();
                    bit_diff += (v ^ prev).count_ones() as u64;
                    prev = v;
                    values.push(v);
                }
                (values, bit_diff)
            })
            .collect();

        let mut total_bit_diff: u64 = results.iter().map(|(_, d)| d).sum();
        let all_values: Vec<Vec<u64>> = results.into_iter().map(|(v, _)| v).collect();

        for i in 1..chunk_heads.len() {
            let last_of_prev = *all_values[i - 1].last().unwrap();
            total_bit_diff += (chunk_heads[i] ^ last_of_prev).count_ones() as u64;
        }

        let mut flat: Vec<u64> = all_values.into_iter().flatten().collect();
        flat.par_sort_unstable();
        let original_len = flat.len();
        flat.dedup();
        assert_eq!(
            flat.len(),
            original_len,
            "COLLISION DETECTED in first {count} ordinals"
        );

        println!("Tested {} IDs with 0 collisions.", flat.len());

        let avg_bit_diff = total_bit_diff as f32 / (count - 1) as f32;
        println!("Average bits changed between adjacent IDs: {avg_bit_diff:.2} / 48");
        assert!(
            (avg_bit_diff - 24.0).abs() < 0.10,
            "Poor diffusion: avg bit diff = {avg_bit_diff}"
        );
    }

    #[test]
    fn test_ordinal_cap() {
        // the maximum valid ordinal works and round-trips
        let name = Name::from_ordinal(Name::MAX_ORDINALS - 1).unwrap();
        assert_eq!(name.to_ordinal(), Name::MAX_ORDINALS - 1);
        assert_eq!(name.to_string(), "kadijiko-sazolija");
    }

    #[test]
    fn test_ordinal_over_cap_errors() {
        assert!(Name::from_ordinal(Name::MAX_ORDINALS).is_err());
        assert!(Name::from_ordinal(u64::MAX).is_err());
    }

    #[test]
    fn test_from_str_first_label() {
        // A full DNS name — should pick the rightmost (apex) label.
        let s = "sub-label.foo.mafojefo-logivada.mns.alt";
        let name: Name = s.parse().expect("parse DNS name");
        assert_eq!(name.to_string(), "mafojefo-logivada");
    }

    #[test]
    fn test_from_str_only_name() {
        let name: Name = "mafojefo-logivada".parse().unwrap();
        assert_eq!(name.to_string(), "mafojefo-logivada");
    }

    #[test]
    fn test_from_str_url_name() {
        let name: Name = "https://mafojefo-logivada.mns.mlkut.org".parse().unwrap();
        assert_eq!(name.to_string(), "mafojefo-logivada");
    }

    #[test]
    fn test_from_str_no_suffix_fails() {
        assert!("example.com".parse::<Name>().is_err());
        assert!("foo.bar.baz".parse::<Name>().is_err());
    }

    #[test]
    fn test_from_str_trailing_dot() {
        // FQDN with trailing dot
        let s = "mafojefo-logivada.";
        let name: Name = s.parse().expect("parse FQDN");
        assert_eq!(name.to_string(), "mafojefo-logivada");
    }

    #[test]
    fn test_from_str_dash_prefix() {
        // A subdomain containing a dash before the apex
        let s = "https://sub-mid.mafojefo-logivada.mns.alt";
        let name: Name = s.parse().expect("parse with subdomain containing dash");
        assert_eq!(name.to_string(), "mafojefo-logivada");
    }

    #[test]
    fn test_from_str_rejects_malformed_hyphen() {
        // Hyphen at wrong position
        assert!("mafojefo-logivada".parse::<Name>().is_ok()); // correct
        assert!("mafoj-efologivada".parse::<Name>().is_err()); // wrong hyphen pos
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_string_roundtrip(ordinal in 0u64..1_099_511_627_775u64) {
            let name = Name::from_ordinal(ordinal).unwrap();
            let encoded = name.to_string();
            let decoded: Name = encoded.parse().unwrap();
            prop_assert_eq!(name, decoded);
        }

        #[test]
        fn prop_ordinal_roundtrip(ordinal in 0u64..1_099_511_627_775u64) {
            let name = Name::from_ordinal(ordinal).unwrap();
            prop_assert_eq!(ordinal, name.to_ordinal());
        }
    }
}
