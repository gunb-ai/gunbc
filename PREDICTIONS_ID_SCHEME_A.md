# Pre-registered predictions — option (A): mint ids that are valid identifiers AND equal to the node's authored name
# Written BEFORE any run, and BEFORE any build (stern-bat-401 has not yet ruled). If the ruling is not (A), this file is never acted on.

## The property being relied on (read, not inferred)
`render_node_type` (src/v1/05_emit.dag:852) renders an unbound `TypeVariable` as a Rust generic
argument ONLY when `tn == var_id`, where `tn = authored_name_at(node)` and `var_id = node.id`;
otherwise it emits `compile_error!("UNRESOLVED_TypeVariable…")`. The fold-accumulator bridge
(src/v1/05_emit_rust.dag:14098) emits `HashMap::<compile_error!("UNRESOLVED_CompilerError"), …>::new()`
whenever the accumulator's type string is empty/`_`/unit-like. Measured: span-keyed ids (`K@file:735-744`)
trip it at `v2_compiler_symbol_index_fill.rs:273:181`; name-keyed ids (`K`) trip a different bridge at
`v2_std_inhabitance.rs:420:174`; main is green at both. So the emitter requires variable identity to BE
the authored name — a property no document states.

## Proposed scheme
For a call-site-minted parameter `g` at span (file, start, end), mint id
`g + "_" + <sanitized-file-stem> + "_" + start + "_" + end` (alphanumeric + underscore only, starts
with a letter, unique per call site), and set the node's authored name to the SAME string, so
`tn == var_id` holds. Non-generics remain byte-identical (empty overlay).

## Predictions (each falsifiable; ANY failure reverts the scheme rather than widening the emitter)
A1  `minted_variables_are_fresh_per_call_site` PASS rc=0, and its row-printed `MINT-INVENTORY` still has
    `positions=24 distinct=4 site_tags=2 bare=0` — site tags are still present (so still call-site
    unique) but now identifier-shaped, i.e. no `@`, `:` or `-` in any id. If `distinct` drops, the
    scheme has re-collapsed two sites and is wrong.
A2  `arguments_present_do_not_trigger_the_call_site_mint` PASS with `minted=0` and `S` present — unchanged.
A3  Regenerated **name-keyed** mutant (`id: g` only): freshness row FAIL rc=101 with `distinct=2 site_tags=1
    bare=2`, pin PASS. If it passes, the discriminator is gone and the repair proves nothing.
A4  `gate_emit_build_14152.sh` at the new head, kind=repair: `SELF_HOST_RC=0` and
    `EMITTED_BUILD_FAILED_LINES=0` on the self-host closure — this is the claim the whole exercise turns
    on. If it is still red at a THIRD site, minting is not carriable by the emitter and the answer is (B),
    not another id format. I will NOT chase site N+1 with format N+2.
A5  Main control unchanged: `gate_emit_build_14152.sh` at the same main SHA, kind=main, `SELF_HOST_RC=0`,
    GUARD `narrow=0 span=0 namekeyed=1`.
A6  Non-generics byte-identical to main: the two-closure hard-diagnostic comparator reports no NEW refusal
    identity versus main, in either closure. Any new identity stops the push and reverts the scheme.
A7  The mirror is regenerated from the tree (never hand-edited) and reaches a fixed point (round 2 installs
    nothing); GUARD prints `narrow=2 pin=1 span=1 fresh=2 acc=0`; the applied patch's hash matches the
    hand-back `FINAL_MIRROR_SHA256`.

## Explicitly out of scope for (A)
No change to `render_node_type`, `render_named_type_base`, the fold-accumulator bridge, `clone_param`, or
`unify_generics`. If (A) requires touching any of those to pass A4, that is evidence for (B) and I stop.
