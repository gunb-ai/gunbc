# PARKED — do not land as-is (operator-lane ruling, bright-ram-778, 2026-08-31)

This branch holds the identity-keyed `recursive_type_set` carrier (name -> declaring
ident_span files, kernel `<kernel:Name>` seeded), its regen fixed point (3 rounds,
first_generation_equal=true), and the enrolled red
`emit_field_layer_keys_on_declaring_identity_not_bare_name`.

Measured disposition at parking time: the enrolled red is NOT greened by this change —
the producing defect is one level below, in the shared type-env (`lookup_type_by_name`
tier-2 `ancestry_str_bindings`, a bare-keyed last-writer merge that resolves module b's
`Nat` to the wrong declarer despite an explicit selective import). The 03_normalize
census is byte-identical pre/post (88 errors), live occupancy of the class is zero
(bold-carp's rustc-JSON split of the 146 E0308s), and the `needs_box_wrapping` recursive
branch may be Rust-unreachable behind the shared_types short-circuit.

REVIVAL CONDITION (the whole check — this branch is revived by it and by nothing else):
after the env-precedence fix (owner: bold-carp-449, emission-follows-resolution lane)
lands and the enrolled red GREENS, re-run this branch's layer-set arm on the repaired
tree and ask whether it then has a discriminating red of its own. A repair underneath
changes what a test above it discriminates: the env fix may unmask a second defect this
change closes, or may prove this change was never a wall. If it discriminates, land it
WITH that evidence; if not, delete this branch — it was a decoration and we learned it
cheaply.

The reproducer + in-process test enroll as the red for the env fix; they are handed to
bold-carp-449 (msg_83cf472b, 2026-08-31) and live in this branch's
`src/v1/compiler_tests_rust.dag` (`ct_recursive_layer_identity_test`).
