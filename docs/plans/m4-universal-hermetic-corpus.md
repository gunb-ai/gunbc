# Plan — M4.1: universal hermetic corpus governance

**Status:** PLAN-FIRST · worker `fierce-crab-852` · DESIGN.md §3/§5/§6 authority — this doc is a dispatch tracker, not a fact ledger.

**Predecessor:** M4.0 (#5188) — `eval_service_call` reads the published mock corpus as the single authority for hermetic realizability, but only from the **entry import closure** (`item_registry`). Filesystem co-locates its corpus via `import extdeps.filesystem.mock_corpus` in `filesystem_io.dag` (M4.0 per-layer opt-in). GovernedProbe keeps corpus in the same module as the service.

**Problem:** Closure-scoped corpus resolution lets an entry **opt out** of governance by omitting the corpus import — the runtime and the M2 `mock_totality_lens` then read **different** models. §3 nicknaming trap: co-location is a transport workaround, not the authority fact.

---

## 0. Thesis — governance is universal, not opt-in

```
  dag/ whole-tree published mock corpora     ← SINGLE authority (same modules M2 lens imports)
    → precompute operation-key set ONCE per batch (whole-tree compile of dag/ roots)
      → InterpContext carries immutable published_mock_keys + governed_services (O(1))
        → eval_service_call: corpus-governed service + unpublished op → §5 fail-closed
          → fixture replay OR inline mock (corpus-free services only)
```

The entry import closure decides **which code runs**; it must NOT decide **which operations are hermetically realizable**. That fact is global over the dag/ tree.

---

## Phase 1 — PLAN-FIRST (model + witnesses before runtime)

| Artifact | Role |
| --- | --- |
| `dag/std/hermetic_replay.dag` | Add `service` + `operation` fields on `PublishedMockCase` (alongside `operation_key` until dissolve) |
| `dag/test/fixture/m4_universal_governed_probe.dag` | Service-only module (no corpus) |
| `dag/test/fixture/m4_universal_governed_corpus.dag` | Corpus-only module (NOT imported by witness entry) |
| `dag/test/claim/m4_universal_corpus_witness.dag` | §5 discriminating pair: GREEN published / RED unpublished **without** corpus in closure |
| `interp_recorded_fixture_test.rs` | Subprocess proof on real `claim_batch` path |

**RED before runtime:** witness entry imports probe service only → M4.0 closure scan misses corpus → `Forbidden` silently uses inline mock (vacuous). **GREEN after M4.1:** whole-tree precompute finds corpus → fail-closed on unpublished op.

---

## Phase 2 — Runtime (load-bearing `v1_interpreter.rs` + `cli_run.rs`)

1. **`precompute_whole_tree_published_mock_keys(source_roots)`** in `cli_run.rs`
  - Scan dag source roots (paths ending in `/dag` or named `dag`)
  - `resolved_graph_from_sources(all modules)` — same tree `dag_compile_clean` validates
  - Evaluate every `data` item declared over `PublishedMockCase` (existing shape gate)
  - Return `HashSet<String>` of operation keys
2. **`InterpContext` extensions**
  - `whole_tree_published_keys: Option<Rc<HashSet<String>>>` — when set, supersedes closure scan
  - `governed_services: RefCell<Option<Rc<HashSet<String>>>>` — service names extracted once; `eval_service_call` uses `governed_services.contains(service_name)` (O(1)) instead of scanning all keys per call
3. **`make_eval_context_with_published_keys`** — thread precomputed keys from `claim_batch` / `handle_run_with_options` (precompute once per invocation, not per witness)
4. **Key extraction** — prefer `service` + `operation` fields when present; fall back to `operation_key` during migration

---

## Phase 3 — Dissolve M4.0 opt-in

| Mark | Action |
| --- | --- |
| `feature:m4-universal-hermetic-corpus-load` | Delete `import extdeps.filesystem.mock_corpus` from `filesystem_io.dag` |
| `feature:hermetic-published-mock-operation-ref` | Delete `operation_key: String` once all rows carry `service` + `operation` |

---

## §5 proof obligations

| Witness | GREEN | RED (teeth) |
| --- | --- | --- |
| `m4_governed_service_witness` (M4.0, corpus co-located) | `Allowed` realizes | `Forbidden` fail-closed |
| `m4_universal_corpus_witness` (M4.1, corpus **outside** closure) | `Allowed` realizes | `Forbidden` fail-closed |
| `filesystem` hermetic tests | Read/Write governed without co-location import | absent op fail-closed |

---

## Open / deferred

- **M5:** RecordedFixture store fork dissolution (unchanged from M4.0)
- **Projection lens:** `feature:filesystem-mock-corpus-from-service-decl` — corpus projected from `service Filesystem` operation set (bind `node://adhoc-a9399cb3-628`)
- **v2 std resolve:** delete `src/v2/std/hermetic_replay.dag` mirror when cross-tree import lands

## Dissolution trigger (DESIGN §6)

Dissolve this tracker when M4.1 universal governance has landed and M4.0 opt-in is gone — `precompute_whole_tree_published_mock_keys` is the single authority, `m4_universal_corpus_witness` is green-by-execution on the floor, and the Phase-3 marks (`feature:m4-universal-hermetic-corpus-load`, `feature:hermetic-published-mock-operation-ref`) have fired (corpus co-location import + `operation_key` deleted) — at which point governance is universal-by-construction and this doc is redundant.
