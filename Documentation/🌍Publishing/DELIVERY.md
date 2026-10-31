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

No live URL; on the bus, `APK.audio/System/Protocols/lib_snmp/config` (retained, `/status` = `disabled`).

## Not automated

- No CI runs the tests (the root `.github/workflows/ci.yml` verifies nothing, by design); the check is the one command above.
- A running node picks up a change only after an operator rebuilds `Node-BareMetal`.

## Delivered 2026-10-31: portable file names (no `:` `|` or case twins)

- The monorepo now checks out on Windows and macOS: no tracked name holds `<>:"|?*`, a trailing dot or space, a device name or a case-only twin. Colon folders became dot folders (the POD.x convention).
- References to the renamed folders updated (`Baremetal.Manager`, `ORCHESTRATOR.hunter.VISA`, `CORE.engine.RUST`). Docs/comments only, no behaviour change.

## Delivered from the 2026-10-30 audit

- **One OID grammar for the tree** — new `SRC/src/oid.rs`: `Oid` (strict `parse`/`FromStr` with `OidError` naming the arc; `Display`/`dotted()`, `dotted_leading()`; `Ord` = SNMP arc order; `starts_with`, `strip_prefix`, `is_child_of`, `parent`, `child`, `longest_prefix` on arcs; `to_ber`/`from_ber` per X.690 §8.19 including multi-byte first sub-identifiers, which PLUGIN.TRENDNET's `read_oid` splits wrongly for `2.x`). `agent.rs` `parse_oid`/`format_oid` now delegate to it (same behaviour, 4 tests unchanged). 5 new tests + 1 doc test incl. the textbook BER vector; `cargo test -p apkaudio-snmp` 13 + 1 pass, `--features python` checks clean. PLUGIN.SNMP (`schema.rs` `is_child_of`/`best_section`/`oid_key`) and PLUGIN.TRENDNET (`snmp.rs` `oid_from_dotted`/`oid_to_dotted`/`encode_oid`/`read_oid`) adopting it: handoffs filed.
- **No longer advertised as live** — `SRC/config.ini` `enabled = false` (with the reason, and "set back with its first running host"). The orchestrator still publishes `/status online` for every module folder regardless of `enabled`: handoff to NODE.BareMetal filed.
- **Tests run per change as one command** — no workflow here (the root CI verifies nothing on purpose); the check is `cargo test -p apkaudio-snmp` in `DISCOVERY.Engine/SRC` (or `cargo test` in `SRC/`, standalone).
- **Builds from its own repository** — `SRC/Cargo.toml` names no workspace, sets `edition = "2024"` and takes `apkaudio-contracts` from APK.plugin.SDK by tag `sdk-v0.5` (the `.apk.scripts/plugin_standalone.py` shape); `.gitignore` keeps the standalone `Cargo.lock` out. The DISCOVERY.Engine workspace reaches it through the link `SRC/members/LIB.SNMP` (landed with this change). Verified: a copy of `SRC/` alone runs `cargo test --offline` green against `sdk-v0.5` (13 tests + 1 doc test); the check is now `cargo test` in `SRC/`.
- **Released `0.2.0`, tag `snmp-v0.2.0`** — `SRC/Cargo.toml` bumped 0.1.0 → 0.2.0 (the `Oid` grammar), annotated tag `snmp-v0.2.0` published (owner-authorised 2026-10-31) so PLUGIN.SNMP and PLUGIN.TRENDNET can take `apkaudio-snmp` by tag; engine lock refresh handed to the coordinator. Verified: `git archive HEAD` of `SRC/` builds and tests alone against `sdk-v0.6`.

Since the audit:

- **SDK tag `sdk-v0.7`** (88b72ec) — `SRC/Cargo.toml` takes `apkaudio-contracts` by `tag = "sdk-v0.7"`. Verified 2026-10-31: `git archive HEAD`, extracted alone, `cd SRC && cargo test` 13 passed + 1 doc test.
- **Bus module `lib_snmp`, not `snmp`** (436a7cd) — `config.ini` `[lib_snmp]` with `topic = APK.audio/System/Protocols/lib_snmp`, `self_test.rs` `MODULE = "lib_snmp"`: the old `[snmp]` put this disabled library's retained status on the live PLUGIN.SNMP's root and the engine census failed (`MODULE 'snmp'` twice). The orchestrator finds the section by its `topic` since NODE.BareMetal 5229192. Verified: `cargo test` 13 + 1 passed.
