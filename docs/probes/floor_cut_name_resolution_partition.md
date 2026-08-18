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
| `floor_cut_name_resolution_partition.py` | **offline authority** for how `partition.tsv` was derived from `census.tsv` |

No workflow, shell runner, or Rust census binary ships in this PR — the measurement was
executed once on a self-hosted runner (GHA run `32080685910`) and the output is committed
as data, same pattern as `docs/plans/measurements/`.

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

- **Call-contract / bare_name_binding (172):** migration list if qualification wins; test
  population if binding-wall wins.
- **Reach_gap (167):** later wave — not the same mechanism as call-contract binding.

Operator decision with crisp-crab-430 (#8282). No corpus edits in this PR.
