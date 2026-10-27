# Seed-build reproducibility — working plan (node adhoc-65499733-aab, Manager 2 clever-crane-830)

Detail lives here; the pinned plan is a short summary. Reordered ahead of E0282/E0425 by root
through manager msg_d8a302b8: one commit hashed differently on three runners (valiant-deer-27's
receipt, seed 3dbf6b88 at main a430c878e5cf), which falsifies every seed-sha receipt.

## Classification (measured, ctrl-build runner, rustc 1.95.0-nightly, commit a430c878e5cf)

| probe | result |
|---|---|
| same checkout path twice, clean tree, cold target dirs | IDENTICAL `b8c24b3d…` |
| same path + one untracked file | DIFFER `eedf8e3c…` (build.rs `-dirty` suffix) |
| two independent clean clones, different absolute paths | DIFFER `ec03f8ac…`; `objcopy` per-section: variance entirely in `.rodata`; `strings` shows the checkout path baked in (`/tmp/rr2/g1/src/v1/stage0/src/cli_run/pre_entry_phase.rs`) |
| manager's own control: two clones, distinct `CARGO_HOME`, `RUSTC_WRAPPER` unset, identity pinned | WITHOUT remap DIFFER (`141b29b0` vs `2941d394`); WITH the four `--remap-path-prefix` pairs + pinned identity IDENTICAL (`442c89e5` both), `strings` shows `/build/checkout`, `/build/target`, `/build/cargo-home`, `/build/rust` |
| regen determinism (b′): two independent clones, one `--regen-round-cost` each, `.git` stripped | IDENTICAL: `diff -r` empty, whole-tree sha256 aggregate `2235b0d8…` |

So: generated `.rs` inputs do not vary (not a correctness bug, no non-deterministic regen); the
variance is compile-grain path bake plus the `-dirty` identity — a receipt-grain bug. Not
toolchain-level, so no staffing exception to the operator is needed.

An earlier `REGEN-DETERMINISM: DIFFER` line came from my own probe re-running regen in place after
`git stash` / `git checkout -- .`, so round two saw a different working state. Discarded; the
two-clone form above is the honest one.

## The four inputs that must be pinned

checkout root, `CARGO_TARGET_DIR`, `CARGO_HOME`, rustc sysroot — each is fused into `.rodata`
(panic `Location` strings, file literals) with nothing normalizing it, and the tree identity
(`GUNBC_MATERIALIZED_TREE_IDENTITY`, consumed by `src/v1/stage0/build.rs`). Declared once in
`gunbc.repo.repo_self_build repo_self_build_env_preamble` and consumed by every seed-build step
(DESIGN §3); workflows are regenerated, never hand-edited.

Two non-obvious consequences recorded because they are each a silent trap:

- `.cargo/config.toml` carries `[build] rustflags = ["-D", "warnings"]`, and an **ambient
  `RUSTFLAGS` overrides that table key rather than extending it**. So the denial travels inside the
  preamble, taken from `gunbc.repo.repo_self_warning_policy` — the same authority the config file's
  own comment names — and leads the string.
- The fleet release-bins key deliberately does **not** key the remap prefixes: they name locations,
  and the key already excludes the location-naming environment variables
  (`release_bins_unkeyed_environment_pattern` drops `CARGO_HOME`/`RUSTUP_HOME`/`CARGO_BIN`). Keying
  them per runner would re-fragment the cache per checkout depth and defeat the repair. What makes
  that sound is the falsifier going red if any prefix is dropped, not an assertion.

## Preamble behavior, verified locally (four branches)

clean tree → `GUNBC_MATERIALIZED_TREE_IDENTITY=tree:<fmt>:<HEAD^{tree}>` pinned; untracked-only →
still pinned (this is the repair: untracked files reach neither rustc nor the identity);
tracked-dirty → left unset so `build.rs` bakes `<head>-dirty` honestly (refusing instead would be a
new CI behavior for a case the suffix already describes); outside a work tree → refusal, because an
empty remap source matches nothing and would leave the absolute path baked in while the step
reported success.

## Control (manager condition 3)

A gunbc test label, not a new CI job: build one commit twice from two clones with the preamble and
require `cmp`-equal sha256, plus a RED arm with the remap disabled that requires the hashes to
differ. Modelled on the repo's existing ignored host-executing falsifier
(`evaluation_budget_consequence_falsifier_host` + its `compiler_tests.rs` test), authored in
`src/v1/compiler_tests_rust.dag`.

## Census: every seed-binary build step and its .dag producer

| YAML | step | producer |
|---|---|---|
| fleet-converge 162, 591 | Build release bins | `fleet_workflow_steps ci_release_build_step` / `ci_release_build_step_for` |
| fleet-converge 613, 616 | Build release bins (dispatched mode, feature-gated retry) | `fleet_release_bins_key release_bins_build_script_dispatch` |
| fleet-converge 752, 815, 2570 | Build the gunbc binary the grant wait / reader key mint runs; Build the compiler pair this boot runs | `fleet_converge_workflow fleet_converge_hosted_gunbc_build_step`, `compiler_pair_release_product().build_line` |
| heal 75 | Build the regenerator | `heal_workflow` via `witness_floor_build_step` |
| heal-publish 68 | Build the publisher | `heal_publisher_workflow` via `witness_floor_build_step` |
| witnesses 149 | Build the witness floor | `witness_floor_workflow witness_floor_build_step` |
| drift-watch 54 | Build the drift watcher | `drift_watch_workflow` via `witness_floor_build_step` |
| instrument-dispatch 255 | Build the compiler | `instrument_dispatch` via `witness_floor_build_step` |
| branch-run-grant-broker 51 | Build the broker | `branch_run_grant_broker_workflow` via `witness_floor_build_step` |
| compiler-gate | Build the compiler | `compiler_gate_workflow` via `witness_floor_build_step` |

Consumption is wired inside the two step builders (`witness_floor_build_step`, and
`repo_self_seed_build_step_run` for the fleet ones) so a step cannot forget the environment.

## RFM row

`dag/gunbc/recurring_failure_mode/seed_sha256_is_runner_relative.dag`: a seed sha256 names a
(commit, runner) pair until this lands; the sound interim citation is **commit + sha256 + host**.
Sibling to `build_cache_key_includes_the_absolute_checkout_path` (same path inputs, cache-key
surface rather than identity-digest surface).

## The measurement, and the instrument that prints it

Instrument: the `seed_build_reproducibility_falsifier` label authored in
`src/v1/compiler_tests_rust.dag` (`ct_seed_build_reproducibility_falsifier_test`), host mirror
`src/v1/stage0/src/compiler_tests.rs::seed_build_reproducibility_falsifier`. It is `#[ignore]`d
because it performs three cold release builds of the seed. Run:

    cargo test --lib -p v1-compiler -- --ignored --nocapture --test-threads=1 seed_build_reproducibility_falsifier

It prints its own line rather than having hashes transcribed into this file. From the run on the
remote runner (linux/amd64, rustc 1.95.0-nightly) at commit `28c994c4e1e54da925c27e9602151f3d898af98f`,
19m05s, `test result: ok`:

    seed-build-reproducibility commit 28c994c4e1e54da925c27e9602151f3d898af98f: preamble
    a=fde094512484e3c7b6ce61d6940381ecca5b8ce69364be85a8ffa799d78629eb
    b=fde094512484e3c7b6ce61d6940381ecca5b8ce69364be85a8ffa799d78629eb  (green arm, must be equal)
    no-preamble c=7a206d54631b4a8fd0a6c16c6ba74050bb936e29ec2ea20cae450541ecc305fa  (red arm, must differ from a)

Both arms discriminate: two independent clones through the preamble give ONE sha256, and the same
commit built the way the tree was built before this repair gives a different one. The before side is
measured too, not asserted — the probes at the head of this file (two clean clones at different paths:
`ec03f8ac…` vs the reference; and the manager's two-clone control without remap: `141b29b0…` vs
`2941d394…`), and the same control with the remap plus a pinned identity: `442c89e5…` on both.

If the preamble is removed from the emitted workflow the test refuses before building; if it loses a
prefix, the red arm collapses toward the green arm's shape and the assertion on `c != a` fails.
