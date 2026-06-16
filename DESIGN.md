# gunbc — Design (working TODO)

The old doc corpus (~94 files / ~30k lines) was bankrupted on 2026-06-16 — it's in git history if
needed. This is the **harvest agenda**: one-liner themes to discuss and turn into the real design,
from first principles. Not a finished document. Work the list; promote a line to prose only when
we've actually discussed it.

`README.md` and `CLAUDE.md` symlink to this file — it is the single source of truth. (v2 ships the
`gunbc` CLI and is v4's seed · v3 was removed, migrated into v2 · v4 is active.)

Legend: `[ ]` to discuss/harvest · `[~]` drafted, needs review · `[x]` settled

## Principles (the review spine)

**Why all of this (the objective function).** A theoretically perfect DRY process is, *by
definition*, the fastest, most efficient, and most reliable one — safety, cost, and speed are jointly
maximized for the domain's constraints. Redundant work is never wanted — that is what *redundant*
means — and "redundant" is broader than duplication: it is anything unwanted, whether **duplicated**
(a forked concept), **unnecessary** (dead / consumer-less code), or **irrelevant** (off-purpose
bloat). So the objective is to rip programming down to the bare minimum along two axes: **horizontal** —
unify a concept across all its breadth and scales (model-local / derive-global; one concept, every
scale) — and **deep** — decompose every concept to its bare atoms. Every principle below serves that
minimization.

- [~] **Loud errors, not warnings — lean toward infra.** This code is (mostly) digital: when
  something is wrong we want a *loud error*, never a warning. In digital logic there is no ambiguity
  about whether a thing works — a bridge doesn't issue a warning, it collapses; when you hit danger you
  alert your neighbors, you don't whisper. Infra has to be unambiguous so others can build on it. The
  rule relaxes on application layers — but only because user experience degrades under maximal
  strictness, and that relaxation is a *tradeoff made under protest*, never a default. Weigh it and
  **lean toward infra** so your work is something others can build on. (This is P3 Fail-Closed as a
  posture: typed located diagnostics, no fabricated output, no warning-as-escape-hatch; a bounded
  "forever" ≠ an "unknown" error.)

- [~] **DRY of concepts — no nicknaming (a correctness concern, not style).** A nickname is a *fork of
  a concept that still means the same underlying thing* — duplicate work at the meaning/communication
  layer. The codebase leans hard on DRY: we generate everything we can, so a forked concept forces us
  to duplicate — then quadruplicate — effort in everything derived from it (testgen, emit, lenses).
  Forking *always* gets consolidated later, so it is compounding debt — treat it as correctness, not
  taste. We can't enforce this programmatically yet (maybe LLMs help here); until then it is
  diligence: faithfully model the accepted, universal frameworks (classical logic, set theory,
  algebra) rather than re-coining them, and if you must introduce a name that relates to an existing
  concept, make the relationship explicit. (This is Decomposition's *reduce* step; reinforces P1
  grounding + P2 single-authority.)

- [~] **DRY of logic — consolidate at the root, then generate.** Duplication anywhere — frontend,
  compiler — is a "why?". Solve it at the root by consolidating onto one authority and generating the
  rest, never by forking. Forked logic looks harmless and exponentiates. (P2 single-authority; "don't
  hand-roll a derived operation".)

- [~] **Decomposition / the atom — `decompress → map → reduce`.** Nothing is opaque that isn't
  *genuinely* atomic. Keeping a rich concept as a bare `String` leaf is anemic, lazy modeling —
  anything can be modeled here, and the synergy (shared lenses, testgen, emit) only pays out when
  concepts unify on one model; forking quadruplicates the effort. *Decompress* the concept to its
  primitives, *map* each part onto the concept that already exists (M9 DFS), *reduce* duplicates and
  nicknames. Net concepts must not grow by re-invention. (drafted w/ operator 2026-06-16)

- [ ] P1 Modeling Faithfulness — grounding is intersubjective; faithfully model accepted universal frameworks; in a closed system heuristics are never necessary
- [ ] P2 Boundary Discipline — one fact, one home; layer DAG (std ← extdeps ← compiler ← workflow); cost-of-change → 1
- [ ] P4 Decidability — bounded forward execution; lowering is the receipt; checker, not discoverer
- [ ] P5 Progress-is-Dissolution — no bridges/deprecations as steady state; scaffolds need a live ratchet; debt-negative default
- [ ] the through-line: how much *stricter* than a normal compiler can we be? (grounded leaves + DRY concepts let us reject what they can't even express)
- [~] **Solve holistically at the root, not the bottleneck.** Don't anchor on one axis: optimizing a single metric/KPI hits the number at the cost of everything else here, and pure qualitative ("do I actually like this?") misses critical quantitative limits — balance all goals at once. And don't just chase the bottleneck: the 80s path tempts you, but the move is to map the cause→effect sequence *across* sections, see how they relate (caching, shared redundancy), and make each as fast and non-redundant as it can be — a 5ms step doesn't get a pass for not being the 80s one (it might be a 5ns step). Root-cause to the language/substrate layer and fix the related systems *together*; local subsystem patches are the forked-logic trap.
- [~] **DRY across scales — one concept, every scale.** This project is DRY on steroids: the *same* concept models a phenomenon at every scale, because at the right layer there is nothing fundamentally different between them. The caching/Realization concept that memoizes a nanosecond computation is the one that caches a build and reconciles a broad infra deployment — content-addressed pure-spec → host-effect, one kernel, N handlers (see Substrate → the Realization pattern). Model it once at the substrate; every scale and every consumer derives from it (model-local / derive-global).

## Worldview
- [ ] program-as-dependency-graph; the compiler is a non-executing causal engine
- [ ] correctness is structural, not behavioral (type/arity/unit/effect/complexity/ownership/idempotency + user dims)
- [ ] model-local / derive-global (N models, not N×M adapters)
- [ ] emission = ingestion = coercion; one total decision procedure; closed mismatch taxonomy
- [ ] Shape A (compiler emits languages/HDLs) vs Shape B (user .dag emits YAML/TF/SQL/SPICE/English) — never blur
- [ ] two groundings: deep static validation vs shallow target realization (must be semantically equivalent)

## Substrate
- [ ] two primitives Node + Edge; 6 connectives + 5 behaviors; the vocabulary closes here (surface = sugar)
- [ ] bounded forward execution (cyclic relations yes, cyclic values no; recursion is sugar over Loop)
- [ ] names are opaque; identity rides a binding-id channel, never structure-or-spelling
- [ ] single equality authority; files leave the pipeline early (content-hash IR)
- [ ] the Realization pattern (content-addressed pure-spec → host-effect; one kernel, N handlers)
- [ ] effects are intrinsic to signature shape (a separate effect taxonomy *is* the bug)

## Epistemic stacking (why grounding pays)
- [ ] operations fall out of inhabitance (Int inhabits OrderedRing → `add` is free)
- [ ] the epistemic chain *is* the emission algorithm; an emitter special-case = an ungrounded concept upstream
- [ ] no third option for a concept: genuine primitive OR unfinished composition (never "treat as opaque")

## How to model (day-to-day)
- [ ] DFS the concept DAG before inventing vocabulary (M9); canonical CS names, no nicknames
- [ ] fact-bundle modeling: invent-or-reuse, never bare-alias; coincidence = structural Node equality
- [ ] project, don't enumerate; don't hand-roll a derived operation; watch over-modeling (nominalization)
- [ ] a finished stage is one fold; non-fold residue = named kernel OR un-migrated modeling (no third)
- [ ] model just-in-time; the mark on the carrier is authority (no parallel-ledger docs)
- [ ] dissolution disposition: dissolve-now / terminal / gated (no fourth)
- [~] **file paths are discriminators, not gospel** — a path helps a human/LLM center on what a cluster of modeling is *for*; organize by them broadly, but don't anchor on the file. A concept's home is its *layer* (it belongs in `std/` or `extdeps/` itself); the file is a movable label.

## Enforcement
- [ ] lenses are the invariant primitive (not grep); pure readers that store nothing; new analysis = zero substrate edits
- [ ] correctness dims: declare lattice → compute at binding → carry → enforce (users declare their own)
- [ ] guarantee tiers; Tier 3 (machinery exists but nothing gates on it) is the trap
- [ ] opacity is single-authority's missing half (the rename test; the metamorphic representation-swap test)
- [ ] OPEN (operator-parked): can a lens mechanically diagnose the *leaf-side* of decomposition?

## Verification discipline
- [ ] tests are TestClaim data; a hand-written .rs test is an unexpressed language feature
- [ ] hermetic, behavior-driven, unit-first; mock a minimal Dag, don't compile end-to-end
- [ ] **HARVEST PROMINENTLY** — the specification-without-execution trap + E-10 (no code without a consumer)
- [ ] "done" = green *by execution* + a discriminating input that goes red when the behavior is wrong

## Self-hosting
- [ ] four facets: written-in-itself / self-emits to a bit-identical fixed point / tests-are-data / recursive-flex
- [ ] hand-Rust target = 0, monotonically; the compiler's ontology dissolves into std/
- [ ] fixed-point *contract* → DESIGN; *reached-or-not* status → ROADMAP

## Hard-won lessons to harvest (paid for once; don't relearn)
- [ ] hollow alias — minimality ≠ grounding; it passes every shape-checker
- [ ] parameterized-family blindness — a 15-variant identical enum passed 4 shape-checks + 2 reviews
- [ ] construct-discard-reconstruct — the cardinal anti-pattern AND the real perf cliff (68× from a 6-line fix)
- [ ] state-space conflation — Option/None standing for >2 meanings; split into named variants (illegal states unrepresentable)
- [ ] cache purity — key on declared-input content; byte-identical cached-vs-cold is the standing purity oracle
- [ ] coercion proven by normalized round-trip, not a golden string; ingest must not grow its own coercion arms
- [ ] reflection evidence ≠ structural proof — prove a read axis by execution with a no-host-enumeration control
- [ ] parallel-representation debt — an honestly-marked scaffold duplicating a canonical fact is still a violation
- [ ] internal review finds missing tests; external review finds missing checks — need both

## Housekeeping (this bankruptcy)
- [ ] README — lean rewrite (entry + nav)
- [ ] ROADMAP — lean pass; drop references to the deleted docs
- [ ] sweep dangling `docs/*.md` references left in .dag comments / CLAUDE.md / PR template
- [ ] `dsl/extdeps/extdeps.md` — substrate-adjacent doc, left in place for now; delete too?
- [ ] secondary cleanup (recon-mapped): `fixtures/v4-mvp1/add` + its v2 parity test, orphan `scripts/ci-merge/`, dead `tools/` (compile_host_runner, ci_timings_collector, ci_affected_components, layering_imports_scan)

## Building & checks (operational)

- `cargo test --workspace` · `cargo clippy --all-targets -- -D warnings` · `cargo fmt --all --check`
- one-time: `.githooks/install-hooks.sh` (pre-push runs `cargo fmt`)
- CI floor is one binary: `cargo run -p ci_claim_gate --release -- --source-root src/v4 --roster-from-discovery --scan-dir src/v4/test`
