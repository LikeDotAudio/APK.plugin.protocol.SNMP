# 🔍 Architecture audit — LIB.SNMP (2026-10-30)

Audited against the APK.audio multi-repo architecture standard: coupling, boundaries, duplication, contracts.
← [Audits](INDEX.md) · Improvements: [🛣Roadmap/IMPROVEMENTS.md](../%F0%9F%9B%A3Roadmap/IMPROVEMENTS.md)

## Bounded context
An OID grammar (`parse_oid`, `format_oid`) and an in-memory OID tree exposed to Python behind the `python` feature. Not an SNMP stack: no socket, no PDU. Rust library `apkaudio-snmp`, 3 files, 378 lines. Everything was read.

## Context map
| Talks to | How (topic / endpoint / file / import) | Relationship |
|---|---|---|
| DISCOVERY.Engine workspace | member through the link `DISCOVERY.Engine/SRC/members/LIB.SNMP` (standalone since 2026-10-30) | Shared build (workspace host) |
| APK.plugin.SDK `apkaudio-contracts` | Cargo git dependency, tag `sdk-v0.5` (`SRC/Cargo.toml`); patched to the local contracts inside the engine workspace | Conformist (self-test report shape), versioned |
| Orchestrator / protocol census | `SRC/config.ini` published retained at `APK.audio/System/Protocols/snmp/config`; census reads `enabled` | Published Language (config only) |
| PLUGIN.SNMP | none — it handles OIDs itself as strings (`PLUGINS/PLUGIN.SNMP/SRC/src/schema.rs:324` `is_child_of`, `parse.rs:271` `oid_tail`) | Separate Ways (overlapping concept) |
| Consumers | none — README "Used by: no crate depends on it" | — |

## Scorecard
| Category | Score (0–3) | Finding |
|---|---|---|
| Coupling & Cohesion | 3 | builds from its own repository; contracts by SDK tag; engine membership through a `members/` link |
| Boundary Integrity | 3 | pyo3 behind a feature; the grammar is plain Rust with typed errors |
| Duplication Control | 2 | `Oid` (parse, format, arc order, prefix, BER) is now the shared grammar; PLUGIN.SNMP and PLUGIN.TRENDNET still carry their own until they adopt it (handoffs filed) |
| Testing & Contracts | 3 | 9 grammar tests (incl. the BER textbook vector) + 4 self tests + 1 doc test, one command |
| Change management | 3 | Released and tagged: `snmp-v0.2.0` (version 0.2.0); consumers pin the tag |
Score key: 3 meets the standard · 2 minor gaps · 1 significant gaps · 0 absent / violated.

## Findings
2. **Good: the grammar refuses bad input.** `SRC/src/oid.rs` `Oid::parse` returns `Err` naming a malformed arc instead of dropping it, and orders by arcs so `.9` sorts before `.10`.
3. **Good: dead code removed honestly.** The empty `oaMIBCache_rs` pymodule and the unread `host`/`port`/`version`/`community` keys are gone (`SRC/src/lib.rs` header, `SRC/config.ini`).
4. **Two OID vocabularies until adoption.** PLUGIN.SNMP keeps OIDs as dotted strings and compares them textually (`PLUGIN.SNMP/SRC/src/schema.rs:324`); PLUGIN.TRENDNET has its own text and BER codec. This crate's side (`Oid`) is delivered; adoption is handed off.
