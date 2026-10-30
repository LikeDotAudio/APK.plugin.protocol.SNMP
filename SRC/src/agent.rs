// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
// Native SNMP agent responder: an in-memory OID tree (set, get, get-next) for
// Python. It is not an SNMP stack -- no socket, no PDU.
//
// THE OID TEXT IS PARSED STRICTLY (LOOSE-snmp-agent). `parse_oid` used to be
// `filter_map(str::parse::<u32>().ok())`, which dropped a malformed, negative,
// empty or over-32-bit arc without a word, so `1.3.x.6` and `1.3.6` became one
// key and a write to either was a write to the other. Now any arc that is not a
// decimal 0..=4294967295 refuses the whole OID, and the Python methods raise
// `ValueError` naming the arc. The pure half below compiles without the
// `python` feature so `cargo test` checks it with no interpreter attached.

#[cfg(feature = "python")]
use pyo3::exceptions::PyValueError;
#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyDict;
#[cfg(feature = "python")]
use std::collections::BTreeMap;
#[cfg(feature = "python")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "python")]
#[pyclass]
struct SnmpAgent {
    tree: Arc<Mutex<BTreeMap<Vec<u32>, String>>>,
}

#[cfg(feature = "python")]
fn parsed(oid: &str) -> PyResult<Vec<u32>> {
    parse_oid(oid).map_err(PyValueError::new_err)
}

#[cfg(feature = "python")]
#[pymethods]
impl SnmpAgent {
    #[new]
    fn new() -> Self {
        SnmpAgent {
            tree: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    fn update_oid(&self, oid: String, value: String) -> PyResult<()> {
        let key = parsed(&oid)?;
        self.tree.lock().unwrap().insert(key, value);
        Ok(())
    }

    fn get_oid(&self, oid: String) -> PyResult<Option<String>> {
        let key = parsed(&oid)?;
        Ok(self.tree.lock().unwrap().get(&key).cloned())
    }

    fn get_next(&self, py: Python<'_>, oid: String) -> PyResult<Option<Py<PyAny>>> {
        let key = parsed(&oid)?;
        let tree = self.tree.lock().unwrap();
        // The first OID strictly greater than the target, in arc order.
        for (o, v) in tree.range(key.clone()..) {
            if o > &key {
                let dict = PyDict::new_bound(py);
                dict.set_item("oid", format_oid(o))?;
                dict.set_item("value", v)?;
                return Ok(Some(dict.into_any().unbind()));
            }
        }
        Ok(None)
    }

    fn clear(&self) {
        self.tree.lock().unwrap().clear();
    }
}

/// OID text to arcs. Leading and trailing dots are optional (`.1.3.6.` and
/// `1.3.6` are one key; `""` and `"."` are the root). Every arc between them
/// must be a decimal number in SNMP's 32-bit range, or the OID is refused.
pub fn parse_oid(oid: &str) -> Result<Vec<u32>, String> {
    let body = oid.strip_prefix('.').unwrap_or(oid);
    let body = body.strip_suffix('.').unwrap_or(body);
    if body.is_empty() {
        return Ok(Vec::new());
    }
    body.split('.')
        .map(|arc| {
            if arc.is_empty() || !arc.bytes().all(|b| b.is_ascii_digit()) {
                return Err(format!("OID {oid:?}: arc {arc:?} is not a decimal number"));
            }
            arc.parse::<u32>()
                .map_err(|_| format!("OID {oid:?}: arc {arc} is above 4294967295"))
        })
        .collect()
}

/// Arcs to OID text, always with the leading dot.
pub fn format_oid(oid: &[u32]) -> String {
    let parts: Vec<String> = oid.iter().map(|n| n.to_string()).collect();
    format!(".{}", parts.join("."))
}

#[cfg(feature = "python")]
#[pymodule]
pub fn oasnmpagent_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SnmpAgent>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    //! `SnmpAgent`'s own methods need a live interpreter to call, so what is
    //! pinned here is the pair of pure functions every one of them goes
    //! through -- the OID text that becomes a cache key -- plus the ordering
    //! property `get_next` depends on.

    use super::*;

    fn ok(oid: &str) -> Vec<u32> {
        parse_oid(oid).unwrap()
    }

    #[test]
    fn an_oid_round_trips_and_a_missing_leading_dot_is_put_back() {
        assert_eq!(ok(".1.3.6.1.2.1"), vec![1, 3, 6, 1, 2, 1]);
        assert_eq!(ok("1.3.6.1.2.1"), vec![1, 3, 6, 1, 2, 1]);
        assert_eq!(format_oid(&[1, 3, 6, 1, 2, 1]), ".1.3.6.1.2.1");

        // Both spellings land on one key, so `update_oid(".1.3.6")` and
        // `get_oid("1.3.6")` are the same node. That is the point of the pair.
        assert_eq!(ok("1.3.6"), ok(".1.3.6."));
    }

    #[test]
    fn the_key_type_gives_snmp_lexicographic_order_and_its_text_would_not() {
        // `get_next` walks a `BTreeMap<Vec<u32>, _>`. SNMP orders OIDs arc by
        // arc, so .1.3.6.1.2.1.10 comes AFTER .1.3.6.1.2.1.9 -- which is what a
        // `Vec<u32>` compares as, and what the text does not.
        let ninth = ok(".1.3.6.1.2.1.9");
        let tenth = ok(".1.3.6.1.2.1.10");

        assert!(ninth < tenth, "arc order: 9 before 10");
        assert!(
            format_oid(&tenth) < format_oid(&ninth),
            "and the text of those same two OIDs sorts the other way, which is \
             why the key is not a String"
        );
        // A prefix is always less than what extends it.
        assert!(ok(".1.3.6.1") < ok(".1.3.6.1.0"));
    }

    #[test]
    fn a_malformed_arc_refuses_the_oid_rather_than_merging_two_keys() {
        // Each of these used to lose an arc silently and collide with `1.3.6`.
        for bad in ["1.3.x.6", "1.-3.6", "1..6", "1.3.4294967296.6", "1. 3.6", "1.+3.6"] {
            let refused = parse_oid(bad);
            assert!(refused.is_err(), "{bad:?} was accepted as {refused:?}");
        }
        // The message names the arc, so the caller can see what to fix.
        assert!(parse_oid("1.3.x.6").unwrap_err().contains("\"x\""));
        // The top of the range is still an arc.
        assert_eq!(ok("1.4294967295"), vec![1, u32::MAX]);
    }

    #[test]
    fn the_empty_oid_and_a_bare_dot_are_the_same_root_and_format_back_to_a_dot() {
        assert_eq!(ok(""), Vec::<u32>::new());
        assert_eq!(ok("."), Vec::<u32>::new());
        assert_eq!(format_oid(&[]), ".");
    }
}
