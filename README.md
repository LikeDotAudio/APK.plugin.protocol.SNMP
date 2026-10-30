# APK:lib:SNMP

SNMP helpers, as a Rust library: an in-memory OID tree (set, get, get-next) for Python. It is not an SNMP stack — it opens no socket and encodes no PDU.

| | |
|---|---|
| **Recipe** | none — a library, not a container: it is linked into the binaries that use it |
| **Source** | `SRC/` (this repository — https://github.com/LikeDotAudio/APK.plugin.protocol.SNMP.git) — crate `apkaudio-snmp` (feature `python` for oaRustCore) |
| **Used by** | oaRustCore (`PODS/POD.THICK/NODE.BareMetal/SRC/Core/CORE:engine:RUST/lib.rs`), which mounts `snmp_agent` (`oaSNMPAgent_rs`) and `snmp_mib_cache` (`oaMIBCache_rs`); a member of the plugin workspace |
| **Workspace** | `PODS/POD.THICK/DISCOVERY.Engine/SRC/Cargo.toml` (shared `target/` at the repository root) |
| **Family / Category** | LIBRARY — protocol helper; no MQTT client, no socket, no port |

Was `APK:BareMetal/PLUGINS/PLUGIN:protocol:SNMP`.

## What it does

| File | What is in it |
|---|---|
| `SRC/src/agent.rs` (feature `python`) | `SnmpAgent` — `update_oid(oid, value)`, `get_oid(oid)`, `get_next(oid)` → `{oid, value}`, `clear()`; keys are `Vec<u32>`, so a walk is in SNMP arc order (`.9` before `.10`) |
| `SRC/src/mib_cache.rs` (feature `python`) | `oaMIBCache_rs` — an empty Python module; it registers nothing |
| `SRC/src/self_test.rs` | the self test, module name `snmp` — the two checks every crate answers (landing spots, `config.ini` section) |
| `SRC/config.ini` | `[snmp]` — `enabled`, `host`, `port` (161), `version` (2c), `community`, `topic` |

A default build (no `python` feature) compiles only the self test. `enabled`, `host`, `port`, `version` and `community` are read by nothing in the tree; they ride along in the published config.

## On the bus

No MQTT client. The orchestrator (`NODE.BareMetal/SRC/Core/ORCHESTRATOR:hunter:VISA/src/mqtt.rs`) publishes `config.ini` retained at `APK.audio/System/Protocols/snmp/config`, with `/status` = `online` and, on a broker the bench owns, `/host`.

## Build and test

```sh
cd PODS/POD.THICK/DISCOVERY.Engine/SRC
cargo test -p apkaudio-snmp                    # 4 self-test tests
cargo test -p apkaudio-snmp --features python  # + the 4 OID tests in agent.rs = 8
```

Checked 2026-10-30: 4 passed by default, 8 with `--features python`. The OID tests only run with the feature, because `agent` is gated whole.

## Philosophy

- **The key type carries the ordering.** OIDs are stored as `Vec<u32>`, not text, because SNMP orders arc by arc and a string sort would put `.10` before `.9`; a test says so (`SRC/src/agent.rs`, `the_key_type_gives_snmp_lexicographic_order_and_its_text_would_not`).
- **Pin what is wrong rather than hide it.** `parse_oid` silently drops a malformed, negative, empty or over-32-bit arc, so two different OIDs can become one key; the test writes it down as current behaviour, not as a decision (`SRC/src/agent.rs`, `a_malformed_arc_is_dropped_and_two_different_oids_become_one_key`). This is where the crate falls short of *refusal over fabrication*.
- **Write the reader first.** No `topic_listen` / `topic_ignore` / `topic_publish` keys, because this crate subscribes to nothing (`SRC/config.ini`, closing comment).
- **Name the lint, don't blanket it.** Two crate-wide `allow`s, each with the reason it cannot be fixed here (`SRC/src/lib.rs`).
- **The self test is a plain function** under the `apkaudio_contracts::self_test` contract, reading `config.ini` through `include_str!` (`SRC/src/self_test.rs`).

---

*© 2026 — Written & Maintained by Anthony P. Kuzub. MIT Licence.*

<!-- wiki:smoke -->
**Smoke test** — ✅ PASS 1/1 since 2026-10-30 12:50 EDT — [results](Documentation/%F0%9F%A7%A9Manual%20and%20Artifacts/SMOKE.md)
<!-- /wiki:smoke -->
