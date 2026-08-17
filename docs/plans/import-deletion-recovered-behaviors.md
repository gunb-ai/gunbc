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

## The catch-up cost is a rate, and the import surface was never frozen

Measured once, on request, to convert "unbounded" into something actionable.
Fork point `64ebefa7416` (2026-08-15 08:31 -0400) to main head `6ada4a59fbf`
(2026-08-16 19:22 -0400), a 34.8-hour window:

```
new .dag files on main since fork:        42
of those carrying authored import lines:  42     <- 100%
authored import lines in them:           249

modified .dag files on main (same window): 76
authored import lines in them:            767

RATE: 29.0 new .dag files/day, 171.7 authored import lines/day
```

**The 42/42 is the finding, not the rate.** Not "most" — every new `.dag` file
authored on main since this cut began carries imports. No partial adoption, no
drift toward the post-cut form, not one file authored in the shape the cut
produces. Independently corroborated on a different window (tidy-pike, since
2026-08-13): 133 new files, 133 import-bearing. Two denominators, same 100%, so
neither measurement is sampling a quiet patch. Standing total on main: 19,608
authored import lines across 3,017 files.

The **modified** count is reported alongside the additions because additions are
not the obligation: modified files produce merge *conflicts* rather than clean
adds, which makes them the more expensive half of the catch-up. A request for
"files added" has a narrower subject than the cost being argued about — the same
class this note records three other instances of.

**What this means, in DESIGN's own terms.** §3 rules that a freeze covers X's
growth surfaces: *a compatibility table inside a frozen X accepts no new rows,
because each row is a deferred modeling obligation the surface's existence
recruited — and a freeze that still accepts rows is not a freeze.* The import era
is X for this cut. At 42/42 and ~29 files/day the surface is accepting rows with
zero exceptions, so **X is not frozen**, and this branch's catch-up cost is not
inattention — it is interest on an obligation main creates at a measured rate.

That is a repository-level decision (freeze the surface, or land the cut early),
not a lane-level one, and it is escalated rather than absorbed. The branch will
not carry tooling to keep winning the race.

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

**Channel 1 (path literals) — ANSWERED, at identity grain.** Extracting every
repo-relative `*.dag` path literal from `dag/**`, `src/**` `.dag` and `.rs`
sources on `origin/main` and on this branch, and diffing the sets that do not
resolve to an existing file:

```
dangling on main:                284
dangling on branch:              283
NEWLY dangling (cut's doing):      0
no longer dangling:                1   src/v2/std/some_other_module.dag
```

**Zero path literals were broken by the cut.** Stating the subject explicitly,
because three earlier passes at this number used the wrong one: the subject is
*repo-relative paths naming a `.dag` file*, which excludes globs, diff fragments
inside witness fixtures, relative paths, and prose that merely contains the
string `.dag`. An unfiltered extraction reports 791 and means nothing. The
absolute counts are also not the finding — most of the 283 are deliberate
sentinels (`does_not_exist_sentinel.dag`, `__bogus_never_imported__.dag`) and
synthetic fixture inputs. **The diff is the finding**, because it is the only
part attributable to this change.

**Channel 2 (directory scans) — STILL OWED, and it is the larger channel** (45 of
62 in the v1 census, against 20 for path literals). A scan cannot "dangle": it
names no files, it enumerates them. So this check cannot cover it by
construction. The failure modes there are different — a scan picking up a file
that no longer parses, or missing one that moved — and answering it requires
comparing enumerated sets, not resolving literals.

**Also still owed:** the reverse direction — a module the rosters/scans reach
that whole-pool resolution does NOT, or vice versa. `source_closure.rs` is the
closest existing machinery.

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

## The remedy table above is WITHDRAWN, and the crash had a higher cut

Two corrections against this note's own earlier text, both owed to external
review of it rather than to my own re-reading.

**The pre-registered 370s/739s decision table is falsified, not amended.** It
offered exactly two outcomes for the 1 MiB arm — ~370s meaning stack use grows
with work, ~739s meaning a fixed deep descent — and the observed 959s matched
neither. A plausible mechanism was then found for the miss (stacker chains
segments, so a smaller native thread stack reaches `maybe_grow` sooner and can
gain total runway, making time-to-fault non-monotonic). That explains the
result; it does not rehabilitate the table. A two-outcome model that observes a
third outcome is refuted, and the honest move is to withdraw it rather than to
widen it after seeing the answer. It is withdrawn. Nothing downstream should
cite either row.

**"The true closure" was the wrong name for what `source_closure` computes.**
This note repeatedly called the ~1,100-module result the true or correct
closure, and used that to argue no better closure builder could help. The
implementation says otherwise in its own comment: binding has not happened, so
a reference-shaped name can still pull an unrelated declaring module, and the
result is an explicit structural OVER-APPROXIMATION whose precision ceiling is
bound-occurrence edges. Zero observed homonym widening does not establish zero
false-positive single-declarer edges. The correct name is *parse-derived
conservative closure*, and the argument built on the old name does not stand.

**The remedy question was below the available cut.** Which stack overflowed is
answerable from the fault address and the maps, and it was answered: a
stacker-allocated 2 MiB segment, faulting on the `PROT_NONE` page 16 bytes
below its base. But that answer cannot choose between cycle refusal, an
explicit worklist, and a justified finite bound — only depth plus subject
identity can. Before spending that, the prior question is whether any consumer
needs the deep descent at all, and for the `FieldOfFractions` witness it did
not. `eval_record_lit` decides its collapse on the type NAME string alone
(`type_name == "GroupCompletion"`, `== "Succ"`); nothing in that arm consults a
resolved declaration. The whole-corpus resolve that witness carried was the
compile-clean gate's subject sitting inside a unit test. Split by subject — the
real authority parsed directly for the shape claim, a minimal same-name
specimen for the runtime claim — the same two assertions run in 0.02s and
0.00s, where the fused form took 803s and died in a SIGSEGV.

**Still open, and not closed by the split:** whether a surviving production
consumer reaches the same descent. If one does, the instrumentation owed is
logical depth plus a stable subject identity at each descent — repeated
identity means a cycle and a typed cycle refusal, unique deep structure means
an explicit worklist, and only an authoritative domain ceiling would justify a
depth bound. A 149th `maybe_grow` is refused either way: it would erase the
symptom before distinguishing those.

## Findings filed against main, not against this cut

- [a record field initializer is not checked against its declared type](main-record-field-type-unchecked-finding.md)
  — surfaced as the control half of a cut comparison that was then withdrawn.
  Below the ordinary compiler floor, reproduces in seven lines, and is a fact
  about `main` rather than about the import deletion. Linked from here so that
  withdrawing the branch-side claim does not orphan the control's own result.

## The deep recursion is reachable from any large `compile_to_resolved`, not just from two bad tests

Recorded because it narrows a question I had left open, and because it was
found by an instrument failing rather than by looking for it.

A branch-local audit harness — counting constructor-resolution refusals across
every `data` item in the corpus — built one source vector of 3,747 modules and
called `compile_to_resolved`. It died at `rc=139`, the same SIGSEGV, after
`compile.frontend` and `compile.normalize` completed.

So the earlier reading that the descent was reachable "only from two
over-scoped witnesses" is **too weak**. What the two witnesses had in common
was not being tests: it was handing `compile_to_resolved` a large source
vector. Any consumer that does that reaches the same recursion.

**The contrast that localizes it.** `gunbc compile --source-root dag
--source-root src/v2` over the SAME corpus, on the same host and binary,
completes: exit 1, 3,096 located diagnostics, no fault. So the fault tracks the
DRIVER PATH rather than corpus size — the CLI's whole-corpus compile does not
reach the descent that `compile_to_resolved` over one vector does.

**What this does and does not establish.** It does not establish that a
production consumer reaches it; the harness is not one, and the CLI path is the
production whole-corpus compile and it survives. It does retire the claim that
the population is two tests. The bounded question — whether the resolver's
production entry can reach a cycling descent in principle — is now sharper: the
two drivers differ, one faults, and the difference between them is the place to
look.

**Consequence for the audit.** The 1,187-site constructor audit cannot be run
at whole-corpus grain through this path until the recursion is repaired or the
harness is scoped. Scoping it is the better instrument anyway: the subject is
157 files, and building the whole world to ask about a known population is the
same over-scoping this branch already cut out of two witnesses.

## Correction: 3,096 is a FLOOR, and the two-driver contrast dissolves

Both corrections come from one check — do two instruments I compared actually
do the same work — prompted by review rather than by my own reading.

**The contrast dissolves.** I reported that `gunbc compile` completes over the
same corpus where a `compile_to_resolved` harness faults, and read that as two
drivers on one subject. They are not one subject:

```
CLI     resolved 2394 sources (transitive import closure)
        1353 indexed modules enter the name census only (NOT compiled)
audit   3747 sources in one vector
```

The CLI does reach FURTHER through the phase sequence — it completes reconcile
and analyses where the harness faults in reconcile — so it is not stopping
early. But it was fed a smaller population, so "one faults and one does not" is
not attributable to the driver. It is not a localizer and I am not building on
it.

**And the headline number is a floor.** The same line corrects a number I have
been quoting all session:

```
main    2621 of 3756 compiled  ->    32 diagnostics
branch  2394 of 3747 compiled  ->  3096 diagnostics
```

The branch is measured over FEWER modules than main, and 1,353 modules were
never compiled at all. So 3,096 is a lower bound on the cut's corpus damage,
not a measurement of it, and the gap is not random: the CLI drives its
whole-corpus compile from the transitive IMPORT closure, which is exactly what
this branch deletes. With imports gone nothing pulls a module that nothing
imports, so the modules most likely to be broken by the cut are precisely the
ones the instrument cannot see.

**That makes the measurement gap the same fact as the defect.** The import
graph is not the reference graph. Measuring this branch through an import-driven
closure is measuring the cut with the instrument the cut removes. Any real
number for the corpus requires a reference closure — the edges the compiler
actually resolved — which is the authority `sleek-moth-351` is building for the
floor's scope derivation. That is one authority, not two, and this lane should
consume theirs rather than fork a second.
