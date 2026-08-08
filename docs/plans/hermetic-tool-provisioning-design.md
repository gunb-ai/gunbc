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
the absorbing fallback DESIGN §5 forbids.

> **Correction (2026-07-30).** An earlier version of this paragraph added that
> "its success arm is also decorative: it locates the file in a specific `PATH`
> directory and then returns the bare name rather than the resolved path." That
> was **false**, and it shipped in #7398. Read against main, every success arm
> returns `candidate.to_string_lossy().into_owned()` — the *resolved* path — for
> the `PATH`, `CARGO_HOME/bin` and `$HOME/.cargo/bin` probes alike. Only the
> terminal arm returns the bare name, which is the fail-open above and the whole
> of the defect. The claim is withdrawn rather than edited silently, because a
> census that overstates a defect is as damaging to prioritisation as one that
> misses it: it invites a fix to a working code path.

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

Its seed realization (`v1_interpreter.rs`, `observe_tool_identity`) digests the tool's
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
exactly the §3 fork this lane exists to remove). `Pin<Subject>` declares the
*expected* `ContentHash` per subject, and P1 reuses `ContentHash` and
`observe_tool_identity`'s digest discipline rather than defining its own.

**The grains did not meet; #7444 closed that, and the gate is discharged** (`review 44388` raised it, merged 2026-07-29).
The earlier draft of this section claimed `resolved_build_context_identity`
already supplies the observed side and that ensure is simply `value_eq` between
the two. That was wrong, and the code says so: `toolchain_identity`
(`src/v1/stage0/src/v1_interpreter.rs:8649-8677`) is seeded from a constant and
then `hash_combine`d over **every** argv in `build_argvs`, plus a second
`observe_tool_identity` call for `rustc` whenever the argv is `cargo`. It is an
*aggregate over the whole build command set*, so comparing one `Pin`'s
`expected_identity` against it is a category error — the aggregate changes when
an unrelated tool in the same build changes, and it cannot say *which* tool
drifted.

The per-tool observed value nonetheless already exists: it is exactly
`observe_tool_identity(requested, version_args, ..)`'s return, computed per argv and then
folded away without ever being named. So the missing piece is a *naming*, not a
new probe — and naming it is the §3-correct move, because minting a second
per-tool digest beside `observe_tool_identity` would be the fork this lane removes.

P2 carried a prerequisite before any reconcile could be written:

> Surface `observe_tool_identity`'s per-argv result as a named per-tool observed identity,
> and re-express `toolchain_identity` as the *derived* fold over those rows
> rather than the authority. Only then is `Pin.expected_identity` comparable
> to an observed value at the same grain, and only then does
> `membership_reconcile`'s `value_eq` mean what this lane needs it to mean.

**That prerequisite is discharged.** #7444 landed it in
`extdeps/realization/emit_on_demand_host.dag`: `ResolvedBuildContext` now carries
`observed_tool_identities: List<ObservedToolIdentity>` as its first field,
`toolchain_identity` is the *derived fold* over those rows rather than a stored
aggregate, and the per-tool read is `observed_tool_identity` with three arms —
`ObservedToolIdentityFound`, `ObservedToolIdentityMissing { tool_name }`,
`ObservedToolIdentityDuplicate { tool_name, count }`. Miss and duplicate are
*distinct typed answers* rather than one `Option`, which is what makes the lookup
usable at a fail-closed boundary at all. No second per-subject digest was minted
beside `observe_tool_identity`, so the §3 fork this lane exists to remove was avoided.

What P2 still owes is the reconcile *itself* — the `membership_reconcile`
instantiation whose `value_eq` is `pin_value_eq` and whose `key_of` is the
per-subject projection. Two obligations on whoever writes it: route every read
through `observed_tool_identity` rather than re-deriving a digest, and answer
its `Missing` and `Duplicate` arms explicitly. Collapsing either into a match
failure or a widened rerun is the absorbing fallback §5 forbids, and would waste
precisely the precision #7444 bought.

Until that lands, treat #7388 as the *aggregate cache-key* consumer it is, not
as the observed half of this lane's ensure.

### Pinning is a dimension, not a property of tools (revision, 2026-07-29)

P1 first landed this as a `ToolPin` record keyed by `tool_name: NonEmptyStr`. The
operator read the shape and rejected it: pinning is a **separate dimension** that
*composes* with a subject rather than a fact belonging to CLI tools. That is
right, and the evidence was already in the tree:

- `extdeps.tools` passes tools **by value** everywhere else — `resolve(tool: CliTool)`,
  `ResolvedTool { tool: CliTool }`, `NotFound { tool: CliTool }`. `ToolPin` was the
  single site in that namespace joining to a modelled type through a **string**,
  putting tool identity in two places (§2 anemic leaf).
- Other pinnable subjects already exist and are **not** `CliTool`:
  `SccacheBinaryArtifact` (release × published musl arch — `extdeps.cache.pin`),
  `ActionRef`, `extdeps/docker` and `extdeps/container/docker_ce`
  images, apt packages, GitHub releases. A type per pinnable thing is the
  ten-integer-types mistake before `Compose<Int, MachineWidth<N>>`.

So the carrier is now `Pin<Subject>` in `extdeps.pin` (the dimension), with
`extdeps.tools.pin` holding only what is subject-specific for `CliTool`: the key
projection and the roster ingest. A new pinnable subject is a **sibling
instantiation**, never an edit to `extdeps.pin` — and if it ever requires one,
that is the signal the dimension was modelled too narrowly.

**Proven by a second consumer, not asserted.** A generic type earns nothing by
existing. `ActionRef { owner, repo, ref }` (`extdeps/github/actions.dag`) is
deliberately unlike `CliTool` — three plain `String` fields against a
`NonEmptyStr` name, an optional `VersionConstraint` and a `List<InstallSource>` —
and it instantiates `Pin`, `pin_value_eq` and `admit_pin_integrity` with zero
edits to `extdeps.pin` (`pin_composes_over_a_structurally_different_subject`,
green by execution). The subject is *consumed, not minted* — it is a live
`ActionRef` row already in the corpus, and **which** row is a fact owned by the
witness, not restated here (`review 44910` caught this paragraph still naming
`upload_artifact_action` after the witness had moved to `checkout_action` to avoid
a double-bound name; a row name with two homes drifts, so this one now has one).
That the subject carries no version while
the *pin* carries one is the point rather than a gap: the version is the pin's own
declared fact, so the dimension supplies it for subjects that have none.

It is also the lane's own next step rather than a synthetic exercise — §2 records
that GHA actions are tag-pinned and that a mutable tag makes SHA-pinning the
hermetic form. `Pin<ActionRef>` *is* that form.

**The soundness condition on the dimension, which two rejected subjects taught:**
a pin subject must not determine its own identity. If the subject already carries
a `ContentHash` then the subject **is** a pin — an OCI descriptor is a content
address — so `Pin` over it necessarily duplicates rather than supplies. The
authority for this is `extdeps.pin`'s
`pin_subject_must_not_be_self_identifying_note`, on the dimension rather than
beside any one instantiation, because it constrains every future `Subject`. It is
stated with its decidability: "the subject type declares no `ContentHash` field"
is decidable by structural inspection, so it is a *wall-after-grounding* wanting a
lens over the `Node` tree — not a permanent ratchet, and not a "never" (§5).

Three corrections recorded with it:

- **A claimed fork that is not one.** When proposing this revision I told the
  operator that `CliTool.min_version` and the pin's `version` were one concept
  forked by rigor. That was wrong. `min_version` is a `VersionConstraint` stating a
  *requirement*; `Pin.version` is a `VersionIdentity` stating a *selection*. A
  requirement and a selection satisfying it are different facts, and dissolving
  them would destroy information. What they have is a checkable **relation** — a
  pin ought to satisfy its subject's constraint — deferred behind
  `feature:version-constraint-satisfaction` because `extdeps.version` has no
  satisfaction fn and writing one in `extdeps/tools` would fork version semantics.
- **The first second-consumer proof was itself unsound** (`review 44662`). It used
  `Pin<SccacheBinaryRelease>`, which failed on both axes this lane polices. *Grain:*
  one release carries two architecture-specific digests while `Pin` carries one
  `expected_identity`, so either artifact's digest could be associated with the
  undifferentiated release. I had recorded that as a tracked `dissolve-on` and
  treated the deferral as sufficient — wrong, and the reason is specific to what a
  proof is. A trigger-only deferral is a legitimate way to carry a known gap on a
  *production* row; a row whose entire job is to prove the dimension composes
  cannot rest on an ambiguous instantiation, because it then demonstrates ambiguity
  rather than composition. *Parallel authority:* it re-minted the aarch64 hex and
  the release version already owned by `extdeps/cache/sccache.dag`, so two copies
  of one fact could drift — the consume-never-fork violation this lane exists to
  remove, committed inside the witness meant to demonstrate the lane. Fixed by
  choosing a sound subject rather than defending the unsound one.
- **The second attempt was unsound too, and this note asserted otherwise**
  (`review 44850`, and `review 44875` for the fact that this paragraph outlived
  the fix). The replacement subject was `OciDescriptor`, and an earlier version of
  the bullet above ended by recommending it: a descriptor "describes exactly one
  blob, so the grain question cannot arise, and it carries `digest: ContentHash`
  natively so `expected_identity` is **derived** from the subject rather than
  copied." The first clause is true; the second was **false and is withdrawn**.
  The construction *copied* the digest, nothing in `Pin<OciDescriptor>` forced the
  two fields to agree, and `admit_pin_integrity` compares only
  `expected_identity` — so a `Pin` whose halves disagreed was writable and would
  have been admitted on the wrong one. A claim of derivation standing over a copy
  is the §5 tell: a declaration that reads like a construction wall while the
  realization lies. Two failures are worth separating here. The modelling failure
  was choosing a self-identifying subject, fixed by `ActionRef` above. The
  *documentation* failure was this bullet surviving that fix — the plan went on
  recommending the rejected pattern after the carriers had retired it, so a reader
  of the plan alone would have reimplemented it. That is the same defect as
  `review 44422` on this PR, where a correction was accepted by *adding* a note
  while the contradicted sentence stood; the rule it produced is that a
  superseded claim is edited at its source, never annotated in place.
- **Multi-artifact grain, LANDED (`feature:pin-artifact-grain`).** For subjects that
  publish several artifacts the pin subject is the *artifact* (release × platform),
  not the release. That observation stood on the corpus rather than on this witness,
  which is why sccache was *rejected* as the second-consumer proof here. The discharge
  is `SccacheBinaryArtifact` + `extdeps.cache.pin` + `sccache_pin_witness_test.dag`,
  with `sha256_digest_content_hash` bridging cited `Digest` to `ContentHash` so the
  artifact pin does not launder a second digest concept into a witness. Remaining
  unfixed, belonging to no lane here: `SourceGitHubRelease` still carries
  `asset_aarch64` / `asset_x86_64` at release grain — a future GitHub-release
  artifact projection would follow the same pattern, not a fork of `extdeps.pin`.

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
  wall already refuses teardown of a system tool (`MemberRemovalRefused`,
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
  (`ExactPin`) or resolved from a moving reference (`TrackedChannel` —
  `latest`, a release channel, a semver range). Re-resolution is an explicit
  action producing a reviewable diff, exactly as `Cargo.lock` and `flake.lock`
  behave.
- **Staleness is observable and counted, never silent.** A tracked pin whose
  currency cannot be established is refused. It does not float forward on its
  own, and it does not sit quietly at a three-year-old version.

### `TrackedChannel` refuses today — the fail-open review 44242 found

The first shape of this model stored **no** resolution evidence on
`TrackedChannel` and took `age_days` as a **caller-supplied parameter**. That
made the freshness verdict entirely the caller's to assert: passing `0` forever
admitted a permanently stale pin, while this document claimed the resolution
was "recorded". Review 44242 called it correctly. It is DESIGN §5's precise
tell — a check satisfiable by editing the caller while the model lies — and the
earlier decision to drop the resolution date as an "unread field" was the wrong
correction. The right one is to make the fold **read** it.

Grounding it needs two things the tree does not have: a carried resolution date,
and calendar day arithmetic to derive an age from it. `v2.std.datetime` models
`CalendarDate` but has **no** day-difference, epoch-day, or days-between
function, and has **zero constructors in use anywhere in the corpus** — so
deriving an age in P1 would mean both becoming that type's first consumer and
forking calendar arithmetic into `extdeps/`, a §3 fork of a std concern.

So P1 does not offer a verdict it cannot ground. The fabricable parameter is
deleted outright, and a `TrackedChannel` pin refuses admission even when its
observed digest matches exactly.

### Integrity and currency are different questions — review 44274

The first correction still exempted the wrong variant. `ExactPin` returned
`PinFresh` **unconditionally**, on the reasoning that an authored pin has no
upstream drift for a window to bound. That was wrong, and wrong against this
lane's own purpose: *authorship proves reproducibility, not currency.* An exact
pin authored three years ago is exactly as reproducible and exactly as rotten,
and reporting it fresh forever is precisely the "pinned and never updated"
failure the refresh axis exists to prevent — the operator's own framing when
they asked for the axis.

One word was carrying two questions:

| axis | question | decidable in P1? |
|---|---|---|
| **integrity** | is this binary the one the pin declares? | **yes** — compare observed digest to `expected_identity` |
| **currency** | is the declared version still the one we want? | **no** — for *either* variant |

They are now separate. `admit_pin_integrity` establishes integrity only and is
named so it cannot be misread; its success arm is `PinIntegrityAdmitted`.
`pin_currency_gap` is **total and has no positive arm at all** — it returns
`AuthoredPinHasNoReviewDate` for an exact pin and
`TrackedPinHasNoResolutionDate` for a tracked one. No arm anywhere reports a
pin current, so currency can never be inherited from an integrity pass; a
consumer that needs it reads the gap and refuses.

`ExactPin` remains usable for what P3 needs (the Rust toolchain is an exact
pin) — its *integrity* is fully checkable. Dissolve-on: `v2.std.datetime` gains
day arithmetic and the pin carries a review/resolution date, at which point
`pin_currency_gap` gains a genuine positive arm derived from stored evidence
and an observed date.

So "allow a latest tag" is satisfied without conceding hermeticity: `latest` is
admissible as a *selection policy*, never as a runtime lookup — and the
companion half of that request, *don't pin and never update*, is honoured by
refusing to claim currency for any pin rather than by exempting authored ones.

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

- **P1 — the pin carrier, alongside `CliTool` (model-only, no consumers).**
  Land `Pin<Subject>` in a new `extdeps.pin` module, with `extdeps.tools.pin` as its `CliTool` instantiation: exact version
  (`extdeps.version.VersionIdentity`, the exact brand — never
  `VersionConstraint`, which is a range) + expected `ContentHash` reusing
  #7388's identity, selection policy (`ExactPin | TrackedChannel`) and a
  refresh window. Both fields are required and non-optional, so a range-only or
  digest-less **`Pin`** is unwritable. The RED is at the ingest boundary:
  converting a legacy `CliTool` whose `min_version` is a `VersionConstraint`
  refuses, because a constraint is not an identity. A `TrackedChannel` pin
  refuses admission outright, since its currency cannot be established without
  a resolution date and day arithmetic (§4 above); the window is carried but
  not yet load-bearing.

  **Scope limit, stated precisely.** P1 does *not* modify `CliTool` and does
  *not* add a digest to `SourceGitHubRelease`. Both remain exactly as they are:
  `min_version: VersionConstraint?` still admits a range or nothing, and
  `SourceGitHubRelease` is still digestless. `Pin` therefore constrains
  only values built as `Pin` — it makes **nothing in the existing corpus
  unwritable yet**. This is the add-replacement half of add-replacement →
  migrate → delete, and calling it more than that would be the
  specification-without-execution trap §5 names.

  The migration is **P2/P4 work, gated on sourcing real digests.** A required
  `digest` on `SourceGitHubRelease` is cheap structurally — `websocat.dag:32`
  is its only construction site — but it must carry the *actual* upstream
  digest. Inventing one to make the field typecheck would be the fabricated
  plausible output §5 forbids, so the field lands when the digests are sourced
  from the upstream release authority, not before.
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

## 7. Discovered by execution: the `none` spelling blocks corpus ingest

Found while witnessing P1's `admit_legacy_cli_tool`, and recorded rather than
worked around. The `Absent` match arm does **not** match a field written with
the `none` literal — a `none`-valued optional evaluates to `Value::Null` and
the match fails with `non-exhaustive pattern match on: null` — while a field
written `Absent` matches normally.

This is a live instance of the documented `Value::Null` open thread ("the
overloaded `None`/`Absent`/miss sentinel"), surfaced by a new consumer. It is
load-bearing for this lane because the corpus overwhelmingly uses `none`:
roughly **568 `: none,` fields against 98 `: Absent,`** in `dag/`, and *every*
shipped `CliTool` row with no version (`jq`, `grep`, `sed`, `sha256sum`,
`sleep`) uses `none`.

Consequence, stated plainly: P1's ingest fn is correct on the model and its
witness is green, but that witness exercises the `Absent` spelling only — it
does **not** demonstrate coverage of the real roster. Ingesting the actual
`CliTool` rows is a **P2 blocker** gated on the `Value::Null` carrier split
(`docs/plans/value-null-split.md`) or an equivalent grounding of the two
spellings onto one representation. The limit is declared on the carrier
(`legacy_ingest_none_spelling_blocker_note`) so it is counted rather than
assumed discharged.

## 8. Non-goals

- Dictating a developer's interactive environment. Mechanical paths only.
- A content-addressed toolchain root shared across worktrees (ruled out above).
- Shipping a C toolchain (Tier 2 keeps `cc` a declared system requirement).
- Re-attempting isolated `RUSTUP_HOME` for CI floor gates before the srv3 flake
  is confirmed fixed; P3 is explicitly gated on that receipt.
