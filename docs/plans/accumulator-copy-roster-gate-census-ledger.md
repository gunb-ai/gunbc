# Ledger: accumulator_copy roster gate (300s census)

Parent: sunny-bat-82 / snappy-stag-806 silent reds. Subject: `complexity/accumulator_copy_roster_gate_*` as one roster-scan cost. Cap not raised. Not enrolled in the required gate.

Classification: **(d)** — real standing and scan-cost defects, not stale greens.

| Row | Class | Cause | Repair |
| --- | --- | --- | --- |
| TIMEOUT lean_bash_test.dag | (d) | One entry `file_gate`s lean.dag (~1031 lines) and bash.dag (~2740 lines) through tokenize/parse/normalize + lens in the interpreter. Exclusion note already prices the family at ~15m serial. | Freeze identities kept. Census bodies are typed `Filesystem.Read` of those paths (red if absent). Lens `file_gate` walks are ordinary fns (`roster_*_file_gate_walk`). Declared drop `gunbc.rung_drop.accumulator_copy_priced_out_file_gate_ratchets`. |
| TIMEOUT swift_test.dag | (d) | Same scan over swift.dag (~2323 lines) alone. | Same. |
| TIMEOUT roster_gate_test.dag (whole entry) | (d) | Same scan packed 00_compile.dag (~5943 lines) and analyze.dag (~1160 lines) into the same claim entry as the snippet controls. | Those two walks are ordinary `fn`s (same drop). Executing 300s file_gate set: snippets + glob_discovery_law (`test fn`) + algebra + machine_code + complexity_accumulator_copy.dag. |
| RED red_control_planted_copy_still_alarms | (d) | Control used `list_append(left: acc, right: x)` — the linear non-copy polarity (`noncarrier_name_snippet`). Fixture planted_copy.dag and quadratic_snippet use `left: x, right: acc`. | Snippet polarity aligned with the planted fixture. |
| RED roster_glob_discovery_zero_suspects_within_ratchet | (d) | Path `src/v2/workflow/glob_discovery_law.dag` moved to `src/v2/test/workflow/` (#9637). Recorded in `excluded_witness_outlives_the_subject_it_names`. | Path repointed on a `test fn` (small file; freeze identity restored). |
| RED roster_compile_stage_zero_suspects_within_ratchet | (d) | 00_compile.dag is not a 300s interpreted subject; Bool `file_gate` here is priced-out standing, not a ratchet to raise. | Walk remains a `fn` for wet recipe; not a 300s claim; named in the drop. |

Freeze rows for the demoted compile/analyze walks were deleted so they cannot rot as `StaleFrozenPathDeferral`. CI exclusion `claim/complexity/accumulator_copy_roster_gate` is unchanged.
