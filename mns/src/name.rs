use std::{fmt::Display, str::FromStr};

// Urbit-derived prefix list (256 entries).
// All 'c' replaced with 'k' for phonetic clarity.
const PREFIXES: [&str; 256] = [
    "doz", "mar", "bin", "wan", "sam", "lit", "sig", "hid", "fid", "lis", "sog", "dir", "wak",
    "sab", "wis", "sib", "rig", "sol", "dop", "mod", "fog", "lid", "hop", "dar", "dor", "lor",
    "hod", "fol", "rin", "tog", "sil", "mir", "hol", "pas", "lak", "rov", "liv", "dal", "sat",
    "lib", "tab", "han", "tik", "pid", "tor", "bol", "fos", "dot", "los", "dil", "for", "pil",
    "ram", "tir", "win", "tad", "bik", "dif", "rok", "wid", "bis", "das", "mid", "lop", "ril",
    "nar", "dap", "mol", "san", "lok", "nov", "sit", "nid", "tip", "sik", "rop", "wit", "nat",
    "pan", "min", "rit", "pod", "mot", "tam", "tol", "sav", "pos", "nap", "nop", "som", "fin",
    "fon", "ban", "mor", "wor", "sip", "ron", "nor", "bot", "wik", "sok", "wat", "dol", "mag",
    "pik", "dav", "bid", "bal", "tim", "tas", "mal", "lig", "siv", "tag", "pad", "sal", "div",
    "dak", "tan", "sid", "fab", "tar", "mon", "ran", "nis", "wol", "mis", "pal", "las", "dis",
    "map", "rab", "tob", "rol", "lat", "lon", "nod", "nav", "fig", "nom", "nib", "pag", "sop",
    "ral", "bil", "had", "dok", "rid", "mok", "pak", "rav", "rip", "fal", "tod", "til", "tin",
    "hap", "mik", "fan", "pat", "tak", "lab", "mog", "sim", "son", "pin", "lom", "rik", "tap",
    "fir", "has", "bos", "bat", "pok", "hak", "tid", "hav", "sap", "lin", "dib", "hos", "dab",
    "bit", "bar", "rak", "par", "lod", "dos", "bor", "tok", "hil", "mak", "tom", "dig", "fil",
    "fas", "mit", "hob", "har", "mig", "hin", "rad", "mas", "hal", "rag", "lag", "fad", "top",
    "mop", "hab", "nil", "nos", "mil", "fop", "fam", "dat", "nol", "din", "hat", "nak", "ris",
    "fot", "rib", "hok", "nim", "lar", "fit", "wal", "rap", "sar", "nal", "mos", "lan", "don",
    "dan", "lad", "dov", "riv", "bak", "pol", "lap", "tal", "pit", "nam", "bon", "ros", "ton",
    "fob", "pon", "sov", "nok", "sor", "lav", "mat", "mip", "fep",
];

// Urbit-derived suffix list (256 entries).
// All 'c' replaced with 'k'.
const SUFFIXES: [&str; 256] = [
    "zod", "nek", "bud", "wes", "sev", "per", "sut", "let", "ful", "pen", "syt", "dur", "wep",
    "ser", "wyl", "sun", "ryp", "syk", "dyr", "nup", "heb", "pog", "lup", "dep", "dys", "but",
    "lug", "hek", "ryt", "tyv", "syd", "nex", "lun", "mep", "lut", "sep", "pes", "del", "sul",
    "ked", "tem", "led", "tul", "met", "wen", "byn", "hex", "feb", "pyl", "dul", "het", "mev",
    "rut", "tyl", "wyd", "tep", "bes", "dex", "sef", "wyk", "bur", "der", "nep", "pur", "rys",
    "reb", "den", "nut", "sub", "pet", "rul", "syn", "reg", "tyd", "sup", "sem", "wyn", "rek",
    "meg", "net", "sek", "mul", "nym", "tev", "web", "sum", "mut", "nyx", "rex", "teb", "fus",
    "hep", "ben", "mus", "wyx", "sym", "sel", "ruk", "dek", "wex", "syr", "wet", "dyl", "myn",
    "mes", "det", "bet", "bel", "tux", "tug", "myr", "pel", "syp", "ter", "meb", "set", "dut",
    "deg", "tex", "sur", "fel", "tud", "nux", "rux", "ren", "wyt", "nub", "med", "lyt", "dus",
    "neb", "rum", "tyn", "seg", "lyx", "pun", "res", "red", "fun", "rev", "ref", "mek", "ted",
    "rus", "bex", "leb", "dux", "ryn", "num", "pyx", "ryg", "ryx", "fep", "tyr", "tus", "tyk",
    "leg", "nem", "fer", "mer", "ten", "lus", "nus", "syl", "tek", "mex", "pud", "rym", "tuk",
    "fyl", "lep", "deb", "ber", "mug", "hut", "tun", "byl", "sud", "pem", "dev", "lur", "def",
    "bus", "bep", "run", "mel", "pex", "dyt", "byt", "typ", "lev", "myl", "wed", "duk", "fur",
    "fex", "nul", "luk", "len", "ner", "lex", "rup", "ned", "lek", "ryd", "lyd", "fen", "wel",
    "nyd", "hus", "rel", "rud", "nes", "hes", "fet", "des", "ret", "dun", "ler", "nyr", "seb",
    "hul", "ryl", "lud", "rem", "lys", "fyn", "wer", "ryk", "sug", "nys", "nyl", "lyn", "dyn",
    "dem", "lux", "fed", "sed", "bek", "mun", "lyr", "tes", "mud", "nyt", "byr", "sen", "weg",
    "fyr", "mur", "tel", "rep", "teg", "pek", "nel", "nev", "fes",
];

// Middle of word vowel, `e` is not as strong so we replace it with `u`.
const PREFIX_EXTRA_VOWEL: [u8; 4] = *b"iaou";
// End of word vowel, `e` is silent, so we skip it.
const SUFFIX_EXTRA_VOWEL: [u8; 4] = *b"yaou";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name(u64);

impl Name {
    /// Creates a unique Name from the ordinal of on-chain registration.
    pub fn from_ordinal(ordinal: u64) -> Self {
        Self(permute_ordinal(ordinal))
    }

    /// Recovers the ordinal used to create this Name (inverse of `from_ordinal`).
    pub fn to_ordinal(&self) -> u64 {
        unpermute_ordinal(self.0)
    }

    /// The internal 40-bit permuted value used for wire encoding and name display.
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    /// Reconstruct a [`Name`] from its internal 40-bit value (the same value
    /// returned by [`Name::as_u64`]).
    pub fn from_raw(val: u64) -> Self {
        Name(val & MASK_40)
    }

    /// Encode the internal 40-bit value as 5 bytes (big-endian) for wire format.
    pub fn to_wire_bytes(&self) -> [u8; 5] {
        let be = self.0.to_be_bytes();
        [be[3], be[4], be[5], be[6], be[7]]
    }

    /// Decode a [`Name`] from 5 wire-format bytes (big-endian, 40 bits).
    pub fn from_wire_bytes(wire: &[u8; 5]) -> Self {
        let mut buf = [0u8; 8];
        buf[3..8].copy_from_slice(wire);
        Name(u64::from_be_bytes(buf))
    }

    /// Returns the canonical zone domain for this Name.
    pub fn canonical_domain(&self) -> String {
        format!("{}.mns.alt", self.encode())
    }

    pub(crate) fn encode(&self) -> String {
        let val = self.0;
        let mut result = String::with_capacity(17); // 8 + '-' + 8
        encode_word((val >> 20) & MASK_20, &mut result);
        result.push('-');
        encode_word(val & MASK_20, &mut result);
        result
    }

    /// Renders a 40-bit value as a 9×9 mirrored pixel grid SVG.
    /// Layout:
    ///   Row 0: bits 0,1,2 — 3 central pixels, mirrored: c=2,6→bit0  c=3,5→bit1  c=4→bit2
    ///   Rows 1–7: bits 3–37 — full 5-col source mirrored to 9 cols
    ///   Row 8: bits 38,39 — 2 central pixels, mirrored: c=3,5→bit38  c=4→bit39
    pub fn render_avatar_svg(&self) -> String {
        let value = self.0;
        const ROWS: usize = 9;
        const COLS: usize = 9;
        const MARGIN: f32 = 16.0;
        const CELL: f32 = 26.0;
        const BOARD: f32 = MARGIN * 2.0 + COLS as f32 * CELL; // 266

        let mut rects = String::new();

        for r in 0..ROWS {
            for c in 0..COLS {
                let bit: u64 = match r {
                    0 => match c {
                        2 | 6 => value & 1,
                        3 | 5 => (value >> 1) & 1,
                        4 => (value >> 2) & 1,
                        _ => 0,
                    },
                    8 => match c {
                        3 | 5 => (value >> 38) & 1,
                        4 => (value >> 39) & 1,
                        _ => 0,
                    },
                    _ => {
                        let src_col = if c < 5 { c } else { 8 - c };
                        let src_idx = 3 + (r - 1) * 5 + src_col;
                        (value >> src_idx) & 1
                    }
                };

                if bit == 1 {
                    let x = MARGIN + c as f32 * CELL + 0.5;
                    let y = MARGIN + r as f32 * CELL + 0.5;
                    rects.push_str(&format!(
                        r#"<rect class="mns-avatar-pixel" x="{x}" y="{y}" width="25" height="25"/>"#,
                    ));
                }
            }
        }

        format!(
            r#"<svg class="mns-avatar" width="100%" height="100%" viewBox="0 0 {BOARD} {BOARD}" xmlns="http://www.w3.org/2000/svg"><style>.mns-pixel{{fill:currentColor}}</style>{rects}</svg>"#,
        )
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
    /// Accepts a bare name (`mokomedu-tasosuna`), a DNS label
    /// (`mokomedu-tasosuna.mns.alt`), or any string containing the
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

const MASK_20: u64 = 0xF_FFFF;
const MASK_40: u64 = 0xFF_FFFF_FFFF;

#[inline(always)]
fn round_f(val: u64, r: u64) -> u64 {
    (val.wrapping_mul(r) ^ (val >> 7) ^ (val << 13)) & MASK_20
}

/// A 40-bit bijective permutation. No collisions across [0, 2^40).
/// Feistel network with 4 rounds
pub fn permute_ordinal(x: u64) -> u64 {
    // Offset by one to avoid permuting 0 to 0.
    let x = x.wrapping_add(1) & MASK_40;
    let mut left = (x >> 20) & MASK_20;
    let mut right = x & MASK_20;
    const R: [u64; 4] = [0x9E377, 0x6C62D, 0xB5A4B, 0xD2F3E];
    for &r in &R {
        let next_right = left ^ round_f(right, r);
        left = right;
        right = next_right;
    }
    (left << 20) | right
}

/// Inverse of `permute_ordinal`.
pub fn unpermute_ordinal(x: u64) -> u64 {
    let mut left = (x >> 20) & MASK_20;
    let mut right = x & MASK_20;
    const R: [u64; 4] = [0x9E377, 0x6C62D, 0xB5A4B, 0xD2F3E];
    for &r in R.iter().rev() {
        let prev_left = right ^ round_f(left, r);
        right = left;
        left = prev_left;
    }
    ((left << 20) | right).wrapping_sub(1) & MASK_40
}

/// Encode a 20-bit value into 8 ASCII characters: PREFIX + VOWEL_P + SUFFIX + VOWEL_S.
fn encode_word(bits: u64, out: &mut String) {
    let hi = (bits >> 10) & 0x3FF; // top 10 bits
    let lo = bits & 0x3FF; // bottom 10 bits

    out.push_str(PREFIXES[(hi >> 2) as usize]); // prefix_idx: bits[9..2]
    out.push(PREFIX_EXTRA_VOWEL[(hi & 0x3) as usize] as char); // vowel_p:    bits[1..0]
    out.push_str(SUFFIXES[(lo >> 2) as usize]); // suffix_idx: bits[9..2]
    out.push(SUFFIX_EXTRA_VOWEL[(lo & 0x3) as usize] as char); // vowel_s:    bits[1..0]
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
    Ok((hi << 20) | lo)
}

/// Decode an 8-byte slice (PREFIX + VOWEL_P + SUFFIX + VOWEL_S) into 20 bits.
fn decode_word(word: &[u8]) -> Result<u64, &'static str> {
    debug_assert_eq!(word.len(), 8);

    let prefix_str = std::str::from_utf8(&word[0..3]).map_err(|_| "invalid UTF-8 in prefix")?;
    let vowel_p = word[3];
    let suffix_str = std::str::from_utf8(&word[4..7]).map_err(|_| "invalid UTF-8 in suffix")?;
    let vowel_s = word[7];

    let prefix_idx = PREFIX_LUT.lookup(prefix_str).ok_or("unknown prefix")? as u64;
    let vowel_p_idx = PREFIX_EXTRA_VOWEL
        .iter()
        .position(|&v| v == vowel_p)
        .ok_or("invalid vowel after prefix")? as u64;
    let suffix_idx = SUFFIX_LUT.lookup(suffix_str).ok_or("unknown suffix")? as u64;
    let vowel_s_idx = SUFFIX_EXTRA_VOWEL
        .iter()
        .position(|&v| v == vowel_s)
        .ok_or("invalid vowel after suffix")? as u64;

    let hi = (prefix_idx << 2) | vowel_p_idx;
    let lo = (suffix_idx << 2) | vowel_s_idx;
    Ok((hi << 10) | lo)
}

/// Slots: (packed_key, index). packed_key == u32::MAX means empty.
struct Lut<const N: usize> {
    slots: [(u32, u8); N],
}

impl<const N: usize> Lut<N> {
    const fn build(list: &[&str; 256]) -> Self {
        let mut slots = [(u32::MAX, 0u8); N];
        let mut i = 0usize;
        while i < 256 {
            let b = list[i].as_bytes();
            let key = (b[0] as u32) | ((b[1] as u32) << 8) | ((b[2] as u32) << 16);
            let mut slot = ((key ^ (key >> 16)) as usize) & (N - 1);
            loop {
                if slots[slot].0 == u32::MAX {
                    slots[slot] = (key, i as u8);
                    break;
                }
                slot = (slot + 1) & (N - 1);
            }
            i += 1;
        }
        Self { slots }
    }

    fn lookup(&self, s: &str) -> Option<u8> {
        if s.len() != 3 {
            return None;
        }
        let b = s.as_bytes();
        let key = (b[0] as u32) | ((b[1] as u32) << 8) | ((b[2] as u32) << 16);
        let mut slot = ((key ^ (key >> 16)) as usize) & (N - 1);
        loop {
            let (k, v) = self.slots[slot];
            if k == u32::MAX {
                return None;
            }
            if k == key {
                return Some(v);
            }
            slot = (slot + 1) & (N - 1);
        }
    }
}

// N = 512: power of two, > 256, keeps load factor < 0.5.
static PREFIX_LUT: Lut<512> = Lut::build(&PREFIXES);
static SUFFIX_LUT: Lut<512> = Lut::build(&SUFFIXES);

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
                Some(i as u8),
                "prefix '{p}' at index {i}"
            );
        }
    }

    #[test]
    fn test_lut_round_trip_all_suffixes() {
        for (i, &s) in SUFFIXES.iter().enumerate() {
            assert_eq!(
                SUFFIX_LUT.lookup(s),
                Some(i as u8),
                "suffix '{s}' at index {i}"
            );
        }
    }

    #[test]
    fn test_no_c_in_lists() {
        for &p in &PREFIXES {
            assert!(!p.contains('c'), "prefix '{p}' still contains 'c'");
        }
        for &s in &SUFFIXES {
            assert!(!s.contains('c'), "suffix '{s}' still contains 'c'");
        }
    }

    #[test]
    fn test_name_length_and_format() {
        for ordinal in [0u64, 1, 42, 1_000_000, 0xFF_FFFF_FFFF] {
            let name = Name::from_ordinal(ordinal).to_string();
            assert_eq!(name.len(), 17, "name '{name}' should be 17 chars");
            assert_eq!(&name[8..9], "-", "dash should be at position 8 in '{name}'");
        }
    }

    #[test]
    fn test_ordinal_roundtrip() {
        for ordinal in [0u64, 1, 2, 42, 255, 1_000, 1_000_000, 0xFF_FFFF_FFFF] {
            let name = Name::from_ordinal(ordinal);
            assert_eq!(
                name.to_ordinal(),
                ordinal,
                "ordinal roundtrip failed for {ordinal}"
            );
        }
    }

    #[test]
    fn test_string_roundtrip() {
        for ordinal in [0u64, 1, 42, 1_000_000, 0xFF_FFFF_FFFF] {
            let name = Name::from_ordinal(ordinal);
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
            (0, "mokomedu-tasosuna"),
            (1, "sikuteby-natubeku"),
            (42, "tabofena-fituregu"),
            (1_000_000, "hanitega-mopuseny"),
            (0xFF_FFFF_FFFF, "dozizody-dozizody"),
        ];
        for &(ordinal, expected) in cases {
            let actual = Name::from_ordinal(ordinal).to_string();
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
            let name = Name::from_ordinal(ordinal);
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
            .map(|c| Name::from_ordinal(c * chunk_size).as_u64())
            .collect();

        let results: Vec<(Vec<u64>, u64)> = (0..num_chunks)
            .into_par_iter()
            .map(|chunk| {
                let start = chunk * chunk_size;
                let end = (start + chunk_size).min(count);
                let mut values = Vec::with_capacity((end - start) as usize);
                let mut bit_diff: u64 = 0;
                let mut prev = Name::from_ordinal(start).as_u64();
                values.push(prev);
                for ordinal in (start + 1)..end {
                    let v = Name::from_ordinal(ordinal).as_u64();
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
        println!("Average bits changed between adjacent IDs: {avg_bit_diff:.2} / 40");
        assert!(
            (avg_bit_diff - 20.0).abs() < 0.10,
            "Poor diffusion: avg bit diff = {avg_bit_diff}"
        );
    }

    #[test]
    fn test_from_str_first_label() {
        // A full DNS name — should pick the rightmost (apex) label.
        let s = "hodolena-ravatudu.foo.risomuty-wanoweta.mns.alt";
        let name: Name = s.parse().expect("parse DNS name");
        assert_eq!(name.to_string(), "risomuty-wanoweta");
    }

    #[test]
    fn test_from_str_only_name() {
        let name: Name = "mokomedu-tasosuna".parse().unwrap();
        assert_eq!(name.to_string(), "mokomedu-tasosuna");
    }

    #[test]
    fn test_from_str_url_name() {
        let name: Name = "https://mokomedu-tasosuna.mns.mlkut.org".parse().unwrap();
        assert_eq!(name.to_string(), "mokomedu-tasosuna");
    }

    #[test]
    fn test_from_str_no_suffix_fails() {
        assert!("example.com".parse::<Name>().is_err());
        assert!("foo.bar.baz".parse::<Name>().is_err());
    }

    #[test]
    fn test_from_str_trailing_dot() {
        // FQDN with trailing dot
        let s = "tabofena-fituregu.";
        let name: Name = s.parse().expect("parse FQDN");
        assert_eq!(name.to_string(), "tabofena-fituregu");
    }

    #[test]
    fn test_from_str_dash_prefix() {
        // A subdomain like "sub-mid.mokomedu-tasosuna.mns.alt"
        let s = "https://sub-mid.mokomedu-tasosuna.mns.alt";
        let name: Name = s.parse().expect("parse with subdomain containing dash");
        assert_eq!(name.to_string(), "mokomedu-tasosuna");
    }

    #[test]
    fn test_from_str_rejects_malformed_hyphen() {
        // Hyphen at wrong position
        assert!("mokomedu-tasosuna".parse::<Name>().is_ok()); // correct
        assert!("mokom-edutasosuna".parse::<Name>().is_err()); // wrong hyphen pos
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_string_roundtrip(ordinal in 0u64..=0xFF_FFFF_FFFFu64) {
            let name = Name::from_ordinal(ordinal);
            let encoded = name.to_string();
            let decoded: Name = encoded.parse().unwrap();
            prop_assert_eq!(name, decoded);
        }

        #[test]
        fn prop_ordinal_roundtrip(ordinal in 0u64..=0xFF_FFFF_FFFFu64) {
            let name = Name::from_ordinal(ordinal);
            prop_assert_eq!(ordinal, name.to_ordinal());
        }
    }
}
