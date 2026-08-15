# Bare cross-module references: two diagnoses, not one class

Status: **census finding, nothing implemented.** Written because folding these
into a single "needs qualification" class hides that one of them is a silent
wrongness rather than a resolution failure.

The import cut exposed bare cross-module references that previously resolved by
pool-membership coincidence. They do not all have the same shape or the same
remedy.

---

## Diagnosis A — genuine homonym (the `Empty` class)

A name declared by a small number of modules, each meaning a **different
thing**. The author intended exactly one.

    Empty -> std.algebra (FreeMonoid constructor)
             std.stack   (Stack constructor)

Measured population: 151 sites, reported by the floor cut's whole-corpus strict
fold. Two candidates, neither on the referencing module's containment chain.

**Resolution verdict:** ambiguous, must refuse. **Remedy:** qualify at the use
site (`std.algebra.Empty`). A qualified reference IS the dependency edge, so
this is the intended end state rather than a workaround.

## Diagnosis B — per-module convention (the `extdeps_external_authority_anchor` class)

A name every module in a family declares **for itself**, as a convention. There
is no single intended referent to qualify toward, because each declarer's value
is different by design.

    extdeps_external_authority_anchor -> 551 declaring modules
    live_tree_disposition             -> 1097 declaring modules

These are not homonyms in the `Empty` sense. A bare reference to such a name
from outside the declaring module is not merely ambiguous, it is **meaningless
by construction**: the convention makes 551 candidates equally valid.

Measured: of 130 modules referencing the anchor, **128 also declare it**, so
their reference binds locally and is correct. Exactly **two** reference it
without declaring it:

- `dag/test/claim/claim_indexed_evidence_witness_test.dag`
- `dag/product/altra_motherboard/attachment_stack.dag`

### Why this one is worse than ambiguity

Each module's anchor holds a **different citation**. So a bare reference that
binds to an arbitrary declarer does not fail loudly — it yields a well-typed
`ExternalAuthority` value carrying the **wrong citation**. That is a plausible
wrong answer, not a refusal: the §5 fabricated-plausible-output class, sitting
in the corpus rather than in the compiler.

Under the pre-cut regime these two sites resolved by whichever anchor happened
to be in the pool, so which authority they cited was a property of the entry's
closure rather than of the source. **Not asserted here:** whether either site
currently cites a wrong authority on main. Establishing that requires resolving
each against its historical pool and comparing to intent; it has not been done.
What IS established is that the binding was not determined by the source text.

### The remedy differs from A

The intended referent is recoverable from context rather than needing a
judgement call. In the product specimen the reference sits beside a
`DeclarationRef` naming its own subject module:

    authority: extdeps_external_authority_anchor,
    subject: DeclarationRef {
      module_path: "extdeps.cpu.ampere_altra_package",

So the authority meant is that module's anchor, and the qualification is
determined, not chosen.

**Open question, deliberately not answered here:** whether 551 qualified
references are the right outcome, or whether a convention that makes a bare
reference meaningless should be examined instead — for example by making the
anchor a field of a single declared authority row rather than a name re-minted
per module. That is a modeling question about the convention, and it is not
this lane's to settle.

---

## Why the distinction is load-bearing for closure assembly

Closure width is driven entirely by diagnosis B. Measured on a five-line entry
over the whole corpus: 1376 of 3711 modules, of which one name
(`extdeps_external_authority_anchor`) accounts for 296 direct pulls because a
single bare reference pulls **every** declarer — the closure layer must not pick
a winner.

So the wide closure is not an algorithm defect. It is the closure faithfully
reporting a corpus authoring problem, and no narrowing of the reference walk can
fix it: subtracting binders and restricting to reference positions moved width
only 1503 -> 1376. Diagnosis A costs nothing in width (2 candidates); diagnosis
B costs everything (551 and 1097).

---

## Measurements banked from the probe instrument (retired 2026-08-15)

These came from `src/v1/tests/src/declaration_index_probe_test.rs`, a measurement
instrument built to drive this census. **The file is deleted**; the numbers it
produced are recorded here because they are the evidence, and the instrument was
never a witness of a claim.

**Why it was retired rather than kept.** It lived in `v1-compiler-tests`, whose
CI gate is a 15-minute budget, and 12 of its 13 tests build a whole-corpus index
— one measured at **26.3s alone, in debug**. Cargo runs tests in parallel, so a
dozen concurrent corpus builds contend for memory and CPU on a capped runner.
DESIGN's rule is direct: *a test is not entitled to arbitrary computation merely
because it is a test*, and a test's size must be **derived from what it proves**.
An instrument answering a one-time question is a benchmark, and a benchmark may
not gate.

### Corpus shape

| quantity | value |
|---|---|
| `.dag` files under the corpus roots | 3713 |
| type-argument occurrences | 9027 |
| — bare | 8978 |
| — dotted | 49 (control-verified: a planted dotted argument IS seen) |
| dotted occurrences in one module (`v2.lens.complexity_accumulator_copy`) | 19 of 49 |
| authored `import` statements remaining | 0 |

### Uniqueness profile of bare type-argument names

| | occurrences | distinct names |
|---|---|---|
| unique (one declarer) | 6269 | 1172 |
| homonym (2+ declarers) | 1949 | 37 |
| undeclared | 760 | 86 |

This **refutes the naive "98% of names are globally unique" extrapolation by
roughly 10x** at the type-argument position. The residue is not a rounding error;
it is the qualification population.

### Closure width

A five-line entry (`specimens.dag`) closes over **1386 of 3713 modules**, with
**524 direct pulls from `extdeps_external_authority_anchor`** alone — because a
bare reference to a name 551 modules declare must pull *every* declarer. Closure
width is a corpus-authoring fact, not a walk defect (§ above).

### Dotted-argument seven-cell matrix — all clean

Cells over self-contained fixture modules compiled without source roots: bare arg
/ dotted arg / dotted-not-an-argument / no-match / direct-type / **alias in
another module** / bare + alias in another module. Every cell clean.

So module-separation of an alias from its target coproduct **does not** reproduce
the reported defect. That eliminates the cheaper of two explanations and leaves
**tree-separation** as the surviving factor — which a controlled fixture cannot
synthesize, because "another tree" is a property of corpus source roots and
supplying them would put ~3700 modules in the pool, making it a corpus
measurement rather than a controlled one. Reproducing it needs a different
harness: two synthetic source roots.

---

## What the cut's own test suite reported (measured 2026-08-15, head 19ae8362f3c)

The `v1-compiler-tests` gate timed out at 15 minutes on CI with only
`running 20 tests` in the log. **That log cannot establish how many tests
completed**: the step pipes cargo through `tee`, so stdout is block-buffered,
and the SIGKILL at the timeout discards any completed-test output under the
buffer size. Reading "zero tests completed" from the absence of `ok` lines is
the false-absence class.

Measured directly instead, running the seventeen non-corpus tests alone:

    12 passed; 5 failed; 3 filtered out; finished in 286.14s

So the timeout was concealing **five real regressions**, not merely three slow
tests. This is the deletion's second census — what X was hiding — arriving on
schedule.

### The five share one root

| test | surface |
|---|---|
| `namespace_only_refuses_fn_parent_homonym_at_call_site` | asserts a typed `AmbiguousReference`; the refusal IS produced correctly, and the test fails on a *third* diagnostic, `function 'pick' not found in scope` |
| 4 × `materialization_provider_resolved_graph_consumer_*` | request-key/semantic-digest failures whose payload is hundreds of `dag/extdeps/**` diagnostics: `unresolved type 'Nat'`, `'FilePath'`, `'NonNegativeInt'`, `undefined variable 'ApiKey'` |

The materialization tests are **self-contained** — three fixture files in a temp
dir, that dir as the only source root. The corpus diagnostics reach them through
the compiler-identity path (`transform_content_digest` feeding
`resolve_closure_request_key_from_digests`), not through their own subject.

So these are not four independent failures and not a digest defect. All five
report the same fact: **bare cross-module names no longer resolve.** The three
timing-out tests are very likely the same root expressed as cost rather than as
a refusal — closure width is driven by the diagnosis-B convention class above.

### Attribution, checked rather than assumed

`build` succeeded on main at `0ed10d7de`, so all twenty pass there. The one red
recent main run (`4b72f9445`) failed on `ci`, a different job — `build` passed.
The five failures and the timeout are this branch's.

### Not established

That qualification alone makes them green. The convention class has no referent
to qualify *toward*; whether it resolves by qualifying 551 references or by
changing the convention is the open modeling question this note already declines
to settle, and any statement about the post-fix corpus is a prediction until
that is decided.
