# Delivery — LIB.SNMP

## Build and test

```sh
cd PODS/POD.THICK/DISCOVERY.Engine/SRC
cargo test  -p apkaudio-snmp                    # 8 tests: 4 OID grammar, 4 self test
cargo build -p apkaudio-snmp --features python  # the PyO3 half
```

Checked 2026-10-30: 8 passed with and without `--features python`.

## Where it lands

| Destination | How |
|---|---|
| `node-baremetal:local` (container `Node-BareMetal`) | `NODE.BareMetal/Docker/Dockerfile` copies `LIBRARY/LIB.SNMP/SRC`; linked with `python` into oaRustCore, a dependency of `apkaudio-orchestrator`. `config.ini` is staged to `/app/APK:DOCKERS/APK:BareMetal/LIB.SNMP/config.ini`. |
| plugin builder image (`DISCOVERY.Engine/Docker/Dockerfile`) | compiled as a workspace member; no plugin binary uses it |
| GitHub | `https://github.com/LikeDotAudio/APK.plugin.protocol.SNMP.git`, a monorepo submodule; pushed by `.apk.scripts/deploy_all.py` |

No live URL; on the bus, `APK.audio/System/Protocols/snmp/config` (retained).

## Not automated

- No CI runs the tests; the root `.github/workflows/ci.yml` only deploys the web site.
- A running node picks up a change only after an operator rebuilds `Node-BareMetal`.
