# The unqualified-reference population: what the cut actually left behind

**2026-08-17.** Supersedes the framing in which the branch's remaining work was
"182 diagnostics in four classes". It is not. That count is an observation over
a biased surface.

## The bias

The qualification pass only ever rewrote a site that PRODUCED A DIAGNOSTIC. A
bare reference whose name is ambiguous corpus-wide, but which happened to
resolve because exactly one declarer was in that compile's pool, was left
untouched and reported nothing.

So the diagnostic count never measured binding correctness. It measured how many
ambiguous bindings were also unlucky enough to be type errors. Reducing it
196 -> 182 removed noisy bindings and told us nothing about silent ones.

This is DESIGN's absorbing fallback with the polarity inverted: not a failure arm
that widens, but a SUCCESS arm that narrows. The instrument reported the subset
that complained.

## Why silence is not safety here

`v1_interpreter.rs` registers every named item twice:

    fn_nodes.insert(name.clone(), item.clone());       // bare leaf
    fn_nodes.insert(qualified.clone(), item.clone());  // module.name

One map. The bare slot is overwritten as modules are traversed, so a bare name
with N declarers resolves to whichever module was visited last. The qualified
key cannot collide, because a module path is unique.

The interpreter also computes

    ambiguous_bare_function_names = bare_name_counts.filter(count > 1)

stores it on `InterpContext`, and never consults it in `lookup_fn`. Its only
consumer is `selected_function_identity`, whose only callers are one unit test.
The refusal is available at the exact site of the silent pick and is discarded.

## The population, measured — and the grain correction

An earlier draft of this note said "5,783 reference sites". **That was wrong and
the error was 6.5x.** The instrument (`binding_identity_oracle.py`, deleted
2026-08-20 with its receipts) emitted one row per `(file, imported name)` pair and
decided membership with `re.search`, so it recorded PRESENCE, not occurrences. The
finding is kept because the failure mode outlived the tool: a presence-grained
instrument reporting an occurrence-grained number understates by whatever the mean
occurrence-per-pair happens to be. Corrected:

    (file, name) pairs                     5,783
    bare occurrences, total               38,694
      inside string literals                1,047   (2.7%)
      in code                              37,647
    distinct names                            348
    files                                   1,913

Of those pairs, only 6 had been qualified.

## Two names are half the population, and neither is a rewrite subject

    Node   9,893
    List   7,573

46% of in-code occurrences. `List` is part of the canonical container surface
(`List`, `Set`, `Map`, `Witness`) rather than an ordinary runtime-dispatched
module item, and blanket-qualifying it because the declaration census finds
homonyms would be wrong. `Node` needs the same scrutiny before it is touched.

This is why the raw census is an upper-bound INDEX and not an edit manifest.

## `import_said` is a candidate generator, NOT an oracle

Each row carries `import_said` — the module the file's import block named for
that spelling before the cut. It is strong evidence of intended authority and
the right way to PROPOSE a replacement. It cannot CERTIFY one:

- it is file-grain while binding is occurrence-grain, so one imported spelling
  can coexist in the same file with a generic parameter, a lambda parameter, a
  pattern binder, a local declaration, and prose (the `C` class is exactly this);
- import syntax records exposure, not resolved-node identity — it establishes
  no declaration kind, variant parent, expected type, or lexical precedence;
- the declarer index it is checked against is itself regex-derived, so the 207
  rows where `import_said` is absent from the declarers may be index omissions
  rather than judgement calls.

### And "a wrong qualifier fails loud" is FALSE as a general property

This note previously argued a bulk rewrite was safe because a bad qualifier
announces itself, citing `parse_diagnostic` — qualified to `v2.std.algebra` from
main's import binding, refused, corrected to `std.algebra`.

That was one lucky case, not a law. It failed loudly because the wrong authority
was INCOMPATIBLE. A wrong target survives silently when it has the same
parameter surface, a structurally compatible record shape, a same-shaped variant,
the same broad return type, or is simply never reached by the measured consumer.

The `emit` incident proves the point rather than refuting it: five declarations
shared one leaf name and the runtime picked by map insertion order. That
particular winner had incompatible parameters, so it complained. A
compatible wrong winner would have executed.

## What this deletion's census is allowed to claim: a LOWER BOUND

DESIGN says "the deletion is the census -- every real dependent refuses loudly."
That is REFUTED as an unconditional rule, and this lane is the evidence. The
corrected form (tidy-pike-117, converged independently with the measurements
below):

    Deletion is a complete census ONLY for dependencies whose binding, dispatch
    and evidence paths are fail-closed and structurally coupled to the deleted
    authority. Otherwise deletion exposes only a LOWER BOUND on the load.

Two silent-survivor mechanisms, both measured in this repository rather than
supposed:

    compile time   AmbientPoolUnique      `global_bare_lookup` resolves a
                                          pool-unique name from anywhere with no
                                          chain check, so identity depends on
                                          unrelated pool membership
    runtime        LastWriteWinsBareMap   `fn_nodes` carries bare and qualified
                                          keys in one map; the bare slot is
                                          overwritten by traversal order and
                                          `lookup_fn` never consults the
                                          ambiguity set it computes

In neither case must deleting the import authority produce a refusal. It may
instead produce a DIFFERENT binding, a different executable function, or
apparently successful execution. The corpus can therefore become GREENER because
the cut removed the evidence of ambiguity -- the exact inverse of the loudness
assumption, and the same shape as this note's opening finding that the diagnostic
count only ever saw the ambiguous bindings that also happened to be type errors.

So the standing this branch may claim is

    DeletionCensusLowerBound { observed_refusals, known_silent_classes,
                               invalidated_controls }

never DeletionCensusComplete. The complete arm requires four properties, and this
branch satisfies none of them yet:

    1. binding identity independent of ambient pool residency
    2. ambiguity REFUSES BEFORE SELECTION
    3. the census ranges over ALL candidate occurrences, not only
       diagnostic-producing ones
    4. the cut PRESERVES OR DISPOSITIONS its evidence controls

Property 4 is the one this branch demonstrably violated, and its rule is sharper
than "keep the fixture": A CONTROL MUST STILL REDDEN UNDER THE MUTATION IT EXISTS
TO DETECT. A control surviving as a FILE while no longer discriminating is worse
than a deleted one, because it reads as coverage --
`module_graph_edge_source_witness_test` exactly. Note also that such a control
need not import or call the deleted authority at all, so a direct-consumer census
cannot find it.

Property 2 carries a named general defect worth checking every gate against:

    PostSelectionAmbiguityObservation, under GuardDoesNotDominateActuation
    A check guards an operation only when an admitted result is REQUIRED BEFORE
    the first selection or actuation. A field attached to a receipt AFTER
    selection is evidence about what happened, not a construction wall.

None of this makes delete-first wrong. It survives with a precondition: prove the
substrate cannot silently survive the deletion by rebinding, overwriting, falling
back, or losing its own controls, BEFORE using breakage as an exhaustive
measurement. Where that is unproven, deletion remains the right discovery
instrument -- it just yields a lower bound, and must say so.

## Every instrument in this lane is blind to the same class, including the borrowed one

`global_bare_lookup` consults containment ONLY on the ambiguous arm. The unique
arm binds and discards the candidate. So when exactly one declarer is resident in
the assembled pool, NO VISIBILITY CHECK RUNS AT ALL — pool-uniqueness is not an
answer produced by the visibility relation, it is a BYPASS of it.

That single fact bounds every number this lane has produced:

    diagnostic counts (196 -> 182, 17 -> 30)
        see only sites that ALSO happened to red

    the (file,name) census and its occurrence expansion
        see only names with 2+ declarers CORPUS-wide

    the peer's [floor-bare-name-ambiguity] counter (96,481)
        sees only picks-among-many; a name with one declarer in scope is
        not counted at all — a LOWER BOUND on unsafe binding, never a
        measure of it (characterised by its own author, after handing it over)

    SilentPickTelemetry (GlobalBareLcpPickSite / GlobalBareLcpTieSite /
    FnParentFirstHitSite)
        all three guard on candidate_count < 2. Coherent for their subject,
        but the family has NO member for the unique arm, so no acceptance
        argument may cite it as silent-pick evidence for this class.

So "picks among many" is measured and "bound without checking" is not — and the
second is the larger and quieter class. If the peer counter is enrolled against
this branch, zero means "no ambiguous bindings remain", NOT "all bindings were
checked".

## The measurement that would not be blind, and needs no semantic change

Count the bare references that bind through the UNIQUE arm and would FAIL the
chain filter. `global_bare_chain_candidates` is already a total filter with no
fallback (keep a candidate iff its module path is an ancestor of the referencing
module; empty -> ModulePathBindingMiss -> refuse), and
NAME_RESOLUTION_POLICY_NAMESPACE_ONLY already defaults true in production, so the
ambiguous arm is containment-checked TODAY and the asymmetry is real rather than
a policy artifact.

The arm-compile-count-discard pattern already exists in this tree:
`compile_dag_diagnostic_census` arms `TYPE_REF_HIT_NE_BIND_MEASURE` host-side for
a nested synthetic compile. Nothing needs to land to take the reading.

Why this number outranks every other one here: failing the chain filter is a
property of EVERY bare reference, red or green, so the refusal count is the
COMPLETE population at that grain rather than a diagnostic-conditioned sample.

WARNING THAT TRAVELS WITH IT: the construction is small and the blast radius may
not be. `GlobalBareCandidate` and `GlobalBareUniqueBinding` carry identical fields
and 04_env.dag already converts between them, so routing the unique arm through
the same relation introduces no new type or helper. But DESIGN's own census says
~98% of names are globally unique and therefore always written bare, so enabling
the filter there would refuse every bare cross-module reference whose declarer is
not an ancestor of the referencing module. MEASURE BEFORE SCOPING.

## The oracle that does hold

For each authored occurrence o:

    pre-cut resolved declaration identity(o)
      == proposed qualified target
      == post-cut resolved declaration identity(o)

where identity carries declaring module, declaration node or stable symbol,
declaration kind, and parent coproduct where relevant. `import_said` supplies the
middle term; the two resolvers verify it.

## Sequence

1. Expand pairs into exact occurrences with parser-provided spans and reference
   roles. Prose then disappears by construction instead of by regex exclusion.
2. Partition structurally BEFORE resolving: prose/comment, embedded DAG-source
   payload, lexical/local/generic binding, kernel primitive, canonical container,
   runtime function/data reference, type reference, record constructor, variant
   constructor, match-pattern constructor, unknown position.
3. Capture pre-cut binding identities from the pinned import-era resolver — do
   not infer them from the import block when the compiler can report them.
4. Rewrite the runtime-dispatchable population first (calls, function values,
   data reads, entry references). Highest harm: it can silently execute the
   wrong code. The nine `emit` qualifications are the exemplar.
5. Rewrite types and constructors using declaration kind, expected type and
   parent coproduct — NOT the function-runtime rule.
6. Only then recompute diagnostics and treat what survives as modeling work.

## Acceptance, at the right grain

    ambiguous bare runtime-dispatch occurrences = 0
    runtime selected-identity mismatches        = 0
    ordinary string literals changed            = 0
    comments/annotations changed                = 0

Not a diagnostic count, and not the peer's 72,480 `(name x scope)` exposure
census — that moves with reach as well as with resolution.

## Code-as-data is a separate, real defect

Some `String` rows carry DAG source or generated source for another consumer, so
a blanket string-literal exclusion is necessary but not sufficient. Source text
and prose are both undifferentiated `String`, forcing every tool to infer the
consumer. The durable fix is a branded carrier (`DagSourceText` / `ProseText` /
`GeneratedSourceText`) or an explicit specimen roster. It is recorded here and
deliberately NOT solved inside this rewrite.
## Decision rule, precommitted before the reading (2026-08-17)

Written and committed BEFORE the unique-arm measurement returned, because a rule
authored after the number is a rationalisation of it. Operator authorised the run;
the side chat's condition on authorising it was that this rule exist first and that
the result not initiate another round of local fixes before the vehicle is decided.

THE PARTITION THAT DECIDES, not the total:

    class A -- name is globally unique corpus-wide, but its declarer is OFF the
               referencing module's ancestor chain.
               Exactly one possible authority exists, so the intended owner is
               recoverable without any pre-cut ledger: mechanically qualifiable
               from the census already in hand.

    class B -- name has 2+ declarers corpus-wide and exactly one of them happened
               to be resident in this pool.
               The binding was decided by POOL COINCIDENCE. The intended owner is
               NOT recoverable from the post-cut tree; it needs the pre-cut import
               ledger or an equivalent exact authority receipt.

A total mixes these, and they have different remedies, so a total cannot decide
anything. Class B is the one that sizes the vehicle.

CONTINUE #8282, under the narrower contract "Import-authority cutover with
binding-safety walls" -- NOT "namespace cut complete" -- only if ALL hold:

    - class B is small and enumerable;
    - class A rows are qualifiable from parser spans, not name-level text
      replacement (this branch has already been bitten once by replay qualifying
      a local binder that shared a module's leaf name);
    - the population does not substantially cross fixtures, embedded source,
      generic parameters, or pattern binders -- each of those needs migration
      machinery that does not exist;
    - no resolver redesign is needed to make the rewrite trustworthy.

CLOSE #8282 UNMERGED AS QUARRY if ANY holds:

    - class B is broad;
    - correct rewriting requires building the occurrence-level pre-cut identity
      ledger anyway (at which point the ledger, not this branch, is the vehicle,
      and a fresh recut from current main is cheaper than repairing a transform
      already applied 3,333 files wide);
    - failures span types, constructors, runtime calls, fixtures and code-as-data
      together;
    - finishing would layer a second corpus-scale rewrite onto the first.

MY PRIOR, recorded so the result can falsify it rather than confirm it: DESIGN's
own census says ~98% of names are globally unique and therefore normally travel
through this arm, so I expect class A to be LARGE. That is survivable -- one
authority each. The number I do not have a prior for, and the one that actually
decides, is class B.

WHAT THIS MEASUREMENT DOES NOT CLOSE, so a good result cannot be read as merge
readiness:

    - runtime dispatch. The interpreter registers items under BOTH bare and
      qualified keys in one map and the bare slot is last-write-wins; a
      compatible wrong selection executes silently. Compile-time containment on
      one lookup path says nothing about it.
    - the 48 disarmed fixture controls.
    - regen's generated authority and the known .dag/seed twin divergence.

So this decides VEHICLE AND MIGRATION SIZE. It does not decide merge.

## SUPERSEDED by operator ruling, 2026-08-17: the bar is compilation and tests

The decision rule above was written against an IDENTITY bar -- prove each
reference still resolves to the declaration it resolved to before the cut. The
operator has ruled that bar out:

    "i wouldn't worry about proving names pointing where they used to -- i would
     only focus on tests/compilation still working"

WHAT THIS RETIRES. Class B -- multi-declarer names bound by pool coincidence,
whose intended owner is not recoverable from the post-cut tree -- was the
population that made a CLOSE recommendation likely, because recovering it needs
the pre-cut ledger and an occurrence-level rewrite. Under the ruling that
recovery is not owed. The close conditions above therefore no longer apply, and
the continue conditions collapse to ordinary work: compile clean, tests green,
regen restored.

WHAT IT DOES NOT RETIRE, stated because the ruling is about EFFORT, not about
what is true. Binding still consults containment only when there are two or more
candidates; with exactly one resident candidate no visibility check runs. So a
green tree is green because of what happened to be loaded, not because a rule
says it should be. The live specimen is dag/extdeps/bmc/types.dag, which compiles
only when unrelated files drag its dependencies into the pool (#6985 Class B).
The consequence under the new bar is not wrong-meaning; it is INSTABILITY -- a
file added later can introduce a second candidate and turn a passing tree red, or
change a binding quietly, at a site unrelated to the change that triggered it.

THE CHEAP ROUTE TO MAKING THE NEW BAR HOLD FOR A REASON, and the one open design
decision: apply the existing containment filter on the one-candidate path too, so
resolution follows the containment tree rather than load order. The machinery
exists (global_bare_chain_candidates is already total, with no fallback), and
GlobalBareCandidate / GlobalBareUniqueBinding carry identical fields with a
conversion already in 04_env, so this introduces no new type or helper. It is a
real semantic change with a real blast radius: bare references are common because
most names are unique, so arming it could refuse a large population. The
measurement in flight sizes exactly that, and NOTHING IS ARMED WITHOUT AN
EXPLICIT OPERATOR DECISION on the measured number.

The measurement itself is retained and still worth taking under the new bar --
not as a merge gate, but because it is the only number that says how much of the
tree's greenness is load-order coincidence, and therefore how exposed the tree is
to unrelated future changes.

REMAINING WORK UNDER THE NEW BAR: compile clean; tests green; regen restored.
Ordinary, and none of it needs the ledger.

## A CONFIRMED live specimen of the unique-arm class, and it is not source-fixable

Found 2026-08-17 while working the residue. This is no longer a predicted class.

    src/v2/std/witness.dag:4    type Witness<C>
                          5       = Holds { value: C }
                          6       | Violates { diagnostic: Diagnostic }

`C` on line 5 is the TYPE PARAMETER declared on line 4. The corpus contains
exactly ONE global declaration spelled `C`:

    dag/extdeps/languages/c/subject.dag:13    type C = | CTarget

-- the C programming language, cited to ISO/IEC JTC1/SC22/WG14. Exactly one
declarer means the UNIQUE arm, which binds without consulting containment. So the
type parameter resolves to the C language, and a consumer

    src/v2/std/grammar.dag:1317
      Holds { value: DeriveGrammarRelationTokensProgress { .. } } =>
      on a Witness<DeriveGrammarRelationTokensProgress>

produces

    dag/extdeps/languages/c/subject.dag:13:1: error: variant
      'DeriveGrammarRelationTokensProgress' not found in type 'C'

The compiler is asking whether DeriveGrammarRelationTokensProgress is a variant of
the C programming language. The diagnostic is located at the DECLARATION it
wrongly reached, not at the site that asked -- which is why this class was
mis-filed as "type parameter shadowed by global C" rather than recognised as the
unique arm. It is the same mechanism, seen from the wrong end.

WHY THIS SETTLES THE DESIGN QUESTION RATHER THAN JUST ADDING A ROW. Every other
member of the residue can, in principle, be repaired in source by qualifying the
reference. THIS ONE CANNOT. There is no spelling of a type parameter that
disambiguates it from a global type: a parameter is lexically bound by its own
declaration, and qualifying it is not a thing the language permits. So this class
is closable ONLY by resolution order -- specifically by the rule that lexical
binders (parameters, generic parameters, lambda and pattern binders, locals) are
consulted BEFORE any global bare lookup.

That is exactly the order the side-chat design states, and this specimen is its
positive control:

    lexical identity  ->  language prelude  ->  qualified container.member
                      ->  bare module-level, VISIBILITY-FILTERED THEN COUNTED

"Cardinality after visibility, never before it" is the one-line statement of the
defect. Here cardinality is consulted first and it answers ONE, and the answer is
the wrong type entirely.

CONSEQUENCE FOR THE BLAST-RADIUS ARGUMENT. I had framed applying the visibility
filter to the unique arm as conditional on the measured population -- turn it on
if cheap. This specimen refutes that framing: a class that cannot be authored
around is not optional to fix regardless of its size, and a LARGE measured
population is evidence that the unsound path is widely relied on, not evidence
that it should stay. The measurement therefore sizes HOW the migration is
executed, not WHETHER the semantics are corrected.

NOT CLAIMED: that every residue row shares this mechanism, or that the count is
known. The whole-tree number is still in flight. What is claimed is that ONE
confirmed instance exists, that it is the second-largest class's real cause, and
that it is unreachable by source edits.

---

## The inversion: a site where QUALIFYING is the defect (2026-08-22)

Everything above describes the pass under-reaching — bare references it never
rewrote. This section is the opposite failure, and it is more dangerous because
every instinct the branch has trained points the wrong way on it.

**`T?` is not `Optional<T>`.** The question mark is a **cardinality** on the node
(`return_cardinality: CardOptional`). `v2.std.optional.Optional` is an ordinary
coproduct, declared in `src/v2/std/optional.dag` — the only `type Optional` in
the pool. They are different things that share the spellings `Present` and
`Absent`.

So in a cardinality-optional position, **bare `Present` is not an unqualified
reference at all** — it is the kernel cardinality form. Qualifying it to
`v2.std.optional.Present` does not make it safer or more explicit. **It changes
the meaning**, and the compiler says so exactly:

```
type mismatch: expected 'Coproduct(WorldRefusal)', got 'Coproduct(Optional)'
```

for a field declared `refusal: WorldRefusal?`.

### The visible symptom, and the invisible one

The pass rewrote **constructors** and left **patterns** alone, so a single
expression ends up carrying two conventions — `product.spatial_world` `find_room`
matched `Present { value: found }` and `Absent` bare while constructing
`v2.std.optional.Present`. That inconsistency is the visible tell.

The invisible one is worse: **the rewrite is semantically wrong even where it
looks internally consistent.** A module whose constructors were all qualified
reads as correctly and uniformly cut. Nothing about its appearance distinguishes
it from a module that was right to be.

### Why no existing number shows it

These modules sit outside the seed closure, so they never appear in the rustc
diagnostic count the branch is driven by. That count was blind to this class by
construction — it is a second axis, not a subset. Confirmed: repairing
`std.orthogonal_topology` and `product.spatial_world` moved the rustc count by
zero.

### What it blocks, which is the reason it is urgent

`gunbc run` loads the whole pool, so **one** module failing to typecheck makes
**every witness on the branch unexecutable**. Not slower, not narrower —
unexecutable. For as long as such a module exists, witness evidence on this
branch is absent rather than weak, and a witness that cannot run is
indistinguishable from a green one unless something forces the distinction.

This population therefore admits no prioritisation: a residue of one costs the
same as a residue of forty.

### Population, stated with its noun

38 files both declare a cardinality-optional and construct the v2 coproduct.
That is an **upper bound on candidates**, not a defect count — a file may
legitimately hold both forms, and the defect is a **site** where a
cardinality-optional position receives a coproduct value. The site-grain join is
the measurement; until it lands, 38 is not a count of anything.

### For whoever runs a qualification pass next

Do not qualify `Present`/`Absent`/`none` in a position whose declared type ends
in `?`. Check the declaration, not the spelling. If the pass cannot distinguish a
cardinality-optional position from a coproduct one, it must refuse the site
rather than guess — guessing produces a module that looks cut and is wrong.
