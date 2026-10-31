// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! The OID grammar: text ⇄ arcs ⇄ BER, prefix tests, and SNMP's arc order.
//!
//! THE one OID vocabulary for the tree (2026-10-30 audit: PLUGIN.SNMP kept
//! OIDs as dotted strings and compared them textually — `schema.rs`
//! `is_child_of` / `best_section` / `oid_key`; PLUGIN.TRENDNET has its own
//! `oid_from_dotted` / `oid_to_dotted` / `encode_oid` / `read_oid`). Plain
//! Rust, no dependencies.
//!
//! * Text is parsed STRICTLY: every arc a decimal `0..=4294967295`, or the
//!   whole OID is refused ([`OidError`] names the arc). A leading and a
//!   trailing dot are optional.
//! * Order is arc by arc (`Vec<u32>`), so `.1.3.6.1.2.1.9` sorts before
//!   `.1.3.6.1.2.1.10` — the text sorts the other way.
//! * Prefix tests are on arcs, so `1.3.6.1.2.1.1` is NOT a prefix of
//!   `1.3.6.1.2.1.10` (the string test `starts_with("1.3.6.1.2.1.1")` says it is).
//!
//! ```
//! use apkaudio_snmp::Oid;
//! let sys: Oid = "1.3.6.1.2.1.1".parse().unwrap();
//! let descr = Oid::parse(".1.3.6.1.2.1.1.1.0").unwrap();
//! assert!(descr.starts_with(&sys));
//! assert!(!Oid::parse("1.3.6.1.2.1.10").unwrap().starts_with(&sys));
//! assert_eq!(descr.to_string(), "1.3.6.1.2.1.1.1.0");
//! assert_eq!(Oid::from_ber(&descr.to_ber()).unwrap(), descr);
//! ```

use std::fmt;
use std::str::FromStr;

/// Why a text or a BER body is not an OID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OidError {
    /// An arc that is not a decimal number (empty, signed, spaced, a name).
    NotDecimal { oid: String, arc: String },
    /// An arc above SNMP's 32-bit range.
    TooLarge { oid: String, arc: String },
    /// A BER body that is empty, truncated mid-arc, or overflows 32 bits.
    BadBer(&'static str),
}

impl fmt::Display for OidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OidError::NotDecimal { oid, arc } => write!(f, "OID {oid:?}: arc {arc:?} is not a decimal number"),
            OidError::TooLarge { oid, arc } => write!(f, "OID {oid:?}: arc {arc} is above 4294967295"),
            OidError::BadBer(why) => write!(f, "BER OID: {why}"),
        }
    }
}

impl std::error::Error for OidError {}

/// An object identifier as arcs. `Ord` is SNMP's lexicographic arc order —
/// the order GETNEXT walks.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Oid(Vec<u32>);

impl Oid {
    /// The strict text grammar (see the module doc). `""` and `"."` are the root.
    pub fn parse(text: &str) -> Result<Self, OidError> {
        let body = text.strip_prefix('.').unwrap_or(text);
        let body = body.strip_suffix('.').unwrap_or(body);
        if body.is_empty() {
            return Ok(Oid(Vec::new()));
        }
        body.split('.')
            .map(|arc| {
                if arc.is_empty() || !arc.bytes().all(|b| b.is_ascii_digit()) {
                    return Err(OidError::NotDecimal { oid: text.to_string(), arc: arc.to_string() });
                }
                arc.parse::<u32>()
                    .map_err(|_| OidError::TooLarge { oid: text.to_string(), arc: arc.to_string() })
            })
            .collect::<Result<Vec<u32>, _>>()
            .map(Oid)
    }

    pub fn from_arcs(arcs: impl Into<Vec<u32>>) -> Self {
        Oid(arcs.into())
    }

    pub fn arcs(&self) -> &[u32] {
        &self.0
    }

    pub fn into_arcs(self) -> Vec<u32> {
        self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// `1.3.6.1` — no leading dot (net-snmp `-On` style is [`Oid::dotted_leading`]).
    pub fn dotted(&self) -> String {
        self.0.iter().map(u32::to_string).collect::<Vec<_>>().join(".")
    }

    /// `.1.3.6.1` — with the leading dot; the root is `.`.
    pub fn dotted_leading(&self) -> String {
        format!(".{}", self.dotted())
    }

    /// Is `prefix` this OID or an ancestor of it? On arcs, not text.
    pub fn starts_with(&self, prefix: &Oid) -> bool {
        self.0.starts_with(&prefix.0)
    }

    /// The arcs after `prefix` (the instance / index part), if it is a prefix.
    pub fn strip_prefix(&self, prefix: &Oid) -> Option<&[u32]> {
        self.0.strip_prefix(prefix.0.as_slice())
    }

    /// Exactly one level below `parent`.
    pub fn is_child_of(&self, parent: &Oid) -> bool {
        self.0.len() == parent.0.len() + 1 && self.starts_with(parent)
    }

    /// One level up; the root has none.
    pub fn parent(&self) -> Option<Oid> {
        (!self.0.is_empty()).then(|| Oid(self.0[..self.0.len() - 1].to_vec()))
    }

    /// This OID with one more arc.
    pub fn child(&self, arc: u32) -> Oid {
        let mut v = self.0.clone();
        v.push(arc);
        Oid(v)
    }

    /// The candidate that is the longest prefix of this OID (a MIB node for
    /// an instance, the section a field files under). Ties keep the first.
    pub fn longest_prefix<'a, I>(&self, candidates: I) -> Option<&'a Oid>
    where
        I: IntoIterator<Item = &'a Oid>,
    {
        let mut best: Option<&'a Oid> = None;
        for c in candidates {
            if self.starts_with(c) && best.is_none_or(|b| c.len() > b.len()) {
                best = Some(c);
            }
        }
        best
    }

    /// The BER contents octets (X.690 §8.19): the first two arcs share one
    /// sub-identifier (`40·a + b`), every sub-identifier base-128 with the
    /// continuation bit on all but its last septet. A one-arc OID (not legal
    /// on the wire) encodes its single arc rather than nothing.
    pub fn to_ber(&self) -> Vec<u8> {
        let mut out = Vec::new();
        match self.0.as_slice() {
            [] => {}
            [only] => push_base128(&mut out, u64::from(*only)),
            [a, b, rest @ ..] => {
                push_base128(&mut out, u64::from(*a) * 40 + u64::from(*b));
                for arc in rest {
                    push_base128(&mut out, u64::from(*arc));
                }
            }
        }
        out
    }

    /// The BER contents octets back to arcs. The first sub-identifier is split
    /// per X.690: below 40 → `0.x`, below 80 → `1.x`, else `2.(v−80)` — so a
    /// `2.999` (multi-byte first sub-identifier) reads back correctly.
    pub fn from_ber(body: &[u8]) -> Result<Self, OidError> {
        if body.is_empty() {
            return Err(OidError::BadBer("empty"));
        }
        let mut subids: Vec<u64> = Vec::new();
        let mut acc: u64 = 0;
        let mut pending = false;
        for byte in body {
            acc = (acc << 7) | u64::from(byte & 0x7F);
            if acc > u64::from(u32::MAX) + 80 {
                return Err(OidError::BadBer("an arc overflows 32 bits"));
            }
            pending = true;
            if byte & 0x80 == 0 {
                subids.push(acc);
                acc = 0;
                pending = false;
            }
        }
        if pending {
            return Err(OidError::BadBer("truncated inside an arc"));
        }
        let first = subids[0];
        let (a, b) = match first {
            0..=39 => (0, first),
            40..=79 => (1, first - 40),
            _ => (2, first - 80),
        };
        let b = u32::try_from(b).map_err(|_| OidError::BadBer("an arc overflows 32 bits"))?;
        let mut arcs = vec![a, b];
        for s in &subids[1..] {
            arcs.push(u32::try_from(*s).map_err(|_| OidError::BadBer("an arc overflows 32 bits"))?);
        }
        Ok(Oid(arcs))
    }
}

fn push_base128(out: &mut Vec<u8>, mut value: u64) {
    let mut septets = vec![(value & 0x7F) as u8];
    value >>= 7;
    while value > 0 {
        septets.push(((value & 0x7F) as u8) | 0x80);
        value >>= 7;
    }
    septets.reverse();
    out.extend(septets);
}

/// `1.3.6.1.2.1.1.1.0` — no leading dot.
impl fmt::Display for Oid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.dotted())
    }
}

impl FromStr for Oid {
    type Err = OidError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Oid::parse(s)
    }
}

impl From<Vec<u32>> for Oid {
    fn from(v: Vec<u32>) -> Self {
        Oid(v)
    }
}

impl From<&[u32]> for Oid {
    fn from(v: &[u32]) -> Self {
        Oid(v.to_vec())
    }
}

impl AsRef<[u32]> for Oid {
    fn as_ref(&self) -> &[u32] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn o(s: &str) -> Oid {
        Oid::parse(s).unwrap()
    }

    #[test]
    fn prefixes_are_on_arcs_not_text() {
        let system = o("1.3.6.1.2.1.1");
        assert!(o("1.3.6.1.2.1.1.5.0").starts_with(&system));
        assert!(o("1.3.6.1.2.1.1").starts_with(&system), "an OID is its own prefix");
        assert!(!o("1.3.6.1.2.1.10.7").starts_with(&system), "the string test says yes; arcs say no");
        assert_eq!(o("1.3.6.1.2.1.2.2.1.2.7").strip_prefix(&o("1.3.6.1.2.1.2.2.1.2")), Some(&[7u32][..]));
        assert!(o("1.3.6.1.2.1.1.5").is_child_of(&system));
        assert!(!o("1.3.6.1.2.1.1.5.0").is_child_of(&system));
        assert!(!system.is_child_of(&system));
        assert_eq!(o("1.3.6").parent(), Some(o("1.3")));
        assert_eq!(Oid::default().parent(), None);
        assert_eq!(o("1.3").child(6), o("1.3.6"));
    }

    #[test]
    fn the_longest_prefix_wins() {
        let sections = [o("1.3.6.1.4.1.28866"), o("1.3.6.1.4.1.28866.2.11"), o("1.3.6.1.4.1.28866.2.1")];
        let f = o("1.3.6.1.4.1.28866.2.11.3.0");
        assert_eq!(f.longest_prefix(&sections), Some(&sections[1]));
        assert_eq!(o("1.3.6.1.4.1.28866.2.111").longest_prefix(&sections), Some(&sections[0]));
        assert_eq!(o("1.3.6.1.2").longest_prefix(&sections), None);
    }

    #[test]
    fn arc_order_is_snmp_order() {
        let mut v = vec![o(".2.11.22"), o(".2.11.3"), o(".2.11"), o(".2.9")];
        v.sort();
        assert_eq!(v, vec![o(".2.9"), o(".2.11"), o(".2.11.3"), o(".2.11.22")]);
    }

    #[test]
    fn text_round_trips_and_refuses_bad_arcs() {
        let d = o(".1.3.6.1.2.1.1.1.0");
        assert_eq!(d.to_string(), "1.3.6.1.2.1.1.1.0");
        assert_eq!(d.dotted_leading(), ".1.3.6.1.2.1.1.1.0");
        assert_eq!(Oid::default().dotted_leading(), ".");
        assert_eq!("1.3.6.".parse::<Oid>(), Ok(o("1.3.6")));
        assert!(matches!(Oid::parse("1.3.x.6"), Err(OidError::NotDecimal { .. })));
        assert!(matches!(Oid::parse("1.4294967296"), Err(OidError::TooLarge { .. })));
        assert!(Oid::parse("1.3.x.6").unwrap_err().to_string().contains("\"x\""));
    }

    #[test]
    fn ber_round_trips_including_large_and_multibyte_arcs() {
        for s in ["1.3.6.1.2.1.1.1.0", "1.3.6.1.4.1.28866.2.11.3.0", "0.0", "2.999.3", "1.3.4294967295", "2.39"] {
            let oid = o(s);
            assert_eq!(Oid::from_ber(&oid.to_ber()), Ok(oid.clone()), "{s}");
        }
        // The textbook vector: 1.3.6.1.2.1.1.1.0 = 2B 06 01 02 01 01 01 00.
        assert_eq!(o("1.3.6.1.2.1.1.1.0").to_ber(), vec![0x2B, 6, 1, 2, 1, 1, 1, 0]);
        // 28866 = 0x70C2 → 81 E1 42.
        assert_eq!(o("1.3.6.1.4.1.28866").to_ber()[5..], [0x81, 0xE1, 0x42]);
        assert_eq!(Oid::from_ber(&[]), Err(OidError::BadBer("empty")));
        assert_eq!(Oid::from_ber(&[0x2B, 0x86]), Err(OidError::BadBer("truncated inside an arc")));
        assert!(Oid::from_ber(&[0x2B, 0x90, 0x80, 0x80, 0x80, 0x80, 0x00]).is_err(), "overflow");
    }
}
