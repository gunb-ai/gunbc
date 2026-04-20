# PB-1 — Data-driven bootstrap loader `(XXL)`

## Context

Today `Dag::new()` runs the full compiler pipeline (tokenize + parse + lower) on `include_str!`'d `.dag` source files every time a `Dag` is constructed. This means:

- **Stage0 requirement**: tokenize, parse, lower — all their Rust deps — must exist as hand-authored Rust at stage0, because `Dag::new()` calls them.
- **Runtime cost**: ~1-2s per `Dag::new()` even in tests that just want a primed DAG (partially mitigated by `cached_compile_to_dag` but only in test harness).
- **Pure-bootstrap gate**: the thesis claim "edit compiler = one `.dag` file, no hand-Rust pair" is structurally blocked until bootstrap loading doesn't need the full compile pipeline.

**v2's proven pattern** (per `docs/design-pure-bootstrap.md`): stage0 reads a pre-built primed DAG from generated Rust constructors. No runtime `.dag` parse at bootstrap. This lane replicates that shape for v3.

Four input authorities currently chained at bootstrap (per `src/v3/compiler/src/bootstrap.rs`):

1. `std_fixtures` — 7 `dsl/std/*.dag` files via `include_str!` (LOGIC_DAG, BIT_DAG, ALGEBRA_DAG, INTEGER_DAG, FLOAT_DAG, STRING_TYPE_DAG, TYPES_DAG)
2. `STAGED_FILES` — `src/v3/std/*.dag` (enumerated by `build.rs`)
3. `V3_SPECS` — `src/v3/spec/*.dag` (Rust/Go/Python target specs)
4. `COMPILER_FILES` — `src/v3/compiler/*.dag` minus `tokenize.dag` (pipeline.dag, operators.dag, regen.dag, runtime_mirrors.dag)

PB-1 must cover **all four**. The design doc spells out staged variants (PB-1a through PB-1d) so landing can incrementalize.

## Read first

- `docs/design-pure-bootstrap.md` — the canonical design for PB. Read §PB-1 in full; the "all four authorities" requirement is the single biggest framing fact.
- `src/v3/compiler/src/bootstrap.rs` — current bootstrap implementation. Note: `Dag::new()` calls `populate_primitive_cache`, `tokenize`, `parse`, `lower_into`. These are the runtime-parse paths PB-1 replaces.
- `src/v3/compiler/build.rs` — where `STAGED_FILES` / `V3_SPECS` / `COMPILER_FILES` / `extdeps_generated` / `gunbc_generated` are built at compile time. The build script is the natural home for PB-1's generator.
- `src/v3/compiler/src/dag/builder.rs` — the Dag builder API that TM-1 landed. `push_value`, `push_transform`, `push_bind`, `push_branch`, `push_loop`, `push_atom`, `push_conj`, `alloc_port_with_shape`. These are what the generated bootstrap loader calls.
- `src/v2/stage0/src/std_*.rs` — v2's reference implementation. These are hand-shaped generated Rust that build v2's primed DAG at bootstrap. Study the shape to understand the output contract.
- `src/v3/compiler/src/serialize.rs` / `serialize_generated.rs` — current serialization (used for fixed-point snapshots, not bootstrap). Useful for understanding the Dag's serializable shape.
- `src/v3/compiler/src/bin/regen_v3.rs` — the existing regen binary pattern; PB-1's bootstrap generator follows this template.

## Work

Staged sub-lanes so an XXL scope stays dispatchable. Worker proposes the final split in first PR's body; example structure:

**PB-1-a — `std_fixtures` (7 `dsl/std/*.dag` files) as generated constructors (~1 PR, M).**

- Build-script-time: compile the 7 std fixtures (reuse the existing v3 compiler) and serialize the resulting Dag additions to a generated Rust constructor module (`bootstrap_std_generated.rs`).
- Runtime: replace the 7 `include_str!` constants + their tokenize/parse/lower loop with a single `include!("bootstrap_std_generated.rs")` + call to the generated `push_*` constructor.
- No change to `STAGED_FILES` / `V3_SPECS` / `COMPILER_FILES` in this sub-lane — they still use the runtime-parse path.

This is the proof-of-concept: one authority migrated, bootstrap still works, `Dag::new()` gets measurably faster for the std portion.

**PB-1-b — `STAGED_FILES` (`src/v3/std/*.dag`) as generated constructors (M).**

Same pattern extended to the v3 staged std files. At the end: `src/v3/std/*.dag` no longer parsed at runtime; their Dag contribution is generated Rust.

**PB-1-c — `V3_SPECS` (`src/v3/spec/*.dag`) as generated constructors (M).**

Rust/Go/Python target specs. At the end: target realizations load from generated constructors, not runtime parse.

**PB-1-d — `COMPILER_FILES` (`src/v3/compiler/*.dag` minus `tokenize.dag`) as generated constructors (M).**

The meta-compiler-over-itself pieces: pipeline.dag, operators.dag, regen.dag, runtime_mirrors.dag.

**PB-1-e — Runtime-path retirement + measurement (S-M).**

Once PB-1-a through PB-1-d land:
- Remove the runtime `tokenize + parse + lower` loop from `Dag::new()` — it's no longer needed.
- Bootstrap becomes pure constructor calls.
- Measure: `Dag::new()` time drops from ~1s+ to <100ms (goal).
- Update `docs/design-pure-bootstrap.md` §PB-1 from planned to landed.

## Acceptance

- Each sub-lane ships its own PR; by close of PB-1-d: zero runtime tokenize/parse/lower calls at `Dag::new()` for std/staged/specs/compiler authorities
- `bootstrap.rs` collapses from ~470 LOC to ~100 LOC or less (whatever residual orchestration remains: populating emit_anchors from generated cache, final diagnostic flush, etc.)
- Four new generated modules exist (or one unified one — worker's call): `bootstrap_std_generated.rs` etc.
- Build script at `src/v3/compiler/build.rs` runs the compile+serialize step at build time, emits the generated constructor modules into `OUT_DIR`
- `Dag::new()` measured 5x faster (or better — target depends on actual cold cost)
- `cargo test -p v3-compiler` passes (bootstrap still primes the DAG correctly)
- DB-8 `self_host_fixed_point` still converges bit-identically (the generated bootstrap must produce the same Dag as the runtime-parse path — this is the acid test)
- SG-0 census updated: any retired handwritten bootstrap scaffolding off the list; any new generated constructor files on the generated partition
- ROADMAP PB-1 status flipped from 🟡 to ✅

## STOP-AND-ESCALATE

- **If `Dag::new()` post-PB-1 isn't measurably faster** — STOP. The whole point is removing runtime compile cost. If the generated constructors add their own overhead that offsets the savings, we've missed something. Name the overhead source.
- **If the DB-8 fixed-point drifts** — STOP, immediately. The generated bootstrap must produce bit-identical output to the runtime path. Drift is either a generator bug, a sort-order issue, or a substrate-representation issue — all must be root-caused before merge.
- **If a specific authority's `.dag` files contain constructs the current compiler can't fully emit** — STOP. That's a gap in the compiler's self-emission capability; the authority might be using parser constructs the emitter doesn't handle, or vice versa. Surface the gap; may block this sub-lane behind a Lane 1e extension.
- **If the generated constructor modules blow up build time** (e.g., rustc OOMs on a 50K-LOC generated file) — STOP. Propose per-authority split, lazy generation, or a different serialization format. Don't ship a build that rustc can't digest.
- **If bootstrap ordering reveals a circular dep** (e.g., generating `std_generated.rs` requires `spec_generated.rs`, but `spec_generated.rs` needs `std_generated.rs` to be emitted first) — STOP. Propose the sequencing / bootstrap-split fix.

## Non-goals

- **Not deleting `tokenize.rs` / `parse.rs` / `lower.rs` / `infer.rs` / `emit.rs`** — PB-1 removes runtime-at-bootstrap *use* of them. Their dissolution as files is separate work (SG-2b, SG-3b proper, SG-4, Lane 1e).
- **Not a format change for the `Dag` at runtime** — generated constructors produce the same `Dag` structure the runtime parse produces.
- **Not a binary blob format** — emitted Rust source (matches v2's proven pattern). The design doc explicitly notes blob was considered and rejected.
- **Not touching `.dag` user-facing syntax** — this is a stage0 runtime change only.
- **Not changing the `Dag` builder API** — TM-1's builders are the surface; PB-1 uses them as-is.

## Size

XXL. 5 sub-lanes. PB-1-a is the proof-of-concept (also the riskiest — establishes the pattern). PB-1-b/c/d are mechanical extensions. PB-1-e is the retirement + measurement. Multi-week scope.

Expected LOC delta at close:
- `bootstrap.rs`: -400 to -470 LOC (dissolved to a shim)
- Generated constructors: +5K to +15K LOC (this is fine — generated code, not census-counted)
- Net hand-Rust census: -1 to -5 files if any of the tokenize/parse/lower helpers become redundant at bootstrap (unlikely in PB-1 scope)

## Dispatch note

Director reviews each sub-lane PR. PB-1-a is the heaviest-reviewed (pattern-setting). DB-8 fixed-point regression is the no-compromise gate — STOP immediately on drift. Worker should be comfortable with build scripts, serialization, and the Dag builder API.
