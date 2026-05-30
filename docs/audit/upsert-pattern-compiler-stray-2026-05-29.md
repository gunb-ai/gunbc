# UPSERT\<T\> pattern canon + compiler stray audit — 2026-05-29

**Operator directive (2026-05-29):** the compiler and tooling must treat
**"do this" = "upsert this"** — never blind create, never blind overwrite.
The operational shape is **fractal** at every scale:

1. **verify-first** — check whether the desired state already holds
2. **satisfy-dependencies-recursively** — prerequisite facts are themselves upserts
3. **create-if-missing** — materialize only when the check says action is needed
4. **cache-outcome** — return the stable handle (decl id, path, memo) whether the
   check hit or create ran

**Canon landed in this PR:** module header in `dsl/std/patterns.dag` (UPSERT\<T\>
section) is the authoritative vocabulary for workflow patterns; this audit is a
one-shot inventory, not a maintained ledger.

**Dissolution trigger:** delete this file once §A actions are executed or
explicitly dismissed in PR review. Inline marks on strays (e.g. `content_upsert`
stub) remain authoritative until dissolved.

---

## §0. Canon (specializations)

| Pattern | Phases | Role |
| --- | --- | --- |
| `ensure<Check, Action>` | 1–3 | Conditional act; returns `{ acted: Bool }` |
| `upsert<Check, Create, Resolve>` | 1–4 | Full fractal; returns `{ value: R }` |
| `content_upsert` | 1–4 (filesystem) | `file_content_matches` + `fs.write` |
| `UpsertEffect` (`std/effects.dag`) | effect witness | Lattice meet on `Map<K,V>` at runtime |

Commented pattern bodies in `dsl/std/patterns.dag` (lines ~127–156) are the
**target wiring**; blocked on pattern-declaration generics (`ROADMAP.md` desired
parser features).

---

## §1. Compiler — conforming (upsert-shaped)

These Rust paths already implement verify-first → create-if-missing → cache:

| Location | Verify-first | Create-if-missing | Cache-outcome |
| --- | --- | --- | --- |
| `infer::ensure_optional_match_disj` | `existing_optional_match_disj_decl` | allocates Some/None/Disj | `set_optional_match_disj` |
| `infer::find_equivalent_*` family | structural equality scan | caller allocates only on `None` | returns existing `DeclarationId` |
| `dag::cardinality_idempotent_target` + `builder::alloc_cardinality_decl` | idempotent target lookup | `push_declaration` only if absent | returns existing id |
| `regen_bootstrap --verify` | `assert_disk_matches` | (write path omitted in verify mode) | N/A — verify-only branch |
| `regen_bootstrap` write path | implicit via operator workflow | writes when not `--verify` | committed snapshot |
| `lower.rs` symbol table | `entry().or_insert` | preserves existing binding | map slot |
| Integration test harnesses | `OnceLock::get_or_init` | first caller populates | cached `Arc`/DAG |

**Assessment:** core **DAG declaration materialization** and **bootstrap snapshot
regen** follow upsert discipline. Naming uses `ensure_*` / `find_equivalent_*`
instead of `upsert_*`; that is vocabulary drift, not behavioral stray.

---

## §2. Compiler — stray or at-risk

| ID | Location | Violation | Severity | Dissolve-on |
| --- | --- | --- | --- | --- |
| **S1** | `dsl/std/patterns.dag` `fn content_upsert` | Stub uses `content == ""` as fake "matches"; no filesystem verify | **High** — callers believe idempotency | Pattern generics + `uses fs`; replace with commented `pattern content_upsert` |
| **S2** | `dsl/shared/dag_util.dag` header | Promised "render-then-upsert helper" never authored | **Medium** — doc drift | `render_document` → `content_upsert` wrapper once S1 lands |
| **S3** | `dsl/tools/bootstrap.dag`, `readme.dag` | Import `content_upsert` expecting real write semantics | **Medium** — depends on S1 | Same as S1 |
| **S4** | `dsl/std/patterns.dag` | `ensure` / `upsert` / `transaction` patterns commented out | **Low** — known parser gap | Parser: generic pattern declarations |
| **S5** | `dsl/tools/codegen.dag` | Uses `when` stamp pattern instead of `content_upsert` | **Info** — intentional until resources bind; comment already states choice | Revisit when S1 lands |
| **S6** | `regen_tokenize.rs` `ensure_classes` | Verify-only gate (no create branch in same closure) | **None** — sub-step of verify-first, not a full upsert | N/A |
| **S7** | `bootstrap::ensure_kernel_bool_lane1e2b_bootstrap_witness` | Verify + repair-or-diagnose (fail-closed), not create-if-missing | **None** — witness repair is upsert-shaped repair path | N/A |

**Operator concern ("compiler strayed") — verdict:** the **Rust compiler pipeline**
for declaration graphs and bootstrap regen is **largely aligned**. Stray is
concentrated in the **DSL workflow layer**: stub `content_upsert`, missing
`dag_util` helper, and tools that import the stub as if authoritative.

---

## §3. Effect / workflow layer (aligned)

- `UpsertEffect` in `dsl/std/effects.dag` and `src/v3/std/effects.dag` models
  keyed convergent writes; DB-18/DB-20 tests exercise classification and
  parallelism fail-closed rules.
- `extdeps/tools.dag` `resolve` is upsert-shaped: `which` check → `Resolved` |
  `NotFound` with install hints.
- `extdeps/cron.dag` `Upsert` operation and gunbc review tools use cron tab upsert
  semantics (idempotent converge).

No additional compiler-Rust stray found for effect enumeration beyond existing
tracked lens/reflection boundaries.

---

## §4. Recommended actions (§A)

| Priority | Action | Owner lane |
| --- | --- | --- |
| A1 | Land pattern-declaration generics; uncomment `ensure` / `upsert` / `content_upsert` in `patterns.dag` | Parser / class-5 |
| A2 | Delete `fn content_upsert` stub; wire real `pattern content_upsert` | DSL tools + patterns |
| A3 | Add `render_then_upsert(document: Document, path: String)` in `dag_util` composing `render_document` + `content_upsert` | DSL shared |
| A4 | Audit `tools.bootstrap` / `tools.readme` after A2 for real `written` semantics | DSL tools |
| A5 | (Optional) Rename internal Rust `ensure_*` dedup helpers to `upsert_*` in a dedicated hygiene PR | v3 compiler — **not** required for correctness |

---

## §5. Test plan for this PR

- Docs + `.dag` comment canon only; no Rust behavior change.
- `cargo test -p v3-compiler m1_3_lens_unused_parameters` — unchanged;
  `content_upsert` synthetic-equivalent test remains the behavioral pin until A2.

---

## §6. Cross-references

- `docs/meta-process-design.md` — process modeling upsert semantics (CI/bootstrap)
- `docs/modeling/other-strong-models.md` — `std/patterns.dag` as strong model
- `docs/substrate-reflection-design.md` — `content_upsert` parser-gap blocker
- `INVARIANTS.md` P1 worked example — `UpsertEffect` lattice meet grounding
- `docs/audit/v4-upsert-stray-catalog-2026-05-30.md` — scoped
  `src/v4/compiler|std|lens` STRAY-FROM-UPSERT catalog with
  `v4-deferral-audit` cross-references
