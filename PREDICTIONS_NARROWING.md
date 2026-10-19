# Pre-registered predictions — narrowing the call-site mint to argument-less calls
# (stern-bat-401 msg_c4d4f3e1: approved, plus the two additions). Written BEFORE any run.

Change: `uninstantiated_call_generics` is now `if (typed_arg_nodes |> count) == 0 { declared params
absent from call_subst } else { [] }` (src/v1/04_infer.dag, ExprCall arm). Mirror still the shipped
un-narrowed one until the regen commit, so step 1 measures the PRE-NARROWING compiler.

N1 (the narrowing RED, measured on the pre-narrowing mirror at the content commit). New row
`arguments_present_do_not_trigger_the_call_site_mint` must FAIL, and its printed
`NARROWING-INVENTORY` must show `minted >= 1` with an id of the form `S@<file>:<start>-<end>`,
because `time_measure<S>(count: Nat)` passes one argument and its `S` is return-only. If it instead
PASSES here, the row does not discriminate and must not be shipped as evidence.
N2 Same run: `minted_variables_are_fresh_per_call_site` must still PASS with the unchanged inventory
`positions=24 distinct=4 site_tags=2 bare=0` -- the fixture's two `empty_map()` calls carry no
arguments, so the narrowing must not touch them. If that inventory changes, the narrowing is
narrower than "argument-less only" and is wrong.
N3 (after the mirror is regenerated from the narrowed .dag) BOTH rows must PASS: freshness with the
same `24/4/2/0` inventory, and the pin with `minted=0` and an `S` present in the id set.
N4 The name-keyed mutant (same narrowing applied, id form `g` only) must still FAIL rc=101 on the
freshness row with `distinct=2 site_tags=1 bare=2`. If it passes, the narrowing has gutted the
discriminator and the repair proves nothing.
N5 Both pre-push gates at the final head: `gate_emit_build_14152.sh` -> `SELF_HOST_RC=0`,
`EMITTED_BUILD_FAILED_LINES=0` (this is the E0277 being gone), and the diagnostic-count gate's
self-host / v2-native type-mismatch counts unchanged from the last measurement at this head. Any NEW
hard-diagnostic identity vs main, by identity, stops the push.
N6 Non-generics byte-identical to main is untouched by this change (the overlay is empty for a call
with no declared parameters either way), so the per-closure comparison should show no new refusal
anywhere; if it does, revert the narrowing rather than widening it.
