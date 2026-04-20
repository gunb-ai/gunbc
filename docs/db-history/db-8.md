# DB-8 History

Collected receipts and reconciliation notes moved out of [ROADMAP.md](../../ROADMAP.md).

### Lane 3 Stage 3c prep — DB-8 fixed-point ratchet (🟡 infrastructure landed)

- **Determinism tests:** `src/v3/compiler/tests/determinism_test.rs` — 5× byte-identical `emit` / `emit_module` per row of `tests/common/determinism_fixtures.rs` (program + module + four-fixture disk matrix). Rust is full-matrix; Go skips `recursive_function_call_six` (no `Loop` in go emit yet); Python skips rows in `PYTHON_EMIT_EXCLUDE` (operator / Loop gaps) until spec/emitter parity matches the Rust matrix. `emit_rs_hash_iteration_debt_is_visible_to_audit` asserts `emit.rs` still documents HashMap/HashSet iteration debt (DB-8 §1–2) until Lane 1e replaces it. **`m1_3_emit_rust_test.rs`** imports the same `PROGRAM_FIXTURES` table from `common/determinism_fixtures.rs` (single authority; line count drops are the moved const, not removed coverage).
- **CI binary:** `cargo run -p v3-compiler --bin self_host_fixed_point` — proves pipeline snapshot fixed-point on `default_fixed_point_source`, probes `dsl/gunbc/compiler.dag`, writes `target/self_host/receipt.json`. Full emit→rustc→run→diff cycle stays **staged** until `compiler.dag` parses under v3 and emitted output is a CLI that can re-emit (see phase-plan §6 answers).
- **Invariant:** `INVARIANTS.md` §Deterministic emission (D-1); `emit.rs` module docs cite D-1 (`feedback_substrate_principle_audit` Q5 — single authority for determinism invariants).
- **Substrate-readiness (phase-plan §6 checklist):** tracked in [`docs/phase-plan-2026-04-18.md`](docs/phase-plan-2026-04-18.md) §6 — rows still 🟡/❌ remain upstream deferrals (1e, 1c Python tail, 2b workflow consumer, 2c runner, 2d, 3b) with dissolution triggers in ROADMAP; no new orphan debt from this audit.
