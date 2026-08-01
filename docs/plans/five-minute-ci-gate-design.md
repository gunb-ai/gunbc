# Five-minute CI gate — program scoping

Scoping only. No implementation lands from this note alone. It registers the
operator-signed product boundary and six sub-lanes in `dag/gunbc/roadmap_authority.dag`
so dispatch, witnesses, and receipts share one program identity instead of scattering
the work across unrelated CI-cost rows.

**Product boundary:** an ordinary source edit reaches a required CI verdict in five
minutes without recomputing any semantic fact whose inputs did not change.

**This is a program, not a PR.** Each sub-lane is independently dispatchable; the
program node states the end-to-end contract the lanes jointly discharge. Receipts
and measurements stay on the sub-lane that owns them — the parent carries no stored
rung or wall-clock field (same discipline as the guarantee-ladder carrier).

---

## Sub-lanes (six)

| id | role |
|---|---|
| `warm-merge-admission` | Stamp and admit merges from warm receipts — fold the resolve/materialization receipt gates into merge admission so the PR path does not pay cold recomputation for facts already fixed for the run. |
| `native-selected-witness-corpus` | Run only the witnesses the affected set selects, natively on the pooled floor — selection shrinks work; native execution avoids per-row process tax. |
| `exact-tree-materialization` | First materialization rung: exact-tree cross-process cache for resolved graphs (opt-in; verified serve, not default CI activation). |
| `module-grain-materialization` | Module-grain semantic memo — editing one module recomputes only that module, semantically affected dependents, and the affected shard; everything else is a verified hit. |
| `pre-index-materialization-lookup` | Warm manifest lookup before corpus-scale index construction — the serve/dashboard fix where index build currently precedes the cache probe. |
| `entry-graph-union-construction` | **Existing row, reparented under this program.** Measure whether repeated typecheck compute exists before building shared-entry machinery; slice 1 banked as #7483. |

### Materialization ladder (operator framing)

The three materialization lanes climb one hierarchy (not three independent caches):

`exact-tree` → `module-grain` → `pre-index lookup`

`entry-graph-union-construction` measures whether module-grain sharing is worth building;
it does not implement the ladder.

### Related work outside this program

- **Placement chain** (`placement-live-roster-preflight` … `placement-atomic-authority-flip`)
  — host/job placement; feeds wall-clock but is a separate lane owner (`ci-placement`).
- **Frontend escape scan** (`frontend-escape-scan-construction`) — tokenizer cost shape;
  landed Lane A from `inner-cost-lanes-scoping.md`.
- **Compile-wall / shard cold pass** — may further compress the five-minute envelope once
  materialization and selection are live; tracked in `compile-wall-endgame.md`, not
  duplicated here.

---

## Dissolution trigger

Delete this note when every sub-lane row is accepted or honestly retired and the
program parent `five-minute-ci-gate` is accepted on fleet receipts showing a
representative leaf `.dag` PR at ≤5 minutes wall with byte-identical verdicts vs a
cold recompute control — or when the operator recuts the program and this note is
superseded by a registered plan row.

DESIGN refs: §1 (time is the value), §2 (one materialization kernel, not N hand caches),
§5 (refuse, never widen — incomplete cache artifacts must not fall through to cold
recompute), §6 (displaced cost priced in receipts, not elegance).
