# Roadmap implementation — living checklist

Companion to [`ROADMAP.md`](ROADMAP.md). **Canonical sequencing** remains in the roadmap phase table; use this file to track execution and verification.

**How to use**

- Check items as they land; add PR/issue links inline when helpful.
- After any change that affects bootstrap output or `.dag` structure, run the **fixed-point** gate.
- Phase 2 may begin once **diagnostics = 0**; L1 work can continue in parallel (see roadmap).

---

## Global verification (run frequently)

| Gate | Command |
|------|---------|
| Unit tests (full workspace) | `cargo test --workspace` |
| Clippy | `cargo clippy --all-targets -- -D warnings` |
| Diagnostics ratchet (Phase 1 exit) | `cargo test -p v2-compiler-tests v2_strict_compile_diagnostic_count -- --ignored` |
| Fixed point (after bootstrap-affecting `.dag` changes) | `cargo test -p v2-compiler-tests v2_bootstrap_fixed_point -- --ignored` |
| Gist pipeline (Phase 2 exit) | `cargo test -p v2-compiler-tests v2_gist_full_pipeline -- --ignored` |

---

## Implementation order (suggested)

1. [x] **M1 (P1.1)** — naming batch in one pass; then fixed point *(landed 2026-03-22)*.
2. [ ] **P1.2 + P1.3** — infer cleanup + `DIAG_RATCHET` → 0 (only Phase 2 blocker).
3. [ ] **Parallel:** [ ] L1 slices (P1.4–P1.8) **and** [ ] Phase 2 gist (after step 2).
4. [ ] **Phase 3** — compile bundle, artifact planning, ownership, v1 off critical path.
5. [ ] **Phase 4** — shared emit spine, `LanguageSpec`, projections, DAG backend boundary.
6. [ ] **Phase 5** — internal `Node`-centric convergence (L2 prep).

---

## Phase 1 — Naming, soundness, L1

### P1.1 Naming cleanup (M1)

- [x] Rename `04_reconcile.dag` → `04_infer.dag`
- [x] Rename `06_pipeline.dag` → `compile.dag`
- [x] Rename `07_complexity.dag` → `complexity.dag`
- [x] Rename `07_ownership.dag` → `ownership.dag`
- [x] Rename `08_artifact.dag` → `artifact.dag`
- [x] Rename `09_trace.dag` → `trace.dag`
- [x] Move `RenderTarget` out of `00_core.dag` into `artifact.dag` (avoids import cycle with `compile.dag`)
- [x] Update imports, bootstrap references, tests, docs (`README.md`, `v2_tests.rs`, `v2_crate_emit.rs`, `lib.rs`)
- [x] **Acceptance:** driver module is `v2.compiler.compile` (`compile.dag`); infer stage is `v2.compiler.infer` (`04_infer.dag`)

### P1.2 Infer cleanup (R5, S3.5, S7)

- [ ] Reduce string-keyed method handling via data tables / structural approach
- [ ] Extract emit metadata leakage from infer (S3.5)
- [ ] Remove fabrication fallbacks where scoped (S7 / roadmap backlog alignment)

### P1.3 Diagnostics ratchet → 0 (blocks Phase 2)

- [x] Enumerate return type *(done per roadmap)*
- [x] Fold accumulator threading *(done)*
- [x] Callable / function-value type *(done)*
- [x] Structured `ErrorCategory` *(done)*
- [ ] `map_insert` / `map_merge` result typing (correct `Map` leaf / structure)
- [ ] Chained field access (after map fixes)
- [ ] Tighten `node_type_equals` **last** (remove permissive `Dynamic` / structural fallbacks once inference stops fabricating)

### P1.4–P1.8 L1 type dissolution (roadmap order)

- [ ] **P1.4** Optional / cardinality — properties in `.dag`; no `n.name == "Optional"`-style reads
- [ ] **P1.5** Containers — `List` / `Map` / `Set` properties; fix bare leaf vs parameterized map inconsistency
- [ ] **P1.6** Primitives — `Int`, `String`, `Bool`, `Float`, `Unit`, `Bytes`, `Json`, `Secret` facts in `.dag`
- [ ] **P1.7** Connective dissolution — remove `connective` from `Node` (after P1.4–P1.6)
- [ ] **P1.8** Delete residual primitives — constructors, predicates, `builtin_type_kind`, etc.

**L1 acceptance (full completion):**

- [ ] `BuiltinTypeKind` deleted
- [ ] `builtin_type_kind()` deleted
- [ ] `node_is_*` predicates deleted or replaced with property reads
- [ ] `optional_node()`, `container_node()`, `pair_node()` (and related) deleted
- [ ] `connective` field removed from `Node`
- [ ] Zero type-name string matching in the compiler
- [ ] Fixed point still holds

### Phase 1 exit criteria

- [ ] `v2_strict_compile_diagnostic_count` passes (ratchet **0**)
- [x] M1 complete
- [ ] Fixed point after each structural change (as applicable)

---

## Phase 2 — Gist end-to-end

**Path:** build stage0 (`v2_bootstrap_fixed_point`) → compile `gist` with stage0 → build/run emitted crate (dry-run). v1 interpreter is not the proof path.

- [ ] **P2.1** Gist pipeline test — real verification via stage0 (`v2_gist_full_pipeline`)
- [x] **P2.2** Service operation bodies *(done per roadmap)*
- [x] **P2.3** `main.rs` workflow dispatch *(done)*
- [ ] **P2.4** Multi-module extdep imports — verified end-to-end
- [ ] **P2.5** Emitted crate build/run — verified end-to-end
- [ ] Manual smoke: emitted gist crate + dry-run (until automated e2e exists if desired)

### Phase 2 exit criteria

- [ ] `v2_gist_full_pipeline` passes
- [ ] Emitted gist builds and runs in dry-run mode
- [ ] No v1-only post-processing required for a buildable crate

---

## Phase 3 — Compile contract, pipeline completion, v1 retirement

Tracks **M2, M3, M4** and **R8, R9**.

- [ ] **P3.1** Parity audit — enumerate what v1 still compiles that v2 does not
- [ ] **P3.2** Ownership + authoritative compile bundle — consolidate unsupported obligations/reporting
- [ ] **P3.3** Artifact planning — real partitioning and per-artifact orchestration (beyond single-artifact compat)
- [ ] **P3.4** Runtime shim dissolution — remaining v1 shim → `.dag` runtime templates
- [ ] **P3.5** Archive v1 — remove v1 from default compile path

### Phase 3 exit criteria

- [ ] One authoritative typed compile result shape
- [ ] Ownership alongside complexity in pipeline output
- [ ] Artifact planning between infer and emit on primary path
- [ ] v1 not required for normal compilation

---

## Phase 4 — Shared emit, projections, backend boundaries

Tracks **M5, M6, M7** ( **M8** after Phase 4 contract solid).

- [ ] **P4.1** `LanguageSpec` single authority for language facts
- [ ] **P4.2** Shared emit fold + per-target adapters (no whole-tree `ExprData` dispatch per backend)
- [ ] **P4.3** Generated tests as first-class projection
- [ ] **P4.4** DAG backend / runtime boundary — canonical DAG artifact; execution downstream
- [ ] **P4.5** Typed backend plumbing and CLI (backend selection not stringly)
- [ ] **P4.6** Equivalence — self-compile + gist still converge

### Phase 4 exit criteria

- [ ] No backend owns a full-tree `ExprData` dispatcher
- [ ] No duplicate whole-tree TCO walkers per backend
- [ ] `LanguageSpec` is the single authority
- [ ] Generated tests are first-class artifact outputs
- [ ] DAG backend emits canonical artifact without embedding interpreter in stages

---

## Phase 5 — Convergence (L2 preparation)

- [ ] **P5.1** Token → `Node` compositions
- [ ] **P5.2** Module/import → `Node` compositions
- [ ] **P5.3** Diagnostic / compile-output dissolution where valuable
- [ ] **P5.4** Service/support type dissolution review
- [ ] **P5.5** Residual semantic enum cleanup

### Phase 5 exit criteria

- [ ] M1 filenames normalized everywhere internally
- [ ] Compiler structure consistently `Node`-centric
- [ ] Each step survives re-bootstrap and fixed point
- [ ] Ready to begin L2 work in roadmap sense

---

## Cross-cutting passes (S\*) — spot-check

| Pass | Status | Notes |
|------|--------|-------|
| S1 | Done | `kernel_types` / `is_kernel_type` |
| S2 | Done | Pipeline vs artifact/trace boundaries |
| S3 | Done | Known-method resolution centralized |
| S3.5 | Phase 1 | Emit metadata out of infer |
| S4 | Phase 1 | Rust-only ownership/render policy out of core + infer |
| S5 | Phase 4 | Fuse duplicated `ExprData` walks |
| S6 | Phase 4 | Shared emit dispatch |
| S7 | Phase 1 / 4 | Remove fabrication / string-keyed cleanup |

---

## Business feature track — Agent workflow (parallel)

- [ ] **AG1** Model cloud agent API in `.dag` (typed lifecycle)
- [ ] **AG2** One end-to-end happy path (auditable)
- [ ] **AG3** Record integration challenges for roadmap feedback

**Timing:** AG1 can start after Phase 2 proves a real emitted program (or overlap late Phase 2 if modeling allows).

---

## End goal (project-wide) — final acceptance

- [ ] L1 complete (zero type-world knowledge in compiler per roadmap)
- [ ] One shared emit walker drives targets via common spine
- [ ] Language facts in `dsl/extdeps/languages/*`; program lowering in adapters
- [ ] Ownership + complexity proofs wired in compile pipeline
- [ ] At least one real program (`gist`) compiles and runs E2E
- [ ] v1 archived
- [ ] Compiler-internal structure converges on `Node` compositions

---

## Notes / decisions log

_Use this section for dated decisions, blockers, and links to PRs._

| Date | Note |
|------|------|
| 2026-03-22 | Checklist created; execution order follows `ROADMAP.md` phase table. |
| 2026-03-22 | **M1 landed:** file renames, `v2.compiler.infer` / `infer()`, `v2.compiler.compile`, `RenderTarget` in `artifact.dag`, emitter + README + tests updated. |
