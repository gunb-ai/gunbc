# The 185-diagnostic residue, decomposed

Snapshot 2026-08-17 at `e229bb584af` (before the type-parameter repair lands).
Written so the remaining work is a list of decisions rather than a number.

| count | class | state |
|-------|-------|-------|
| 50 | `Nat` <- integer literal in field/named-arg | BLOCKED on the two-`Nat`-fork decision |
| 22 | type parameter shadowed by global `C` | repair pushed, awaiting measurement |
| 18 | cross-source-root reference (see below) | needs a root/ownership decision |
| ~11 | lost dependency edges | REPAIRED (196 -> 185) |
| 7 names | ambiguous owner | one decision each |
| rest | uncharacterized | |

## Cross-source-root references — a class the cut creates

`src/v2/test/manual/ownership_movable_test.dag` carries 18 of the remaining
diagnostics, all of them names reported as undeclared:
`EdgeKind`, `EdgeClassification`, `OwnershipProof`, `BindingUsage`, `Consumed`,
`Projected`, `Threaded`, `build_movable_set`.

They are NOT undeclared. They live in `src/v1/ownership.dag`. The compile's
source roots are `dag` and `src/v2`; `src/v1` is not among them. So a v2 test
depends on v1-world declarations across a root boundary that the reference
closure cannot cross, because the declaring file is in no scanned root at all.

An `import` used to bridge that gap by naming the module directly. A reference
closure cannot, because it can only pull files it can see. This is the same
"protection by absence became a live problem" shape as the shadowed type
parameter, inverted: there, absence stopped protecting; here, absence stops
providing.

Three candidate dispositions, and the choice is a modeling one:
- add `src/v1` to that compile's roots (widest, and re-imports v1-world names
  into a v2 compile that deliberately excludes them);
- move the ownership types to a root both worlds see;
- retire the test, if v1's ownership model is not v2's subject.

## A correction to this document's own instrument

The first pass reported eight of those names as NO DECLARER FOUND, because the
declaration index scanned `dag/**` and `src/v2/**` and not `src/v1/**`. The
names were declared the whole time. That is a denominator error of exactly the
class this branch keeps producing -- an instrument's silence read as the world's
emptiness -- and it is recorded rather than quietly fixed because the same index
backs the binding risk census, whose `UNKNOWN (no declarer found)` bucket
(1,135 rows) is subject to the identical bound and should be re-run over all
three roots before anyone treats that number as a defect population.

## The 7 ambiguous-owner names

Each needs one decision, not a script -- and per the parse_diagnostic lesson,
the repair must name the declaration that OWNS the name, which main's import
list is not a reliable guide to.

```
nat_max     std.nat | v2.std.nat                         (subtree decides)
Accepted    gunbc.workflow.types | v2.std.diagnostic
Rejected    std.markup | v2.std.diagnostic
Present     v2.std.optional | v2.std.execution_surface | gunbc.srv3_install_media_fetch
Absent      std.upsert_decision | v2.std.execution_surface | gunbc.host_converge | ...
Missing     extdeps.coverage | std.filesystem.types | v2.workflow.ci_placement
```

`Present`/`Absent` are the kernel-optional family and must NOT be qualified --
qualifying them degrades the scrutinee to a generic `T`. They need the
kernel binding, which is a resolver question rather than an authoring one.

## DISPOSITION 2026-08-17: the 18 cross-source-root rows — retired with a receipt

`src/v2/test/manual/ownership_movable_test.dag` is deleted. It carried 18 of the
residue's diagnostics, all names reported undeclared (`EdgeKind`,
`EdgeClassification`, `OwnershipProof`, `BindingUsage`, `Consumed`, `Projected`,
`Threaded`, `build_movable_set`) which are in fact declared in `src/v1/ownership.dag`
— a root this compile does not scan.

WHY RETIREMENT RATHER THAN A ROOT WIDENING OR A MOVE. The three candidate
dispositions this document listed are not equally available post-cut. Adding
`src/v1` to the roots re-admits v1-world names into a compile that deliberately
excludes them, and its closure pulls `src/v1/04_resolve.dag`, which the file's own
note records as carrying a pre-existing dark type error. Moving the declarations
to a shared root is a real modeling change to v1's ownership model, made to satisfy
a test rather than a consumer.

WHAT IS ACTUALLY LOST, taken from the file's own dissolution note rather than from
my assessment of it: it was OFF THE AUTO-PATH (not reached by
`claim_batch --roster-from-discovery`, since `src/v1` is not a discovery scan dir),
NOT CI-ENROLLABLE (the floor witness run is dag+src/v2-scoped and cannot resolve
the v1 reference), and NOT RESOLVABLE ON CLEAN MAIN. Its own words: "this is NOT a
live .dag regression guard today. The live v1 ownership logic is carried by the
compiled seed (v1_compiler_ownership.rs) and exercised via rust_tests — that is
where the real coverage sits, not here."

So the executed-coverage delta is ZERO, and this is the #8146 shape: a witness
that had already stopped executing, kept alive by its file existing. What is lost
is the AUTHORED INTENT that a v2-side successor should eventually exist — which
was carried in the dissolution trigger, and is preserved here rather than in a
file that cannot run.

WHAT THE CUT ACTUALLY CHANGED, stated because it is the general fact and not a
property of this file: an `import` could NAME a module outside the scanned roots;
a reference closure can only pull files it can SEE. So cross-root dependencies are
expressible under imports and structurally inexpressible after the cut.

CORRECTION to that sentence (side chat, 2026-08-17): cross-root references are
not INHERENTLY impossible. They are impossible when the target root is not part
of that compilation's declared module universe -- and that is the CORRECT
fail-closed result, not a capability lost. The rule is simply:

    a compilation can name only modules in its configured module universe.

Under imports, a reference could name a module the compilation had no way to
read, and the era tolerated the gap. The cut does not remove an ability; it
stops the tolerance. Adding the target root to a compilation's universe remains
available and is a configuration decision, which is exactly why the disposition
for this file was a judgement about its coverage rather than a forced deletion. Any other
file in this position has the same three dispositions and the same reasoning —
this is a class, not an incident, even though its current population is one.

## ONE ROOT CAUSE, three confirmed instances (2026-08-17)

The residue is not a list of unrelated authoring defects. Three sites confirmed
by reading declarations, and they are the same mechanism:

    1. TYPE PARAMETER -> global type
       src/v2/std/witness.dag:4   type Witness<C> = Holds { value: C } | ...
       binds C to dag/extdeps/languages/c/subject.dag:13  type C = | CTarget
       (the C programming language; the corpus's ONLY global `C`)
       => "variant 'DeriveGrammarRelationTokensProgress' not found in type 'C'"

    2. VARIANT CONSTRUCTOR -> global type of the same name
       dag/gunbc/ci_workflow.dag:1061  PullRequest { branches: .., types: .. }
       intends dag/extdeps/github/actions.dag:67
         | PullRequest { branches: List<String>, types: List<PullRequestActivity> }
       a 2-field VARIANT of the workflow-trigger coproduct, sitting in a list
       beside Push { .. } and MergeGroup.
       binds instead to dag/extdeps/github/pulls.dag:30  type PullRequest { .. },
       the GitHub REST object, 10+ fields
       => 10 x "missing required field '<f>' in literal of type 'PullRequest'"

    3. same shape at src/v2/compiler/materialization_carriers.dag:4:78 for
       `Terminal` (no global `type Terminal` -- the reverse direction, and not
       yet fully traced; recorded as a candidate, not a confirmation)

THE COMMON STATEMENT. A name whose meaning is determined by CONTEXT -- a type
parameter bound by its own declaration, a variant constructor determined by the
coproduct or expected type -- is instead resolved by GLOBAL BARE LOOKUP, and that
lookup silently succeeds whenever exactly one global declarer happens to exist.
Context-determined names are being answered by a global search that was never
asked the right question, and the unique arm means nothing checks whether the
answer was even visible.

WHY IT SHOWS UP AS MANY DIFFERENT-LOOKING CLASSES. The diagnostic is emitted
wherever the WRONGLY-REACHED declaration lives, and its text describes that
declaration's shape. So one root produces "variant not found in type 'C'",
"missing required field 'base'", "non-exhaustive match: missing Present, Absent",
and "type mismatch: expected Coproduct(C)" -- four different-looking classes, one
cause. This is why the residue decomposed into many small buckets: the buckets are
SYMPTOM SHAPES, not causes.

ESTIMATED SHARE, labelled as an estimate because it is not yet joined at site
grain: of the 101 unique sites in the CI-scoped compile, the classes whose text is
consistent with this root -- missing-required-field (12), variant-not-found (7),
non-exhaustive Present/Absent (8), expected-Coproduct type mismatches (9) -- total
roughly 36. Three are confirmed by reading declarations; the rest are consistent
but unverified. DO NOT quote 36 as measured.

CONSEQUENCE. Instance 1 cannot be repaired in source at all (a type parameter has
no qualified spelling). Instance 2 could be -- but the correct repair is not to
qualify the variant; it is for a variant constructor to resolve against the type
it is constructing rather than by global search. Editing these sites would be
editing correct authoring to satisfy an incorrect resolution order, the same shape
the Nat repair rejected. So the residue is substantially a RESOLVER population,
not a SOURCE population, and the remaining work is smaller than 101 suggests but
concentrated in one place.

## MERGE DEBT from the post-#8283 integration (2026-08-17) — NOT a clean merge

Recorded because the merge commit reads as resolved and two of its resolutions are
deferrals wearing a resolution's clothes.

WHAT HAPPENED. Three uniform rules were applied to 131 conflicts. Two of them had
the same flaw, discovered only by building:

  "take main's version for generated .rs"
      -> produced a seed assembled from TWO .dag corpora. Main's seed files
         reference resolved_imports / ResolvedImport / source_visible_names, which
         this branch's seed removed. 17 build errors, all one class. Fixed by
         restoring src/v1/stage0/src/ wholesale from the pre-merge tip.

  "take main's .dag content minus import statements"
      -> stripped the import STATEMENTS and re-admitted the import MACHINERY.
         src/v1/04_infer.dag went from 0 references to resolved_imports/
         ResolvedImport pre-merge to 24 after. Regen then failed on exactly those.

The general lesson, and it is the third instance today: removing the SYNTAX of a
deleted class is not removing the class. A one-sided add re-enters through
anything that carries the class's vocabulary -- an import line, a diagnostic
variant, a field name, a helper. Only an invariant that names the CLASS catches
it; conflict detection never will, because there is no conflict.

THE DEBT, stated exactly. Both sides made large independent changes to two files:

    src/v1/04_infer.dag     main +400/-32     this branch +427/-717
    src/v1/05_emit_rust.dag (same shape)

Both are restored to THIS BRANCH's version, so main's ~400 lines of #8283 work in
04_infer.dag are NOT in the tree. That is a real loss, not a formality: they are
authored compiler changes, not derived artifacts, so nothing regenerates them.

WHY THAT CHOICE: taking main's version re-admits the import machinery the cut
exists to remove, and regen refuses on it -- so main's version is not merely
undesirable here, it does not function. Re-applying main's 400 lines onto a file
whose own delta is +427/-717 requires understanding both changes, which is real
reconciliation work and not a merge rule.

OWED: a deliberate reconciliation of src/v1/04_infer.dag and 05_emit_rust.dag
against main at 611fd027708, replaying main's semantic changes onto the cut's
version. Until then this branch is behind main IN CONTENT for those two files
while reporting 0 behind IN COMMITS -- which is precisely the kind of gap that
looks closed and is not.

## The cut widens every compilation closure (2026-08-17)

Regen run 8 returned 25 diagnostics, and **not one of them is in `src/v1`**. They are in
`dag/std/observation.dag`, `dag/std/attribution.dag`, `dag/product/pcb/physical.dag`,
`dag/product/pcb/stackup.dag`, `dag/product/placement_supply.dag`.

Measured against the pre-cut ledger at merge-base `616d34604115498f25ac777d7ccb9b0b5ce759a4`,
the number of `import` paths from anywhere in `src/v1` to any of those five modules is **zero**.
Regen's closure never contained them. It does now.

That is not a regression and not an accident. The import graph was the thing that BOUNDED a
compilation closure; `05_emit_rust.dag` declared 32 imports and regen compiled their transitive
closure. With imports deleted, the pool is the whole source root, so regen typechecks the entire
corpus. Every closure in the repository widened the same way, silently, in one commit.

Two consequences, stated separately because they have different owners:

1. **Regen's bar got strictly harder, by construction.** "Regen green" post-cut means the whole
   corpus typechecks, where pre-cut it meant one 32-import closure did. Progress measured as
   "diagnostics remaining" is therefore not comparable across the cut boundary; the denominators
   are different populations.
2. **These are pre-existing latent defects, newly visible.** They were never wrong-and-caught;
   they were never compiled. This is DESIGN's second census -- what X was hiding -- and it is the
   half the rule says deletion cannot discover any other way.

### The Nat class is a binding fact, not a declaration fact

The `expected 'Product(CommutativeSemiring)', got 'Primitive(Int)'` rows were read earlier as a
wrong `Nat` declaration, and a repair was attempted and reverted (see
`nat-literal-field-init-refusal.md`). The ledger refutes that reading:

    dag/product/pcb/physical.dag       import std.integer { Int }
    dag/product/placement_supply.dag   import std.types   { ..., Int }

Two different `Int` declarations, told apart pre-cut by the import line and by nothing else.
Post-cut a bare `Int` reaches whichever one pool-uniqueness selects. This is the branch's central
finding -- cardinality consulted before visibility -- appearing inside the class that was blocking
regen, and it means the repair is **qualification at the reference sites**, never a redeclaration
of `std.nat.Nat`. The reverted declaration change was aimed at the wrong layer, which is why it
closed a definitional cycle instead of fixing anything.

## CI `witnesses` failure is era skew between the .dag side and the seed (2026-08-17)

PR #8282 @ `fd9aaea` fails `witnesses` with:

    claim_executor: unknown argument: --required-floor

This is not a witness failure. It is the merge debt recorded earlier, surfacing at the one
place it can: main's #8283 replaced the CI floor with a single invocation,

    claim_executor --required-floor --source-root dag --source-root src/v2

and this branch carries **main's `.dag` side** of that change (`gunbc.witness_floor_workflow`,
`gunbc.ci_layer_roots`, `gunbc.design_document` all mention `required-floor`) while its **Rust
seed predates it** -- `src/v1/stage0/src/` was restored wholesale from pre-merge tip
`738791252ba` after the mixed-seed failure, and neither `bin/claim_executor.rs` nor `cli_run.rs`
carries the flag. The emitted workflow therefore calls a binary that cannot answer it.

### Why the obvious repair is the wrong order

`git diff HEAD origin/main` on the two files:

    src/v1/stage0/src/bin/claim_executor.rs    +92  -491
    src/v1/stage0/src/cli_run.rs             +3279 -1585

`claim_executor.rs` is a clean take -- main SHRINKS it, because the floor cut deleted the plan,
batch, worker and selection machinery it used to carry. `cli_run.rs` is not: main is +3279
relative to this branch precisely because **this branch deleted the import machinery main still
has**, so taking main's copy re-admits the import era wholesale. That is the same trap that
produced the earlier "main's content minus imports" failure, and it must not be repeated.

The narrow port is `run_required_floor` (main `cli_run.rs`, ~900 lines) plus its
`required_floor_claims_from_admission` helper, folding `v2.workflow.required_floor`.

**But porting it does not turn this check green, and it must not be attempted as if it would.**
`run_required_floor` strictly typechecks the whole corpus during preparation, before executing
any witness. On this branch that preparation refuses -- which is the same fact as the residue
below. Landing the port first would replace `unknown argument` with a preparation refusal and
change nothing else, while adding ~900 lines of delicate seed surgery to an already-diverged
file. Sequencing is therefore: close the corpus residue, then port the floor entry, then the
check can pass for the first time.

Recorded so that a later session does not read `unknown argument` as a small flag-parsing fix.

## Run 10 (regen_stage0, the real binary): 27 rows, and two corrections

Runs up to 9 are not comparable with run 10 and must not be quoted as one series. Run 9 was
invoked as `gunbc regen`, which is not a subcommand -- it exited 2 having compiled nothing,
printing zero diagnostics, which is byte-identical to what a clean run prints. Only the exit
code distinguished them. Regen is the separate `regen_stage0` binary. **Run 10 is the baseline;
earlier counts came from a different tool with a different closure and are retired as a series.**

Result: RC 1, 27 diagnostics.

### What landed

- `std.nat.Nat` as an opaque carrier: **works**. Zero `CommutativeSemiring` mismatches remain,
  down from ~11 rows. An opaque carrier accepts integer literals and closes no cycle, and it
  makes the v1 and v2 declarations agree.
- `MacAddress` -> `extdeps.dhcp.v4.MacAddress`: **works**, zero rows.

### What regressed, and was reverted

Qualifying `fold_list` to `v2.std.algebra.fold_list` **made it worse** and has been reverted.
It traded 4 clean `function 'fold_list' not found in scope` rows for 5 `undefined variable 'v2'`
plus 4 `method 'fold_list' cannot be resolved: receiver type 'Primitive()'`. `dag/` and `src/v2`
are separate source roots; from a `dag/` file the leading `v2` parses as a variable.

The repair was reasoned from passing files that use the qualified form -- but both forms appear
in passing files (`secret_manager.dag` bare, `iam.dag` and `effect.dag` qualified), so **form was
never the discriminator**; those files are simply outside regen's closure. Reasoning from "files
that pass use form X" was invalid because passing was not caused by the form.

### `fold_list` is the unregistered-primitive gap, not a qualification target

`fold_list` is declared in `src/v2/std/algebra.dag`, which is not among regen's source roots,
and pre-cut `dag/std/attribution.dag` carried **no import for it** -- it resolved as a kernel
builtin. DESIGN's determinism thread already names this exact gap: `fold_list` "has a native
interpreter arm while appearing in neither the algebra Map surface nor the builtin registry."

So there is nothing correct to qualify it to, and no reference-site edit can fix it. It needs
the **language-prelude step** -- resolution order lexical -> prelude -> qualified -> bare -- which
is the same authority as the open resolver decision. 11 of the 27 rows are this one cause.

### `src/v1` is NOT clean -- earlier claim retracted

Reported earlier as zero on run 8's tool. Run 10 shows six rows:

    4x variant 'Present' not found in type 'Node'   src/v1/00_core.dag
    1x no field 'inferred' on type 'Unit'           src/v1/04_infer.dag
    1x no field 'body'     on type 'Unit'           src/v1/04_infer.dag

The `Present` rows are the qualification deliberately declined earlier (kernel-optional, no
pre-cut import owner) -- the refusal was right, and this is what it looks like when it comes due.
The two `Unit` rows are the recorded 04_infer merge debt. The earlier "src/v1 is clean" claim
came from a different tool with a different closure and is withdrawn.

## `fold_list` is a source-root gap, and every cheap repair is forbidden (2026-08-18)

Measured, not reasoned: `regen_stage0.rs` line 533 sets

    let roots = vec![workspace.join("src/v1"), workspace.join("dag")];

**`src/v2` is not a regen source root.** `fold_list`'s only declarer is `src/v2/std/algebra.dag`
(`module v2.std.algebra`). So under regen:

- bare `fold_list` has no declarer in the pool -> `function 'fold_list' not found in scope`
- `v2.std.algebra.fold_list` has no module `v2` in the pool -> `undefined variable 'v2'`

Both spellings fail, for the same reason. The qualification attempt and its revert were both
chasing a spelling for a name that is not reachable at all. `dag/extdeps/github/app.dag` uses the
qualified form without erroring only because it is outside the closure actually compiled.

These files were never in regen's closure before the cut (zero import paths from `src/v1`), so
the reference was never resolved. Latent defect, newly visible -- the same census the cut opened.

### Why the obvious repairs are refused

**Registering `fold_list` in `v1.compiler.infer_method builtin_function_registry` is forbidden.**
The v1 seed is frozen X, and DESIGN section 3 is explicit: "a host-builtin registry, an
escape-hatch arm, a compatibility table inside a frozen X accepts no new rows, because each row
is a deferred modeling obligation the surface's existence recruited ... and a freeze that still
accepts rows is not a freeze." The convenience of this repair is exactly what the rule exists to
refuse; the 2026-08-15 receipt (38 of 125 host builtins accreted into one driver file) is what it
costs when accepted.

**Declaring `fold_list` in a `dag/` or `src/v1` module is a section 3 fork** -- a second authority
for a function that already has one.

That leaves three candidates, all design decisions rather than mechanical fixes:

1. **Add `src/v2` to regen's source roots.** Smallest diff, but it changes what regen compiles
   for every consumer, and `cli_run::regen_source_roots` is a second place asserting the same
   pair, so the two must move together.
2. **Move `fold_list`'s authority** into a root regen can see. A relocation, not an addition --
   no new authority is minted -- but it moves a `src/v2` std surface, which needs its own ruling.
3. **The `PrimitiveDefinition` identity join** DESIGN's determinism thread already names as the
   dissolve-on for exactly this class. The modeled answer, and much the largest.

**Operator decision required.** 11 of the 27 rows are this one cause, and the branch cannot reach
a regen fixed point without choosing. Recorded rather than guessed, because the cheapest option
here is the one the authority docs forbid, and a session under merge pressure is precisely the
reader most likely to take it.
