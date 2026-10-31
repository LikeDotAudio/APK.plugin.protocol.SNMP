# Delivery — LIB.SNMP

## Build and test

```sh
cd PODS/POD.THICK/DISCOVERY.Engine/SRC
cargo test  -p apkaudio-snmp                    # 13 tests + 1 doc test — the one check
cargo build -p apkaudio-snmp --features python  # the PyO3 half
```

Checked 2026-10-30: 13 passed + 1 doc test; `cargo check -p apkaudio-snmp --features python` clean.

## Where it lands

| Destination | How |
|---|---|
| `node-baremetal:local` (container `Node-BareMetal`) | `NODE.BareMetal/Docker/Dockerfile` copies `LIBRARY/LIB.SNMP/SRC`; compiled as a member of the plugin workspace; oaRustCore no longer links it. `config.ini` is staged to `/app/APK:DOCKERS/APK:BareMetal/LIB.SNMP/config.ini`. |
| plugin builder image (`DISCOVERY.Engine/Docker/Dockerfile`) | compiled as a workspace member; no plugin binary uses it |
| GitHub | `https://github.com/LikeDotAudio/APK.plugin.protocol.SNMP.git`, a monorepo submodule; pushed by `.apk.scripts/deploy_all.py` |

No live URL; on the bus, `APK.audio/System/Protocols/snmp/config` (retained).

## Not automated

- No CI runs the tests (the root `.github/workflows/ci.yml` verifies nothing, by design); the check is the one command above.
- A running node picks up a change only after an operator rebuilds `Node-BareMetal`.

## Delivered from the 2026-10-30 audit

- **One OID grammar for the tree** — new `SRC/src/oid.rs`: `Oid` (strict `parse`/`FromStr` with `OidError` naming the arc; `Display`/`dotted()`, `dotted_leading()`; `Ord` = SNMP arc order; `starts_with`, `strip_prefix`, `is_child_of`, `parent`, `child`, `longest_prefix` on arcs; `to_ber`/`from_ber` per X.690 §8.19 including multi-byte first sub-identifiers, which PLUGIN.TRENDNET's `read_oid` splits wrongly for `2.x`). `agent.rs` `parse_oid`/`format_oid` now delegate to it (same behaviour, 4 tests unchanged). 5 new tests + 1 doc test incl. the textbook BER vector; `cargo test -p apkaudio-snmp` 13 + 1 pass, `--features python` checks clean. PLUGIN.SNMP (`schema.rs` `is_child_of`/`best_section`/`oid_key`) and PLUGIN.TRENDNET (`snmp.rs` `oid_from_dotted`/`oid_to_dotted`/`encode_oid`/`read_oid`) adopting it: handoffs filed.
- **No longer advertised as live** — `SRC/config.ini` `enabled = false` (with the reason, and "set back with its first running host"). The orchestrator still publishes `/status online` for every module folder regardless of `enabled`: handoff to NODE.BareMetal filed.
- **Tests run per change as one command** — no workflow here (the root CI verifies nothing on purpose); the check is `cargo test -p apkaudio-snmp` in `DISCOVERY.Engine/SRC` (`cargo test` in `SRC/` once the standalone conversion lands).
