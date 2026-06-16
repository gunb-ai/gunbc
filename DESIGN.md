# gunbc — Design (working TODO)

The old doc corpus (~94 files / ~30k lines) was bankrupted on 2026-06-16 — it's in git history if
needed. This is the **harvest agenda**: one-liner themes to discuss and turn into the real design,
from first principles. Not a finished document. Work the list; promote a line to prose only when
we've actually discussed it.

Legend: `[ ]` to discuss/harvest · `[~]` drafted, needs review · `[x]` settled

## Principles (the review spine)
- [~] **Decomposition** — `decompress → map → reduce`; nothing opaque that isn't *genuinely* atomic; net concepts must not grow by re-invention (drafted w/ operator 2026-06-16)
- [ ] P1 Modeling Faithfulness — grounding is intersubjective; in a closed system heuristics are never necessary
- [ ] P2 Boundary Discipline — one fact, one home; layer DAG (std ← extdeps ← compiler ← workflow); cost-of-change → 1
- [ ] P3 Fail-Closed — typed, located diagnostics; no fabricated output; "forever" (bounded) ≠ "unknown" (error)
- [ ] P4 Decidability — bounded forward execution; lowering is the receipt; checker, not discoverer
- [ ] P5 Progress-is-Dissolution — no bridges/deprecations as steady state; scaffolds need a live ratchet; debt-negative default
- [ ] the through-line: how much *stricter* than a normal compiler can we be? (grounded leaves let us reject what they can't even express)

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
