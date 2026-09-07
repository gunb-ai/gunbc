# Plan — compile-clean gate force-check (ROADMAP §1 floor-coverage / §0 inference fail-open)

**Status:** historical diagnosis (execution-proven) + live **(A) partition design**. The 10-name / 102-site blast radius is a 2026-06-21 capture, not a live census. **DESIGN §5 (fail-closed) is authority.** Sibling of [fail-closed-lockdown.md](fail-closed-lockdown.md) §3.

**Re-verified against the live tree 2026-07-31.** `v1.compiler.infer_method` `builtin_function_registry` remains a flat global name→type map; `std.encoding` `utf8_decode_bytes` is now grounded and its old registry row is gone. Symbols, not line positions, are receipts.

## 0. Verdict — the brief's mechanism is wrong; the leak is a global seed allowlist

At the 2026-06-21 capture, the compile-clean gate (`dag/gunbc/instruments/dag_compile_clean_gate.dag` → `gunbc compile --target rust` over `dag/` with `src/v2` as the import pool) was **fail-open**: `gunbc compile` on `main` returned 0 diagnostics / EXIT 0 even though `extdeps.cloud.gcp.secret_manager` `utf8_secret_from_access_payload` called `utf8_decode_bytes`, which was then **defined nowhere in `dag/` or `src/v2`**. The current tree now defines `std.encoding` `utf8_decode_bytes`; this paragraph is a historical execution receipt, not a claim that the literal leak remains live.

The brief framed this as "unreached fn bodies escape typecheck." **That was false** — bodies were always visited. At the captured run, the execution-proven mechanism (§1 below) was two independent fail-open holes:

1. **Registry leak at capture** — `utf8_decode_bytes` resolved because it was a hardcoded entry in the global `builtin_function_registry()`, then an **explicitly-marked BRIDGE scaffold**. The registry was *not scoped to the tree being compiled*, so v1-seed runtime intrinsics leaked into the dag-substrate compile.
2. **Return-type fail-open** — a function whose body's inferred type ≠ its declared return type is not flagged (`#5293` closed only the record-field hole, not return types). Independent of the gate; a member of ROADMAP §0's "inference fail-open (return-type after #5293)".

## 1. Execution-proven mechanism (receipts)

`gunbc` built from `src/v1/stage0`; each probe is a single-file `--source-root` compile.

| Probe | Input | Result | Reading |
| --- | --- | --- | --- |
| undefined **variable**, uncalled fn | `fn f()->Int{ xyz }` | **1 error** "undefined variable 'xyz'" | bodies *are* visited; "unreached" hypothesis is false |
| call to name **not** in registry | `fn f()->Int{ totally_undefined_fn_xyz(q:3) }` | **1 error** "function '…' not found in scope" | unregistered call names already fail-closed |
| call to **registered** seed builtin | `fn f()->Int{ utf8_decode_bytes(payload:3) }` | **0 diagnostics** | registry absorbs the name → `string_type`, no def required, no arg check |
| plain return-type mismatch | `fn f()->Int{ "a string" }` | **0 diagnostics** | declared return type unenforced |

**The real on-main witness** — `extdeps.cloud.gcp.secret_manager` `utf8_secret_from_access_payload`:

```
fn utf8_secret_from_access_payload(payload: Bytes) -> Secret {
  utf8_decode_bytes(payload: payload) as Secret
}
```

In the captured run, `utf8_decode_bytes` resolved via the registry; the `as Secret` cast satisfied the return type. So the **literal witness was hole #1 alone** (return-type enforcement would not have caught it because of the cast).

## 2. The seam — `builtin_function_registry`

At capture, `v1.compiler.infer_method` `builtin_function_registry` held 76 names → return-type `Node`s, including a `utf8_decode_bytes → string_type` row. That row is absent now, but the live registry remains a flat global map. `std.encoding` `utf8_decode_bytes_host_realization_marker` and `std.bytes` `bytes_seam_host_realization_marker` exist, but both carry a stale `DeclarationRef` to `std.bytes` `builtin_function_registry`, where no such declaration exists. They are unresolved scaffold debt, not verified bindings to the live v1 registry.

The flat live registry remains a known §3 fork (duplicate authority); the two broken marker references are a second §3 gap that must repair or dissolve with it. Two registry properties make the resolution seam fail-open:

- **Global, not tree-scoped.** The same registry is consulted whether the entry root is `src/v1` (the seed, which legitimately needs seed-only intrinsics such as `scan_while`) or `dag/` (the substrate, which must not depend on seed intrinsics). A name in the registry resolves in *any* tree.
- **Resolution without definition or arg-check.** A registry hit yields a fabricated return type with no parameter typing and no requirement that a definition exist in the compiled closure — DESIGN §5's "fabricated plausible output" anti-pattern, at the call-head resolution seam.

(`v1.compiler.infer_method` `resolve_builtin_call_type`'s `Absent => unit_type` is *not* a live leak for call syntax — unregistered names are caught upstream as "function not found in scope". It is a latent fail-open kept for non-call uses; out of scope here, noted for the audit.)

## 3. Construction-correct direction (DESIGN §5/§3)

The gate's claim is "the dag substrate is well-typed **and self-contained**." Made *unwritable* (§5 construction, not a post-hoc lens): **the set of resolvable builtins must be derived from the tree being compiled, not a global seed allowlist.** This is exactly the registry's own stated deletion point — builtins become real `.dag` definitions resolved through normal import/definition resolution, the global registry is deleted, and a substrate that neither defines nor imports a live seed-only name such as `scan_while` fails closed on it ("function not found in scope", identical to probe 2). At that point the leak is dead by construction, not by a roster.

Full dissolution of the flat registry into `.dag` definitions is a large migration, out of this node's scope. The bounded options below are stepping stones; (A) closes the leak class *by construction*.

## 4. Scope decision (parent-confirmed) — (A), as design + measure only

Parent `quick-ant-298` confirmed: this node ships **(A)'s design + the blast-radius measurement**, and **lands nothing enforcing**. (B) and (C) are split out:

- **(A) Tree-scoped builtin availability / registry partition** — the live direction. Split the registry into *substrate-available* builtins (real `.dag`/std defs, or the sanctioned primitive surface) and *seed-only kernel* intrinsics; admit the seed-only set only when the entry root is the v1 seed itself. A dag-substrate compile then fails closed on a seed-only name. The enforcing flip is **load-bearing** (inference scope) + **changes what compiles** → operator-gated; it must also repair or dissolve the stale marker `DeclarationRef`s rather than treating them as authority.
- **(B) Return-type enforcement** — separate. It did not close the historical literal witness because the `as Secret` cast satisfied the return type; do not bundle it with registry partitioning. **PARTIALLY LANDED** as the P0 live-system lane's declared-type conformance wall (`v1.compiler.infer` `declared_type_conformance_note`): the §1 probe that returned 0 diagnostics for `fn f()->Int{ "a string" }` is now RED, and `data d: Int = "a string"` with it, witnessed by `declared_type_conformance_witness`. Scope is deliberately narrow and the narrowing is the finding: the wall judges only where BOTH sides are ground kernel scalars, because running the general relation over the corpus found four classes of CORRECT code that `node_type_compatible` calls a mismatch (optionality in two representations, unpeeled brand aliases, anonymous record literals, and cardinality absent from the resolved type node). Those four are the promotion triggers, carried on the note; the wall does not chase them with exemptions.
- **(C) Grounding `utf8_decode_bytes` as a real std fn — complete.** `std.encoding` now declares `utf8_decode_bytes`, and the historical registry row is gone. Its host realization still carries `utf8_decode_bytes_host_realization_marker`, which dissolves with the registry fork.

### HAND-RUST GATE receipt — CompilerDiagnostic seed projection

**Explicit deferral. ROADMAP row: hand-MAINTAINED Rust to zero at v2 self-host.** THE PARAGRAPHS THAT FOLLOW ARE RENDERED from the typed rows in `gunbc.compiler_diagnostic_seed_projection`, which is the single authority for this receipt. Those rows replaced a `String` data row in `v1.compiler.core` that seven lanes had written into (review 62020; DESIGN §4c — a receipt, a deferral schedule and a count are not program data). This document restated that row in four hand-authored paragraphs until the migration, which is the §3 fork this section had already recorded itself committing once (review 45565 quoted figures back from here after they had been re-derived on the carrier). Editing the rendered text without editing the rows is reverted on the next heal.

**Seed disposition — `v1_compiler.cli_run.compile_clean compile_clean_diagnostic_histogram_key` is MODIFIED, never added, and originates no capability.** Extending `v1.compiler.core CompilerDiagnostic` forces the arms; without them the seed does not compile, so they are the mechanical consequence of the `.dag` change rather than host capability chosen in Rust. BOUNDARY: src/v1/stage0/src/cli_run/compile_clean.rs compile_clean_diagnostic_histogram_key carries BOTH total matches over CompilerDiagnostic -- the class match and the name match -- so extending the coproduct forces two arms in ONE hand-Rust item and adds none. The item's own signature, callers and behaviour are untouched by a coproduct extension; what changes is arm count. Severity and gate disposition are NOT decided here: v1.compiler.core diagnostic_disposition is the single total authority for both and is exhaustive by construction, so a variant with no stated disposition does not compile.

**Standing obligation.** Add one arm to each total match in compile_clean_diagnostic_histogram_key. Not enforced by a gate and not needing one: the seed does not compile without them, so the class is structurally impossible rather than validated, and it disappears with the seed.

**Discharged by a climb**, authority `v1.compiler.core diagnostic_disposition`. The obligation was: Add a matching entry to the compile-clean ADVISORY ALLOWLIST for a non-blocking variant, or the frontier it represents is invisible to the gate meant to bound it. This obligation existed because compile_clean_diagnostic_is_advisory was a CLOSED ALLOWLIST rather than the complement of is_hard, so a variant in neither predicate was rendered to the terminal while the gate counted zero of it -- a third Boolean state both aggregates hid. WHAT DISCHARGED IT: The classifier now takes ONE complete population -- the diagnostics vector compile-clean consumes -- and projects it into an exhaustive binary partition: advisory is the exact complement of compile_clean_diagnostic_is_hard, so no diagnostic can enter neither bucket and no separately classified population can drift. Adding a variant no longer requires an allowlist edit that can be forgotten, so the obligation has no subject left. This is a climb that deletes the invisible state, not a lowered rung: blocking classification is unchanged and every non-hard diagnostic becomes counted. The remaining accepting direct-call boundary is declared independently by gunbc.rung_drop direct_call_unbound_carried_generic_compat, with its own population and restoration trigger.

**Discharged by deletion** of `v1_compiler.cli_run selection_control_input_sources`. The obligation was: Carry cli_run selection_control_input_sources in the gate's list of explicit deferrals owing a dissolution schedule. WHAT DISCHARGED IT: Deleted with affected-set selection on 2026-08-15, so the deferral was discharged by deletion rather than by a schedule. It stayed cited for weeks afterwards, naming nothing -- the DESIGN section 3 stale-citation class, inside a receipt whose own argument was that an unverifiable citation rots silently.

**The transcribed figures are WITHDRAWN and are not restated here.** Every figure is unreachable rather than merely stale. Four were measured against origin/main, a MOVING ref, so the command reports a different number without anyone touching the change it certifies -- the fourth lane measured that drift happening, 2 added / 0 removed becoming 6 added / 64 removed within hours, the 64 removals being main's work. The three measured against a named merge-base name commits on squash-merged branches that no longer exist. Two more name cli_run.rs, which stopped carrying the subject at the sixth lane's file split. The one fact they all asserted -- that the hand-Rust carrier census was FLAT -- is preserved as the NoNewCapability disposition above, which is a claim about the change rather than a number about a tree. WHAT WAS WITHDRAWN: Seven lanes' hand-Rust census figures for the CompilerDiagnostic seed projection: per-lane fn counts over the subject file at base and head (747, 884, 890, 895, 896, 38), per-lane numstat added/removed (23, 4, 8, 2, 2, 4), and per-lane counts of added lines declaring a fn (0 in every lane). THE INSTRUMENT THAT RE-DERIVES IT: `gunbc.instruments.rust_item_census_instrument census`, run rather than quoted.

Dissolution is INHERITED and not minted: the arms cannot live anywhere but the seed's projection of the coproduct, and they disappear exactly when the seed does, at the ROADMAP row `v1-zero-hand-maintained-rust`. The gate's other explicit deferrals — the emit-surface retirement rows — are DECISION SURFACES that could live in `.dag` and are deferred for a stated reason, so they do owe a schedule of their own; an exhaustiveness arm owes none, and that distinction is the receipt.

`compiler_tests.rs` and `v1_std_core.rs` are GENERATED, not hand-written, so their growth is authored in `.dag` and is not hand-Rust surface. That classification is DERIVED by `gunbc.stage0_rust_source_lifecycle_scaffold` `derived_generated_stage0_repo_paths` and `gunbc.generated_artifact` rather than asserted here, and the receipt's dispositions name no generated file.

## 5. (A) partition design

**Two facts must be separated** at the call-head builtin-resolution seam:

1. *Is this name a host intrinsic at all?* (registry membership — today's only question).
2. *Is this name in-scope for the tree being compiled?* (new: substrate-available vs seed-only).

**Construction shape (the registry's dissolve-on, staged):** each builtin row carries an **availability tag** — `SubstrateAvailable` vs `SeedOnly` — instead of a flat name→type map. The call-head resolver admits a `SeedOnly` row **only when the compile's entry root is the v1 seed**; for a substrate entry root (dag/ + v2 pool) a `SeedOnly` hit is treated as *not a builtin* → falls through to the existing fail-closed "function not found in scope" (probe 2). `SubstrateAvailable` rows are the sanctioned primitive surface and, per the dissolve-on, migrate to real `std` `.dag` defs over time; once a name has a real def it leaves the registry entirely and resolves through normal func-env lookup.

- *Entry-root signal*: the existing entry-vs-pool distinction (`v1.compiler.emit_rust` `emit_compile_match_arm`: "entry modules = all .dag in the FIRST source root; additional roots are dependency pools") already separates the compiled tree from its pools. The infer scope must carry one bit — "entry root is the v1 seed" — derived from whether `src/v1` is the primary `--source-root`. (Threading this into `InferScope` is the load-bearing part; out of scope for this node — captured here for the enforce PR.)
- *Why a tag, not a second map*: a second allowlist map would be a new parallel authority (§3). One row per builtin with an availability field keeps single authority and reads as construction, not a lens.
- *Non-enforcing intermediate (this node)*: the empty-registry measurement in §6 already enumerates the exact leak set without any code change shipping. The enforce PR turns each `SeedOnly` substrate hit into the fail-closed path behind operator sign-off.

## 6. Blast radius (MEASURE-FIRST) — execution-proven, 10 names / 102 sites

**Method (DESIGN §5, real consumer green-by-execution, not grep):** in a throwaway worktree off `origin/main`, `builtin_function_registry()` was replaced with an empty map (MEASURE-ONLY, uncommitted), `gunbc` rebuilt, and the **real gate compile** run — `gunbc compile --source-root dag --source-root src/v2 --dependency-pool-index primary-precedence --target rust`. Every registry name a substrate module **free-calls without a real `.dag` def** then surfaces as a "function 'X' not found in scope" diagnostic. (Grep over-counted ~50 — most registry names are invoked as *methods* and resolve via the structural method path, not the free-call registry.)

Result: **102 diagnostics, all "not found in scope", 10 distinct names** (all in `dag/`; none in `src/v2`):

| builtin | sites | leaf area | class |
| --- | --: | --- | --- |
| `string_contains` | 51 | dag/extdeps/formats, dag/test/claim | general string primitive |
| `to_string` | 25 | broad (examples, extdeps/*, product, tools) | general primitive |
| `concat` | 11 | extdeps/git, languages/go, version, product, **std** | general primitive |
| `filesystem_read` | 5 | dag/test/claim | **lens host-reflection intrinsic** (known §3 `Filesystem.Read` fork) |
| `set_contains` | 3 | dag/std | set primitive |
| `set_insert` | 2 | dag/std | set primitive |
| `count` | 2 | dag/gunbc/tools | general primitive |
| `utf8_decode_bytes` | 1 | **dag/extdeps/cloud/gcp** | **historical brief witness — domain leak later grounded under completed (C)** |
| `hash_combine` | 1 | dag/std | hashing primitive |
| `atom_identity_hash` | 1 | dag/std | hashing primitive |

**Reading of the captured number:** on 2026-06-21, 8 of the 10 names (96 sites) were general-purpose primitive calls (`string_contains`, `to_string`, `concat`, `count`, `set_contains`, `set_insert`, `hash_combine`, `atom_identity_hash`), `filesystem_read` (5) was the known lens-reflection fork, and `utf8_decode_bytes` (1) was the sole domain leak. Completed (C) has since grounded `std.encoding` `utf8_decode_bytes` and removed its registry row, so this capture does **not** identify a live domain-code failure or establish today's rollout size. Before enforcing (A), re-run the real compile measurement against the current registry, classify each remaining live row as `SubstrateAvailable` or `SeedOnly`, then prove the partition with the `scan_while` red in §7.

## 7. Discriminating witness (must go RED when the behavior is wrong)

The enforce PR's receipt is the same shape as §6's method: a planted substrate module that free-calls a live seed-only builtin such as `scan_while` must make `gunbc compile` over the dag pools **fail**; the identical call compiled under the `src/v1` seed must **pass**. Green-by-execution against the real `gunbc`, not a typecheck/grep. (DESIGN §5: spec-without-execution is not done.)

## Dissolution trigger (DESIGN §6)

Delete this doc when (A) lands as construction: the builtin registry carries a `SubstrateAvailable`/`SeedOnly` availability tag, a dag-substrate compile fails closed on a live seed-only name such as `scan_while` while the v1 seed admits it, and the stale `DeclarationRef`s in `utf8_decode_bytes_host_realization_marker` / `bytes_seam_host_realization_marker` are repaired onto a live authority or dissolve — at which point the leak is dead by construction (DESIGN §5).
