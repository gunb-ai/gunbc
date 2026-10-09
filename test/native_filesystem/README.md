The native source-root door uses the existing `Filesystem.Read` and `Filesystem.List`
transport, realized by `v1.compiler.emit_rust.emit_file_call`. Traversal and read
refusal live in `gunbc.source_root_read`; witness construction lives in
`v2.compiler.source_authority.source_root_ingest_from_files`. The native CLI calls
`v2_cli_main` once, binding one emitted Filesystem service at its entry.

The listing's existing error-kind channel now distinguishes NotADirectory. A child
with that observation can be a file; a root cannot. Missing entries, permission
failures, incomplete enumeration, and unrepresentable names all refuse the walk.
Read refusal uses the existing `FilesystemExactRead` coproduct. The legacy
content-only `filesystem_read` builtin is not called by this acquisition path.

Effectful tail calls share the pure emitter's loop lowering. Service/resource
bindings remain fixed; value parameters become loop slots. Non-tail effectful
self-recursion still refuses at emission. The old tail-refusal witness is now a
positive loop control; the non-tail refusal remains enrolled.

Run the native controls with a seed rebuilt from this checkout, inside an
enforceable memory envelope:

```sh
systemd-run --user --scope -p MemoryMax=8G target/release/gunbc compile \
  --source-root dag --entry dag/gunbc/source_root_read.dag \
  --output-dir target/native-filesystem-controls-crate
mkdir -p target/native-filesystem-controls-crate/tests
cp test/native_filesystem/controls.rs target/native-filesystem-controls-crate/tests/native_filesystem.rs
cargo test --manifest-path target/native-filesystem-controls-crate/Cargo.toml \
  --test native_filesystem
```

This compiles the actual `.dag` acquisition module and its import closure, then
executes Rust tests against those unmodified emitted modules: missing-file typed
absence, nested traversal and ordering, root refusal, unreadable UTF-8, listing
encoding refusal, and a large population exercising both tail loops. These tests
are local qualification, not a new required CI lane.

The existing `//gunbc/instruments:v2-native-cli` producer retains missing-root and
unreadable-source controls in `walk_cli_door`. Both require exit 2, empty stdout,
and a diagnostic locating the subject and its host error kind. A panic cannot
pass. That instrument is operator-invoked; it is not required CI coverage.

The source-root eval driver also calls the shared acquisition fold. Its remaining
argv, host-fact decoding, timing, output and orchestration obligations keep
`source_root_eval_driver_seed_growth_justification` live. This filesystem cut does
not retire that row or establish generation-two compiler correctness.

Qualification on `native-filesystem-fold`: the old seed emitted both acquisition
self-calls verbatim and Rust refused them with E0733. After rebuilding with the
shared loop lowering, all six native controls passed (4.12 seconds for the test
execution, including the 4,096-file control). The three pure traversal-policy
witnesses also passed. Full-driver and mirror validation are recorded separately;
these local controls do not stand in for either.
