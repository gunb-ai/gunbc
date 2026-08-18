# Floor-cut name-resolution failure partition (Wave 1)

**Status:** read-only measurement for operator decision (qualification vs binding-wall).  
**Tree:** `main` @ 611fd027708+ (census executed 2026-08-18).  
**Does not:** qualify references, rename declarations, widen reach, or edit `floor_expected_red`.

## Finding (two causes, not one)

The initial hypothesis was one root cause (bare-name insertion-order binding) surfacing as
three error shapes. The partition refutes a single mechanism:

| reach_vs_binding | count | failure classes |
| --- | --- | --- |
| `bare_name_binding` | **172** | all `call_contract_mismatch` |
| `reach_gap` | **167** | all 151 `no_such_function` + 16 `undefined_variable` |
| `reach_or_binding_unresolved` | 0 | — |

**172** rows are wrong-function-with-matching-bare-name (163 with >1 visible candidate).  
**167** rows have empty `visible_candidate_set` — reach gaps, not binding ambiguity.

## Artifacts (committed output, not CI)

| file | role |
| --- | --- |
| `floor_cut_name_resolution_census.tsv` | raw identity-grain census (808 held rows) |
| `floor_cut_name_resolution_partition.tsv` | import-analysis + `reach_vs_binding` enrichment |
| `floor_cut_name_resolution_partition.py` | derivation from `census.tsv` → `partition.tsv` (see below) |

No workflow, shell runner, or Rust census binary ships in this PR — the measurement was
executed once on a self-hosted runner (GHA run `32080685910`) and the output is committed
as data, same pattern as `docs/plans/measurements/`.

## Derivation authority (not a reconstruction)

Two links with different epistemic status:

| artifact | status | recourse if numbers look wrong |
| --- | --- | --- |
| `census.tsv` | **receipt** — one-shot observation | run a **new** census (transport deleted; cannot re-run) |
| `partition.tsv` | **reproducible** — derived from census | re-run `partition.py` on `census.tsv` |

1. **`census.tsv`** — produced by the one-shot `required_floor_failure_census` run on GHA
   `32080685910`. The Rust transport that produced it was scaffolding and is deleted from
   this tree; **`census.tsv` cannot be regenerated from source here.** Trust the committed
   file and run id, or commission a new census — not a re-run of this one.
   Each row records witness module, bare name, selected decl module, candidate decl modules,
   failure class, and error message as observed at eval time.

2. **`partition.tsv`** — produced by running the committed script against that census file:

   ```bash
   python3 docs/probes/floor_cut_name_resolution_partition.py \
     docs/probes/floor_cut_name_resolution_census.tsv
   ```

   The script adds `intended_declaration_identity` (import scan of witness module under
   `dag/` + `src/v2/`) and `reach_vs_binding` (rule table in the script). Re-running at
   HEAD reproduces `partition.tsv` byte-for-byte; it is the authority for those two columns,
   not a cleaned-up retelling.

## Execution receipt

- Main floor run `32076934126`: 820 enrolled expected-red, 795 held, 25 now-passing.
- Census run `32080685910`: 808 held rows written to `census.tsv`.
- Head at measurement: `4137cba` on `session/valiant-hawk-198`.

```
PARTITION_CLASS call_contract_mismatch=172
PARTITION_CLASS no_such_function=151
PARTITION_CLASS undefined_variable=16
PARTITION_CLASS other=469
```

Name-resolution subset (three classes above): **339** rows. Remaining **469** `other` rows are
unrelated failure classes (budget, host-tool, etc.).

## Partition columns

| column | meaning |
| --- | --- |
| `reference_site` | witness module + bare name under analysis |
| `intended_declaration_identity` | import-derived intended `module.name` (empty if ambiguous) |
| `actually_selected_declaration_identity` | scope registry winner at eval time |
| `visible_candidate_set` | all in-scope modules declaring the bare name |
| `failure_class` | `call_contract_mismatch` \| `no_such_function` \| `undefined_variable` \| `other` |
| `reach_vs_binding` | `bare_name_binding` \| `reach_gap` \| `reach_or_binding_unresolved` \| `other` |

## Disposition

- **Reach_gap (167):** later wave — not the same mechanism as call-contract binding.

### Operator decision (2026-08-18) — bare_name_binding (172)

**Do not qualify.** The 172 `call_contract_mismatch` / `bare_name_binding` rows are not a
migration list for this lane. Qualifying every bare name at the call site would be 172 edits
in the vocabulary of the mechanism the namespace cut replaces (insertion-order binding) — work
that tidies X internally and dies with X (DESIGN replacement-migration attractor). The
alternative is to wait for the namespace cut to land a correct binding authority and let the
class dissolve; that is the chosen path. Qualification is not a safe mechanical transform: a
related lane qualified 39 references, typechecked, and produced 11 new failures — qualified
names mis-bind when the scrutinee's type is never checked, the same failure mode as bare-name
insertion-order binding with a more careful-looking diff. **These rows stay enrolled as
expected-red** — known, named, counted debt; the roster is designed to red the moment they
start passing. **Dissolution trigger:** namespace cut lands binding authority (not a calendar
date). #8282 remains the governing integration branch; it is not ready to merge. This partition
corrected a one-root-cause claim before a wave was dispatched at the wrong denominator; the
decision it enabled is to not do 172 qualification edits.

No corpus edits in this PR.
