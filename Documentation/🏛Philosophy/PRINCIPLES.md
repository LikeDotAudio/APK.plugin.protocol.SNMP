# Principles — LIB.SNMP

The principles `apkaudio-snmp` keeps, each pointed at the code that keeps it. Ecosystem rules: the monorepo's [Bare Metal golden rules](../../../../../../Documentation/%F0%9F%8F%9BPhilosophy/Bare%20Metal/goldenrule.md) and [Refusal over fabrication](../../../../../../Documentation/%F0%9F%8F%9BPhilosophy/Doctrine/readme.Refusal%20over%20fabrication.md); cited, not copied.

## Kept

1. **Not an SNMP stack, and says so.** An in-memory OID tree for Python: no socket, no PDU, no MIB. — `SRC/src/agent.rs` header (lines 3–4), `SRC/src/lib.rs` doc.
2. **The key type carries the ordering.** OIDs are `Vec<u32>`, not text, because SNMP orders arc by arc and a string sort puts `.10` before `.9`; `get_next` walks a `BTreeMap` range on that key. — `SnmpAgent.tree` and `get_next` (`agent.rs:57`); test `the_key_type_gives_snmp_lexicographic_order_and_its_text_would_not` (`agent.rs:135`).
3. **Refuse, don't merge** (refusal over fabrication). A non-decimal, signed, empty or over-32-bit arc refuses the whole OID with a message naming the arc; the Python methods raise `ValueError`. Two different OIDs can no longer become one key. — `parse_oid` (`agent.rs:80`), `parsed` (`agent.rs:32`); test `a_malformed_arc_refuses_the_oid_rather_than_merging_two_keys` (`agent.rs:153`); self-test check `an_oid_with_a_bad_arc_is_refused_not_merged` (`oid_grammar`, `SRC/src/self_test.rs:75`). Fixed 2026-10-30, LOOSE-snmp-agent.
4. **Only what needs the interpreter is gated.** `parse_oid` / `format_oid` compile in the default build so `cargo test` checks them; only `SnmpAgent` and `#[pymodule]` need `python`. — `agent.rs:14–36` and `:103`.
5. **Write the reader first** (golden rule 1). `config.ini` keeps `enabled` (the census reads it) and `topic` (the orchestrator publishes under it); the `host`/`port`/`version`/`community` nothing read, and the empty `oaMIBCache_rs` nothing imported, are gone. — `SRC/config.ini`, `SRC/src/lib.rs` (LOOSE-snmp-mib-cache).
6. **Bake in, don't look up** (golden rule 6). `include_str!("../config.ini")` — `SRC/src/self_test.rs:41`.

## Where the code falls short

- **A tree nobody fills.** The Python surface is not mounted: oaRustCore dropped it, because no Python under `PODS/` imported it (`NODE.BareMetal/SRC/Core/CORE:engine:RUST/lib.rs`, the "NOT MOUNTED" comment), and no Cargo.toml in the tree enables the `python` feature. A Python caller re-adds the dependency with `features = ["python"]` and its `add_submodule!` in the same commit as the import. The SNMP that talks to switches is `PODS/POD.THICK/PLUGINS/PLUGIN.SNMP`, which does not use this crate.
