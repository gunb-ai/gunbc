# RC_SCALAR_WRAP — corrected burn-down + stale-binary hazard (2026-07-25)

Doc-only receipt. **No emitter code changes in this PR** — the actual fix
(`value_inferred_type_is_rc_wrapped` in `emit_data_def_body`,
`src/v1/05_emit_rust.dag`) is already merged via #7190. This PR's earlier commit
added temporary DEBUG instrumentation to verify that fix's effect; that
instrumentation was reverted in `fd0f7b1481`. The full diff vs `main` is this
file plus the `HandAuthoredDocBind` row registering it in
`dag/gunbc/doc_graph_roots.dag` (required by the doc-reachability gate) —
no emitter/hand-Rust/shell delta.

## 1. Corrected measurement: #7190 is NOT a no-op

#7190's merged PR body understated its own effect (described the change as a
no-op). Re-measured against a verified-fresh local build across the "deep
seven" v2 compiler modules (`04_infer`, `05_emit`, `05_eval`, `06_translate`,
`emit_host`, `emit_module`, `materialization_carriers`), counting
`grep -c 'found `Rc<'` against each module's `cargo.log` from the
`curated_cargo_probe_one_keep.sh` harness (`CSSL_STD_SEED_LINK=1`, emit →
`cssl_assemble` → `cargo build --release --lib`):

| state | RC_SCALAR_WRAP count (deep-seven total) |
|---|---|
| pre-#7190 baseline | 461 |
| post-#7190 (fresh build) | 174 |

**461 → 174, a 62.3% reduction.** The fix is real and working. Six of the
seven modules dropped sharply; `materialization_carriers` was flat (15 → 15)
— see §3, a distinct sub-class #7190 does not touch.

A one-line comment has been added to #7190 linking this correction so its
"no-op" claim does not mislead a future reader.

## 2. General hazard: `ctrl-build` can silently build in a remote sandbox and never touch the local binary

This is what produced the *original* wrong "zero burn-down" conclusion in this
lane (corrected upstream of this PR, in direct conversation with
sharp-bee-290) — recorded here as a durable trap for any session doing an
edit → regen → rebuild → measure cycle on gunbc binaries, not specific to
RC_SCALAR_WRAP.

**Symptom:** `ctrl-build -- cargo build --release --bin <bin> -p <pkg>`
reports success (exit 0, `Finished `release` profile`) but the local
`target/release/<bin>` file's mtime does not advance past the source edit
that supposedly triggered the rebuild. Every subsequent "measured by
execution" check against that binary is silently testing stale code.

**Tell, visible in the build log:**
- paths rooted at `/root/workspace/repo-root/...` (not the local worktree path)
- `Uploading artifacts from /root/workspace/artifacts/command-0 ... Uploaded 0 artifacts`

**Fix / verification:**
1. Before trusting any execution-based measurement, check
   `ls -la --time-style=full-iso target/release/<bin>` and confirm its mtime
   is *after* the edit + regen it's supposed to reflect.
2. If it isn't, force a genuine local build:
   `CTRL_BUILD_BYPASS_SHIMS=1 cargo build --release --bin <bin> -p <pkg>`
   (or plain `cargo build` outside `ctrl-build` entirely).

(Also recorded in this session's cross-conversation memory as
`ctrl-build-remote-sandbox-stale-local-binary`, for reuse in future sessions.)

## 3. Residual RC_SCALAR_WRAP sub-class — named, not fixed here

`materialization_carriers` stayed flat at 15 RC_SCALAR_WRAP occurrences
before and after #7190. Tracing the residue: this is a **distinct** failure
shape from the cached-`data`-def double-wrap #7190 fixes — a **generic
function return-type substitution mismatch in call-argument position**. E.g.
`nanosecond_duration_count`'s emitted return type resolves to
`Rc<CommutativeSemiring<Magnitude>>` for the wrong type parameter, where the
call site expects a bare `i64`.

Per sharp-bee-290's ruling, this sub-class is **explicitly out of scope for
this PR/session** — it converges with eager-crane's `Measure`/
`GroupCompletion` residue and silent-badger's Root-4 arith-derivation work
into a shared "Root-4 lane" to be scoped separately by the manager. Not
started here.
