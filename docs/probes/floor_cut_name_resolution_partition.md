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

## Counts (main @ 611fd027708, census run 32080685910, 2026-08-18)

Main floor run `32076934126`: **820** enrolled, **795** held, **25** now-passing (build failure by design).

Census (`required_floor_failure_census` on `4137cba`): **808** held rows written (12 additional
roster rows passed during expected-red-only eval).

```
PARTITION_CLASS call_contract_mismatch=172
PARTITION_CLASS no_such_function=151
PARTITION_CLASS undefined_variable=16
PARTITION_CLASS other=469
```

**Name-resolution hypothesis subset** (three classes above): **339** rows.

| reach_vs_binding | count (name-resolution subset) |
| --- | --- |
| `bare_name_binding` | 172 (all call-contract rows; 163 with >1 visible candidate) |
| `reach_gap` | 167 (all 151 `no_such_function` + 16 `undefined_variable`) |
| `reach_or_binding_unresolved` | 0 |

The **172 call-contract** rows all carry a selected declarer module and are classified
`bare_name_binding` — wrong function with matching bare name.

The **151 no_such_function** and **16 undefined_variable** rows are **not** automatically the
same mechanism; inspect `visible_candidate_set` and `reach_vs_binding` per row in the partition TSV.

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
