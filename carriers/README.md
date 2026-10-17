# Throwaway measurement carriers for #14152 (scratch branch only — never lands on a PR branch)

Why these exist, and why they are here rather than on the PR branch (msg_d7ee8f8a ruling: the census
instrument must NOT land in #14152; cite it by commit):

- `my_04_infer.dag` — the fold as it stands, including the two fixes that are stuck in the merge index
  and cannot be committed mid-merge: `binding_has_type_variable_leaf` passing children RAW, and
  `generic_alias_unfold` reading the populated `TypeBinding.resolved`.
- `peer_arm_04_types.dag` — a UNION SCAFFOLD: quiet-eagle-395's `04_types.dag` at 835cbd38e1e plus the
  four definitions that exist only on my branch (`peel_alias_rhs_once`, `type_head_carrier`,
  `type_head_is_declared_carrier_alias`, `type_heads_agree`). Needed only so ONE measurement can have
  both the peer's relation and my fold's dependencies. Built by `/tmp/build_carrier.py`, whose slicing
  rule and three checks are documented there. Throwaway: never committed to a PR branch.
- `my_04_types_backup.dag` — my merged `04_types.dag`, byte-identical to HEAD for this file, so the
  scaffold can be put back after a run.
- `converged3/` — the stage0 mirrors at the fixed point for `my_04_infer.dag` against origin/main's
  mirrors, verified to carry both fixes (raw-children recursion present; no `resolved_type` inside
  `binding_has_type_variable_leaf`). This is the BASELINE SEED. It must be main's-mirrors-plus-my-fold,
  NOT main's mirrors alone: `cross_coproduct_generic_locate_test.rs` calls `unify_generics` with the
  `TextJudgment` argument my branch adds, so against main's mirrors the test crate does not even
  compile (E0061 + E0432).
- The census itself lives at `tools/rebind_census.sh` in this branch (verified byte-identical to the
  copy the dispatches used). Dispatches fetch it from here; it is NOT copied into carriers/.

The runner workspace `/root/workspace/repo-root` is SHARED and carries other sessions' uncommitted
edits between dispatches, so nothing measured there is trustworthy (sharp-boar-42, via stern-bat-401:
"main does NOT drift"; my earlier `FAIL generated surface drift: v1_compiler_infer.rs` was that
dirtiness, not main). Every dispatch that reports a number clones fresh from origin inside the dispatch
and prints the SHA it cloned.
