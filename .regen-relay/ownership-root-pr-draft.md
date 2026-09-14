Title: Preserve whole-value reads across ownership branch joins

The ownership fold could discard a binding's whole-value Read when another branch had more projections. A preceding return then made build_movable_set classify that binding as movable, so emitted Rust moved an Rc into a constructor before reading a later field and failed with E0382.

max_usage_by_fan_out now retains Read evidence from either binding usage while retaining the existing fan-out selection and Projected distinction. Shared-prefix reads are deduplicated. The existing emitter clone decision consumes the corrected ownership result; no emitter rule changes.

The exact-main reduction compiles the baseline, rejects the early-return/move-first variant with E0382, and compiles the reordered diagnostic control. Both constructor orders conservatively clone after repair: Read is non-tail residue in the existing model, not proof of a borrow. This is a bounded Rc-clone cost residue, not a correctness gap. Its next-rung trigger is per-path last-use evidence at the ownership fold.

The native rebuild exposed why this root repair must precede the Optional representation migration: the old seed emits generation one using its old ownership fold. Combining the repair with the newly triggering Optional source produces an unbuildable first generation. This PR changes only ownership.max_usage_by_fan_out, its three .dag controls, and the transaction-adjudicated mirrors.

Validation so far: exact-main authored-source reduction is RED; scoped claim_batch on the patched .dag passes branch_read_survives_larger_projection_path, projected_before_consumed_stays_movable, and whole_read_then_consume_stays_non_movable, each within the 500 ms ceiling. Parsing and formatting pass.

Pending before publication: regenB0.SJqrpc fixed-point receipt, regenerated native evidence, actual mirror and clone-gaining declaration census. Those instrument results must replace this paragraph; no native or fixed-point success is claimed yet. The bug was loudly mitigated by rustc's borrow check; retention of branch Read evidence is now constructed at the ownership fold. This does not claim a complete per-path ownership model.
