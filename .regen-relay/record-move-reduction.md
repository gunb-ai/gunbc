# Record move/projection reduction

Exact committed main seed ed853cf430e, isolated b-main-tgt libraries, 22 GiB address-space bound. b_move_probe.rs compiles three authored source variants and prints their ownership proofs; b-move-rustc.log compiles the emitted Rust (unmodified module content) against the same seed dependencies. Baseline passes, early-return plus move-first refuses E0382, same early-return plus projection-first passes. The field-order permutation is a diagnostic control, not a production workaround.

rendered_use_site_type already carries the identical record literal and field order on main. Its main mirror uses resolved.clone() in the nested Resolved node. B adds the early return but does not author this field sequence.

ownership.walk_expr classifies a direct field access as Projected. whole_value_borrow_count excludes Projected. More critically, max_usage_by_fan_out selects one entire branch usage list: the higher-fan-out branch of projections discards the other branch's Read. The new early return supplies Consumed; the retained projection-only branch leaves whole_value_borrow_count zero, so build_movable_set marks the parameter movable. emit_var_ref applies that function-wide membership at the nested constructor field. emit_typed_record_lit renders fields in authored order and the following projection borrows the moved Rc.

Proposed bounded realization location: emit_typed_record_lit restricts the movable context for field emission where later field expressions reference the same binding, including nested constructors, preserving authored evaluation order and using emit_var_ref's existing clone route. This needs an existing expression-reference walk and no duplicate ownership vocabulary. However, the measured branch-summary loss affects other expression forms too: an emitter-only cut must declare this residual, or the gatekeeper can choose to correct ownership branch-use preservation at the root. No production fix made pending that scope ruling.


## Root ruling and model boundary

Gatekeeper msg_99679899 admits ownership-root repair in B and rejects emitter restriction. The requested borrow-first-still-moves control cannot follow from merely retaining whole-value Read at the join: in both executed permutations the constructor's item occurrence is Read, while only the early return is Consumed. whole_value_borrow_count therefore blocks both if the Read survives. ownership.EdgeKind explicitly documents Read as non-tail residue rather than borrow evidence and forbids using it as a borrow optimization without the use-context split (prior work #8689). The existing v2.test.manual.ownership_movable.witness_whole_read_then_consume_is_not_movable confirms that conservative contract.

A model decision was requested before production edits: extend the root with the minimal consumption-context and per-path order evidence needed to preserve the literal borrow-first positive, or retain conservative cloning and use the existing Projected-before-Consumed positive. Neither changes emitter evaluation order. Do not silently reinterpret Read or drop its evidence to make the positive pass.


## Admitted join repair

Decision msg_8206034f selects the conservative join-only correction. max_usage_by_fan_out retains every missing Read from the other binding usage after selecting the fan-out path; existing Read edges are matched by kind, site and span_start to avoid duplicating shared-prefix evidence. Projected classification and emitter rules remain unchanged. Authored-source native witness branch_read_prevents_record_move_before_projection is RED on the exact main seed (attached log). The two constructor orders now owe clones, not moves. Direct claim controls cover the smaller Read branch, Projected-before-Consumed without Read, and whole-Read-before-Consumed conservatism. Parsing passes; scoped claim execution is pending.

The safety defect was loudly mitigated by rustc E0382. Read retention is now constructed at the ownership branch join; full emitted and fixed-point evidence remains owed. Conservative Rc clones are a bounded cost residue, not a correctness gap. Next-rung capability: per-path last-use evidence at the ownership fold. Report the actual count of affected mirrors and clone-gaining declarations from the next fixed-point transaction; no count is claimed before that instrument runs.
