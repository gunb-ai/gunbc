# Audit — PB-Zero: canonical read surfaces for v2-side row-authority consumption (STOP matrix)

**Status:** planning / audit only (no implementation claim).  
**Unlocks from:** merged consumer-gap audit [`docs/audit/pb-zero-v2-method-template-row-authority-consumer-gap.md`](pb-zero-v2-method-template-row-authority-consumer-gap.md) (#1242 on main).

**Non-goals:** no compiler or substrate code; no fixtures or tests; no `v3.std.*` import bridge; no snapshot schema design; no consumer implementation; no statement that PB-Zero has shipped the consumer path.

## Live authority vs missing infrastructure (unchanged fact)

| Layer | State |
|-------|--------|
| **Row authorities** | **Live** at `src/v3/std/{rust,python,go}_method_template_contracts.dag` (plus carrier / registry authorities cited in the gap audit). |
| **v2 structural consumer** for those rows | **Not built** — same conclusion as the gap audit and the in-file `.dag` headers. |

This document compares **candidate read surfaces** only; it does **not** select a canonical surface for implementation.

## Routing: substrate vs PB territory

Substrate-owned **snapshot format**, **carrier shape**, or **first-hook** placement for facts that must not fork between Grounding, emit retirement, and PB bootstrap work is **not** something PB documentation may invent here. Route those decisions through **[`INVARIANTS.md`](../../INVARIANTS.md)** (repository root — **P1–P5** are the top-level `##` sections there; per-rule IDs such as `INVARIANTS.md#c-8` and long-form split-outs under [`docs/invariants/`](../invariants/) are **separate** navigation surfaces).

Use these **verbatim markdown heading lines** from `INVARIANTS.md` as `rg` anchors (approximate line numbers on `main` at authoring time):

- `## P1: Modeling Faithfulness` (~L23)
- `### Problem shape: Unnamed substrate target` (~L86)
- `### Procedure: substrate-fact introduction (decision procedure for new modeling)` (~L94)
- `## P2: Boundary Discipline` (~L144)
- `### Problem shape: Parallel authority` (~L152)

[`docs/modeling-discipline.md`](../modeling-discipline.md) maps modeling practices to those **P1**/**P2** headings for checklist-style review. **R2 Substrate Manager** / Grounding briefs are the owning intake; see **[`docs/briefs/r2-substrate-manager.md`](../briefs/r2-substrate-manager.md)**.

PB-Zero / **(γ)** program context remains [`docs/design-pure-bootstrap-zero.md`](../design-pure-bootstrap-zero.md) (bootstrap-as-data; build-step break of chicken-and-egg).

## Verified on main: v2 “extension” reality (narrow)

The v2 tree has **no** dedicated bootstrap-Dag consumer that lifts `MethodTemplateContract` rows into emit. What exists today is only the ordinary **compile pipeline over discovered `.dag` sources**:

- `collect_dag_sources` recursively walks a configured directory and accumulates `SourceFile` inputs for `compile_sources` — see `src/v2/tests/src/pipeline.rs` (`collect_dag_sources`, and call sites such as `full_dsl_compiles` / workspace-wide discovery). That is **file inclusion + parse/compile**, not a semantic bridge from v3 std **row tables** into v2 emit’s legacy `dsl/extdeps/languages/*/emit.dag` template maps.
- Self-compile / stage tests pass **`--source-root`** for `src/v2` and `dsl` — see `src/v2/tests/src/bootstrap.rs` (`run_self_compile`). Expanding roots in a **future** implementation would be a product and boundary decision, not evidenced here as a row consumer.

So the “existing v2 extension point” candidate is **mechanical source discovery + compile**, not a ready-made row-authority reader.

## STOP matrix — candidate canonical read surfaces

| Candidate surface | Authority source (facts live) | Required owner (decision / contract) | Viable? | Implementation prerequisites (high level) | STOP / escalation |
|-------------------|-------------------------------|----------------------------------------|---------|-----------------------------------------------|-------------------|
| **Committed generated snapshot** (e.g. extend today’s v3 regen / include pattern so v2 **reads bytes** from a generated artifact, not `use v3.std.*`) | Still the `.dag` rows + lowering pipeline; generated file is a **projection** | **Substrate Manager (+ Director)** names schema and single-authority rule so Grounding and emit retirement do not fork; PB may **consume** only after contract exists | **Plausible** if schema is substrate-ratcheted | Regen ownership, stability/versioning, P1-named carrier or table for “published” template facts | **STOP** PB-only authoring of snapshot schema; escalate to Substrate if “canonical” is undefined |
| **Bootstrap-Dag / staged load order (γ)** | Future `bootstrap.dag` (or equivalent) + load invariants per design doc | **PB-Bootstrap-Process** (program) + **Director** gate on evaluator-trampoline readiness; still needs **same** non-fork agreement with row `.dag` authority | **Plausible in-program** | Minimal interpreter or build-step evaluator per [`docs/design-pure-bootstrap-zero.md`](../design-pure-bootstrap-zero.md); agreement with v3 compiler load order | **STOP** if bootstrap slice would duplicate row text in v2/dsl as a second authority (violates gap audit + **INVARIANTS.md `## P2: Boundary Discipline`** parallel-authority discipline) |
| **PB-Bootstrap-Process–owned consumer hook** (Rust hook that runs inside PB-owned build/evaluator boundary, **not** v2 `use v3.std`) | Row `.dag` + bootstrap data as above | **PB program** implements hook **only** under named substrate contract; hook does not redefine row semantics | **Plausible as consumer code location**, not as authority origin | Depends on rows + snapshot or bootstrap data being **already** canonical elsewhere | **STOP** if hook is used to bypass Substrate-owned snapshot decision — route to Substrate row first |
| **Direct `v3.std.*` import in v2 crate graph** | Would re-home authority in Rust deps | N/A | **Non-option** | — | **STOP immediately** — violates architectural boundary in gap audit |
| **v2 pipeline source-root / `collect_dag_sources` expansion** | Files on disk under added roots; **not** by itself a row semantic | **Director + Release** (v2 retirement coordination) with Substrate agreement on what parse inclusion means for authority | **Insufficient alone** — can include more `.dag` files but does **not** connect emit to `MethodTemplateContract` rows without additional consumer work | Parser/module graph must not smuggle `v3.std` imports; cross-tree hazards noted in `rust_method_template_contracts.dag` header | **STOP** treating “add path” as equivalent to “canonical read surface” |
| **Cross-binary extract (v2 build invokes v3 tool)** | v3 compiler output artifact | **Director** splits ownership: contract for artifact is Substrate/PB boundary | **Speculative** — no verified first-class extension on main today | Process + CI + typed artifact contract; same single-authority discipline | **STOP** if artifact becomes informal stdout scraping without named carrier |

## STOP-condition report (dispatch acceptance)

### Do existing docs already pick one canonical surface?

**No.** The merged gap audit explicitly lists “canonical read surface” as an **open** question among regen include, `bootstrap.dag` slice, or other artifact, and requires Substrate/PB agreement before implementation ([`pb-zero-v2-method-template-row-authority-consumer-gap.md`](pb-zero-v2-method-template-row-authority-consumer-gap.md) § “Scope clarification needed before implementation dispatch”). [`docs/design-pure-bootstrap-zero.md`](../design-pure-bootstrap-zero.md) commits to **(γ)** but does not select which concrete artifact v2-side retirement will read first.

### What exact decision remains gated?

1. **Which artifact is canonical** for template-row facts consumed outside the v3 std module graph (generated snapshot vs bootstrap-declared load vs future hybrid), and **versioning / ratchet** so consumers cannot drift.  
2. **Who authors the typed contract** for that artifact (Substrate-first per P1 vs PB-bootstrap-owned slice with substrate-named targets).  
3. **Whether v2 emit retirement** attaches to **build-step** consumption only, **test oracle** consumption only, or both — still coordinated **post-R3** per [`docs/r2-structure.md`](../r2-structure.md); consumer infra may precede full retirement.

### Who owns the next decision?

| Decision | Primary owner |
|----------|----------------|
| Named substrate target / new carrier or published snapshot row | **R2 Substrate Manager** + INVARIANTS § P1 procedure |
| Row list + Grounding test corpus alignment | **R2 Grounding** / LanguageSpec lanes (see [`docs/briefs/t-ground-tests.md`](../briefs/t-ground-tests.md)) |
| Bootstrap-process / (γ) evaluator and build-step story | **Pure Bootstrap** program / [`docs/briefs/r2-pure-bootstrap-manager.md`](../briefs/r2-pure-bootstrap-manager.md) |
| v2 retirement sequencing | **R2 Release Manager** (post-R3 operational framing) |

## Cross-refs (link-only)

- Prior audit (row files × gap matrix): [`docs/audit/pb-zero-v2-method-template-row-authority-consumer-gap.md`](pb-zero-v2-method-template-row-authority-consumer-gap.md)
- PB program / (γ): [`docs/design-pure-bootstrap-zero.md`](../design-pure-bootstrap-zero.md); [`docs/briefs/r2-pure-bootstrap-manager.md`](../briefs/r2-pure-bootstrap-manager.md)
- Substrate ownership / P1 routing: [`INVARIANTS.md`](../../INVARIANTS.md) (`## P1` / `## P2` headings); [`docs/briefs/r2-substrate-manager.md`](../briefs/r2-substrate-manager.md)
- Modeling practices ↔ P-mapping checklist: [`docs/modeling-discipline.md`](../modeling-discipline.md)
- Structure / v2 retirement timing: [`docs/r2-structure.md`](../r2-structure.md)
