# R1C-F demo — user-authored lens rejects violating program

**Running artifact:** `demo_user_authored_lens_rejects_violating_program_suite` in
[`src/v3/compiler/tests/t_demo/t_demo_fixtures.dag`](../../src/v3/compiler/tests/t_demo/t_demo_fixtures.dag);
runner harness `t_demo_user_authored_lens_rejects_violating_program_passes` in
[`src/v3/compiler/tests/integration.rs`](../../src/v3/compiler/tests/integration.rs).
Reproduce: `cargo test -p v3-compiler --test integration t_demo_user_authored_lens_rejects_violating_program_passes`.

**What this demonstrates.** A user writes a 13-line `.dag` lens
([`src/v3/lenses/named_function_count.dag`](../../src/v3/lenses/named_function_count.dag) — the same
GREEN T-LensAPI `user_authored_lens_compiles` lens) declaring the dimension *"count every
top-level named binding."* The violating fixture program declares three named bindings
(`a`, `b`, `c`); the compiler runs the user's lens against the program's substrate, the lens
emits `3`, the `LensOutputEquals` predicate matches the expected violation count, and the
gate Passes — meaning the user-authored rule correctly rejected the program. The mechanism
is byte-identical to the built-in complexity / ownership / parallelism lens demos in the same
fixture file; the only thing that changed is who wrote the lens. This operationalizes
[`THESIS.md` §"User-defined dimensions"](../../THESIS.md): the ceiling of what gunbc can prove
is user-extensible, not compiler-baked — adding a new correctness dimension is one `.dag` file,
not a compiler patch.
