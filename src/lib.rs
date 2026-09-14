// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! `apkaudio-snmp` — SNMP helpers.
//!
//! **This crate is a PyO3 extension shim.** Its real implementation lives in the
//! sibling module(s) below and is gated behind the `python` feature, which is NOT
//! enabled by default. A default `cargo build` therefore produces an empty library
//! — that is expected, not missing code. Build with `--features python` to compile
//! the implementation.
//!
//! (The `cargo new` template `add()` that used to sit here has been removed: a
//! function asserting 2 + 2 as the crate's public entry point misrepresented the
//! crate as unimplemented.)
//! TWO lints are allowed crate-wide, and neither is fixable here:
//! `unsafe_op_in_unsafe_fn` fires inside pyo3's `#[pymethods]` expansion, and
//! `non_snake_case` covers `oaMIBCache_rs`, whose spelling is the key
//! `Core/src/lib.rs` registers into `sys.modules` for Python callers.
//! The four-lint blanket this replaced also hid `dead_code`, `unused_imports`
//! and an unused parameter.
#![allow(unsafe_op_in_unsafe_fn, non_snake_case)]

#[cfg(feature = "python")]
pub mod agent;

#[cfg(feature = "python")]
pub mod mib_cache;

/// This crate's own self test — what it can prove about itself with nothing
/// plugged in. Reached from the bus through `outgoing/selftest_run`; see
/// `apkaudio_contracts::self_test` for the two inputs and the withdrawal the
/// first of them performs. PLAN-459.01.
pub mod self_test;
