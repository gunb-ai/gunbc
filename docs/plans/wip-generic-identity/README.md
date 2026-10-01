# WIP, not for merge: generic-identity repair attempts (clever-lynx-801, 2026-10-01)

These files are preserved at wind-down so they are not lost. They are not an artifact to land. The model and the class row
are #12913 (docs/plans/derived-node-identity-design.md, gunbc.recurring_failure_mode
generic_identity_decided_by_spelling); the carrier is #12914 (std.derived_type_origin).

- `three_edits.patch`: v1 infer edits that did NOT fix the srv3 regression. It builds the call-arg expectation from
  `substitution_basis`, and stops a list literal from passing an element expectation that is not fully
  resolved (raw-child check included). It applied on main b793732902.
- `optionA.patch`: option A, applied on main at the time. A `type_variable_identity` predicate (TypeVariable mark, direct or one Resolved
  hop) keys unify_generics, substitute_generics_apply and the self-binding guard, and deletes the name branches.
  The seed builds, but the next generation refuses its own sources (3x `expected Primitive(K), got
  Coproduct(KeySource)` in dag/std/effects.dag: generic record field types arrive unmarked).
- Guard-only variant (not saved as a patch, one line): the self-binding guard in substitute_generics_apply compares
  identity instead of `type_node_label(c) == tv_id`. It restores srv3, and newly refuses 4 rows in
  src/v2/compiler/00_compile.dag (native-lane nested fold_list, m: Unit).
- `genprobe_summary_b793732902.txt`: a ONE-OFF local probe, transcribed (do not cite as an instrument; step 2's
  first task is the `gunbc test` label). It counted events where unify/substitute bound or substituted a
  generic, with whether the node carried Node.declaration TypeParameter and its two nearest infer callers. The probe
  was an eprintln in the generated v1_compiler_infer.rs at the SUB_TV arm, the SUB_NAME bare-name branch, and
  the unify_generics map_insert. Its caller was taken from std::backtrace frames matching `v1_compiler_infer::`.
- `srv3_repro.dag.txt`: bright-cat-556's 17-line repro of the srv3 `join not found` refusal.
