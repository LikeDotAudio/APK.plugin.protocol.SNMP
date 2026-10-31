# 🛣 Improvements — LIB.SNMP (from the 2026-10-30 architecture audit)

What this repo should change to meet the standard, most valuable first. Source: [🔍Audits/ARCHITECTURE-AUDIT.md](../%F0%9F%94%8DAudits/ARCHITECTURE-AUDIT.md).
← [Roadmap](INDEX.md)

Delivered items are recorded in [🌍Publishing/DELIVERY.md](../%F0%9F%8C%8DPublishing/DELIVERY.md).

| # | Improvement | Category | Effort (S/M/L) | Evidence |
|---|---|---|---|---|
| 1 | Make the crate standalone: drop `workspace =`, own edition, contracts by `sdk-v0.5` tag. **Waiting on DISCOVERY.Engine (handoff filed):** must land together with the engine's `members/LIB.SNMP` link (cargo refuses a non-hierarchical member without `package.workspace`); the LIB side is a ready, verified patch | Coupling | S | `SRC/Cargo.toml:2,8` |
| 2 | Tag releases once consumed. **Needs owner:** publish a tag (e.g. `snmp-v0.2.0`) after PLUGIN.SNMP / PLUGIN.TRENDNET adopt `Oid` (handoffs filed) | Change management | S | `SRC/Cargo.toml` |
