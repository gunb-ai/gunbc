# gunbc — Design

`README.md` and `CLAUDE.md` symlink here — this is the single source of truth. (v1 ships the `gunbc`
CLI and is v2's seed · v3 was removed, migrated into v1 · v2 is active.)

This document is reasoned **serially**: each section is a consequence of the one before it, or an
independent peer — never a restatement of it. The principles below apply recursively, including to
this document. It is a living draft rebuilt from first principles (the prior ~30k-line doc corpus was
bankrupted 2026-06-16; it remains in git history). Examples are verified against the live tree or a
git-history receipt; open threads are collected at the end.

---

## 1. The objective (the axiom)

To *solve a problem* is to find its **minimal, safe, efficient solution** — and those three are not
preferences we layer on; they fall out of the meaning of "problem" itself:

- **minimize cost** — input sustainability (cheap to run and to feed);
- **maximize safety** — harm reduction (it never silently does the wrong thing);
- **minimize complexity** — solution + maintenance sustainability (it stays cheap to change).

Underneath, all three are **time**: cost is time-to-run, complexity is time-to-change, and an unsafe
answer is time-to-recover — paid later, at interest. Time is the one quantity every agent intrinsically
values (**time is life**), which is what makes minimal/safe/efficient an intersubjectively grounded
objective (§4), not a house preference.

The aim, stated at its limit, is to **reduce intersubjectivity to physics** — physics being the
efficient, time-bound description of interactions that satisfies a goal (i.e. §1 itself, turned on
grounding). Every convention replaced by necessity is agreement that no longer has to be negotiated,
and the shared framework §4 points at is, at the floor, physics — the description nothing arbitrary
survives in. This is the deep reason §3 models the universal frameworks and real upstream rather than
re-coining them: a nickname is convention standing where physics was available.

The whole project optimizes these three jointly. Everything below is a mechanism for one or more of
them.

## 2. Minimize redundancy (the master move for cost and complexity)

Redundant work — **duplicated, unnecessary, or irrelevant** — loses on all three of §1's quantities at
once: it costs more to run, it widens the surface where harm hides, and it adds complexity to maintain.
So a perfectly DRY process is, *by the meaning of redundant*, the minimal/efficient one; minimizing
redundancy is therefore the master move for §1's cost and complexity axes (the safety axis is §5).
Through §1's time lens this is *why* DRY matters at all: redundant work **defers** cost into the
future — it shoves a problem onto a later fixer, or builds a process destined to be thrown out — so DRY
is the refusal to spend someone's future time to buy the author's present convenience; it values time
**holistically**, across every party and all of it, not just the here-and-now. Redundancy is removed
along exactly two directions — not separate "DRY rules," but one move seen two ways:

- **Horizontal — one concept, every scale and breadth.** Model a concept once; derive every use
  (model-local / derive-global). At the right layer there is nothing fundamentally different between
  scales, so the *same* concept spans nanosecond memoization and broad infra deployment — the
  Realization pattern (content-addressed pure-spec → host-effect; one kernel, N handlers).
  - *e.g.* `dsl/std/integer.dag`: `Int8`…`UInt128` are 10 `Compose<Int, MachineWidth<N>>` rows, one axis not 10 types. Realization spans resolve-cost (ns) → sccache → §10 OS provisioning on one content-hash. Cost of *not* doing it: eleven hand-rolled v1 `HashMap` caches, and v2 still recurs (`ParseTable`).
- **Deep — every concept decomposed to grounded atoms.** Nothing is opaque that isn't *genuinely*
  atomic. The move is `decompress → map → reduce`: reveal the structure the source names, **map** each
  part onto the concept that already exists (DFS the concept DAG first), **reduce** duplicates. A
  `String` leaf hiding named parts is anemic modeling.
  - *e.g.* `"LGA4926"` → `CpuSocket { package: LandGridArray, contact_count: 4926 }` (the number is a grounded `Int`); `Cost = Time|Space|Energy` → a record (every cost has all three); still-anemic today: `DramModuleCatalogRow.{ddr4_pc4_class, rank_label}` are bare `NonEmptyStr`.

The test that an edit actually *reduced* redundancy rather than moving it: **net concepts must not grow
by re-invention.** Decomposing a leaf by minting a fresh authority for a concept that already exists is
a failed decomposition.

## 3. Single authority (what keeps §2 from being undone)

Minimization holds only if each fact lives in exactly one place. The recurring violation is
**nicknaming — a second name for one concept** — which duplicates work at the meaning layer and, since
we generate from concepts, duplicates it again in everything derived (testgen, emit, lenses). A fork
always gets consolidated later, so it is a correctness concern, not a style one. We cannot enforce this
programmatically yet; until then it is diligence — faithfully model the accepted universal frameworks
(classical logic, set theory, algebra), and in `extdeps/` the real upstream spec (cite the source, keep
its real names, declare its version, model what the API actually returns), rather than re-coining them.
Corollaries: the layer DAG is
strict (`std ← extdeps ← compiler ← workflow`, imports point toward std); a fact's home is its *layer*,
not its file (paths are discriminators, not gospel); below-boundary representation is opaque (the
rename test).

- *e.g.* `CpuArchitecture` and `TargetArchitecture` are byte-identical enums (the latter's header denies the parallel it declares); `ModulePath` was a nickname for `QualifiedName` (renamed, `ModulePathSegment` deleted); one "vendor" concept forks by rigor — `CpuVendor` closed enum vs `GpuFacts.vendor` stringly. Counter-example done right: `std/cpu` owns the catalog *shape*, the vendor SKU rows live in `extdeps/cpu/ampere`; `compute_fabric` moved std→product as a domain model.

## 4. The closed, grounded substrate (what makes §2–§3 decidable)

You can unify and decompose *mechanically* only in a closed, grounded system — so the substrate is
built for it. A program is a dependency graph over two primitives (`Node` + `Edge`) and a closed
vocabulary (6 connectives + 5 behaviors); surface syntax is sugar that adds no power. Execution is
**bounded and forward** (cyclic relations via acyclic encodings, never cyclic values; recursion is
sugar over `Loop`), so decidability and termination *fall out* rather than being separately proved.
**Grounding is intersubjective** — point at a shared framework, not an internal taxonomy — and in a
closed system **a heuristic is never necessary**: the richer source always exists or can be written.
This is *why* the structure is acyclic — a DAG in the substrate, serial reasoning in this document
(recursively, per the preamble): intersubjective agreement holds only **across time** (§1), which
demands claims that stay stable under it, so each is written syllogistically — a consequence-chain you
can re-interrogate and that never wavers (algebra), never a cycle that could quietly redefine itself.

Because the substrate is closed and grounded, the wins of §2 fall out for free: operations come from
*inhabitance* (no per-type ops); and emission, ingestion, and coercion are **one** total decision
procedure run in different directions (the epistemic chain *is* the emission algorithm — N models, not
N×M adapters; every refusal a located, typed mismatch).

- *e.g.* `dsl/std/algebra.dag` derives `Int.add` from `Int` inhabiting a ring (ops aren't listed per type); idempotency dissolved from an `idempotent: Bool` flag into the `EffectShape` variant; termination is *checked, not discovered* — `DescentEvidence = Strict | NonIncreasing | DescentUnknown` inhabits a `BoundedLattice` with bottom = fail-closed.

## 5. Fail-closed (§1's safety axis — harm reduction)

Minimizing cost and complexity (§2–§4) is worthless if a wrong thing passes silently — this is §1's
safety axis made concrete. This code is digital: a wrong answer is a **loud error, never a warning** —
a bridge collapses, it does not warn. Every path succeeds fully or
fails with a typed, located diagnostic; no fabricated plausible output (a bounded "forever" ≠ an
"unknown" error). Relax toward application-layer leniency only under protest, and lean to infra so
others can build on your work. The deepest trap is **specification-without-execution**: a typecheck and
a `.contains()` grep are *not* consumers — "done" means a real consumer **green by execution** plus a
discriminating input that goes *red* when the behavior is wrong. (For the LLM agent: fluent,
type-checking, grep-passing output is precisely the artifact that looks finished without running. Treat
your own output as unverified until a consumer runs it green.)

- *e.g.* a compiler-sized `.dag` corpus typechecked and passed its grep claims, yet `emit` hung >600s on `fn add` → rebuilt `emit = serialize_target ∘ translate` (43 lines), proven by *running* `emit(add)` against the literal `fn add(x: i32, y: i32) -> i32 { x + y }`. Removing a scan-all-keys fail-open heuristic (`0331b526ee`) exposed 8 real inference deficits its fabrication had hidden; the parser's dummy `LitNull` nodes once fed inference bogus types.

## 6. How to work (given §1–§5 — these coexist)

- **Model:** DFS the concept DAG before inventing vocabulary; fact-bundle modeling (invent or reuse on
  proven coincidence, never bare-alias); a finished stage is one fold (any non-fold residue is either a
  named irreducible kernel or un-migrated modeling — there is no third); model just-in-time and let the
  mark on the carrier be the authority (no parallel-ledger docs); every scaffold lands with a named
  dissolution trigger.
- **Prioritize holistically, not by the bottleneck:** balance the quantitative and the qualitative —
  don't anchor on one KPI (you'll hit it at a cost) or on pure taste. Map the cause→effect across
  sections; a 5ms step doesn't get a pass for not being the 80s one (it might be a 5ns step).
  Root-cause to the language layer and fix related systems *together* — a local subsystem patch is the
  forked-logic trap.
- **Enforce with lenses,** not grep: a lens is a pure reader over the same `Node` tree, storing
  nothing, so a new analysis costs zero substrate edits. Beware the tier where the machinery exists but
  nothing gates on it — coverage by illusion.
- *e.g.* one catamorphism `fold_node` is reused by all 7 v2 stages; #4699 dissolved `06_translate` 4,912→3,973 lines (`_go` accumulators 35→0); a 6-line `merge_envs` root fix cut reconcile from 81% of the pipeline to 6% (~2× self-compile) — the symptom recurs wherever the root is unfixed (v2 still hand-rolls `ParseTable` because the Realization carrier is staged, not inhabited).

## 7. Self-hosting (the principles applied to the compiler itself)

The compiler is a pure transform and an ordinary substrate fact, analyzable by its own lenses. It is
written in itself, self-emits to a bit-identical fixed point (the `.dag` graph is the truth; Rust is
one realization, a seed that shrinks to zero), and its tests are data. Its ontology dissolves into
`std/` — no dual representation at the compiler/user boundary. This is the recursion: every principle
above governs the system that implements them, and this document.

---

## Recurring failure modes (instances of §3–§5, kept for pattern-matching)

hollow alias (minimality ≠ grounding) · state-space conflation (an `Option`/`None` meaning >2 things —
split into named variants) · cache impurity (key on declared-input content; byte-identical
cached-vs-cold is the purity oracle) · reflection evidence ≠ structural proof (prove a read axis by
execution, with a no-host-enumeration control) · coercion proven by normalized round-trip, not a golden
string · parallel-representation debt (an honestly-marked scaffold duplicating a canonical fact is
still a violation) · internal review finds missing tests, external review finds missing checks.

## Open threads

- can a lens mechanically diagnose the *leaf-side* of decomposition (§2)? (operator-parked)
- sweep deleted-doc references in `.dag` comments (~100 files, mostly `Practice 4 (docs/modeling-discipline.md)` ledger marks) — fold into the dep-graph reform, not a blind repoint.

## Building & checks

- `cargo test --workspace` · `cargo clippy --all-targets -- -D warnings` · `cargo fmt --all --check`
- one-time: `.githooks/install-hooks.sh` (pre-push runs `cargo fmt`)
- CI floor is one binary: `cargo run -p ci_claim_gate --release -- --source-root src/v2 --roster-from-discovery --scan-dir src/v2/test`
