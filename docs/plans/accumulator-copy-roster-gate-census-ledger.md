# Ledger: accumulator_copy roster gate (300s census)

Parent: sunny-bat-82 / snappy-stag-806 silent reds. Subject: `complexity/accumulator_copy_roster_gate_*` as one roster-scan cost. Cap not raised. Not enrolled in the required gate.

Instrument: out-of-gate 300s census of those entries (not `wc -l`; file sizes are not an oracle). Classification: **(d)** — real standing and scan-cost defects, not stale greens.

Typed standing for priced-out lens walks: `gunbc.rung_drop.accumulator_copy_priced_out_file_gate_ratchets`. Offline invocation: `gunbc.offline_local_recipe` rows whose `policy` is `accumulator_copy_roster_gate_exclusion_note`. Exclusion rationale stays `gunbc.ci.ci_layer_roots` `accumulator_copy_roster_gate_exclusion_note` (unchanged from main).

| Row | Class | Cause | Repair |
| --- | --- | --- | --- |
| TIMEOUT `accumulator_copy_roster_gate_lean_bash_test.dag` | (d) | Interpreted `file_gate` of `src/v2/extdeps/languages/lean.dag` and `src/v2/extdeps/languages/bash.dag` in one census entry. | Freeze identities kept. Census bodies: typed `Filesystem.Read` of those paths. Lens walks: `roster_lean_file_gate_walk` / `roster_bash_file_gate_walk`. Named in the drop. |
| TIMEOUT `accumulator_copy_roster_gate_swift_test.dag` | (d) | Same scan of `src/v2/extdeps/languages/swift.dag`. | Same pattern; `roster_swift_file_gate_walk`. |
| TIMEOUT `accumulator_copy_roster_gate_test.dag` | (d) | Same scan packed `src/v2/compiler/00_compile.dag` and `src/v2/lens/complexity_accumulator_copy/analyze.dag` with snippet controls. | Those walks remain ordinary `fn`s. Remaining executing `file_gate` `test fn`s: snippets plus `roster_std_algebra_zero_suspects_within_ratchet`, `roster_machine_code_zero_suspects_within_ratchet`, `roster_lens_cost_model_zero_suspects_within_ratchet`. |
| RED `red_control_planted_copy_still_alarms` | (d) | Control used `list_append(left: acc, right: x)` — linear non-copy polarity (`noncarrier_name_snippet`). Fixture `planted_copy.dag` and `quadratic_snippet` use `left: x, right: acc`. | Snippet polarity aligned with the planted fixture. |
| RED `roster_glob_discovery_zero_suspects_within_ratchet` | (d) | Path moved to `src/v2/test/workflow/glob_discovery_law.dag` (#9637). Recorded in `excluded_witness_outlives_the_subject_it_names`. Interpreted `file_gate` of that path also interrupted the required floor's 8s changed-witness wall (exclusion does not skip changed-witness). | Freeze identity kept. Census body: typed `Filesystem.Read` of the new path. Lens walk: `roster_glob_discovery_file_gate_walk` (named in the drop). |
| RED `roster_compile_stage_zero_suspects_within_ratchet` | (d) | `00_compile.dag` is not a 300s interpreted subject. | Ordinary `fn`; named in the drop. |

Freeze rows for the demoted compile/analyze walks were deleted so they cannot rot as `StaleFrozenPathDeferral`. CI exclusion substring `claim/complexity/accumulator_copy_roster_gate` is unchanged.
