# Floor-cut name-resolution failure partition (Wave 1)

**Status:** read-only census for operator decision (qualification vs binding-wall).  
**Branch:** `session/valiant-hawk-198` (off `main` @ 611fd027708+).  
**Does not:** qualify references, rename declarations, widen reach, or edit `floor_expected_red`.

## Hypothesis (one root cause, three error shapes)

Bare names in the widened reference closure bind by insertion-order precedence in the
per-claim scope registry. When two unrelated modules declare the same leaf name, the
later module in scope order wins silently; the failure surfaces one step later as:

| failure class | role |
| --- | --- |
| `call_contract_mismatch` | wrong function selected with matching bare name |
| `no_such_function` | may be binding or reach gap |
| `undefined_variable` | may be binding or reach gap |

Population counts are filled from the main CI floor run (or census execution on that
tree); do not treat stale brief numbers as current.

## Method

1. `required_floor_failure_census` — prepares the subject once, evaluates **only**
   enrolled expected-red witnesses (`GUNBC_REQUIRED_FLOOR_FAILURE_CENSUS_ONLY=1`).
2. For each held expected-red failure, records witness identity, error class/message,
   bare reference name (when parseable), selected declarer module (call-contract rows),
   and all in-scope declarer modules for that bare name.
3. `docs/probes/floor_cut_name_resolution_partition.py` — adds `intended_declaration_identity`
   from import analysis and `reach_vs_binding` classification.

Artifacts:

- `docs/probes/floor_cut_name_resolution_census.tsv` — raw census
- `docs/probes/floor_cut_name_resolution_partition.tsv` — identity-grain partition
- `docs/probes/floor_cut_name_resolution_census.log` — run receipt

Run: `docs/probes/run_floor_cut_name_resolution_census.sh`

## Partition columns

| column | meaning |
| --- | --- |
| `reference_site` | witness module + bare name under analysis |
| `intended_declaration_identity` | import-derived intended `module.name` (empty if ambiguous) |
| `actually_selected_declaration_identity` | scope registry winner at eval time |
| `visible_candidate_set` | all in-scope modules declaring the bare name |
| `failure_class` | `call_contract_mismatch` \| `no_such_function` \| `undefined_variable` \| `other` |
| `reach_vs_binding` | `bare_name_binding` \| `reach_gap` \| `reach_or_binding_unresolved` \| `other` |

## Counts (filled by execution)

```
PARTITION_CLASS call_contract_mismatch=…
PARTITION_CLASS no_such_function=…
PARTITION_CLASS undefined_variable=…
PARTITION_CLASS other=…
```

## Disposition notes

- **Call-contract rows** carry their own evidence: wrong function with the right bare name.
  These are the migration list if qualification wins; the test population if the binding
  wall wins.
- **no_such_function / undefined_variable** are **not** automatically the same mechanism.
  Rows with empty `visible_candidate_set` are reach gaps (later wave). Rows with
  candidates but no selected binding are unresolved without deeper site analysis.

## Next step

Operator decision with crisp-crab-430 (#8282): qualify references vs construction wall.
This session stops after delivering the partition; no corpus edits until that decision.
