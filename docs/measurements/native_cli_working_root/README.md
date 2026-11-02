# Native CLI entry, declared working root

**Goal:** prove that a compiler-route native CLI entry receives its working root from the argument
`std.compiler_entry` declares, that the absence of that argument refuses BEFORE any handler is built,
and that a cargo failure returns through the declared output fields instead of a panic. The subject is
the emitted program built by `v1.compiler.emit_rust emit_native_cli_driver_main_rs` for
`test.fixture.native_cli_working_root_entry`, run as a process. Nothing here is measured through the
interpreter: the interpreter's own grammar (`src/v2/cli/compile_cli.dag`) is exercised by claims, not
by these rows.

**Subjects.**

- Entry module: `dag/test/fixture/native_cli_working_root_entry.dag` (module
  `test.fixture.native_cli_working_root_entry`). It reads NO argv of its own -- every path it passes to
  `extdeps.rust.cargo_build_service CargoBuild.Build` is relative to the working root the BINDING
  carries -- and it carries the argv it was handed into its outcome text, so a run can also show that
  the emitted main passed the FULL argv down (one argv, two readers).
- Fixture crates: `dag/test/fixture/native_cli_working_root_fixture/ok` and `.../broken`. Both declare
  the package name `native_cli_working_root_fixture`, so the only thing a control varies is the source:
  `ok` compiles, `broken` fails with a type error.
- Model rows and the closed sum: `std.compiler_entry` `native_cli_working_root_argument`,
  `native_cli_working_root_absent_cause`, `native_cli_declared_execution_mode`,
  `native_cli_effect_binding`. Their claims are
  `dag/test/claim/native_cli_working_root_witness.dag` and, for the argv grammar half,
  `v2.test.claim.cli.v2_native_cli` (the rows that build their flag spelling FROM the row).

**Setup** (one dispatch, from a checkout of the head under test):

```sh
HEAD=$(git rev-parse HEAD)
SEED=<the seed binary built from $HEAD>          # record its sha256 and build time below
OUT=/tmp/ncwr/emitted
"$SEED" compile --target rust --output-dir "$OUT" \
  --source-root dag --source-root src/v2 \
  --entry dag/test/fixture/native_cli_working_root_entry.dag
(cd "$OUT" && cargo build --release)
BIN="$OUT/target/release/v1_compiled"            # the emitted crate's package is v1_compiled
```

**The emitted-text discriminators** (these are what go red when the route falls back; run them before
the process rows, against the emitted `src/main.rs`):

```sh
MAIN="$OUT/src/main.rs"
grep -q 'std_compiler_entry::native_cli_working_root_argument()' "$MAIN"      # the flag spelling is READ from the row
grep -q 'std_compiler_entry::native_cli_effect_binding(' "$MAIN"               # the refusal decision is the model's, not the emitter's
grep -q 'CargoBuild { working_dir: Some(' "$MAIN"                              # the binding names the root explicitly
grep -q 'DryRunMode(' "$MAIN"                                                  # and names the mode explicitly
! grep -q 'CargoBuild::new(Default::default())' "$MAIN"                        # no ambient construction for THIS service
```

**Row (i) -- the RED: an argv without the declared argument refuses before any binding is built.**

```sh
cd /tmp/ncwr && mkdir -p /tmp/ncwr/elsewhere && cd /tmp/ncwr/elsewhere
"$BIN" ; echo "exit=$?"
```

Expected: exit code 2, and the single line
`REFUSED: native effect binding: missing working_root: <native_cli_working_root_absent_cause>`
printed BEFORE any line naming a service binding (the emitted text has no binding statement to reach).
It must not print cargo output: nothing was spawned.

**Row (ii) -- the control: with the declared argument, the build happens AT THE DECLARED ROOT and
nowhere else.** The process cwd is deliberately somewhere else, so an ambient-cwd fallback would build
the wrong crate and be caught by the last two lines.

```sh
rm -rf /tmp/ncwr/ok /tmp/ncwr/elsewhere/target && mkdir -p /tmp/ncwr/ok
cp -r dag/test/fixture/native_cli_working_root_fixture/ok/. /tmp/ncwr/ok/
cd /tmp/ncwr/elsewhere
"$BIN" --work-root /tmp/ncwr/ok ; echo "exit=$?"
test -x /tmp/ncwr/ok/target/release/native_cli_working_root_fixture          # the artifact landed under the DECLARED root
test ! -e /tmp/ncwr/elsewhere/target                                          # and NOT under the process directory
```

Expected: exit 0; `stdout` carries the fixture's line and the entry's `argv_count=2`; the binary exists
at the declared root; the cwd stays clean. Reading the artifacts from the filesystem is the
discriminator -- the program's own claim of where it built is not evidence.

**Row (iii) -- the failure path: a nonzero cargo exit returns through the declared fields.**

```sh
rm -rf /tmp/ncwr/broken && mkdir -p /tmp/ncwr/broken
cp -r dag/test/fixture/native_cli_working_root_fixture/broken/. /tmp/ncwr/broken/
cd /tmp/ncwr/elsewhere
"$BIN" --work-root /tmp/ncwr/broken ; echo "exit=$?"
```

Expected: nonzero exit; the reported `stderr` contains the rustc type error for the broken fixture; the
program does not panic (no `no mock data available for dry-run operation` line, which is what a
dry-run-mode binding would produce).

**Provenance every recorded run must name:** the PR link, the exact head sha, the seed binary's sha256
and build time, the cargo/rustc version on the host (`cargo --version`, expected 1.93.0), and the host.

**Standing:** not yet run. These rows need a seed built from the head that carries the emitter and model
changes; a receipt from an unmatched binary does not establish them. Recorded here when run.
