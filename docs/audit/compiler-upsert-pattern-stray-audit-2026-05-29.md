# Compiler UPSERT&lt;T&gt; pattern stray audit — 2026-05-29

**Operator directive (work item `adhoc-342cfea2-653`):** the compiler must treat every
operational "do this" as **upsert this**. The fractal canon lives in
`dsl/std/patterns.dag` (UPSERT&lt;T&gt; header): verify-first → satisfy-dependencies-recursively →
create-if-missing → cache-on-content.

**Scope:** `src/v3/compiler/**` mutation and cache paths; cross-reference `dsl/std/patterns.dag`
workflow patterns.

**Ledger standing:** point-in-time snapshot per `CLAUDE.md` ledger exception (same class as
`docs/audit/v4-deferral-audit-2026-05-29.md`). Inline code comments and `patterns.dag` canon
remain authoritative; dissolve this file after remediations land or are dismissed in review.

---

## Canon (single sentence)

Never write without reading; never create without verifying absence; never memoize without a
content key; every dependency step is itself an upsert.

---

## Audit matrix

| Site | Pattern step | Verdict | Notes |
| --- | --- | --- | --- |
| `dsl/std/patterns.dag` UPSERT&lt;T&gt; header | canon | **landed** | Four-step fractal + naming rule (`apply_*` without verify = stray). |
| `ensure_optional_match_disj` (`infer.rs`) | verify → create → cache map | **conformant** | `existing_optional_match_disj_decl` before materialize; `set_optional_match_disj` indexes. |
| `ensure_kernel_bool_lane1e2b_bootstrap_witness` (`bootstrap.rs`) | verify → repair-if-missing | **conformant** | Reads `inhabits`; wires only when absent. |
| `populate_primitive_cache` (`dag.rs`) | rebuild cache from authority | **conformant** | Idempotent re-lookup; bootstrap diagnostics already fail-closed. |
| `populate_target_clean_emission_bindings` | clear + rebuild from declarations | **conformant** | Duplicate-language guard; authority is declaration table. |
| `regen_bootstrap --verify` (`bin/regen_bootstrap.rs`) | verify-first; write only on regen | **conformant** | Content-equality gate before write; write path is explicit operator regen. |
| `cached_compile_outcome` (`tests/integration/common/cached_compile.rs`) | cache-on-content | **conformant** | `(source, file)` key + `OnceLock`; outcome variant preserved. |
| `BOOTSTRAPPED_*` `LazyLock` (`dag.rs`) | cache-on-content | **conformant** | Generated snapshot ≡ fresh compile ratchet (`regen_bootstrap`). |
| `try_register_lane2_workflow_effect` (`dag.rs`) | verify-first upsert | **remediated** | Was blind overwrite; now no-op on equal workflow, `false` on conflict. |
| `apply_authored_lane2_loop_witness` (`r3_fc_lane2_loop_witness.rs`) | harness side channel | **accepted debt** | Pattern A author-now; dissolution paired with lowering-owned `lane2_workflow`. Uses upsert register API. |
| `attach_diagnostic` | append-only errors | **intentional non-upsert** | Diagnostics are a log, not idempotent state; multiple errors are desired. |

---

## Stray remediated in this dispatch

**`Dag::try_register_lane2_workflow_effect`** previously assigned `lane2_workflow = Some(...)`
without checking an existing carrier — a second call with a different `WorkflowEffect` silently
clobbered the first (pure "do"). Fixed via `upsert_lane2_workflow_on_node`: absent → insert;
equal → no-op success; different → `false` (fail-closed).

Unit test: `try_register_lane2_workflow_effect_upserts_without_silent_overwrite` in
`dag.rs` tests.

---

## Follow-on (not blocking this audit)

1. **Rename harness API:** consider `upsert_lane2_workflow_effect` alias when lowering owns the
   field and deletes the witness scanner.
2. **DSL pattern generics:** uncomment `pattern upsert<Check, Create, Resolve>` in
   `patterns.dag` when parser supports pattern type parameters (ROADMAP "Desired Parser Features").
3. **`content_upsert` stub:** replace `fn content_upsert` stub with real `ensure` wiring when
   filesystem resource binding lands.

---

## Reproduction

```bash
# Canon header
rg -n 'UPSERT<T> canon' dsl/std/patterns.dag

# Register upsert unit test
cargo test -p v3-compiler try_register_lane2_workflow_effect_upserts -- --nocapture
```
