# Audit — PB-Zero: v2-side consumer path for v3 method-template row authorities

**Status:** planning / audit only (no implementation claim).  
**Dispatch context:** Director pre-author audit surfaced PB-Zero substrate target: row authorities under `src/v3/std/*_method_template_contracts.dag` are lifted in v3, but v2 cannot import `v3.std.*` and has no bootstrap-Dag consumer infrastructure to read those rows via a snapshot path; full v2 emit retirement is gated behind that consumer story.

## STOP — scope of this document

- **Not redundant with existing authorities:** [`docs/design-pure-bootstrap-zero.md`](../design-pure-bootstrap-zero.md) commits to shape **(γ)** and defines **bootstrap as data** (evaluator reads bootstrap authority, constructs `Dag`, hands control to compiler-as-data). It does **not** enumerate per-target method-template row files or a v2-side read matrix. The **live row authorities** already state in-file that v2 lacks a bootstrap-Dag consumer and defer retirement to PB-Zero scope (see citations below). This audit **centralizes** the consumer-gap matrix and explicit architectural boundary for implementation dispatch.
- **Ownership split (do not over-assign to PB alone):** **Substrate / Grounding** own the row list, carrier shape (`MethodTemplateContract` in `emit_model.dag`), registry alignment (`MethodRef` / `methods.dag`), and open Phase 1.5 / classification items called out in the `.dag` headers. **PB-Zero / (γ)** own the *bootstrap-process* story: a consumer path that reads **structural** row authority from committed bootstrap / snapshot-shaped artifacts **without** introducing a second, v2-local authority for the same facts. **R2 Release** coordinates **v2 retirement post-R3** per program structure ([`docs/r2-structure.md`](../r2-structure.md)). Before coding, Director should confirm whether the first v2-side hook lands under **PB-Bootstrap-Process** work items in the zero-floor program vs. a **substrate-owned** “published snapshot” contract consumed by a thin build boundary.
- **If this audit is sufficient:** treat implementation as **dispatch-gated** on (1) canonical artifact choice (e.g. regen output vs. explicit bootstrap slice), (2) non-duplication rule (no parallel v2 authority), (3) `v2 ∌ v3.std.*` preserved.

## Architectural boundary (load-bearing)

**v2 must not import `v3.std.*`.** Any v2-side retirement of legacy emit maps in `dsl/extdeps/languages/*/emit.dag` must consume **structural** facts through a path that does **not** treat v3 std modules as Rust dependencies of the v2 crate graph. PB-Zero’s **(γ)** “bootstrap is data” model is the natural owner *conceptually* for “read the lifted rows without a code import bridge,” but the **exact** hook (build script, committed generated include, bootstrap DAG slice, or other) is **not** specified here — only the requirements and gaps.

## Live authorities cited

| Authority | Role |
|-----------|------|
| [`docs/design-pure-bootstrap-zero.md`](../design-pure-bootstrap-zero.md) | LIVE program; shape **(γ)**; bootstrap-as-data breaks chicken-egg at **build step**; evaluator + bootstrap authority |
| [`docs/briefs/r2-pure-bootstrap-manager.md`](../briefs/r2-pure-bootstrap-manager.md) | R2 PB manager scope; cross-refs to design + structure |
| [`docs/briefs/t-ground-tests.md`](../briefs/t-ground-tests.md) | Row authorities at `src/v3/std/{rust,python,go}_method_template_contracts.dag`; Stratum A/B routing stability; `MethodTemplateContract` + `MethodRef` registry |
| [`docs/r2-structure.md`](../r2-structure.md) | v2 persists as oracle; **v2 retirement = post-R3** operational cleanup; PB manager owns post-R1 PB program work |
| `src/v3/std/rust_method_template_contracts.dag` | In-file: v2 emit still uses legacy `rust_*` chain; **v2 has no bootstrap-Dag consumer**; retirement deferred to PB-Zero / v2 retirement (#1133) |
| `src/v3/std/python_method_template_contracts.dag` | Defers scope/authority notes to Rust sibling |
| `src/v3/std/go_method_template_contracts.dag` | Same deferral to Rust sibling |

Rust header (authoritative for shared scope note):

```6:17:src/v3/std/rust_method_template_contracts.dag
// single-authority for future emission. v2's emit pipeline still
// consumes the legacy `rust_simple_method_specs` /
// `rust_method_templates()` / `rust_method_wraps_result()` chain in
// `dsl/extdeps/languages/rust/emit.dag` because v2 has no
// bootstrap-Dag consumer infrastructure to read these rows. Full
// retirement of those legacy authorities is **deferred to
// Pure-Bootstrap-Zero / v2 retirement scope** (manager dispatch
// #1133 inbox 4344956791). The row-list here is net-additive until
// then. (Phase 2 retired the dead `MethodTranslation` declarations
// in `runtime.dag` — zero v2 consumers — but emit-side authorities
// continue serving v2.)
```

## Distinction: live row authorities vs. consumer infrastructure

| Layer | State |
|-------|--------|
| **Row authorities (v3 std `.dag`)** | **Live** — populated rows per target; headers document legacy v2 emit still authoritative for several method families until retirement |
| **v2 bootstrap-Dag consumer** | **Not built** — no `Dag::new()`-style or equivalent read path in v2 pipeline that ingests these rows as structural data |
| **Import bridge `v3.std.*` → v2** | **Out of scope / forbidden** by architectural boundary above |

## Matrix — method-template row files → PB-Zero consumer gap

| Target row-authority file | v2 consumer gap | Snapshot / bootstrap-Dag read requirement | Likely PB-Zero owner surface | Gating dependencies | STOP conditions | Non-goals |
|---------------------------|-----------------|---------------------------------------------|------------------------------|---------------------|-------------------|-----------|
| `src/v3/std/rust_method_template_contracts.dag` | v2 emit reads legacy Rust templates from `dsl/extdeps/languages/rust/emit.dag`; cannot `use v3.std.*` | Need a **read-only** structural path (committed regen artifact, bootstrap DAG declaration, or equivalent) that exposes row facts to the process that retires v2 emit **without** duplicating template strings in v2 sources | PB-Bootstrap-Process / (γ) evaluator-trampoline work: “bootstrap is data” + build-step emission of consumer | Higher-order / dual-template methods still legacy-only (Phase 1 omission in same file); Phase 1.5 substrate decision | If Director places **first** hook under **Substrate-only** “published snapshot” with PB consuming second — **stop** PB-only dispatch and follow that contract | v2 crate importing `v3.std.rust_method_template_contracts`; copying rows into `dsl/` as a second authority |
| `src/v3/std/python_method_template_contracts.dag` | Same pattern: `python_method_templates` map in `dsl/.../python/emit.dag` | Same class of requirement as Rust row | Same bootstrap-process family | `string_contains` classification open; several methods row-only vs legacy map (per file header) | Same as Rust row | Same |
| `src/v3/std/go_method_template_contracts.dag` | Same pattern: `go_method_templates` map in `dsl/.../go/emit.dag` | Same class of requirement as Rust row | Same bootstrap-process family | `chars` / tokenizer `Unparsed` edge called out for Phase 2 | Same as Rust row | Same |

## Scope clarification needed before implementation dispatch

1. **Canonical read surface:** Is the first supported consumer a **committed generated** include (today’s regen pattern extended), a **bootstrap.dag** slice consumed by a minimal interpreter, or another artifact? Substrate and PB must agree so Grounding tests and emit retirement do not fork on different “sources of truth.”
2. **Who authors the hook’s contract:** If the snapshot format is substrate-owned, Grounding/Substrate briefs should name the stable schema; PB-Zero then only consumes it. If PB-Zero owns the bootstrap DAG that *loads* std rows, the contract still must not duplicate row text in v2.
3. **v2 retirement timing:** [`docs/r2-structure.md`](../r2-structure.md) places v2 retirement **post-R3**; consumer infrastructure may land earlier as **enabling** work, but full emit retirement remains program-coordinated, not a silent side effect of row lift alone.

## Non-goals (repeated from dispatch)

No v2 compiler code changes in this audit PR; no v3 substrate edits; no `v3.std.*` import bridge; no row-authority edits; no emitter retirement implementation.

## References (line anchors for cross-ref grep)

- `docs/r2-structure.md` — search `v2 retirement` / `post-R3` for explicit non-scoping of v2 removal from R2/R3 thesis gates.
- `docs/briefs/t-ground-tests.md` — lines ~13–14 (substrate single-authority paths), ~112–114 (`MethodTemplateContract` landed), ~143–145 (Stratum A/B corpus keyed to row files).
