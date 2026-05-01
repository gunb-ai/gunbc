# PR-D Companion Audit — Cross-Target Equivalence Observation Policy

**Status:** AUDIT — docs-only companion to Worker A's primary
`docs/design-cross-target-equivalence.md` design draft (in flight; this
audit is **input** to that doc, not a parallel design). This brief
spells out **why** algebraic equality across emit targets needs a typed
observation carrier, what's already in place, and how PR-D's wording
should phrase value-domain observation **without taking substrate
ownership**.

**Parent / consumer:** Worker A's `docs/design-cross-target-equivalence.md`.
**Adjacent authorities:**
[`docs/briefs/r2-pr-d-cross-target-equivalence-harness-primitives.md`](r2-pr-d-cross-target-equivalence-harness-primitives.md)
(slice 0 + 1 landed),
[`docs/briefs/r2-pr-b-2-runner-extension-bundle.md`](r2-pr-b-2-runner-extension-bundle.md)
(W1 / W3 routing),
[`docs/briefs/r2-evaluator-test-runner-authority-ratchet.md`](r2-evaluator-test-runner-authority-ratchet.md).
**Substrate references (read-only):**
[`src/v3/std/verification.dag`](../../src/v3/std/verification.dag),
`TestPredicate` / `ForAllTargets` / `DifferentialEquals` /
`ProgramOutputBind`.

This brief does **not** propose a substrate carrier. The
`ProgramOutputObservation` shape question has been routed to the
Substrate Manager via INVARIANTS §P1 per the bundle brief; that work is
out of this audit's hands.

## 1. Why algebraic equality needs a typed observation carrier

L5 verification is **algebraic equivalence over a curated corpus** — the
same `.dag` program emitted to Rust / Python / Go must produce the same
*runtime* answer. PR-D's job is to put primitives in place so R3 can
later assert that. The fundamental question PR-D's design must answer
is: **what does "the same answer" mean when the answer crosses an
emit-target boundary?**

Three moves are categorically wrong, and naming them up front prevents
PR-D from accidentally adopting any of them:

1. **Byte-equality of emitted source.** Trivially fails on Rust vs
   Python vs Go because the surfaces are different languages. Even
   within one target, formatter / reordering noise breaks byte
   equality. (`r2-pr-d-cross-target-equivalence-harness-primitives.md`
   §Scope item 3 already calls this out: "L5 compares computational
   results across targets, not byte identity.")
2. **Exit-code equality only.** Today's `ForAllTargets`
   (`verification.dag:160`) carries `{ command, args, expect_exit_code }`
   — the entire observation surface is the process exit code. That
   is enough to assert *compiles + does not crash*; it is **not** enough
   to assert "same value computed". A program that returns `42` vs `43`
   passes by exit code; one that prints different structured outputs
   passes by exit code; the harness silently green-lights divergence.
   Bundle brief W3 §"Structural observation authority" makes this
   explicit.
3. **Convention-only stdout parsing.** Regex over stdout, ad-hoc
   `^-?\d+$` matching, environment-variable signaling. The bundle
   brief's "Runner authority discipline" §explicitly **forbids** this:
   "if a fact is observed, the substrate must name it." A
   convention-only observation creates a Rust-side parallel
   test-predicate authority that the ratchet brief
   (`r2-evaluator-test-runner-authority-ratchet.md`) is meant to
   ratchet down, not grow.

Therefore the algebraic-equality predicate must compare values of a
**typed comparable** that the substrate names. Without that carrier,
PR-D cannot safely strengthen `DifferentialEquals` or `ForAllTargets`
beyond their current cost-only / exit-code-only forms.

## 2. What's already typed: the `CostLookup` precedent

The runner already operates one well-formed differential observation:
the Lane-E cost-lineage path in
`src/v3/compiler/src/test_runner.rs:201-212`
(`eval_lane_e_differential_cost_lineage`). It works because:

- Both sides (`v3_program_cost` host fold, `v2_oracle_cost` emit-side
  `cost_of`) compute a `CostLookup` value at the same `PortId`.
- `CostLookup` is `Hit(n) | Miss` — a typed comparable enum.
- Equality is structural enum equality, not string parsing.
- Single comparable type → single equality semantics → no per-side
  normalization drift.

This is the shape PR-D's value-domain observation should aim for, in
miniature. The pattern PR-D should generalize:

> *Both sides compute a value of the same typed comparable carrier, and
> the predicate asserts structural equality of that carrier.*

The new carrier (call it `ProgramOutputObservation`, P1 routing)
generalizes `CostLookup`'s typed-comparable role to cover the
runtime-value domain instead of just the cost domain.

## 3. What's missing: a generalized `ProgramOutputObservation`

`CostLookup` is only good for cost lineages. For value-domain
comparisons across targets, the observation must carry actual
inhabitants of the runtime `Value` shape (`src/v3/std/runtime.dag`):
`LiteralValue(LiteralBits)`, `RecordValue(List<NamedField>)`,
`VariantValue { tag, payload }`, `NodeRef(NodeId)`, `CardinalityValue(LoopBound)`.

A candidate carrier shape (Substrate Manager owns; PR-D consumes):

```
type ProgramOutputObservation
  = ExitCodeOnly { expect_exit_code: Int }
  | StructuredValue { channel: ObservationChannel, observed: Value }
```

The two pieces PR-D's design must depend on but not author:

- **`ObservationChannel`** — a typed surface (stdout / stderr / declared
  output file / declared output port). The runner extracts the
  `observed` value from this channel per target.
- **Per-`Value`-variant rendering rule per target** — a
  Rust-renders-`LiteralBits::Int`-as decimal string-after-trim is one
  rule; Python's is the same; Go's is the same; but per-target rules
  for `RecordValue` / `VariantValue` differ and must be declared. This
  is exactly the W3 sequencing the bundle brief lays out (Int → Bool →
  Record sequential ship; String / Variant / List / Cardinality
  deferred).

PR-D's design doc should phrase its dependency on this carrier
explicitly: "L5 algebraic-equality receipts assert
`StructuredValue.observed` equality at the `Value` level; the carrier
is owned by Substrate via INVARIANTS §P1 and is a hard gate."

## 4. Float policy

Floating-point equality across emit targets is **not** structural
`Value` equality — IEEE-754 round-tripping, denormal handling, and
target-runtime intrinsics (LLVM intrinsics in Rust, libpython in
CPython, Go runtime in Go) all introduce divergence the substrate
cannot model away.

PR-D's design doc should adopt one of these stances explicitly (the
audit's recommendation: option B):

- **A. Defer floats entirely.** No `Value::LiteralValue(LiteralBits::Float)`
  observations in PR-D's L5 receipt scope. Algebraic equality is
  defined only over discrete-domain values (Int / Bool / Record-of-discrete /
  VariantValue-of-discrete / NodeRef / CardinalityValue). Float
  comparison routed to a separate slice with its own tolerance
  semantics.
- **B. ULP-tolerance comparator with a declared tolerance edge.**
  Algebraic equality over `Float` uses a *declared* ULP / epsilon
  tolerance carried on the comparison (not hidden in the comparator's
  default). Tolerance becomes part of the substrate carrier; equality
  is `|subject - oracle| ≤ declared_tolerance` where the carrier names
  the tolerance.
- **C. Bit-exact `f64` equality.** Strictest; will fail on legitimate
  cross-target IEEE-754 implementation differences. **Recommended
  against** — would create false-positive PR-D failures the bundle
  cannot resolve.

The audit's recommendation is **A for PR-D's first scope; B as the
named follow-on**. Floats are a meaningful slice, not a free-with-the-bundle
extension. Note that this is the PR-D design doc's call to make; this
audit just enumerates the options so Worker A can pick one
deliberately.

## 5. Side-effect normalization

Cross-target equivalence must also decide what counts as **"the
output"**. Programs that touch external state (file system,
environment, time, random, network) have observations that vary even
when the program is "the same":

- **File system.** A program that writes to `./out.txt` produces a
  side-effect, not a `Value`. PR-D's L5 corpus should restrict to
  programs whose output is the bound port's `Value` only; side effects
  are out of L5 scope until a separate slice declares the
  effect-observation carrier.
- **Environment / time / random.** Non-determinism; cross-target
  comparison is meaningless. PR-D corpus must be deterministic by
  construction. The `effects.dag` (`src/v3/std/effects.dag`) already
  carries effect kinds — corpus admission should require the program's
  effect set to be empty (or declared "observable inputs only" with no
  side-effecting writes).
- **Stdout vs structured output.** If the typed observation channel is
  stdout, the program's actual computation result must be **the only
  thing written to stdout**. Trailing newlines are part of the
  per-target rendering rule (Rust `println!` adds one; Go `fmt.Println`
  adds one; Python `print` adds one — convergent), but logs / debug
  prints break the observation. PR-D's corpus admission should require
  programs that write exactly one rendered value to the declared
  channel.

The audit's recommendation: PR-D's design doc declares **"side-effect
free + single observation channel write"** as the L5 corpus admission
criterion, and routes any expansion (filesystem, network, time,
multiple writes) to separate slices with their own observation
semantics.

## 6. W1 / W3 carrier dependency notes (input to PR-D framing)

The bundle brief's W1 (`DifferentialEquals` lineage producers) and W3
(`ForAllTargets` per-target dispatch) both depend on the same P1
substrate carrier. PR-D should know:

- **Sequencing.** PR-D design lock can land before the carrier; PR-D
  *strict* L5 receipts cannot. Slice 0 + 1 of the existing
  harness-primitives brief are correctly framed as "primitives, not
  corpus" for this reason.
- **Single comparator authority.** When the carrier lands, PR-D must
  consume the **same** Rust-side comparator helper W2 / W3 use (single
  authority — no per-PR fork). The bundle brief's sequencing table
  already calls out the shared comparator helper as a sequenced step
  blocking both Commutativity and W3 scalar normalization; PR-D L5
  consumption joins that dependency edge.
- **Cost vs value comparators are distinct.** `CostLookup` equality
  (the existing `eval_lane_e_differential_cost_lineage` path) is **not**
  the value-domain comparator; the two should not merge. PR-D should
  preserve cost-lineage receipts as their own well-formed predicate
  inhabitant alongside the future value-domain receipts, not collapse
  them.
- **No NotYetImplemented runner arms in PR-D scope.** The bundle's
  separate routing already covers that path; PR-D should not introduce
  parallel runner-side scaffolding while the carrier is pending.

## 7. Phrasing recommendation for PR-D's design doc

When Worker A's `docs/design-cross-target-equivalence.md` reaches the
"how does L5 observe and compare values" section, the audit recommends
the following structure:

1. **State the goal:** algebraic equivalence is `subject_observed ==
   oracle_observed` at the `Value` level for a single declared
   observation channel, where `subject` and `oracle` are the same
   `.dag` program emitted / evaluated through different targets.
2. **Name the typed comparable:** point at `ProgramOutputObservation`
   (Substrate-owned per P1) as the carrier; do not redefine it in
   PR-D.
3. **Per-`Value`-variant rendering:** name Int / Bool / Record as the
   first ship-tier; defer String / Variant / List / Cardinality to
   later slices; defer Float to its own slice with a declared
   tolerance carrier.
4. **Side-effect rule:** L5 corpus admission requires effect-free
   programs that write the rendered output exactly once to the
   declared channel.
5. **Existing typed precedent:** cite `CostLookup` as the
   already-working typed-comparable pattern PR-D's value-domain work
   generalizes.
6. **Dissolution / dependency:** PR-D's strict L5 receipts gate on the
   `ProgramOutputObservation` carrier; until it lands, the harness
   stays at slice 0 + 1 (compile + cost-differential) and does **not**
   silently broaden the observation surface.

## 8. Out of scope (this audit)

- Authoring the substrate carrier — Substrate Manager owns per P1.
- Authoring the primary PR-D design doc — Worker A.
- Adding any runner code, fixture, or `NotYetImplemented` arm.
- Choosing the float policy on Worker A's behalf — the audit
  enumerates options; Worker A picks.
- Defining the corpus contents — R3 (T-Verification-L5-Corpus).

## 9. If Worker A's PR-D design lands first

Per dispatch: rebase and shrink this audit to a supplemental cross-link
note inside the merged design doc, or fold it into the existing
harness-primitives brief as a §"Observation policy notes" subsection.
Either way, single-authority discipline says this audit must not
out-live or compete with the merged design doc.
