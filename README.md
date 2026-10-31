# APK:lib:SNMP

SNMP helpers, as a Rust library: **the tree's one OID grammar** (`Oid` — strict parse, format, SNMP arc order, prefix tests, BER encode/decode) and a dormant in-memory OID tree (set, get, get-next) for Python. It is not an SNMP stack — it opens no socket and encodes no PDU.

| | |
|---|---|
| **Recipe** | none — a library, not a container: it is linked into the binaries that use it |
| **Source** | `SRC/` (this repository — https://github.com/LikeDotAudio/APK.plugin.protocol.SNMP.git) — crate `apkaudio-snmp` (opt-in feature `python`; nothing enables it) |
| **Used by** | adoption pending: PLUGIN.SNMP (`schema.rs` string prefix tests) and PLUGIN.TRENDNET (`snmp.rs` `oid_from_dotted`/`encode_oid`/`read_oid`) — handoffs filed 2026-10-30. oaRustCore no longer mounts `snmp_agent`. A member of the plugin workspace; `enabled = false` on the bus until a process runs it |
| **Workspace** | `PODS/POD.THICK/DISCOVERY.Engine/SRC/Cargo.toml` (shared `target/` at the repository root) |
| **Family / Category** | LIBRARY — protocol helper; no MQTT client, no socket, no port |

Was `APK:BareMetal/PLUGINS/PLUGIN:protocol:SNMP`.


**Wiki:** [Documentation/README.md](Documentation/README.md) — principles, architecture and delivery.

## What it does

| File | What is in it |
|---|---|
| `SRC/src/oid.rs` | `Oid`: `parse`/`FromStr` (strict; `OidError` names the arc), `Display`/`dotted()` (`1.3.6.1`), `dotted_leading()` (`.1.3.6.1`), `Ord` = SNMP arc order, `starts_with`/`strip_prefix`/`is_child_of`/`parent`/`child`/`longest_prefix` on arcs, `to_ber`/`from_ber` (X.690 §8.19, incl. multi-byte first sub-identifier) |
| `SRC/src/agent.rs` | `parse_oid(text)` → `Result<Vec<u32>, String>` and `format_oid(arcs)` (always compiled); `SnmpAgent` (feature `python`) — `update_oid(oid, value)`, `get_oid(oid)`, `get_next(oid)` → `{oid, value}`, `clear()`; keys are `Vec<u32>`, so a walk is in SNMP arc order (`.9` before `.10`); a malformed arc raises `ValueError` |
| `SRC/src/self_test.rs` | the self test, module name `snmp` — landing spots, the `config.ini` section, and the OID grammar refusing a bad arc |
| `SRC/config.ini` | `[snmp]` — `enabled = false` (read by the protocol census; nothing runs this library), `topic` |

The empty `oaMIBCache_rs` module and the unread `host`/`port`/`version`/`community` keys were removed on 2026-10-30 (LOOSE-snmp-mib-cache). The SNMP that talks to real switches is `PODS/POD.THICK/PLUGINS/PLUGIN.SNMP`.

## On the bus

No MQTT client. The orchestrator (`NODE.BareMetal/SRC/Core/ORCHESTRATOR:hunter:VISA/src/mqtt.rs`) publishes `config.ini` retained at `APK.audio/System/Protocols/snmp/config`, with `/status` = `online` and, on a broker the bench owns, `/host`.

## Build and test

```sh
cd PODS/POD.THICK/DISCOVERY.Engine/SRC
cargo test  -p apkaudio-snmp                    # 13 tests: 5 Oid, 4 agent grammar, 4 self test; + 1 doc test
cargo build -p apkaudio-snmp --features python  # the PyO3 half
```

Checked 2026-10-30: 13 passed + 1 doc test; `cargo check --features python` clean.

## Philosophy

- **The key type carries the ordering.** OIDs are stored as `Vec<u32>`, not text, because SNMP orders arc by arc and a string sort would put `.10` before `.9`; a test says so (`SRC/src/agent.rs`, `the_key_type_gives_snmp_lexicographic_order_and_its_text_would_not`).
- **One OID vocabulary.** Prefix tests and ordering are on arcs (`Oid::starts_with`, `Ord`), never on text, where `1.3.6.1.2.1.1` would "prefix" `1.3.6.1.2.1.10` (`SRC/src/oid.rs`, `prefixes_are_on_arcs_not_text`).
- **Refuse, don't merge.** `parse_oid` refuses a malformed, negative, empty or over-32-bit arc instead of dropping it, so two different OIDs can never become one key (`SRC/src/agent.rs`, `a_malformed_arc_refuses_the_oid_rather_than_merging_two_keys`; LOOSE-snmp-agent).
- **Write the reader first.** No `host` / `port` / `version` / `community`, and no topic quad, because nothing reads them (`SRC/config.ini`).
- **Name the lint, don't blanket it.** One `allow`, only under `python`, with the reason (`SRC/src/lib.rs`).
- **The self test is a plain function** under the `apkaudio_contracts::self_test` contract, reading `config.ini` through `include_str!` (`SRC/src/self_test.rs`).

---

*© 2026 — Written & Maintained by Anthony P. Kuzub. MIT Licence.*

<!-- wiki:smoke -->
**Smoke test** — ✅ PASS 1/1 since 2026-10-30 12:51 EDT — [results](Documentation/%F0%9F%A7%A9Manual%20and%20Artifacts/SMOKE.md)
<!-- /wiki:smoke -->
