# Behaviors recovered from the deleted import tests

Status: **design note, nothing implemented.** This exists so that deleting the
tests that asserted import *semantics* does not also delete the *properties*
some of them were the only proof of.

The import statement is gone from the corpus (`^import` count in `.dag` is
zero) and `import` is a parse error. The tests below asserted the mechanism
that is gone, so they were deleted with it. Each entry records the property
worth re-establishing, in namespace terms, once closure assembly is rebuilt.

Nothing here is a plan to restore the deleted mechanism. A property is only
worth re-expressing if it is still true under namespace-only resolution.

---

## 1. Pool membership must not be sufficient for binding

From `class_b_trim_specimen_test` (deleted; it was already a declared scaffold
whose stated dissolution was "delete this Rust module").

**The property:** a bare name must not resolve merely because some module that
happens to define it is present in the compilation pool. Under the import
regime this was proven by the pair "explicit import binds / bare use refuses
even when `std.algebra` is already in the pool".

**Why it survives the deletion of imports.** The refusal arm is the half that
matters and it is not about imports at all: it says binding must follow from a
*declared, derivable* relationship, not from incidental co-presence. That is
the Class-B pool-membership-coincidence class
(`import-strip-witness-discovery-cascade-diagnosis.md`), and this lane makes it
sharper rather than moot — with imports deleted, *every* module's dependencies
are derived rather than declared, so "did this bind for a real reason or a
coincidental one" becomes the central question rather than an edge case.

**Re-expression, once closure assembly is reference-derived:** a name whose
definer is in the pool but is NOT reachable through the reference closure from
the entry must refuse. The positive control is the same name reachable through
a real reference. Note this is only meaningful if the pool can ever be *wider*
than the closure; if assembly makes them identical by construction, the
property becomes structurally impossible to violate (§4b) and needs no test —
which is the better outcome and should be checked for first.

## 2. A unique variant projection resolves without qualification

From `decl_facts_dimensionless_projection_test`'s two `explicit_import`
tests (deleted). Sibling tests in that file survive.

**The property:** where exactly one module in scope exports a given variant
arm, a use of that arm resolves to it unambiguously, and the decl-facts
projection agrees with the resolver about which declaration it found.

**Re-expression:** this is precisely unique-on-chain with a candidate set of
one, so it should hold *more* readily without imports than with them. Worth an
explicit witness because the interesting case is the boundary: exactly one
candidate resolves, two candidates must refuse as ambiguous rather than
first-hit. The surviving `namespace_unique_on_chain_policy_test` cases already
cover the general form; what was import-specific here was only how the single
candidate got into scope.

## 3. The historical ImportScoped policy

From `import_scoped_default_resolves_homonym_fixture_clean` (deleted).

**Not a property to preserve.** This asserted that with the policy bracket set
to `false`, resolution kept the pre-namespace behavior verbatim: nearest-wins
for types, first-hit for functions. First-hit is the silent-pick class that
`NamespaceOnlyY` exists to refuse, and the policy's other arm is already the
default in the executing seed.

Recorded here only so that its deletion is not later mistaken for an accidental
loss of coverage. The behavior it pinned is the one being replaced. The
sibling `namespace_only_refuses_*` tests, which assert the *refusal*, are
retained.

## 4. Ephemeral generated source roots participate in resolution

From `helpers::tests::resolver_imports_ephemeral_generated_source_root`
(deleted — it tested the import-driven closure helper itself).

**The property:** a module written into a temporary generated source root is
found and loaded by closure assembly, not only modules that exist in the
checked-in tree.

**Re-expression:** this is a real requirement of whatever replaces
`extract_imports`, and it is about the *index*, not about imports — the
module-source index must span every configured root including ephemeral ones.
It should be re-asserted directly against the new assembly path.

---

## "Imports deleted, tree green" is a weaker claim than it looks

Measured on the sibling v1 lane (neat-bee), partitioning all 62 deleted v1 `.dag`
modules by which demand channel could have made a *survivor* complain:

```
import graph      0  of 62
path literal     20  of 62
directory scan   45  of 62
-----------------------------
union witnessed  53
residue           9
```

**Zero.** No surviving `.dag` imported any of the 62 deleted modules. Every real
dependency ran through a roster naming a path literal, or a bin doing `read_dir`
over a directory. An import-graph census would have reported that entire deletion
as unwitnessed — a rigorous-looking negative result that was worthless.

This corroborates a finding from this lane's own measurements: authored import
lists under-declared the true reference closure by roughly 30×. Both are the same
fact from opposite ends — **imports were not where dependency lived**, so deleting
them removes a representation that was already not carrying the load.

**The unreassuring half, and it bounds what this cut has proven.** If path
literals and directory scans are the channels actually carrying demand, deleting
imports does not remove them, and the verification cannot be import-shaped
either. `R1_IMPORTS 0` plus a green tree proves *nothing needed the import
declarations*. It says nothing about whether the reference closure the resolver
now computes agrees with what the surviving path-literal rosters and directory
scans reach. **Those are different populations, and only one of them is affected
by this change.**

**OWED, and not answered by anything in this branch today:** after the cut, is
there a module reachable by a surviving path-literal roster or directory scan
that whole-pool resolution does NOT reach — or the reverse? That is a **set
comparison at identity grain**, not a count, and `source_closure.rs` is the
closest existing machinery to being able to answer it.

Recorded here rather than left implicit because it is the third time in this lane
that a guard's subject turned out narrower than the claim it was used to license
(oracle: `.dag` files vs `.dag` content; control: within-job vs cross-job;
receipt: branch vs merge ref). This one is the same shape at the level of the
cut's whole verification story.

---

## What is deliberately NOT in this note

The tests that fail today with `unresolved type 'Nat'`, `'FilePath'`,
`'FieldOfFractions'` are **not** listed here and were **not** deleted:
`field_of_fractions_construction_test`, the four
`materialization_provider_resolved_graph_consumer_test` cases, and
`namespace_only_refuses_fn_parent_homonym_at_call_site`.

None of them tests import semantics. They are the evidence that the closure
rebuild worked, and deleting them would remove the only executing check on the
lane's keystone.

**Why they failed, corrected 2026-08-16.** The sentence above previously read
"they fail because closure assembly is currently import-driven, so their
dependencies never load." That diagnosis was superseded by its own lane:
`resolve_imports_transitively_with_source_roots` now delegates to
`closure_for_entry`, which is reference-driven. The real cause was found by
diffing the failing files against main rather than by reasoning further about
the resolver, and it was two things:

1. This cut's corpus pass had stripped `import` lines out of `.dag` fixtures
   that live inside **Rust string literals** and under `dag/test/fixture/`,
   without qualifying what those imports had bound. The string-literal oracle
   did not catch it because the oracle's subject is `.dag` FILES and those
   fixtures are `.dag` CONTENT in a `.rs` file — outside its denominator by
   construction.

2. `closure_inner` looked every referenced name up in an index keyed on
   **simple** declaration names, so a qualified reference
   (`std.algebra.FieldOfFractions`) never matched and the miss arm was a bare
   `continue` — the dependency was silently dropped and the closure came back
   short. Since qualification is what this cut substitutes for `import`, the
   closure builder had no way to follow the edge the cut creates.

A mechanism worth keeping in mind, though it is NOT established as the cause of
anything below: bare cross-file names match EVERY declaring module (the index
deliberately refuses to pick a winner), which in a densely interconnected corpus
multiplies closure width. An earlier revision of this paragraph asserted that the
runtime cost and the crash "are that width". That was refuted by execution — the
closure fix landed and both tests still crash — and the sentence is corrected
here rather than deleted, because the same over-reach was then repeated twice
more in the section below.

## The segfault: cause UNRESOLVED, with a pre-registered remedy decision

Two v1 tests exit `rc=139` after ~12 minutes on a parse-clean tree. Measured,
release binary, changing only the spawned-thread stack size:

```
2 MiB  (cargo test default)   rc=139 SEGFAULT   739s
64 MiB                        rc=124 TIMEOUT   1500s   <- my timeout, not a crash
```

Those two rows are from DIFFERENT remote jobs, which is the flaw that runs
through everything below: at the time this was written it read "one variable,
crash gone — it is stack exhaustion." Two variables changed, not one. The 64 MiB
arm is also censored: under linear scaling it would not be predicted to crash
until ~6.6 hours, so stopping at 1500s reached 6.3% of its own predicted failure.

A null control (same arm twice, **same job**) gives 845s / 834s — 1.3% spread.
That floor licenses WITHIN-job comparison only, and was wrongly used below to
license cross-job ones.

**The discriminator is a 1 MiB arm, and the remedy is written down BEFORE it
reports so the result cannot be read to license the more convenient fix:**

| 1 MiB outcome | what it means | remedy |
|---|---|---|
| crashes at **~370s** (half of 739s) | depth GROWS WITH WORK — stack consumed progressively, unbounded in practice | `stacker` is "raise the stack" wearing a library; it converts a 12-min segfault into a long run ending in heap exhaustion. Remedy is an explicit worklist that **refuses with a located diagnostic**. |
| crashes at **~739s** (unchanged) | recursion reaches a FIXED depth exceeding both sizes; stack size decided only whether it survived, not when it died | bounded-deep; `stacker::maybe_grow` is the codebase's own idiom (151 existing sites) and using it is consistent, not evasive. |

**Why the arm must precede the fix:** adding `stacker` to `source_closure.rs`
makes the symptom disappear under *both* hypotheses, destroying the ability to
tell which one held.

### The arm reported, and BOTH pre-registered rows are wrong

```
 1 MiB   rc=139 SEGFAULT   980s      predicted ~370s
 2 MiB   rc=139 SEGFAULT   739s
64 MiB   rc=124 TIMEOUT   1500s      (censored — my timeout)
"overflowed its stack" in the test's own stdout:  0 occurrences
```

**The arm was uninformative, and the first version of this section over-read it
in the opposite direction.** It initially concluded "halving the stack made it
crash later, so the exhaustion model is dead and a memory-safety fault is live."
That is circular, because the non-monotonicity and the cross-job variance are
*the same two numbers*:

```
as a STACK effect     2 MiB 739s (job B) -> 1 MiB 980s (job D)   +32.6%
as CROSS-JOB noise    739s (job B)       vs        980s (job D)   +32.6%
```

Identical pair, identical delta. The ~32% noise figure was *derived from* the
comparison then used to judge it. One observation pair cannot be both the
measurement of the noise and a signal assessed against that noise.

The honest reading: **the 1/2/64 MiB comparison is entirely cross-job and carries
no information about stack size.** Stack exhaustion is therefore neither
established nor refuted — both hypotheses sit exactly where they were before the
arm ran. Retracting "it IS stack exhaustion" was correct (it was never
established); asserting the model was dead was the same over-read pointed the
other way.

**The missing overflow message is a WEAK absence, not a real one.** `cargo test`
captures test output by default, and this suite is already known to lose buffered
output when a process dies mid-crash. Rust prints `has overflowed its stack` to
the crashing thread's stderr — exactly the buffer that gets lost. Zero
occurrences is consistent with *no overflow* and with *an overflow whose message
died with the process*. `--nocapture` makes the string stream instead of
buffering, which converts it into a real absence either way.

**The usable cross-job floor is 3.9–5.2%**, from `field_of_fractions` at fixed
stack size (803s vs 845s/834s) — not the 32%, which is contaminated by the stack
variable.

**A method error found in the same round, recorded because it invalidates a
second claim.** The null control measured the same arm twice *within one remote
job* (845s/834s, 1.3%). That floor was then applied to comparisons spanning
*different* jobs. Same arm across jobs:

```
field_of_fractions (fixed):  803s   vs  845s / 834s    ~4.5% cross-job
decl_facts:                  739s   vs  980s            ~32% cross-job
```

So the "+7–13%, far outside the noise floor" claim compared 748s from one job
against 803–845s from two others — a cross-job comparison judged against a
within-job floor. **Withdrawn.** It is the oracle-denominator failure applied to
an instrument rather than to a rewrite: the control was correct and complete for
its own subject, and its subject was narrower than the comparisons it was used
to license.

**Standing after this round:** the crash is deterministic and reproducible
(`rc=139`, four runs, two tests). Everything else is open. Stack exhaustion is
neither established nor refuted — the arm built to decide it was confounded.
That the closure fix increases work is likewise neither, for the same reason.
The pre-registered table is INTACT with its question still open: an uninformative
arm does not observe a third row, it observes nothing.

**Next instrument:** ONE job performing the whole comparison — same runner, same
build, 1/2/64 MiB back to back — plus the closure-membership set diff, which
answers correctness rather than cost and is unaffected by timing variance.

**A finding worth stating regardless of the outcome:** the seed uses
`stacker::maybe_grow` at 151 sites as its established idiom for deep structural
recursion. `source_closure.rs` — the 429-line file this cut introduced — uses it
zero times while performing recursive tree walks.

**Owed, and not answered by any clock:** whether the dependencies the closure fix
newly loads are ones the tests actually need. The fix demonstrably does more work
(748s → 803/845/834s, outside the 1.3% floor), but more work is equally
consistent with correctly loading required modules and with over-collecting. The
instrument is a set diff of closure MEMBERSHIP before and after, checking each
added module against the test's reachable references — a correctness question the
timing cannot answer at any precision.

**Next-rung trigger for the `continue`.** It is repaired for qualified names and
still silent for genuinely unknown ones, so the class sits at *mitigatable*. It
should become a refusal — but not before the tree's remaining unresolved
population reaches zero, because converting it earlier would require a
suppression list, and a suppression list at a refusal arm is the escape hatch
§5 forbids.
