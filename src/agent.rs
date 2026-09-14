// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
// oaComSNMP/Methods/oaSNMPAgent_rs/mod.rs
// Author: Anthony Peter Kuzub (via Gemini)
// Version: 20260413.1400.1
//
// Description: Native SNMP agent responder. Manages OID trees 
// and MIB responses in Rust to ensure consistent performance 
// regardless of Python GIL load.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

#[pyclass]
struct SnmpAgent {
    tree: Arc<Mutex<BTreeMap<Vec<u32>, String>>>,
}

#[pymethods]
impl SnmpAgent {
    #[new]
    fn new() -> Self {
        SnmpAgent {
            tree: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    fn update_oid(&self, oid: String, value: String) {
        let mut tree = self.tree.lock().unwrap();
        let parsed_oid = parse_oid(&oid);
        tree.insert(parsed_oid, value);
    }

    fn get_oid(&self, oid: String) -> Option<String> {
        let tree = self.tree.lock().unwrap();
        let parsed_oid = parse_oid(&oid);
        tree.get(&parsed_oid).cloned()
    }

    fn get_next(&self, py: Python<'_>, oid: String) -> Option<Py<PyAny>> {
        let tree = self.tree.lock().unwrap();
        let parsed_oid = parse_oid(&oid);
        
        // Find the first OID strictly greater than the target
        for (o, v) in tree.range(parsed_oid.clone()..) {
            if o > &parsed_oid {
                let dict = PyDict::new_bound(py);
                let _ = dict.set_item("oid", format_oid(o));
                let _ = dict.set_item("value", v);
                return Some(dict.into_any().unbind());
            }
        }
        None
    }

    fn clear(&self) {
        let mut tree = self.tree.lock().unwrap();
        tree.clear();
    }
}

// Inline comment: Logic for parse_oid
fn parse_oid(oid: &str) -> Vec<u32> {
    oid.trim_matches('.')
        .split('.')
        .filter_map(|s| s.parse::<u32>().ok())
        .collect()
}

// Inline comment: Logic for format_oid
fn format_oid(oid: &[u32]) -> String {
    let parts: Vec<String> = oid.iter().map(|n| n.to_string()).collect();
    format!(".{}", parts.join("."))
}

#[pymodule]
// Inline comment: Logic for oasnmpagent_rs
pub fn oasnmpagent_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SnmpAgent>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    //! The first tests this crate has ever had. `SnmpAgent`'s own methods need
    //! a live interpreter to call, so what is pinned here is the pair of pure
    //! functions every one of them goes through -- the OID text that becomes a
    //! cache key -- plus the ordering property `get_next` depends on and
    //! nothing states.

    use super::*;

    #[test]
    fn an_oid_round_trips_and_a_missing_leading_dot_is_put_back() {
        assert_eq!(parse_oid(".1.3.6.1.2.1"), vec![1, 3, 6, 1, 2, 1]);
        assert_eq!(parse_oid("1.3.6.1.2.1"), vec![1, 3, 6, 1, 2, 1]);
        assert_eq!(format_oid(&[1, 3, 6, 1, 2, 1]), ".1.3.6.1.2.1");

        // Both spellings land on one key, so `update_oid(".1.3.6")` and
        // `get_oid("1.3.6")` are the same node. That is the point of the pair.
        assert_eq!(parse_oid("1.3.6"), parse_oid(".1.3.6."));
    }

    #[test]
    fn the_key_type_gives_snmp_lexicographic_order_and_its_text_would_not() {
        // `get_next` walks a `BTreeMap<Vec<u32>, _>` and returns the first key
        // strictly greater than the target. SNMP orders OIDs arc by arc, so
        // .1.3.6.1.2.1.10 comes AFTER .1.3.6.1.2.1.9 -- which is exactly what a
        // `Vec<u32>` compares as, and exactly what the text does not.
        let ninth = parse_oid(".1.3.6.1.2.1.9");
        let tenth = parse_oid(".1.3.6.1.2.1.10");

        assert!(ninth < tenth, "arc order: 9 before 10");
        assert!(
            format_oid(&tenth) < format_oid(&ninth),
            "and the text of those same two OIDs sorts the other way, which is \
             why the key is not a String"
        );

        // A prefix is always less than what extends it, so a walk from a subtree
        // root reaches the subtree.
        assert!(parse_oid(".1.3.6.1") < parse_oid(".1.3.6.1.0"));
    }

    #[test]
    fn a_malformed_arc_is_dropped_and_two_different_oids_become_one_key() {
        // `filter_map(str::parse::<u32>().ok())` discards anything that is not
        // a number instead of refusing the OID. The arc does not survive, and
        // no caller is told.
        assert_eq!(parse_oid("1.3.x.6"), vec![1, 3, 6]);
        assert_eq!(parse_oid("1.3.6"), vec![1, 3, 6]);
        assert_eq!(
            parse_oid("1.3.x.6"),
            parse_oid("1.3.6"),
            "two OIDs that are not the same OID reach the cache as one key -- a \
             write to either is a write to the other"
        );

        // An arc above the 32-bit range SNMP allows goes the same way: silently.
        assert_eq!(parse_oid("1.3.4294967296.6"), vec![1, 3, 6]);
        // As does a negative one, and an empty arc from a doubled separator.
        assert_eq!(parse_oid("1.-3.6"), vec![1, 6]);
        assert_eq!(parse_oid("1..6"), vec![1, 6]);
    }

    #[test]
    fn the_empty_oid_and_a_bare_dot_are_the_same_root_and_format_back_to_a_dot() {
        assert_eq!(parse_oid(""), Vec::<u32>::new());
        assert_eq!(parse_oid("."), Vec::<u32>::new());
        assert_eq!(format_oid(&[]), ".");
    }
}
