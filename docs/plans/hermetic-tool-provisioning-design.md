# Hermetic tool provisioning — one pinned tool identity, ensured not probed

Operator-directed 2026-07-29. Every mechanical external CLI dependency is a
declared, pinned, repo-local member reconciled by ensure/upsert. Ambient `PATH`
divination, hardcoded host paths, and existence-as-readiness are deleted.

Scope is **mechanical paths only** — CI, hooks, floor gates, probes, dispatch,
emitted workflows. A developer's interactive shell remains their own business;
this lane never dictates what is on a human's `PATH`, only what the machine is
permitted to *depend on*.

---

## 1. The defect: "where is this tool" is answered five ways, none hermetic

The pre-push `cargo: command not found` incident
(`docs/plans/recursive-workflow-advancement-design.md` §3, "The publication gate
had its own ambient dependency") is not a hook bug. It is one instance of a
mechanism forked five ways — a §3 nickname fork at the resolution layer, which
is why repairing the hook alone would only mint a sixth:

| Site | Mechanism | Terminal arm |
|---|---|---|
| `src/v2/workflow/ci_regen_rustfmt_path_emit.dag` | 4-location bash ladder (`CARGO_HOME/bin` → `/opt/cargo/bin` → `$HOME/.cargo/bin` → `rustup which`) | refuses |
| `src/v1/stage0/src/v1_interpreter.rs:8296` `resolve_host_tool_program` | `PATH` scan → `CARGO_HOME/bin` → `~/.cargo/bin` | **returns the bare name** |
| `src/v2/workflow/ci_workflow_run_emit.dag:46` | `$CARGO_HOME/bin/cargo` else `command -v cargo` | refuses |
| `docs/probes/curated_cargo_probe_one.sh:47` | `[[ ! -x ]]` existence | rebuilds |
| `dag/extdeps/tools.dag` `resolve` | `shell.Which.Check` | `NotFound { hint }` — instructs a human |

Two are independently defective:

**`resolve_host_tool_program` fails open.** After three probes miss it returns
`name.to_string()` and hands the bare name to `Command::new`, so the failure
arm *widens* to "try it and see" and surfaces later as an opaque spawn error —
the absorbing fallback DESIGN §5 forbids. Its success arm is also decorative:
it locates the file in a specific `PATH` directory and then returns the bare
name rather than the resolved path, discarding what it learned and
re-resolving ambiently at spawn time.

**`curated_cargo_probe_one.sh` conflates existence with freshness.** A stale
binary is executable, so the rebuild arm never fires. This already produced a
retracted measurement — `docs/probes/rc-scalar-wrap-fresh-build-correction-2026-07-25.md`
§2, where a "zero burn-down" conclusion was wrong because the local binary was
never rebuilt (the real figure was 461 → 174). Its prescribed remedy is a human
mtime check, i.e. validation standing where construction was available.

### Census of mechanical dependencies

- **Bare names in `.dag`:** `cargo` ×30, `cc` ×5, `python3`, `go` ×2, `node`,
  `npx`, `claude`, `true`
- **Hardcoded absolute host paths** — non-portable *and* ambient:
  `/usr/bin/{git,jq,tmux,tee,sh,ln,install}`, `/usr/bin/codex`, and
  `/home/briansrls/.cargo/bin/{cargo,rustfmt,rustc}` in
  `dag/gunbc/roadmap_dashboard_instance.dag` and two witness tests
- **Rust `Command::new`:** `cargo`, `git`, `rustfmt`, `diff`, `date`, `sleep`,
  `true` across `bootstrap_witness.rs`, `pre_push.rs`, `regen_stage0.rs`,
  `v1_interpreter.rs`
- **Emitted CI `run:` steps assume:** `git` ×86, `tar` ×7, `sed` ×4, `awk` ×3
- **`CliTool` coverage: 12 rows, 7 versioned, 5 `none`.** The tools that carry
  the pipeline — cargo, rustc, rustfmt, git, tar, node, python3, cc — have **no
  row at all**.

`jq` is the fork in miniature: a `CliTool` row with `min_version: none`, a
modeled `argv: ["jq", …]`, *and* a hardcoded `/usr/bin/jq` elsewhere. Three
representations, no pin.

## 2. What is already right

This lane extends existing carriers; it does not rebuild them.

- **`rust-toolchain.toml`** — an exact pin (`1.93.0` + `["clippy","rustfmt"]`)
  whose header already declares it the "Sole in-repo rustup channel authority
  (CI + local)".
- **`InstallSource.SourceGitHubRelease { repo, tag, install_path, asset_aarch64,
  asset_x86_64 }`** — already nearly the hermetic-installer shape; it needs a
  content digest to become content-addressed.
- **CI already isolates** — `ci_workflow_run_emit.dag` wipes and sets
  `HOME`/`CARGO_HOME`/`RUSTUP_HOME` under `$RUNNER_TEMP`. The end state
  generalizes this to every mechanical context, rather than inventing it.
- **`.gitignore` already reserves `/.cargo-home/`** (`gitignore_emit.dag:45`,
  `LocalCargoHome`) — the repo-local root, declared and currently unused with
  no writer anywhere.
- **GHA actions are tag-pinned** (`@v5`, `@v1.16.0`). Tags are mutable, so
  SHA-pinning is the hermetic form, but the discipline exists.

### `ResolvedBuildContext` (#7388) is the observed half — this lane is the other half

Merged 2026-07-28, `dag/extdeps/realization/emit_on_demand_host.dag`:

```
type ResolvedBuildContext {
  toolchain_identity: ContentHash
  environment_identity: ContentHash
  cargo_configuration_identity: ContentHash
}
```

Its seed realization (`v1_interpreter.rs`, `observe_tool`) digests the tool's
**logical name + executable bytes + version-probe stdout/stderr**, probes with
the materialized workspace as `current_dir` so `rust-toolchain.toml` selection
is observed, and admits only `env_clear`-constructed hashed environment rows.
That is genuine content-addressed tool identity, already landed.

The relationship is complementary, and it fixes this lane's seam:

- #7388 **observes** — *which* toolchain did we actually get? Hash it so a
  changed toolchain cannot reuse a warm artifact. It never refuses and never
  provisions; a wrong-but-consistent toolchain is faithfully recorded and
  silently used.
- This lane **ensures** — get the *declared* one, or refuse.

So the pin does **not** mint a second toolchain-identity concept (which would be
exactly the §3 fork this lane exists to remove). `ToolPin` declares the
*expected* `ContentHash`; `resolved_build_context_identity` already computes the
*observed* one; ensure is the reconcile between them, which is precisely
`membership_reconcile`'s `value_eq`. P1 therefore reuses `ContentHash` and
`observe_tool`'s digest discipline rather than defining its own.

Two census entries #7388 adds rather than removes:

- `v1_interpreter.rs:8672` — `std::env::var("RUSTC").unwrap_or_else(|_| "rustc")`,
  a fresh ambient bare-name resolution.
- `ci_native_cache_root_toolchain_segment_command`
  (`ci_workflow_run_emit.dag:31`) is **unchanged** and still fabricates
  `toolchain-unresolved` when `rustc -V` fails, so every host that cannot
  resolve its toolchain shares one stable cache root. ⊤-as-ignorance treated as
  ⊤-as-answer; it refuses under P1's refresh/identity arms.

## 3. Existence is the wrong question — tools are reconciled members

The five resolvers all ask *"is this tool present?"*. That is the host-shaped
question `dag/extdeps/rust/rustup.dag` already warns produces "a true-sounding
answer to a question it did not mean" — a component is a property of a
TOOLCHAIN, not a machine, and `~/.cargo/bin` entries are shims dispatching to
whichever toolchain is active. This box proves it: `which cargo` →
`/usr/local/ctrl-build-shims/cargo` → `/opt/cargo/bin/cargo`, itself a symlink
to *rustup* dispatching on `argv[0]`. Four layers, and finding "a cargo" still
says nothing about which toolchain executes.

The right question is **ensure**, and the carrier already exists:
`gunbc.membership_reconcile` (`membership_reconcile<M,K>`), the grain-agnostic
desired-vs-observed diff. Tools are one more instantiation with their own
bundle — zero spine change, per its own authority note ("one fn, N bundles — a
Realization; a forked reconcile means the genericity bought nothing"):

- `key_of` = tool identity (the tool's name/role), stable, never content — so a
  version bump is `Modified → upsert`, i.e. **reinstall in place**, not
  teardown-and-reinstall.
- `value_eq` = pinned version **+ content digest** — a drift at the same key
  re-upserts, which is exactly the ensure semantics wanted.
- `ownership_of` = `Owned` for tools provisioned into the repo-local root
  (removable), `Ensured` for system-required tools — so the R5 construction
  wall already refuses teardown of a system tool (`MemberTeardownRefused`,
  which has no effect arm in any apply dispatch), and ownership-unknown refuses
  rather than assuming.

The tiering in §5 therefore lands on existing types instead of new ones.

`extdeps.tools.resolve` correspondingly gains its missing arm. Its present
result — `Resolved | NotFound { hint }` — terminates in a human instruction;
the ensure path terminates in a *provisioning action*, and refuses only when
provisioning itself fails (typed, located, counted).

## 4. Pinning without freezing — the refresh axis

An exact pin that is never revisited is its own defect: it rots, and nothing
ever says so. But resolving a moving reference at *execution* time is worse —
it is non-determinism at the substrate boundary and would breach
`v2.std.determinism`. Both are avoided by separating **what executes** from
**how it was chosen**:

- **Execution is always exact.** Every run resolves a specific version with a
  specific content digest. There is no "latest" at execution time, ever.
- **Selection carries a policy.** A pin is either authored directly
  (`ExactPin`) or resolved from a moving reference and *recorded*
  (`TrackedChannel { channel, resolved_to, resolved_at }` — `latest`, a release
  channel, a semver range). Re-resolution is an explicit action producing a
  reviewable diff, exactly as `Cargo.lock` and `flake.lock` behave.
- **Staleness is observable and counted, never silent.** A tracked pin past its
  declared refresh window yields a typed diagnostic that reds. It does not
  float forward on its own, and it does not sit quietly at a three-year-old
  version. This is the §5 discipline applied to the pin itself: the failure
  refuses and is countable, so it ranks for fixing.

So "allow a latest tag" is satisfied without conceding hermeticity: `latest` is
admissible as a *selection policy*, never as a runtime lookup. The recorded
resolution is what runs.

`min_version: VersionConstraint?` does not survive this. It is a *range*
(`curl` pins `">= 7.68"`), it is optional, and a range admits a different
binary on every host. For mechanical tools the pin is exact and required.

## 5. The tiering, and the irreducible floor

Full hermeticity has a bottom. Naming it is the honest form; pretending
otherwise is the §5 trap.

- **Tier 1 — hermetic, repo-local, digest-pinned.** rustup/cargo/rustc/rustfmt/
  clippy, node/npx, jq, curl, gh, codex, claude, xorriso, socat, websocat,
  nbdkit, busybox. All are fetchable static releases; this is the bulk of the
  census and where the win is. Ownership `Owned`.
- **Tier 2 — system-required, declared and version-*asserted*.** `cc` + linker
  + libc (rustc needs a system linker; shipping a C toolchain is a separate
  order of work), plus daemon-backed tools — docker, systemd, sudo, tmux.
  Ownership `Ensured`; teardown refuses by the R5 wall.
- **Tier 3 — the bootstrap floor.** `sh`, coreutils, `tar`, and one fetcher.
  You cannot hermetically install the thing that installs things. This is a
  short, explicitly declared set asserted at startup — DESIGN §5's genuinely
  unstructurable residue, not a silent assumption.

`git` is the open judgment call: portable and fetchable, but it is the
transport for the very checkout that would contain the pinned copy. It starts
in Tier 2 and is revisited once Tier 1 is proven.

Storage is per-worktree (operator ruling 2026-07-29: worktrees are not shared,
and ~1.5 GB is cheap). No content-addressed cross-worktree root in this lane.

## 6. Phases

Each phase names its own RED control. Model-before-implement: P1 lands with no
consumers.

- **P1 — the pin model.** Extend `CliTool` into a pinned identity: exact version
  (`extdeps.version.VersionIdentity`, the exact brand — never
  `VersionConstraint`, which is a range) + expected `ContentHash` reusing
  #7388's identity, `SourceGitHubRelease` gains the digest, selection policy
  (`ExactPin | TrackedChannel`) with recorded resolution and a refresh window.
  Both fields are required and non-optional, so a range-only or digest-less pin
  is **unwritable** rather than validated — the RED lives at the ingest
  boundary instead: converting a legacy `CliTool` whose `min_version` is a
  `VersionConstraint` refuses, because a constraint is not an identity. A
  tracked pin past its window reds.
- **P2 — one resolver.** Instantiate `membership_reconcile` for tools with the
  pin as desired and #7388's observed identity as observed, and collapse the
  sites onto it. Delete `resolve_host_tool_program` and the bash ladder — the
  ladder's own dissolve-on trigger already asks for exactly this ("DISSOLVES
  WHEN rustfmt resolution is a typed toolchain probe on `host_effect_apply`
  rather than emitted bash"). RED: an unpinned tool refuses before spawn, not
  after; a digest mismatch refuses rather than recording a new identity and
  proceeding.
- **P3 — Rust hermetic.** Provision the pinned toolchain into the declared
  repo-local root. Shortest path to a working proof and it covers 30+ census
  hits. Gated on confirming the srv3-07/08 `exit 127` rustup extract flake
  (`ci_floor_gate_toolchain_note`, 2026-07-21) is actually resolved before
  floor gates lean on isolated `RUSTUP_HOME`.
- **P4 — residue to zero.** Migrate the bare names and `/usr/bin/*` literals
  onto resolved tools; a lens reds a new one. Construction half: argv positions
  take a resolved tool, not a `String`, so the bad state is unwritable; the lens
  covers the residue.
- **P5 — declare the floor.** Tier 2/3 rows with asserted versions and a
  startup check, so the ambient set is small, named, and observable.

## 7. Non-goals

- Dictating a developer's interactive environment. Mechanical paths only.
- A content-addressed toolchain root shared across worktrees (ruled out above).
- Shipping a C toolchain (Tier 2 keeps `cc` a declared system requirement).
- Re-attempting isolated `RUSTUP_HOME` for CI floor gates before the srv3 flake
  is confirmed fixed; P3 is explicitly gated on that receipt.
