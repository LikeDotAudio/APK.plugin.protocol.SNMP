// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! `apkaudio-snmp` — SNMP helpers.
//!
//! **An in-memory OID tree for Python, around a pure OID grammar.** `agent`'s
//! `parse_oid` / `format_oid` compile in the default feature set, so `cargo test`
//! checks them and `self_test` asks them questions; only the `SnmpAgent` pyclass
//! and its `#[pymodule]` sit behind the `python` feature (off by default).
//!
//! `mib_cache` (`oaMIBCache_rs`) was removed under LOOSE-snmp-mib-cache: it was a
//! `#[pymodule]` that registered nothing, mounted by oaRustCore as
//! `snmp_mib_cache` and imported by no Python in the tree.
//!
//! ONE lint is allowed crate-wide and it is not this crate's code:
//! `unsafe_op_in_unsafe_fn` fires inside pyo3's `#[pymethods]` expansion.
#![cfg_attr(feature = "python", allow(unsafe_op_in_unsafe_fn))]

pub mod agent;

/// This crate's own self test — what it can prove about itself with nothing
/// plugged in. Run by `cargo test`; nothing answers `outgoing/selftest_run`
/// for it on the bus since PLAN-3316.01 (see `self_test.rs`). The report shape
/// is `apkaudio_contracts::self_test`'s. PLAN-459.01.
pub mod self_test;
