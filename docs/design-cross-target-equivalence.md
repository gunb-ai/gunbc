# Cross-target equivalence design

**Status:** DESIGN LOCK for R2 Evaluator PR-D / R3 T-Verification-L5-Corpus. This document defines the equivalence relation that L5 consumes; it does not add runner arms, corpus files, target enumeration, or substrate variants.

**Parent authorities:**
- [`docs/r3-structure.md`](r3-structure.md) design challenge 3: L5 is algebraic equivalence over a curated corpus, not byte equality.
- [`docs/briefs/r2-pr-d-cross-target-equivalence-harness-primitives.md`](briefs/r2-pr-d-cross-target-equivalence-harness-primitives.md): R2 PR-D harness primitive scope and existing `TestClaim` hooks.
- [`docs/briefs/r3-verification-manager.md`](briefs/r3-verification-manager.md): R3 Verification Manager lanes for L4/L7 direct checks and L5 corpus.
- [`INVARIANTS.md`](../INVARIANTS.md) P1/P2/P5: substrate facts must have an authority, equality surfaces must be typed rather than string-shaped, and scaffolds need dissolution triggers.

## Decision

Cross-target equivalence compares **semantic observations** of the same `.dag` program across targets. It never compares emitted source bytes, host diagnostic text, target-specific pretty-printing, or formatting artifacts as the equality authority.

For L5, a target run contributes an observation only after the program has passed the target's declared compile/run boundary. The observation is then compared by algebraic equality:

- Successful pure computations compare the semantic result value.
- Failed computations compare typed failure category only when the claim explicitly has failure behavior in scope.
- Setup failures, target tool failures, timeouts, and unsupported feature reports are not equivalent program results; they fail closed as harness failures.

This is the only PR-D equality policy that R3 L5 corpus authoring may consume unless a later design explicitly amends it.

## Equality Domain

The primary equality domain is the runtime value model locked by PB-Runtime and R2 Evaluator PR-A:

- `LiteralValue` compares by literal kind and canonical literal payload.
- `RecordValue` compares by field name and recursively equal payload values. Field order is not an emitted-byte authority; any ordering used by a runner is a normalization step, not the semantic fact.
- `VariantValue` compares by tag declaration identity and recursively equal payload value.
- `NodeRef` compares by declared program identity, not by target-side pointer, address, or generated symbol spelling.
- `CardinalityValue` compares by the structural `LoopBound` value.

This document does not introduce a new `Value` shape. If future L5 observations need a value not expressible by the PR-A / PB-Runtime domain, the gap routes through `INVARIANTS.md` P1 before any new substrate fact, predicate variant, or runner-private carrier is accepted.

## Corpus Policy

The L5 corpus is curated, not "all possible programs" and not a byte-output golden-file suite. A corpus row is valid only when it carries all of the following information:

- A single source `.dag` program, with stable declaration identity for the entry point under test.
- The required target set. Full L5 closure requires all Shape A targets named by R3 for this lane; partial rows are tracked as gated scaffolds, not passing L5 evidence.
- The declared input sample or finite input family.
- The expected semantic observation or oracle authority.
- The effect class for the program: pure, controlled stdout, typed failure, or deferred effectful behavior.
- The numeric policy, including whether floating-point behavior is excluded or covered by an explicit future policy.
- A coverage reason tying the row to a language construct, runtime value shape, target realization edge, or previous L4 corpus row.

Programs enter L5 when their semantics are already grounded enough that a cross-target disagreement is meaningful. Programs are excluded or held as blocked when they depend on unspecified target behavior, unmodeled effects, unsupported target features, or a numeric policy this document marks deferred.

The R3 `T-Verification-L5-Corpus` lane consumes the L4 corpus as seed coverage. New L5 rows may expand that set, but each expansion must state the coverage reason; "one target happened to print this output" is not a valid corpus rationale.

## Oracle Policy

A valid oracle is one of:

- A hand-authored expected runtime `Value` for a small program whose semantics are direct from the `.dag` source.
- A `.dag` evaluator result once the evaluator body execution path is live and the claim is an L4/L5 runtime comparison.
- A previously accepted algebraic law witness or structural declaration that computes the same semantic value.
- A `DifferentialEquals` subject/oracle pair where both sides are declared producers of semantic observations, not emitted text.

An invalid oracle is:

- Raw emitted Rust, Python, Go, or generated file contents.
- Raw stdout bytes unless the claim explicitly says stdout is the program's semantic output channel and provides a parser/normalizer for that channel.
- One target chosen as the reference target for all others without a separate semantic authority.
- Diagnostic string matching, substring checks, or target-private error text.

When no valid oracle exists, the corpus row is not ready. The correct state is a tracked blocker, not a weak equality rule.

## Float Policy

Floating-point programs are **excluded from strict L5 equivalence by default**.

Exact equality is allowed only when the program's float surface is structurally grounded enough to state a deterministic finite representation and all participating targets are known to implement that representation. Tolerance-based equality is deferred: it requires a typed tolerance policy with a named authority, not an ad hoc epsilon in a runner or corpus row.

Fail-closed cases:

- NaN payloads, infinities, signed zero, platform math-library behavior, target rounding drift, and decimal formatting differences are not silently normalized.
- If a program reaches one of those surfaces before the typed policy exists, it is excluded from L5 strict-fire evidence.
- A target-specific tolerance or string-rendered float comparison is a P2 violation unless a later design routes it through a typed substrate carrier.

## Side-effect Policy

Initial L5 evidence is for pure programs and explicitly controlled process observations.

In scope:

- Exit status as a harness observation, separated from program semantic value. A nonzero target tool exit is not a program result unless the claim is explicitly about typed failure behavior.
- Stdout only when the claim declares stdout as the semantic output channel and provides a structural parse/normalization path to the equality domain.
- Stderr only for explicitly typed failure claims. Diagnostic text is not compared for successful semantic equivalence.

Deferred:

- Filesystem mutation and file contents.
- Time, randomness, environment variables, network access, process identity, concurrency scheduling, and nondeterministic IO.
- Target-global state and host resources not declared in the claim's requirements.

Deferred effects fail closed. A corpus program using them cannot satisfy `l5_cross_target_consistency` until a later substrate/runner design gives the effect a typed observation surface and isolation policy.

## TestClaim Integration

This design consumes the existing verification substrate:

- `TestClaim` remains the claim envelope.
- `DifferentialEquals` is the L4 / oracle-comparison primitive for a subject and oracle on a shared input.
- `ForAllTargets` is the L5 per-target scaffold for applying one claim across the declared target set once LanguageSpec, Shape A grounding, and corpus prerequisites are live.
- `Compiles` may remain only as a slice-0 structural hook. It is not semantic equivalence evidence.

No new `TestPredicate` variant is introduced by this design. If future implementation cannot express the required typed observation through the existing predicates and runner surfaces, the worker must stop and route the substrate fact through `INVARIANTS.md` P1 instead of adding a local enum case or string-coded side channel.

The current raw-command fields inside `ForAllTargets` and related host-execution predicates are scaffolded in `src/v3/std/verification.dag`. L5 consumers may use them only under the existing scaffold discipline. Their dissolution trigger remains typed target capability / observation facts, not permanent command-string equality.

## R3 Gates

Once this design lands, R3 may consume these decisions as follows:

- **T-Verification-L5-Corpus** may author corpus rows and worker plans against this equality/oracle/effect policy. It still gates strict execution on Lane 1 corpus availability plus Rust/Python/Go Shape A grounding and LanguageSpec support.
- **T-Verification-L4-L7-Direct** may use the same semantic equality relation for L4 emit/eval comparisons, while keeping L4 separate from L5. L4 proves a target matches `.dag` evaluation; L5 proves targets agree with one another over the curated corpus.
- **PR-E / Evaluator integration planning** may cite this document as the PR-D semantic lock. It may not treat this document as runner implementation or as authorization to add target enumeration, body evaluator logic, or new value variants.

The R2 PR-D primitive fixtures continue to provide the structural import surface. Strict L5 `ForAllTargets` receipts remain gated on the dependencies listed in the PR-D harness brief.

## Dissolution And Deferred Work

Current scaffolds and deferrals have these dissolution triggers:

- `Compiles`-only PR-D hooks dissolve when their named claims are strengthened to `DifferentialEquals` / `ForAllTargets` rows over real semantic observations.
- Raw command execution scaffolds dissolve when target capability and observation facts are typed in substrate/runner tables.
- Float exclusion dissolves only when a typed exact or tolerance policy lands with a P1-compliant authority.
- Deferred side effects dissolve only when each effect surface has typed observation, isolation, and equality rules.

Until those triggers fire, the correct behavior is to exclude the program from strict L5 evidence or record a blocked row. The harness must not manufacture equivalence through bytes, strings, target-private conventions, or fabricated target variants.
