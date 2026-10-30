# Architecture — LIB.SNMP

`apkaudio-snmp` is an in-memory OID tree (set, get, get-next) for Python, as a Rust library in the plugin workspace `PODS/POD.THICK/DISCOVERY.Engine/SRC/Cargo.toml`. It opens no socket and encodes no PDU.

## Using it

From Python — only once a caller re-mounts it in oaRustCore with the `python` feature; today it is not mounted: oaRustCore dropped it, because no Python under `PODS/` imported it (`NODE.BareMetal/SRC/Core/CORE:engine:RUST/lib.rs`, the "NOT MOUNTED" comment), and no Cargo.toml in the tree enables the `python` feature. A Python caller re-adds the dependency with `features = ["python"]` and its `add_submodule!` in the same commit as the import:

```python
import oaRustCore
agent = oaRustCore.snmp_agent.SnmpAgent()           # also oaSNMPAgent_rs
agent.update_oid(".1.3.6.1.2.1.1.5.0", "bench-switch")
agent.get_oid("1.3.6.1.2.1.1.5.0")                  # 'bench-switch' — leading/trailing dots optional
agent.get_next(".1.3.6.1.2.1.1")                    # {'oid': '.1.3.6.1.2.1.1.5.0', 'value': 'bench-switch'}
agent.get_oid("1.3.x.6")                            # ValueError: OID "1.3.x.6": arc "x" is not a decimal number
```

From Rust: `apkaudio_snmp::agent::{parse_oid, format_oid}` and `apkaudio_snmp::self_test`.

## Inside

| File | Role |
|---|---|
| `SRC/src/lib.rs` | `pub mod agent` (always), `pub mod self_test`; one pyo3 lint allowed under `python` |
| `SRC/src/agent.rs` | `parse_oid` (strict: every arc a decimal `0..=4294967295`, `""`/`"."` = root), `format_oid` (always a leading dot); `SnmpAgent` with an `Arc<Mutex<BTreeMap<Vec<u32>, String>>>`; `oasnmpagent_rs` |
| `SRC/src/self_test.rs` | `MODULE = "snmp"`; landing spots, `[snmp]` section, the OID grammar |
| `SRC/config.ini` | `[snmp]` — `enabled`, `topic` |

## Data flow

Every `SnmpAgent` method parses its OID text through `parse_oid` first; a refusal is a Python `ValueError` before the lock is taken. `config.ini` is staged by the NODE.BareMetal image build and published retained at `APK.audio/System/Protocols/snmp/config` by the orchestrator (`publish_protocol_configs`).
