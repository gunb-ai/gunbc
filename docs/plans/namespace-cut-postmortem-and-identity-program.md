# Postmortem: gunbc#8282, and the identity program it revealed

> **On the numbers in this document (2026-08-25).** Every count below is a
> transcription unless it names the command that re-derives it, and under the
> operator's *name the instrument, never transcribe its output* ruling a
> transcription is debt. The counts retained here are retained on DESIGN's own
> declared-exception test — **the magnitude is the subject of the claim**, not a
> reading taken of something else: *zero* emitted newtypes and *zero* emission
> reads of `sole_constructor` are the findings themselves, and a finding that a
> mechanism is never reached is unreadable without the number that is zero.
> Where a count is load-bearing, the deriving command is given inline so a
> reader re-runs it rather than trusting it. Where one is not, it has been cut.
>
> This is not a formality and the receipt is in §11.2: `sole_constructor` was
> published here as **176** production sites and corrected to **79** twenty
> minutes later — 21 of the difference were `data …: String` prose rows
> *discussing* the feature. The wrong number had already been sent to a lane as
> the basis for a fan-out estimate. **A grep is a hypothesis; a count with no
> producer behind it is not a measurement.** The standing repair is §11.2(4a) —
> ask whether the subject already publishes its own answer — and the terminal
> one is a lens over the `Node` tree, which is §7's open item.


**Subject:** why three generations of import deletion failed, what single defect
they were all standing on, and the terminal shape and landing order for each
piece of it.

**Status:** postmortem + direction. **It supersedes nothing on its own
authority.** It does record one operator supersession — the 2026-08-25 reordering
that moves grammar deletion to last, §7a — which is a ruling relayed, not a
conclusion this document reached; the ratified plan's ROOT is unchanged by it.
[namespace-cut-replacement-plan.md](namespace-cut-replacement-plan.md) is ratified
— DESIGN §3 links it as one of two cut programs — and its 2026-08-15 operator
ruling *predicted this document's findings by name*: "delete all the grammar/import
up front, then solve each problem as it is revealed — expecting the deletion to
also reveal problems import was standing up (**the concealment census**)." What
follows is that census's output. The defects below are the cut working, not the
cut failing, and any reading of this document as an argument against the cut is a
misreading of it. Contributes the compiler as the missing consumer to
[keying-relation-design.md](keying-relation-design.md) §6, and does not restate
its model.

**Provenance.** Sections 1–6 are measured in this worktree by the commands they
name. Section 4a (pipe-position lowering), the branch-phase state in §1a, and the
open questions in §9 are **relayed from `deep-ant-102`**, whose own write-up is
[the artifact linked in its 2026-08-25 handoff]; they are attributed rather than
restated as measurements taken here. `crisp-crab-430` owns #8282 and is reporting
separately. deep-ant-102 publishes a calibration list of six same-day withdrawals
and characterises five of them as "two verified facts joined by an unverified
arrow" — DESIGN's *authority substitution* class — so relayed items are marked and
each names the fixture that would re-derive it.

**Instruments, per the 2026-08-24 ruling — every count below names the command or
commit that re-derives it, and none is a standing figure.**

---

## 1. What happened

Three generations, all from `calm-ram-435`, all attempting to delete `import`:

| gen | PR | opened → ended | base | how it ended |
|---|---|---|---|---|
| 1 | #8084 | 08-09 → 08-10 | `session/valiant-boar-65` | closed after one day |
| 2 | #8115 | 08-10 → 08-24 | main | **quarried** 08-15, frozen at `d35b4fec309` |
| 3 | #8282 | 08-15 → open | main | **frozen** 08-25, kept as offline oracle |

Gen 1 attempted incremental deletion stacked on another session's compiler
branch. Gen 2 attempted it on main and was quarried in favour of "a separately
staffed clean namespaced-subject cutover PR built from current main" — which is
gen 3. Gen 3 completed the source axis (imports 21258 → 1, parse-clean,
whole-corpus preparation at zero) and froze on the emission axis.

### 1a. Measured state of the branch (relayed, `deep-ant-102`, at `383c28f`)

| phase | branch | main at the merge-base `c271b7582` |
|---|---|---|
| parse | **FAIL** — `src/v2/extdeps/coercion_widening.dag:29:49`, pre-existing at `4e8f5b4d9b8`, not merge-manufactured | OK, 3972 |
| regen (v2 self-compile) | **REFUSED** — 410 hard diagnostics across 64 `.dag` files | `first_generation_equal=true` |
| v2-emission | refused, downstream of the one unparseable source | clean |
| witness floor | **never run on this branch** — entirely unmeasured | `failed=0` |
| seed candidate | 13 rustc errors (down from 2958 same-day) | n/a |

Two corrections this table forces on any summary of #8282, including earlier
paragraphs of my own: **"13 errors" is the seed candidate rustc count only** — the
branch's regen phase is refused at 410 — and the 410 is branch-caused, not
inherited, because main's regen runs a v2 self-compile on every push and passes.

**Two different deaths, one generation apart.** Gen 1 and 2 died of *approach* —
incremental deletion does not converge, which is DESIGN §3's delete-first rule
arriving as a measurement. Gen 3 died of *integration*: `git rev-list --count
--merges origin/main..origin/integration/namespace-cut` counts 222 merges of main
against roughly 380 first-parent commits of its own work. Better than one merge
per two commits of progress.

An atomic cutover fixes the second death. It does not touch the first, and it
does not touch what follows.

## 2. The integration tax was a symptom, not the disease

Reading gen 3's log, the merge churn decomposes into three named hazards, and
only the first is inherent to long-lived branches:

- **generated mirrors cannot be 3-way merged** — `124dc105fe` restores the seed
  to "its last building state" for exactly this; `74cbb4ff2d` restores four
  mirrors deleted without their install.
- **merges silently reverted branch work** — `22b2ab4fc7` ("Restore the import
  wall a main integration silently reverted"), `bcb1a55aee`, `a06a51b15d` ("three
  main commits my own merge resolution had dropped").
- **the mechanical qualifier was wrong in ~15 distinguishable ways**, each found
  only by running the compiler: let-binders, lambda binders, match-arm pattern
  heads (2428 de-qualified in one commit), field names, argument labels, type
  parameters, `^symbol` references, pipe targets, kernel container heads, string
  literals (`a03dac4200` repairs 154 corrupted literals including cited URLs and
  pinned upstream versions, wrong since `c01e52d2418`), and comment lines.

That third bucket is the disease. Every item in it is one question asked wrongly:
**given this authored text, what does it denote?** The qualifier had to answer it
by rule because nothing in the tree would answer it by construction.

## 3. The root, reasoned forward

### 3.1 From the axioms

DESIGN §3 requires each fact to live in exactly one place, and names *nicknaming*
— a second name for one concept — as the recurring violation. A qualified and a
bare spelling of one declaration are two names for one concept. That is not a
style question: since we generate from concepts, the fork duplicates into
everything derived.

DESIGN §4 says operations come from inhabitance and refusals are located, typed
mismatches — which presupposes that the thing being matched has an identity the
substrate can compare. §5 then says the stronger move is making the bad state
unwritable rather than catching it.

So the question "what is the key of this lookup" is not an implementation detail
downstream of the namespace work. It is §3 applied to identity, and it is
upstream of every name-resolution decision the compiler makes.

### 3.2 What the compiler actually does

```
// src/v1/00_core.dag
fn authored_name_at(source_indices, node) -> String {
  match node.ident_span {
    Present { value: span } => source_text_at(index: ..., span: span)
```

`authored_name_at` returns **the bytes the author typed**. It is a
provenance/rendering function. It is used as the **identity** function, as a key
into `shared_types: Set<String>`, `variant_to_enum: Map<String, String>`, the
item registry, `is_container_type`, and the use-line collector.

Re-derive the reach with
`for f in src/v1/0*.dag; do echo "$f $(grep -c authored_name_at $f)"; done`:
roughly 500 call sites, concentrated in `04_infer` and `05_emit_rust`. Meanwhile
`src/v1/04_occurrence_binding.dag` — `OccurrenceId`, `entries_by_id`,
`references_by_id`, `resolve_reference_occurrence_binding`, whose own note says
it takes **only** an `OccurrenceId` "precisely so caller-supplied span facts
cannot bypass the validated carrier" — has zero mentions in `04_resolve.dag` and
`05_emit.dag`.

The identity authority exists, is fail-closed, and is unconsumed by the two
stages that most need it. That is DESIGN's *authority substitution* failure: one
authority borrowed to answer another's question, with no relation claimed between
them.

### 3.3 What imports were actually masking — three regimes, not one

The key-space collapse below is real but it is the *third* mechanism, and an
earlier revision of this document presented it as the only one. Corrected against
the side-chat analysis relayed 2026-08-25, whose two load-bearing claims I
verified in this worktree before adopting.

**(a) Imports were graph-construction edges, not visibility syntax.** Three
separate loaders decided *which files exist in the subject* by walking import
statements: v1 test helpers, the compile loaders, and `regen_input_sources` via
`extract_import_paths`. Delete imports and the provider module is never loaded, so
the resolver correctly reports an absent declaration — reported as *name
resolution failed*, actually *the compiler was never given the provider file*.
Already extracted and merged as **#9088**, whose own framing is the one to keep:
"admitting the right files" and "resolving them correctly once admitted" are
different questions, and gen 3 needed both.

**(b) Imports were suppressing an alternate resolver regime.** This is the
sharpest finding in the investigation and it is not a keying fact at all.
`src/v1/stage0/src/cli_run.rs:8610` — verified in this worktree:

```rust
if source_declares_import_lines(&sf.content) {
    continue;
}
bare_scan_eligible.insert(file_rel.clone());
```

A file containing **any** import statement was skipped by the bare-reference
scanner. So deleting imports did not merely remove disambiguation — it
**activated a dormant whole-pool scanner for nearly every file in the corpus at
once**, whose lexical blind spots had until then had almost no population to act
on. The scanner then treated parenthesis-free lambda parameters (`t => ...`), a
module's own coproduct variants (`World`, `Measured`, `Volume`), declaration type
parameters, and local binders as external references, pulling unrelated PCB,
spatial and roadmap modules into the v1 seed closure. Guards for the immediate
cases reached main via **#9090 / #9102**; #9102's own account still describes the
scanner as hand-written Rust re-deriving parser-owned facts from raw source text,
so the terminal construction is parser-owned occurrence and binder facts, not a
second partial grammar. Beneath it sat ambient whole-pool binding — *lexical miss
→ globally unique bare symbol in the assembled pool → accept* — and a
module-path-proximity tiebreak, which together make a program's meaning a function
of which unrelated files happen to be present. DESIGN §4 rules the heuristic never
necessary in a closed system; this is that rule with a receipt.

**(c) Imports were collapsing the key space.** With imports, every authored
spelling is bare by convention. Spelling and identity coincide, so an identity
lookup keyed on spelling is *accidentally* correct at every site. **Imports were
not hiding a set of bugs; they were holding the corpus in the single state where a
whole class of key defect could not be observed.**

The three compose, and the ordering matters for anyone reading gen 3's diagnostic
counts: (a) decides what is in the subject, (b) decides what regime resolves it,
(c) decides whether the answer is keyed correctly. A count taken before (a) and
(b) were understood is a measurement of a different question.

Delete them and the class becomes observable everywhere at once. Gen 3's board
starts at 2958 candidate compiler errors, and its own body records the correct
reading: *"The cut did not break six mechanisms — it exposed one assumption in
six places, that an authored name is bare."* The Rc-placement story earlier
revisions told is withdrawn there; 2605 of 2609 mismatched pairs differing in Rc
nesting depth, in both directions, is downstream of one line — `render_node_type`
asking a bare-keyed `shared_types` about a qualified name.

`7f3b6fe035d` closed 2493 of 2529 E0308 by peeling that one lookup.

### 3.4 The precise statement, and why "peel to leaf" is not the remedy

`deep-ant-102`'s side chat sharpened the framing, and the sharper version is the
one this document adopts:

> **A consumer was given a name in a different identity domain from the one its
> key expects.**

That is strictly better than "the key was over-specified", because **the remedy
differs by consumer** and peeling is actively wrong for several:

| consumer | identity domain its key expects | remedy |
|---|---|---|
| container / algebra leaf (`is_container_type`) | declared leaf | peel to declared leaf |
| exact-declaration registry (`shared_types`, item registry) | declaration identity | carry identity — **peeling is a collision bug** |
| provider / dependency edge (use-lines) | the **defining** module | neither the leaf nor the import source |
| pipe-position callee | syntactic position | syntax-aware lowering, not a key at all |
| Rust target renderer | exact type identity | preserve identity through emission |

So the law in §5 is not "peel everything to its minimal form". It is *name the
domain the key ranges over, and make only that domain spellable in key position*.
Two consumers in the table want **more** than the leaf, not less.

`crisp-crab-430`, who executed the cut, states the domains as a closed set — and
this is the version to use, because it names one the consumer table does not:

```
AuthoredPath     the full source spelling. Good for diagnostics, nothing else.
DeclarationLeaf  the unqualified declared name.
DeclarationId    exact owner module + declaration.
KernelSpelling   the closed intrinsic vocabulary — List / Map / Set.
```

**A domain being the right one to consume is not the same as it being the right
one to have been built on.** `shared_types` is genuinely a `DeclarationLeaf` set
— `build_shared_types` folds type-summary keys and container pascal names, and
nothing in it is ever dotted — so peeling to the leaf is the correct consumption
and gunbc#9199 is right to do it. But *that a set of shared types is keyed on
bare leaves at all* is a latent collision one layer down: two modules declaring
the same leaf, one shared and one not, collapse into one answer, and no qualified
reference is needed to reach it. That row is **pre-existing on main, independent
of the cut, and not #9199's to fix** — the PR makes qualified spellings behave as
bare ones already do, which is the law's first half and strictly an improvement.
It belongs on this ledger as its own Track C item: *the registry's key domain is
correct for its consumers and wrong for its subject*, which is the shape to watch
for wherever a peel is judged correct.

`KernelSpelling` is the one that makes a uniform sweep dangerous rather than
merely imprecise: the kernel container names are a **closed vocabulary**, not
declarations, so they are neither peelable nor identity-bearing, and a rule
phrased over declarations has nothing correct to say about them.

**And the law has two halves, which is the sharpest statement of it anyone has
produced:**

> **same identity, different spelling → must behave the SAME**
> **same spelling, different identity → must remain DIFFERENT**

The first half is this branch's defects. The second is `smart-wolf-868`'s #9075,
arrived at from the opposite side — two accepted functions both spelled
`contains` requiring different emitted authorities. **A repair aimed at one half
alone breaks the other**, which is why every fixture in this program carries the
converse control, and why `qualified_last_segment` applied uniformly repairs some
sites and silently collapses others.

This also prices the leverage honestly: one file, one line — `render_node_type`
keying `shared_types` on the peeled leaf — closed 2493 of 2529 E0308. **Diagnostic
count and defect count are not the same quantity**, and any plan that sizes this
work by error counts will mis-rank it.

### 3.5 The fix on the branch is the same defect, mirrored

`qualified_last_segment(...)` peels to the bare leaf. That is now
**under-specified**: a bare leaf is not unique across modules, which is precisely
what the namespace cut exists to fix. The corpus went spelling (over-keyed) →
leaf (under-keyed), and never once sat at the minimal unique address.

The commits say so themselves — `e9a03d1ec95` calls its own sweep "uniform rather
than minimal", and #8282's body records that `4e8f5b4d9b8` has **no measured
population effect**, because a peel can be right and wrong at different sites and
so cannot be validated as a class.

Gen 2 had already ruled both directions out. Its quarry note's do-not-port list
reads: *"`module_declarations_by_segment` and suffix-based head lookup · any
second reconstruction of an already-accepted head"* — the under-key and the
over-key, named as anti-patterns a month before the emitter paid for them.

## 3a. A second, independent class — qualified callee in pipe position

**Not a keying defect, and this document does not fold it into one.** Relayed from
`deep-ant-102`, reproduced on **main** in four lines:

```dag
treatment:  a.b.f(xs: [1,0,2]) |> count      -> 0 diagnostics
control:    [1,0,2] |> a.b.f() |> count      -> method 'a' not found on receiver
                                                Container(List, Primitive(Int))
```

A qualified callee in pipe position has its **root segment lowered as a method on
the receiver**. The branch's own diagnostics match the shape exactly (`method 'v2'
not found on receiver Container(...)`). Census family 115 + 4 = **119 of the 410**;
mechanism verified by execution, individual instances not each attributed.

Three consequences:

1. It is a **main defect with a red fixture already**, extractable on its own
   merits, independent of every other item in this document.
2. It falsifies "every #8282 finding is one class". Qualification revealed **at
   least two** independent roots: identity-domain mismatch, and syntax-aware
   lowering of a qualified head. Both were masked by imports for the same reason —
   no authored name had ever been dotted — but they share a *trigger*, not a
   *mechanism*, and merging them would be the authority-substitution failure this
   document elsewhere warns about.
3. The masking observation generalises past keying: **`import` held the corpus in
   the single state where "authored name is one bare segment" was true, and every
   consumer that assumed it — for any reason — was accidentally correct.**

## 3b. A fourth consumer, and the one that is already below floor

Relayed from `deep-ant-102`, found while building a fixture to check a reviewer's
push-back rather than by any census. `src/v1/05_emit.dag`
`order_typed_call_args` (line 326, verified here) opens:

```dag
let has_unnamed = args |> any(arg => arg_name_at(...) == none)
if has_unnamed { args } else { ...reorder by name... }
```

If **any** argument is unlabelled the emitter does no reordering at all and emits
source order. Measured on main:

```dag
fn f(a: Int, b: Int) -> Int { a }
f(b: 2, 1)
    interpreter -> a == 1      (label-correct)
    emitter     -> f(2, 1), i.e. a == 2
```

**The emitted program computes a different answer than the interpreted one.** It
compiles clean, so no census surfaces it.

This belongs here for two reasons. First, it is a fourth consumer answering the
same *argument-to-formal identity* question with its own third rule — the same
shape as §3.4's table, one layer down from types into call arguments. Second,
**DESIGN §4b already names this exact specimen** as its illustration that a
class's rung is the minimum across its in-scope paths — "the interpreter refuses a
mislabeled call that the emitter reorders into positional leftovers." What was a
worked example in the doctrine now has an executed receipt, and the receipt says
the emitter does not merely reorder wrongly: it declines to reorder at all.

Pre-existing, unrelated to the cut, and explicitly **out of scope** for #9192 —
whose author is recording it in that PR's body with the measurement so the PR
cannot be read as having closed it, and enrolling the source-language half as a
regression witness that states its own non-coverage. It is a Phase 0 row here: an
independent below-floor defect with a fixture, extractable on its own merits, with
corpus-wide ordering blast radius that argues for its own lane rather than a
rider.

## 3c. A fifth consumer — one bare-name index, and a retraction worth keeping

**This section previously reported something else and was wrong.** It claimed
that renaming one private function changed anonymous-record *type resolution* in
four unrelated files, that the resulting 8 diagnostics arose at reconcile, and
that compile outcome was therefore a function of the whole corpus name set. That
claim is **retracted by its own author** (`smart-ram-730`, same day) and nothing
in this program depends on it. The retraction is recorded rather than quietly
deleted, because how it failed is itself one of the classes this document is
about.

### What is real

`v1.compiler` `build_data_body_index` indexes **every item with a body under its
bare unqualified name, across all 3927 modules, functions included,
last-write-wins.** A witness module's `fn owner` collided with a production
`data owner`, and that collision was the **sole blocker of whole-corpus
emission**. Confirmed by execution in both directions: removing the function
clears it, and filtering functions out of the index clears it.

That is this document's class exactly — a bare name standing in for an identity,
in an index whose key admits two different subjects — and it is a Track C row. It
is simply an ordinary instance rather than a corpus-wide coupling.

### What was wrong, and why it is worth a paragraph

The 8 diagnostics arise at **emit**, not reconcile, and they were **masked**.
`emit_rust`'s body is a sequence of early returns: the workflow-default check runs
first and returns on any hit, so while those 2 diagnostics stood, the
anonymous-record check **never executed**. Both classes are emit-stage; one gated
the other.

The confirming run is what makes this certain rather than merely plausible: with
the 2 cleared by a **compiler-only** patch — zero `.dag` renames, no source edit
anywhere — the **same 8 appear in the same four files**. A compiler-internal
change and a source rename produced identical populations, which is impossible if
the rename caused them. And the corroborating detail that was on screen the whole
time: emit took 43s in the baseline and 78s with the 2 cleared. **It ran longer
because it got further.**

So the 8 are ordinary pre-existing ambiguities nobody could see because an earlier
gate returned first. They still block emission and are worth fixing —
`DeclarationRef` vs `RustItemDeclarationRef` among them is plausibly this class —
but as independent ambiguities carrying none of the severity first attached.

### Why this stays in the document

This is **execution-provenance loss**, which DESIGN already names in its recurring
failure modes: *if a count of zero and an unreached stage produce the same output,
the number is not an observation.* Here the shape is one step subtler than the
canonical form — not a stage that refused, but a stage that **returned early on a
different class**, so a real emit run reported a real population that was a
*prefix* of the truth.

Two things generalize. First, **the producing stage must be read, not inferred
from a diagnostic's wording** — the whole false conclusion rests on reading
"ambiguous record literal" as a reconcile-stage message. Second, **corroboration
is not confirmation**: a second lane agreed from adjacent prior experience about
bare-name lookup collisions, which are real, and that agreement made an inference
feel verified when it had only been echoed. Both are cheap to guard and neither
was.

The methodological receipt is worth more to this program than the retracted claim
would have been, because every remaining measurement here is a staged-pipeline
measurement with the same exposure.

## 4. The realization half — and it changes the diagnosis

The v1/v2 split, re-derivable with
`grep -rhoE 'Map<String,|Set<String>' --include='*.dag' <root> | wc -l` against
`grep -rhoE 'Map<[A-Z][A-Za-z0-9_.]*,' ... | grep -v 'Map<String,'`:

| root | `String`-keyed | typed-key |
|---|---|---|
| `src/v1` | 2093 | 31 |
| `src/v2` | 52 | 327 |
| `dag` | 179 | 80 |

The obvious reading is that v2 has the discipline and v1 does not. **That reading
is wrong, and the correction is the most consequential finding in this document.**

`grep -rhoE "^pub type [A-Za-z0-9_]+ = String;" src/v1/stage0/src/` returns 48
lines over 46 distinct names (anchored; an unanchored grep returns 50/47 by
picking up indented aliases, and the anchored figure is the reproducible one —
count corrected by `deep-ant-102`, 2026-08-25):

```
Symbol  CommitSha  Sha256DigestHex  Sha1DigestHex  Sha512DigestHex  Hash
Secret  SecretName  Email  ArtifactId  IntentId  IssueId  CommentId  GistId
RunKey  SignalKey  WorkerId  WorkflowRunId  WorkflowObserverId  ...
```

`grep -rhoE "pub struct [A-Za-z0-9_]+\(String\)" src/v1/stage0/src/` returns
**nothing**.

So every branded identity scalar in the model erases to `String` in emitted Rust.
`Secret` and `String` are one Rust type. `CommitSha` and `Email` are
interchangeable. And `Map<Symbol, V>` — v2's 236-instance typed-key discipline —
emits as `Map<String, V>`.

The mechanism is not an oversight. `src/v1/05_emit_rust.dag`:

```dag
fn rust_nominal_identity_carrier_type_eligible(type_name: String) -> Bool {
  false
}
```

**The nominal-identity emission path is authored and unconditionally disabled.**

Three consequences:

Measured on the *emitted* side by `neat-fox-901`, which is the denominator that
matters for key position: **2,548 `HashMap<String,` against roughly 9 named-key
maps, every one of those a `String` alias** — and 62 transparent primitive
aliases against exactly 1 emitted newtype, which is hand-written
`v1_interpreter.rs` rather than emitted at all. The source side mirrors it and
confirms the rung split: `type Symbol` in v2 is **bodyless, with zero functions
returning it and zero casts anywhere in `src/v2`**, so a spelling cannot become a
`Symbol` in v2 source. Structurally impossible in the model, absent in the
realization, and per §4b(1) the honest rung is the minimum — *absent*. v2's
remaining 52 `String`-keyed sites are concentrated rather than diffuse: 50 of them
in three lens files (`module_impact_query` 28, `reference_deps` 12,
`module_graph` 10).

1. **v1's 2093 `Map<String, _>` are not 2093 independent acts of sloppiness.**
   They are the model faithfully describing what the realization can express —
   though *the model had no alternative* would be too strong, and review flagged
   that overreach: v2 shows source-level typed keys can exist even where lowering
   erases them, so some v1 declarations **are** genuinely under-modelled and do
   need repointing after realization. The accurate statement is a
   **model/realization co-defect whose terminal repair must start at
   realization.** Repointing them to
   typed keys while the emitter erases brands would move the fork rather than
   close it — DESIGN §4b rung honesty: a class's rung is the *minimum* across its
   in-scope paths, and citing the model path while the emission path stays silent
   is inflation.
2. **v2's typed keys are a source-level property that does not survive lowering.**
   The discipline is real in `.dag` and absent in the executing artifact.
3. This is the same class as §4b's existing `UriValidatedScalar` receipt, where a
   `sole_constructor` type's emitted mirror is a `pub` struct with a `pub` field
   that admits values the mint refuses. Brand erasure is the general case; that
   receipt is one instance of it.

**The identity work is one program with two ends**, and the emitter end goes
first — **for the parts that depend on it.** Adversarial review narrowed that
dependency, and the narrowing matters because the earlier phrasing put the whole
program behind one track:

| work | needs nominal realization first? |
|---|---|
| unify closure construction | no |
| remove whole-pool / proximity fallback | no |
| produce exact occurrence bindings | no |
| key exact registries by structured `DeclarationRef` | usually no |
| change `Map<String, V>` → `Map<SubjectKey<K>, V>` | **yes** |
| claim identity is preserved end-to-end | **yes** |
| use emitted brands as runtime map keys | **yes** |

Realization is a hard predecessor of **Track D's activation and of the rung
claim**, not of Tracks B and C as a whole: those are compiler-semantic facts and
do not require `Symbol` to lower as a Rust newtype.

## 5. The keying law, stated so it is enforceable

The observed rule — *key on the minimally unique address, nothing more* — is
sound as an observation and **undecidable as stated**. Minimality quantifies over
populations that do not exist yet: a key unique on today's corpus is not unique
after the next module lands, so any checker for it is a snapshot, and DESIGN §5's
oracle rule already forbids a measurement of the current tree standing as an
oracle.

One restatement makes it top-rung:

> **A key is the element's declared identity carrier. Nothing else is spellable
> in key position.**

**That phrasing is wrong, and it was refuted by adversarial review before anything
was built on it** (`neat-fox-901`, 2026-08-25). It is right about the two error
terms and wrong about the quantifier. Two legitimate key roles violate it:

- **A cache determinant** answers *is the prior result semantically reusable for
  this computation*. Its minimal correct form includes **every result-determining
  input** — source or semantic digest, direct dependency interfaces, compiler
  identity, target realization, any option that changes the result. That is
  deliberately *more* than the subject's identity and it is correct, not
  over-keyed. DESIGN already carries the role independently in its recurring
  failure modes — *cache impurity (key on declared-input content)* — so the
  sentence above contradicts the repository's own vocabulary, and a program
  enforcing it literally would break every cache in the tree.
- **A grouping key** partitions into equivalence classes. Multiplicity is
  *expected*, and it must never pass through a unique roster — which T1/T3's
  route through `keyed_roster_build` would force on it.

### The phrasing that survives

```
key_R(x) = key_R(y)   ⟺   same_R(x, y)
```

**Forward direction broken → under-keying** (collapse: two subjects share a key).
**Reverse direction broken → over-keying** (aliasing: one subject spells two
keys). Because **R is a parameter**, cache determinants, grouping keys and
locators are *ordinary instances* rather than exceptions carved out of a
subject-identity rule. The earlier version had to name subject identity
specifically, which is exactly why it needed a carve-out it did not have.

This is strictly better on two counts: it **generates** both error terms instead
of asserting them, and it kills the degenerate outcome where a keying program
becomes *wrap every map key in `SubjectKey`* — **the role must be classified
before the carrier is chosen.**

It is also what `keying-relation-design.md` said from the beginning — *keying is a
relation, not an `id` field*, and "the key of X" is not well formed while "the key
of X under relation R in scope S" is. Both independent derivations
under-generalized a law the ratified design already had, which is a caution worth
carrying: convergence between two sessions is evidence about a restatement's
shape, and it is **not** evidence that the restatement is complete.

Three derivations now agree, and the third makes the biconditional legible:
`crisp-crab-430`'s two halves — *same identity, different spelling → behave the
same* and *same spelling, different identity → remain different* — are precisely
its two directions, arrived at from executed defects rather than from the census.

**Occupancy is zero and the counts do not move**: exactly one genuine cache-keyed
`String` map exists corpus-wide (`census_cache` in `v2.lens.reference_deps`), and
it is outside the v1+`dag` denominator; the memo-sounding population (`seen`,
`visited`, `depth_map`, `accepted_map`, the `*_index` family) keys on declared
identities and was already classified correctly. The vocabulary gap is real even
though nothing occupies it, and it is recorded rather than papered over —
**a bucket that does not exist cannot receive a site, so its absence is invisible
in the counts and would otherwise have been cited as coverage.**

Minimality stops being a search and becomes a definition — §3, one fact one home.
**Two sessions derived that sentence independently on 2026-08-25**, from
different evidence — this lane from the emitter's `shared_types` lookup,
`neat-fox-901` from the corpus keying census — and converged on the same wording
*including the same reason for rejecting the minimality phrasing*. Recorded
because independent convergence is the only cheap evidence available that a
restatement is the subject's own shape rather than one author's preference.

### The specimen that puts both error terms in one function

`neat-fox-901`'s tightest statement of the class: `v1.04_infer`
`nominal_call_arg_brand_mismatch` — **the mechanism that exists to enforce brand
distinctness** — compares `authored_name_at` (the spelling: over-keyed) under
`qualified_last_segment` (the leaf: under-keyed). Both error terms, inside the
wall built to stop exactly this. The namespace cut is the same class one level
up, which is why peel-to-leaf greens a build and re-arms the defect the cut
exists to fix.

### The hole is *key position* specifically

`src/v2/lens/module_impact_query.dag` carries a note claiming, on a 2026-08-08
operator ruling, that `ModuleIdentity` is a record so a path cannot be passed
where an identity is required — the confusion unwritable rather than merely
labelled. **That is true at parameter position and false at key position.** The
carrier is destructured at all 7 sites (`key: module_id.name`, `key: m.name`,
`key: from.name`) into `Map<String, List<ModuleIdentity>>` and
`seen: Map<String, String>`. Typed value, `String` key, path perfectly writable.

That is worth more than its site count. It says the repo's existing identity
walls are not weak — they hold where they were aimed — and that **key position is
the aim they were never pointed at**. "Nothing but the identity carrier is
spellable in key position" is therefore not a new discipline but the existing one
extended to the single position that was skipped.

This is not new doctrine: it is the terminal shape
[keying-relation-design.md](keying-relation-design.md) already argues for, and
`dag/std/key_relation.dag` already names its own next-rung trigger,
`key_relation_identity_wall`:

> introduce `SubjectKey<K>` / `ResourceLocator<L>` / `ContentIdentity<H>` impostor
> carriers and constrain production `key_of` projections to return `SubjectKey`
> only, so a content-derived key stops compiling.

That design's §6 says "nothing here is built until the first real consumer needs
it," and lists four steps of which three have landed (Tailscale specimen,
`KeyRelation` in `std`, `membership_reconcile` converted). **This postmortem's
contribution is the consumer: the compiler, at 2093 sites, with a measured
2958-error receipt for what its absence costs.** The design note's specimen list
(§3) contains Tailscale, the dashboard, `content_hash` and `occurrence_identity`;
the emitter is a fifth and by volume the largest.

Two halves are needed and only one exists:

- **uniqueness — exists.** `std.keyed_roster` `keyed_roster_insert` refuses a
  repeated key before a second value can exist, and refuses to widen an
  already-invalid roster. Its note declares its own ceiling: *structural
  impossibility — branded `KeyedRoster` carrier whose success arm is reachable
  only through `keyed_roster_build`/`insert`*, trigger
  `feature:keyed-roster-construction-wall`.
- **what counts as a key — missing.** Every one of these carriers takes
  `key_eq: fn(K, K) -> Bool` **from the consumer**. DESIGN §4b already names this
  defeat: `std.interval` `closed_interval` accepts reversed bounds under a lying
  predicate. `key_eq` on spellings and `key_eq` on identities produce different
  rosters over one population and neither is refused.

The missing piece is that **equality is a property the key type declares, not an
argument the caller passes** — DESIGN §4 inhabitance, the same move by which
`Int.add` comes from `Int` inhabiting a ring. `String` does not inhabit a key
algebra: free text has no declared identity. `Symbol`, `ModuleId`, `OccurrenceId`,
`ContentHash`, `GitObjectId` do.

### Ladder placement, per class

| class | decidable? | ceiling | why |
|---|---|---|---|
| over-keying (spelling as key) | yes, once the identity type exists | **structurally impossible** | give identity a minted type; passing a spelling where an identity is declared stops being a lookup that returns false and becomes a type error |
| under-keying (collision) | yes, over a declared population | **structurally guaranteed** | the mint refuses the duplicate at construction; the collision has no representation |
| brand erasure at emission | yes | **structurally impossible** | a newtype has no `String` constructor; the switch is already written |
| *is the declared relation the right one* | **no** | ratchet forever | whether a workdir is per-instance or host-global is a claim about the world. Compiler errors for key **violations**, never for key **mistakes** — keying-relation-design.md §5 |

### One constraint on where it lands

`src/v1/stage0/src/std_trait_derive_shape.rs` carries the invariant, learned
expensively:

> **A module reachable by import from `src/v1` cannot import v2 anything.**
> `required_regen_host` compiles stage0 from `regen_input_sources`, whose roots
> are hardcoded to `src/v1` and `dag` — `src/v2` is not a regen root and never
> will be, because stage0 *is* the v1 seed and a seed that reached into `src/v2`
> would depend on the successor it bootstraps toward.

The key algebra is consumed by the v1 seed. **It lands in `dag/std/`, never in
`src/v2/std/`.** An intermediate revision of the trait-derive migration violated
exactly this and required-regen refused before comparing any bytes.

## 6. Terminal shapes

**T1 — key algebra in `dag/std/`.** `FinitelySupportedFunction<K, V>` and
`FinitePowerSet<T>` admit only inhabitants of a declared key algebra. `key_eq`
ceases to be a parameter anywhere in `std.keyed_roster`, `std.key_relation`,
`std.change`. `Map<String, _>` remains spellable only where the domain genuinely
is free text — **and that carve-out is 0.35% of the compiler population, not a
comfortable residue.** `neat-fox-901`'s census (→ [keying-census.md](keying-census.md),
gunbc#9202) measures 8 `FreeText` sites in 2272, all of them in `extdeps/`,
against 838 `ResourceLocator` and 817 `SubjectKey`-as-text. An earlier revision
of this sentence read as though free text were a broad escape hatch; on
measurement it is a rounding error, and the sentence is corrected rather than
left to be read generously.

Two justifications sit under those 8 and only one of them is "unstructured":
POSIX env names and upstream GCP IAM attribute names are external vocabularies
this repo does not own, while the `ts_keyword*` rows are the sharper case — **the
key IS the spelling and the spelling IS the identity under the upstream grammar
relation.** That is a *correct* `String` key, not a tolerated one, and T1 should
name the upstream-vocabulary case explicitly because its justification differs
from a filename's.

**T2 — impostor separation.** `SubjectKey<K>` / `StateRevision<R>` /
`ResourceLocator<L>` / `ContentIdentity<H>` / `DisplayLabel`, with production
`key_of` returning `SubjectKey` only. This is `key_relation_identity_wall`,
**two arms, not five, and entirely unbuilt** — measured by `neat-fox-901`:
`SubjectKey` / `ResourceLocator` / `ContentIdentity` have **zero declarations** in
the corpus and exist only inside prose strings in `key_relation.dag` itself, as do
`KeyMultiplicity` and its arms, and **zero `src/v2` modules import
`key_relation`**. What landed is a naming authority, which its own note says
plainly. Nothing here is half-done; it is undone, and the trigger is the real
part. When built, a content-derived key stops compiling.

**The first landing is `SubjectKey` and `ResourceLocator` only.** The census finds
817 and 838 sites for those two and **zero** for `StateRevision`,
`ContentIdentity` and `DisplayLabel` in this population — `ContentIdentity` has an
obvious consumer one layer over in `std.content_hash`, but none here. Per
`keying-relation-design.md` §6's own rule that nothing is built until a real
consumer needs it, authoring five arms would be authoring three carriers ahead of
any consumer. The two that land are not interchangeable at a single site, so both
do real work immediately.

**T3 — `KeyedRoster` construction wall.** **Only for relations declared unique.**
A grouping relation expects multiplicity and must not pass through a unique
roster at all — `keying-relation-design.md` already states this as multiplicity
being *declared, not discovered*, with `keyed_roster_build`'s duplicate refusal a
legitimate policy for relations declared unique rather than a law imposed on
relations that were never unique. Success arm reachable only through
`keyed_roster_build`/`insert`. Already declared as
`feature:keyed-roster-construction-wall`.

**T4 — a sealed carrier survives emission sealed.** A `sole_constructor`
declaration emits a Rust carrier that cannot be forged: private field, no
derived `Deserialize`, construction only through the mint. This subsumes §4b's
`UriValidatedScalar` forgery receipt, which is the single below-floor item in
that paragraph.

*T4's subject was corrected on 2026-08-25 and this row states the corrected
terminal, not the superseded one.* It previously read
*`rust_nominal_identity_carrier_type_eligible` returns a real predicate; a
branded scalar emits as a newtype* — which contradicts §7a's A0 correction, where
that predicate is **deleted** rather than repaired, and where the reason is not
scheduling: `type Brand = String` admits `take_string(s: b)` with zero
diagnostics on unpatched main, so brand transparency is a **model** fact and the
emitter follows the alias RHS because there is nothing else to follow. Brand
rendering is therefore blocked upstream on a model fact that does not exist, and
naming it as a terminal here would have been a second authority for one
construction — the §2/§3 violation this document spends §11 recording. What
reaches emission at the cut's terminal is the seal, which is already authored on
79 production declarations and read by the emitter zero times.

**T5 — one identity authority in the compiler.** `04_occurrence_binding` is the
sole answer to *what does this reference denote*. `authored_name_at` survives with
its honest job — rendering and diagnostics — and is unreachable from any lookup
key. `build_emit_graph_info` folds on declaration identity with a collision arm
instead of `Map<String, TypeSummary>` with silent last-write-wins.

**T7 — the migration unit is an occurrence, not a string.** Gen 3's transformer
qualified against the module where a name was *visible* rather than where it was
*declared*: `Present` is imported into `v2.std.collection` and declared in
`v2.std.optional`, and the cut produced `v2.std.collection.Present`. That is not a
resolver gap — it is a binding-preservation failure in the transform. So the
migration unit is not `import statement + matching strings` but
`reference occurrence -> exact declaration identity`, and the oracle is a ledger
the *current* resolver mints while imports still work:

```dag
type OccurrenceBinding {
  occurrence_id:     OccurrenceId
  source_module:     ModuleId
  source_span:       SourceSpan
  authored_spelling: String
  binding_basis:     ReferenceBindingBasis
  declaration:       DeclarationRef
  declaring_module:  ModuleId
  visible_through:   VisibilityRoute?
}
```

The load-bearing separation is `declaring_module` (where the declaration
structurally lives) against `visible_through` (the import or re-export surface
that made it visible today). **The qualifier comes from `declaration`, never from
`visible_through`.** And `ReferenceBindingBasis` names the two regimes §3.3(b)
exposed — `AmbientPoolBinding` and `HeuristicNearestModuleBinding` — so they
become counted migration debt that new instances refuse, rather than an unnamed
fallback. This is `04_occurrence_binding`'s existing carrier grown two fields, not
a new authority.

**T6 — namespace cutover.** `import` grammar, AST and name-universe deleted;
resolution by containment; qualification is a corpus-migration decision only. Gen
2 proved the producer already handles both spellings — `module_named_by_qualifier`
admits exact path and suffix, and `v2.std.live_tree.LiveTreeDisposition` and
`live_tree.LiveTreeDisposition` resolve to the same terminal declaration by
execution.

## 7. Landing order

### 7.0 First: this does not defer the deletion, and here is the test

DESIGN §3 is delete-first, and the 2026-08-15 operator ruling is explicit that the
grammar goes up front. **The ordering below must not be read as re-litigating
that, and there is a mechanical test for whether it does:** does any phase below
require `import` to still exist, or restore a capability the deletion removed? No
phase does. Every one of A–C is independently correct on a corpus that still has
imports and on one that does not.

What is being sequenced is not *when to delete* but **where the census output
lands**. The deletion already ran and already produced its census. #8282 is frozen
and its ~5600 commits will never merge, so every fix currently sitting on it is,
by DESIGN §6, work presumed thrown away. Extracting to main is the only motion
that makes the census's output survive — that is *banking* the deletion's result,
not deferring the deletion.

The dependency inside the extraction is not a preference. Gen 3 ran T6 without T5
and paid 2958 errors; T5 without T4 would repoint 2093 model-side keys onto types
that erase to `String` one layer down, satisfying a lens while the realization
keeps lying.

**Phase 0 — the two already-red mains, in parallel with everything below.**
Neither depends on any other phase and both have a discriminating fixture today:
the pipe-position lowering of a qualified callee (§3a — the fixture is four lines
and already red on main), and the parse failure at
`src/v2/extdeps/coercion_widening.dag:29:49`, which is pre-existing on the branch
at `4e8f5b4d9b8` and blocks its regen and v2-emission phases outright. Add the emitter's
argument-ordering divergence (§3b) — pre-existing, below floor, its own lane. Land
these first because they cost nothing to sequence and the pipe class alone is ~119
of the branch's 410.

**Phase A — realization (v1 emitter, `src/v1/05_emit_rust.dag`).**
Land T4. Turn `rust_nominal_identity_carrier_type_eligible` into a real
predicate; emit branded scalars as newtypes. Green on main today, no dependency
on anything else here, and it is the only phase whose absence silently
invalidates the others. Its discriminating RED already exists as §4b's
`UriValidatedScalar` receipt. Sequenced first *because* it is invisible to every
model-side check.

**Phase B — substrate (`dag/std/`).**
T1, then T2, then T3, in that order — each is the next-rung trigger of the one
before, and all three are already declared with dissolution conditions. Lands in
`dag/std/` for the seed-closure reason in §5. `TotalMap`/`TotalPolicy` have zero
code consumers today and stay unauthored-against until `table_decision_tree`
lands; the uninhabited 2×2 corner stays uninhabited.

**Phase C — compiler (v1, then v2).**
T5. Repoint the 12 censused emitter sites (#8848), starting with
`build_emit_graph_info` — it is upstream of `build_shared_types`,
`derive_variant_to_enum` and `is_dag_value_type_name`, so none of those is
repairable above it. Recover the **29/29 `ReferencePath` acceptance controls**
from `backup/8115-import-deletion-quarry` @ `d35b4fec309` as the enrolled
evidence rather than re-authoring them; under §4b(4) they stay enrolled after the
climb. The eight un-extracted #8282 emitter commits are the **site census** for
this phase, not its patches — they peel to bare leaf, which T1 makes unspellable.

This phase is green on main *with* imports (bare spelling → same terminal
declaration) and green after the cut (qualified spelling → same terminal
declaration). That is the whole point of its placement: it lands as ordinary PRs
against main, carries no integration debt, and makes the cut a no-op for the
class that killed gen 3.

**Phase C+ — the derived import-free projection.**
T7, and it replaces "re-derive the cut mechanically" as the vehicle. Do **not**
commit a mass qualification. Generate an overlay under `target/` from the binding
ledger: parse current source, drop import declarations in the derived
representation, rewrite only reference occurrences whose exact declaration is
non-local, use the declaration's canonical path, and touch no binder,
declaration, shadowed use, annotation or string literal. Emit a receipt mapping
each changed occurrence to its old and new binding.

This is a **lens over one authority, not a second authority** — main's source stays
imported, the projection is mechanically derived and authors no semantic fact of
its own, so it is not the parallel-representation debt DESIGN §2/§3 forbids. Its
acceptance invariant is the one a compile cannot give you:

```
for every surviving occurrence:  old_resolved_declaration == projected_resolved_declaration
```

A projection that merely compiles is insufficient: it can compile while silently
binding a different `Node`, `Present`, `Connective` or `Cardinality`. Gates, all
on current main: projected imports 0 · projected parse failures 0 ·
unresolved/ambiguous projected occurrences 0 · **binding-identity changes 0** ·
`AmbientPoolBinding` 0 · `HeuristicNearestModuleBinding` 0 · closure edges without
occurrence provenance 0 · v1 candidate rustc errors 0 · v2 self-compile hard
diagnostics 0 · generation 1 == committed candidate · generation 2 == generation 1.
Plus the perturbation falsifier gen 3 itself derived: *adding or removing an
unrelated source module may not change an entry's closure, bindings, ambiguity
results or visible services* — which is the direct control for §3.3(b).

**Phase D — the cut (one motion, from a fresh main SHA).**
T6. Fresh branch from the exact current main SHA, in a temporary worktree rather
than a maintained PR (§8). Regenerate the projection once from that SHA, apply the
rewrite mechanically, flip production resolution and closure authority, delete the
import grammar/visibility/graph routes *and every ambient fallback path* in the
same change, regenerate stage0, run the full acceptance set, merge without
absorbing a newer main — and if the landing window is lost, **restart from a fresh
SHA rather than integrating**. Gen 3's receipt for that last clause: dozens of
import-bearing files merged with no conflict at all, because git reports
overlapping text and cannot report that cleanly-merged content violates the new
language invariant.

By this point the emitter class is closed by construction, the pipe class by Phase
0, and the projection has already proven the rewrite binding-preserving on main —
so the cut's remaining failure modes are the ones the ratified plan's step 5 always
intended to be fixing forward on.

**v1/v2 split, stated plainly.** A, C and D are v1 seed work — that is where the
2093 sites, the emitter and the regen closure live. B is `dag/std/`, shared. v2
needs no repointing: it already keys on typed identity in source and gets a
correct realization for free once A lands. The seed shrinks toward zero on
schedule; none of this cements Rust into templates.

## 7a. The PR sequence, derived backward from the goal

Operator framing (2026-08-25): work backward from the goal, and **the grammar
deletion is the last step, not the first**.

**This reverses a ratified ordering, and the reversal is recorded here rather
than absorbed into a re-derivation of the root.** The ratified
[namespace-cut replacement plan](namespace-cut-replacement-plan.md) states its
root verbatim as *"the `import` concept — the grammar production, the parse
surface, and the import-name universe"*, under an operator ruling of 2026-08-15:
*"delete all the grammar/import up front, then solve each problem as it is
revealed."* An earlier revision of this paragraph reconciled the two by asserting
that the root "is import-derived resolution and closure, not import syntax" —
i.e. by narrowing the ratified root while also saying that plan was not
superseded. That is the move review 55844 correctly refused: a root narrowed in
prose converts delete-first into deferred cleanup, and does it invisibly, because
both documents then read as agreeing.

The actual situation is simpler and is stated as such. **The root is unchanged —
grammar, parse surface and import-name universe are all inside it.** What changed
is the ORDER in which that root is cut, by a later ruling from the same authority
(2026-08-25, recorded above; the same exchange also ruled the cut "doesn't have
to be atomic technically"). The 2026-08-15 ruling is superseded on order and on
nothing else. Neither this document nor the replacement plan may claim the two
rulings agree.

**What the reordering costs, stated because a supersession that reports no cost
is the tell that it was not priced.** Deleting the grammar first makes the census
a single refusal storm — every real dependent refuses at once, which is exactly
the property DESIGN §3 credits delete-first with. Deleting it last spreads that
census across the sequence and makes each consumer's move *individually*
reversible, which is the property that lets an eleven-PR sequence land without a
dual-authority interval on main. The exposure that buys is the one §3 names: while
the grammar stands it is an attractor, and every question asked near it gets
answered in import vocabulary. The bound on that exposure is the invariant below
plus a terminal that names grammar deletion as a required step rather than a
cleanup item — if the sequence completes and the production still parses
`import`, the cut did not complete.

> **For each production semantic contract, exactly one producer supplies every
> production consumer. Alternative producers may exist only as derived,
> non-production observations. The producer changes once.**

Dual authority is what delete-first forbids; inert syntax is not. The shape this
licenses is what makes incremental landing safe: consumers move one PR at a time
onto a common typed relation — `DependencyFacts` — whose **body** stays
import-derived throughout preparation, with the reference-derived body existing
only as a comparison receipt, and the body swapping once at F. Consumers may be
adapted individually because they still receive **one** authoritative answer.
What must never exist is `compile → reference closure` while
`regen → import closure` and `topological_sort → import edges`: multiple live
answers to one question, even with no individual consumer carrying a fallback.

An earlier revision of this invariant said *each repoint moves one consumer all
the way across*, which is **insufficient**, and the objection is the side chat's:
per-consumer atomicity still permits two simultaneously-live consumers to
disagree with each other before the authority flip. Each is internally coherent;
they answer the same question differently. That is not a hypothetical — it is
#9088 exactly, where regen's subject was walked from imports while other
consumers derived theirs elsewhere, so a green regen was green over a different
corpus than the one it claimed.

The consequence for the wave order is concrete: **B.1 is not one wave-1 item
among several, it is what makes per-consumer repointing safe at all.** Once
closure has a single authority, consumers cannot disagree about it by
construction, and a repoint becomes a local change rather than a straddle. Every
fact repointed after B.1 inherits that property; anything repointed before it
must move all of that fact's consumers in one PR.

### The backward chain

| | for this to be true… | …this must already hold |
|---|---|---|
| **G** | `import` grammar/AST deleted | nothing reads an import |
| **6** | nothing reads an import | every import consumer repointed to references |
| **5** | consumers can repoint | the corpus resolves without imports |
| **4** | the corpus resolves without imports | the spelling migration was binding-**preserving** |
| **3** | the migration is binding-preserving | an exact occurrence→declaration ledger exists |
| **2** | the ledger is truth, not a guess | ambient/whole-pool + proximity fallback is gone |
| **1** | those delete safely | one closure authority feeds every subject |
| **0** | any of it is measurable | every measurement names the stage that produced it, and no stage silently returns a prefix |

Step 0 survives §3c's retraction in a weaker and more general form. The original
reading — *a rename in an untouched file changes diagnostics elsewhere* — is
withdrawn. What remains is what the retraction itself demonstrated: **`emit_rust`
returns early on the first failing class, so a real run reports a population that
is a prefix of the truth**, and every gate in Waves 4–5 is a staged-pipeline
measurement with exactly that exposure. The requirement is therefore not a new
mechanism but a property of the gates: each names the stage that produced it, and
a gate that could be short-circuited by an earlier class says so. That is
`E.2`'s obligation, not a Wave-1 blocker.

### Tracks

**Track 0 — independent, no ordering cost.** 0.1 pipe-position lowering (§3a,
fixture red) · 0.2 `order_typed_call_args` (§3b) · 0.3 `build_data_body_index`'s
bare-name index (§3c) · 0.4 the `coercion_widening.dag:29:49` parse failure.

**0.3 does not close in this document's sense when its PR merges.** It arrives as
a PR handoff — gunbc#9207, open and mergeable — and that PR's body is explicit:
**"this is a population fix, not an identity fix."** A data body index containing
function bodies is wrong on its own terms whatever the keying scheme, and
filtering it to data items closes that. **The key is still a bare name**, so two
*data* declarations sharing a spelling still resolve last-write-wins. The
population half closes at 0.3; **the keying half stays open under Track C** —
recorded here so Resolution does not inherit a wave item it believes is finished.

**And its review found something the program should carry: this class of fix
silently self-reverts under ordinary branch hygiene failures.** #9207 sat at
REQUEST_CHANGES because a patch applied from a stale working copy clobbered main's
newer `05_emit_rust.dag`, and what it silently reverted was *another fix of this
same class* — `emit_data_def`'s `needs_rc` routed back to a bare-spelling key,
with the `preserves_declared_brand` branch dropped. Nothing detected it; it
surfaced in review.

That is worth a row rather than a note, because the vulnerability is structural
and specific to this program's shape. **An identity-keying repair is typically one
line, in a large file, changing which key a lookup consumes — semantically
enormous and visually indistinguishable from noise in a whole-file diff.** So the
ordinary hygiene failure (copy a file across branches) reverts exactly this class
in preference to others, and the reverted state still compiles, still passes, and
reads as the file's normal content. Two consequences: **every fix in this program
needs its discriminating RED enrolled, not just authored** — an enrolled red
refuses the revert while a merely-authored fixture does not — and the whole-file
copy is a specific hazard to name in each lane's brief, not general advice.

Its measurement carries a generalization worth more than the fix. Clearing the two
blocking diagnostics revealed 8 pre-existing ones behind them, so **the corpus's
blocking population is not knowable from one run — clear one class, discover the
next, and every intermediate count reads like a total.** That is M0 stated from
the measuring side, and it is why M0 is a proof barrier rather than a checklist
item.

**Track A — realization. NOT a one-line switch**, and an earlier revision of this
document said it was. The dormant path is real — the predicate has 5 call sites
waiting and `rust_nominal_identity_carrier_def` directly above it already emits
`pub struct Name(pub String)` — but adversarial review found three reasons it is
a skeleton rather than a flip:

1. **The predicate takes a `String`.** Implementing it as a name whitelist would
   **recreate the exact defect this program removes.** Terminal eligibility
   consumes an exact declaration or an already-derived carrier fact, never an
   authored or bare name.
2. **The dormant newtype's derives do not include `Hash`.** A carrier used as a
   Rust `HashMap` key needs an explicit hash story before Track D can key on it.
3. **`pub struct Name(pub String)` preserves a *brand*; it does not make a
   *validated scalar* unforgeable.** The claim that A subsumes §4b's
   `UriValidatedScalar` forgery class is therefore **too broad** — a brand
   carrier and a validated carrier have different construction contracts, and
   only the first is delivered here.

The track splits, which also removes a hidden A↔C cycle:

**Name collision, live until the other tree republishes:** the observation tree's
document also has a "Track A", meaning *absorb the census into main*. This one
keeps the label — five items are sequenced against the A0/C0/A1/D decomposition
and the letter is load-bearing inside it; theirs is being renamed to *census
absorption*. Until then an unqualified "Track A" across the two documents is
ambiguous; cite this one as **Track A (realization)**.

```
A0  the Rust emitter consumes `sole_constructor` — a carrier's construction
                        contract is derived from its declaration, so a
                        cross-module forge does not compile.
C0  semantic carrier identity — exact declaration → carrier kind.
                        No String-name whitelist, and no eligibility
                        predicate at all.
A1  gap 2 + gap 3     — `Hash` on the carrier derive; the two carrier kinds
                        kept distinct with different construction contracts.
D   repoint production key relations                       [gated on A1]
```

**A0's subject was corrected on 2026-08-25, and the correction is recorded
rather than silently applied because the earlier specification was mine and it
was unbuildable.** A0 previously read *target capability — emit a distinct
nominal carrier … no production eligibility yet*, with activation deferred to
A1. `silent-eagle-146` measured that specification and it does not stand, in
two independent ways.

First, **activation alone emits a dead struct.** Patching the eligibility
predicate to admit one fixture name emits `pub struct Brand(pub String);` while
*every reference position still renders `String`* — `resolved_type` follows
`.inferred` to the alias RHS, so the brand name is gone before the emitter sees
the use site. The emitted crate still `cargo check`s. That is not a partial
win: it is an unreferenced decoration that looks like a win in a diff, which is
§4b's worse-than-absent shape arrived at from a new direction.

Second, and this is what moved the subject rather than merely repairing it:
**the brand arm has no model fact behind it.** Measured on unpatched main,
`type Brand = String` admits `take_string(s: b)` with **zero** diagnostics.
Brands are transparent *in the model*, not merely erased in Rust. So gap 4 is
not a defect beside the erasure — it is the erasure's mechanism, and the
emitter follows the alias RHS because in the model there is nothing else to
follow. Building brand rendering would be the realization half of a model fact
that does not exist.

What replaced it was already sitting in the corpus. `sole_constructor` is
authored on **79 production type declarations** — `grep -rnE '^\s*type\s+[A-Za-z0-9_]+\s+sole_constructor' dag/ src/v2/ --include=*.dag | grep -v '^dag/test/' | wc -l` —
parsed (`grep -c sole_constructor src/v1/02_parse.dag` → 17), consumed by
inference (`grep -c sole_constructor src/v1/04_infer.dag` → 12), and read by the
emitter **zero** times: `grep -c sole_constructor src/v1/05_emit.dag
src/v1/05_emit_rust.dag`. The zero is the claim; the other three size it.

*(That count was published as 176 for about twenty minutes on 2026-08-25 and
is corrected here rather than quietly restated. 176 is the raw occurrence count
of the token; **21 of those sit inside `data …: String` prose rows** that merely
discuss `sole_constructor`, and the remainder are references rather than
declarations. The declaration count is 79. The error is the one §11.2 records
immediately below — a population derived by grepping a corpus rather than by
asking the producer — committed by this document's author while writing the
section that names it, which is why the section says the discipline is not
knowledge but a habit.)* The consequence is visible in the
committed mirror:

```rust
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UriUnicodeScalar   { pub cp: i64 }          // plain record
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UriValidatedScalar { pub admitted_cp: i64 } // sole_constructor
```

Byte-identical. So the validated arm's defect is not that it fails to emit
nominally — it already does — but that the nominal carrier is **forgeable**,
and this is the single **below-floor** item in DESIGN.md's §4b paragraph, the
one with an executed receipt reaching produced output. Everything else there is
a decidable gap in a working wall.

Three things follow, and together they are why the correction improves A0 rather
than shrinking it. The eligibility predicate is **deleted**, not repaired, so a
spelling roster has no name in scope to be written against — construction rather
than discipline. The fact is already in production on 79 declarations, so
`compile_dag_rust_emit_check`, which *does* run in the required floor over
emitted bytes, sees the change directly: no dependency on `src/v1/tests/claim`
(which the floor does not run), no floor-root widening, and no §4b(2) trigger
naming the namespace cut. And **the inertness proof inverts** — regen divergence
0 was to have been the evidence that A0 was safely dormant; under the corrected
subject it is the evidence of *failure*, because if the emitter consumes the
fact and nothing changes, the fact did not reach emission.

The standing requirement on A0 is therefore a measurement before a shape: the
production fan-out of private-field-plus-no-`Deserialize` across 79
declarations is the delete-first census, and a forging call site it breaks is a
finding rather than an obstacle — never repaired by widening what the mint
admits, and never by an escape hatch. A.2 the `UriValidatedScalar`
forgery becomes A.1's enrolled regression control (§4b(4)). *Precedes every key
repointing, because it is invisible to model-side checks.*

**Track B — one closure authority (chain step 1).** **B.1 is the hard
prerequisite of the whole program**, per the per-fact invariant above — route
every subject construction through one authority — ordinary compile, test helpers, floor
preparation, generation, emitter entry (#9088 did regen; the rest remain).
B.2 every closure edge carries source occurrence + exact provider identity.
B.3 replace the raw-text reference scanner with parser-owned occurrence and
binder facts, which #9102 explicitly leaves open.

**Track C — identity through the pipeline (chain steps 2–3).** C.1 delete
whole-pool-unique and nearest-module fallback. C.2 `build_emit_graph_info` folds
on declaration identity with a collision arm — **first**, since three of the 12
censused sites are computed from it. C.3 the remaining sites per §3.4's
domain/remedy table. C.4 the `OccurrenceBinding` ledger (T7).

**Track D — substrate, parallel with C, gated on A.** D.1 key algebra in
`dag/std/` → D.2 impostor separation (two arms) → D.3 `KeyedRoster` construction
wall. **Sized in decisions, not sites: ~238, not 2093 — and CONTESTED.** 238 distinct binding
names cover the 1958 named sites; the top name is 37% of them and the top five are
61%. `source_indices` (726) plus its abbreviation `si` (106) is 832 sites that are
**one** `Map<String, NewlineIndex>` threaded through the compiler by parameter
passing — one keying decision replicated by threading, not 832 decisions. 132
names occur exactly once. Pricing this track per site overstates it by more than a
third. **Contested by the census's own adversarial review**: 832 threaded
positions may still each need editing even though they express one decision, so
238 may be the flattering denominator rather than the honest one. Treat it as an
upper bound on *judgments* and a lower bound on *edits* until that resolves.
A second open judgment moves the largest bucket: whether path-as-key for
`source_indices` (838 sites, 36%) is a `ResourceLocator` or is genuinely the
file's identity under the filesystem relation.

**Track E — the migration (chain steps 4–5).** E.1 the derived projection.
E.2 the acceptance gates, each naming its producing stage per step 0. E.3 the
perturbation falsifier the branch itself derived — *adding or removing an
unrelated module may not change an entry's closure, bindings, ambiguity results
or visible services*. It stands on its own merits as the control for §3.3(b)'s
ambient binding, which is measured and unretracted.

**Wave 4 is sequenced behind a PR this lane does not own, and that is correct.**
M0 — *no gate may report a prefix of the truth as the truth* — and E.2's
acceptance gates are closed by **a required phase that actually emits over a
closure**, which is `cool-hawk-324`'s gunbc#9203 in the observation tree, not by
anything on this side of the implementation/observation seam. DESIGN already
declares that class as sitting **outside the ladder**, with an uncounted and
unbounded population. The seam's normal direction is *we fix, they gate*; this is
the one required dependency running the other way. **The response is to consume
their phase, never to build a second emitting gate to escape the wait** — a second
gate would be the parallel-authority debt this document is about, reached by
impatience. Needing it earlier than they plan to land it is an escalation, not an
engineering decision.

**Track F/G — one semantic motion, then the corpse.** **Corrected by adversarial
review; this was the sequence's largest defect.** An earlier revision had F.1 flip
the authority, F.2 then repoint "the last import readers"
(`source_visible_names`, `resolve_module_imports`, v2 `admit_imports`, translate,
body-lowering, `topological_sort`'s ordering), and G delete the grammar. That is
incoherent:

> **If any F.2 reader remains reachable and affects a production result, the
> authority transition did not finish in F.1. If it is unreachable, it should be
> deleted as corpse — not repointed.**

So the rule the operator's grammar-last framing actually licenses is narrower
than what was written:

> **Grammar deletion may safely trail the atomic cut. Live import readers may
> not.**

```
F — one semantic motion
    apply the binding-preserving rewrite
    switch the single resolution/closure PRODUCER
    move every live ordering / lowering / emission consumer to the derived facts
    delete the old import-derived producers and all ambient fallbacks
    retire import-semantic diagnostics and their witnesses
    regenerate

G — immediate, bounded corpse deletion
    delete the import token, grammar, AST variant and parser production
    delete unreachable import structures
    retire the syntax-acceptance compatibility control
    regenerate
```

There is no post-F "last reader repoint".

**And the trailing interval must be fail-closed.** If G trails F, a state where
`import` is *accepted silently and ignored* may not merge — that is fabricated
successful no-op syntax, DESIGN §5 exactly, and it is the one shape this program
would be embarrassed to ship. Either remaining import syntax produces an explicit
temporary **`ImportSyntaxRetired`** refusal for the interval, or **G rides the
same merge train as F**. Dead parser machinery may trail; a silently-accepted
corpse may not.

**Terminology, because the alias is what kept the F/G question confused:** stop
saying *big-bang deletion*. It names two events with two different rules —
a **big-bang semantic cutover** (F: one motion, no straddle, every live consumer
moves) and a **bounded grammar cleanup** (G: corpse removal, may trail, must
refuse rather than ignore). Every ruling about "big-bang" in this document's §8
is about the first.

**Corpse needs a behavioural definition, not grep-zero.** *Nothing calls this
import function* is necessary and insufficient: import syntax can still reach
occurrence allocation, authored ordering, candidate iteration,
source-visible-name populations, topological order, diagnostics, cache keys,
generated output and stable semantic ids. The criterion is

```
semantic_result(P) == semantic_result(add / remove / reorder / rewrite every import declaration in P)
```

over exact occurrence→declaration bindings, dependency edges, closure, reconcile
decisions, diagnostic identities, target output and generation result — source
positions may move; declaration identities and diagnostic causes may not. The
implementation form: **at the end of F, `ImportDecl` is outside the transitive
dependency cone of denotation, closure, ordering, inference, emission and
semantic diagnostics.** Then G is genuinely bounded, and **if G changes any
semantic fixture beyond turning accepted syntax into a parse refusal, F was
incomplete.**

**The last import's witness retires with F, not with G.** Also corrected here.
`dag/test/claim/import_shadowed_by_local_definition_witness_test.dag` holds the
last import in the corpus and its import is its subject: it witnesses an explicit
import silently discarded by a same-name local definition, the class main made
refusable in #9166. `crisp-crab-430` states this as *one import must survive any
cut*; an earlier revision of this document put its retirement at G. Both are
wrong for the same reason — **it is not a grammar witness, it witnesses a
semantic import-resolution decision.** Once F makes imports semantically inert
that subject has retired, so the `ImportShadowedByLocalDefinition` diagnostic and
its witness retire *with F*; a parser-only *imports remain accepted during the
cleanup interval* control may survive to G. Waiting for G leaves either a live
semantic import consumer after the claimed flip, or a passing witness whose
subject no longer exists — the evidence-outliving-its-subject failure DESIGN §3
step 6 names, reached from the direction that looks most conservative.

### Two decisions taken (operator, 2026-08-25)

- **Target spelling — FINAL FORM (ruling v2, 2026-08-25).** The canonical
  production spelling is the **fully qualified DECLARING identity**.
  `visible_through` is provenance and is **never** the qualifier — that is the
  binding-preservation failure #8282's transformer made, and naming it in the
  decision closes it by construction rather than by diligence. **Shortest unique
  suffix is quarry evidence for a possible DISPLAY format, not a live option in
  this program**, revisitable only under a later operator ruling after this
  program closes. Nobody optimizes spelling while the cut is underway.
- **The projection stays derived, never committed.** From the general rule the
  operator states, which is worth recording past this case because it decides
  more than one question here: **commit something only if it is not cheaply
  derivable.** That is §2 read from the storage side — a committed copy of a
  derivable fact is a second representation with no authority, and it is the same
  reasoning that bankrupted `docs/probes/`. It also settles, without a separate
  argument, why the projection is a lens rather than a branch.

## 7b. Milestones, waves, and the three lanes

### Milestones are verifiable states, not activities

| | milestone | the check that closes it |
|---|---|---|
| **M0** | **Measurable.** No gate can report a prefix of the truth as the truth | every acceptance count names its producing stage; a gate reachable only past an earlier early-return declares that |
| **M1** | **Emission preserves identity.** A branded scalar is a distinct Rust type | a `Secret` cannot be passed where a `String` is expected in emitted Rust; `UriValidatedScalar` forgery refused |
| **M2** | **One closure authority.** Every subject is built the same way | no second `extract_imports`; every closure edge names its occurrence and provider |
| **M3** | **Identity survives the pipeline.** Resolution's answer reaches emission intact | the 12 censused emitter sites key on declaration identity; `build_emit_graph_info` has a collision arm |
| **M4** | **Keys are typed.** A spelling is unspellable in key position | `key_eq` is not a parameter anywhere; a content-derived `key_of` does not compile |
| **M5** | **The projection is green.** A binding-preserving import-free corpus exists, derived | `old_resolved_declaration == projected_resolved_declaration`, 0 binding-identity changes |
| **M6** | **Semantic authority flipped AND imports semantically inert** — every production consumer on the reference-derived authority, all import-derived producers and ambient fallbacks gone | acceptance set green; stage0 regenerated; no live import reader remains |
| **M7** | **Grammar corpse removed** | `import` refuses at parse; token, AST and production gone |
| **M8** | **Grammar deleted.** | `import` refuses at parse; the AST and productions are gone |

M0–M4 are independently valuable on main *with imports*. M5 onward is the cut.

### Wave order

```
WAVE 1   0.1 pipe lowering   0.2 arg ordering   0.4 coercion parse
         A.1 nominal identity switch    A.2 forgery control
         B.1 closure authority          B.2 edge provenance
         D.1 key algebra authoring
         0.3 build_data_body_index: bare-name index over 3927 modules

WAVE 2   B.3 parser-owned reference facts        [needs B.1]
         C.1 delete ambient + proximity fallback [needs B.1]
         C.2 build_emit_graph_info collision arm [needs A.1]
         D.2 impostor separation, two arms       [needs A.1]

WAVE 3   C.3 remaining emitter sites   [needs C.2]
         C.4 OccurrenceBinding ledger  [needs C.1 — a ledger over a heuristic records the guess]
         D.3 KeyedRoster wall          [needs D.2]

WAVE 4   E.1 derived projection  [needs C.4]
         E.2 acceptance gates    [needs E.1, and BLOCKED ON ANOTHER TREE — below]
         E.3 perturbation falsifier — the control for ambient binding

WAVE 5   F   one semantic motion             ← single owner
             rewrite · switch the producer · move EVERY live consumer
             · delete old producers and ambient fallbacks
             · retire import-semantic diagnostics and witnesses · regenerate

WAVE 6   G   bounded corpse deletion — grammar, token, AST, production
```

The two hard edges: **C.4 must follow C.1** — a ledger built while ambient
binding is live records a heuristic's guess as truth — and **F.1 is one motion
with one owner**, because it is the authority transition.

### Three lanes, divided by question rather than by file

The boundary that stays mutually exclusive under pressure is not a directory, it
is the question each lane answers. Files follow from it.

| lane | the question it owns | surface |
|---|---|---|
| **Emission** | *how is an identity rendered into a target* | `05_emit.dag`, `05_emit_rust.dag`, the stage0 mirrors |
| **Resolution** | *what does this reference denote* | `02_parse`, `03_resolve`, `03_normalize`, `04_infer`, `04_env`, `04_occurrence_binding`, `cli_run.rs` closure, v2 `03_name_resolve` |
| **Substrate** | *what may be a key at all* | `dag/std/` |

Assignment: Emission owns 0.2, A, C.2, C.3. Resolution owns 0.1, 0.3, 0.4, B, C.1,
and C.4. Substrate owns D. **Resolution SUPPLIES COMPONENTS to F; it does not own
a post-flip phase, because there is no post-flip phase.**

**F and G are not this lane's at all.** Authority transitions — applying the
derived projection to canonical source, the single production authority flip, and
grammar retirement — belong to `warm-hawk-909` (ruling v2, 2026-08-25), together
with cross-seam adjudication. This lane supplies E's projection and every semantic
component F consumes; it does not perform the transition. An earlier revision put
F.1 and G with "the integrator", meaning this session, which was wrong in the
direction that matters: **the one motion nobody may straddle should not be owned
by a party that also owns components entering it.**

`nominal_call_arg_brand_mismatch` is the boundary case worth stating: it lives in
`04_infer`, which Emission does not own, and it is a *denotation* question — does
this argument's brand match — so it is Resolution's, by the question rule rather
than by proximity to a brand concept.

## 8. Adjudicating the vehicle: big-bang vs per-subtree

`deep-ant-102` relays a dissent from its side chat and calls it "the single most
consequential open question": that the ratified design specifies **shortest unique
suffix** qualification and **per-subtree** migration with import-grammar deletion
as the *terminal* step, whereas #8282 is whole-repository big-bang **full**
qualification — so re-deriving the cut mechanically would repeat the wrong
vehicle.

**It is decidable by reading the two documents, and it is decided.** The dissent
is reading `docs/plans/namespace-resolution-design.md`, which does say exactly
that — §8 is titled *"Migration — per-subtree, behind a swappable policy"* and
step 4 reads "flip the policy to `namespace-only-Y` **per subtree** as each
converges … **not big-bang**". The dissent is not inventing it.

But `namespace-cut-replacement-plan.md` — the later document, the one DESIGN §3
actually links as ratified — quarries it **by name**:

> **Existing designs** covering this region (namespace-resolution-design, the
> layering repoint design, and kin) are quarry — terminal shapes and mechanisms
> are evidence; **their sequencing never defers the deletion** (operator ruling,
> 2026-08-15).

So the split is clean, and it falls exactly along the line that clause draws:

- **shortest unique suffix** is a *terminal shape* question, so it survives as
  evidence rather than as a live option. **The target spelling is decided and its
  single home is §7a: full qualification** (operator, 2026-08-25). This row does
  not restate that question and does not reopen it — an earlier revision said
  full qualification was "not obviously the right target", which read as live
  against §7a's decision and cost a peer lane a blocked escalation. The evidence
  value that remains is narrow: if the sweep ever needs a second output format,
  `namespace-resolution-design` §8 is where that shape is already worked out.
- **per-subtree migration** is a *sequencing* question. It is explicitly
  superseded by the ruling above. Big-bang deletion stands — one authority
  transition, not a per-subtree policy flip. That is a different axis from §7a's
  2026-08-25 reordering, which moves grammar deletion to the END of the sequence
  without splitting the transition: the producer still changes once.

**What survives from the dissent regardless, and should be adopted:**

- **The nine commits are three kinds of thing and must not ride one label**
  (`crisp-crab-430`, who authored them):
  - **general compiler corrections**, true on main today and unrelated to imports
    — the empty-list `List<Unit>` fabrication, open as gunbc#9200 with a
    red-first two-arm receipt. Its mechanism is worth reading past its size:
    `List<Unit>` **reads as fully resolved**, so the fabrication switched *off*
    the if-join's own re-inference repair. **The defect consumed the mechanism
    that existed to correct it** — a shape worth watching for elsewhere, because
    it makes a defect self-masking rather than merely present.
  - **spelling-equivalence corrections** — real compiler defects reachable only
    *through* a qualified reference, so latent on main rather than false there:
    `render_node_type`'s unpeeled key, the resolver replacing an alias
    declaration's identity with its RHS target's, `trait_derive_emit` losing K/V
    `Clone` bounds for a qualified generic head, `is_container_type`, and the
    parser's qualified constructor literal in no-brace position.
  - **migration-generator corrections** — defects in the cut's own tooling, which
    **must never become PRs**. The `collection.Present` class is this: the
    qualifier wrote the module where a name was *reached* as though it were where
    the declaration *lives*. It dies when the cut is re-derived.
  Two of the nine are additionally a repair plus its own dissolution
  (`56dbb62` → `cf432d1`), and the use-line pair is worse than that — the second
  **restores the prefix the first peeled**, because the bare registry is
  last-write-wins. Main takes the final construction as one PR, never a
  known-incomplete step followed by its correction.
- Extract **mechanisms**, not the nine commit boundaries. Several of those commits
  are a repair plus its own later dissolution (`56dbb62c3f3` → `cf432d1922a`
  dissolves into `with_authored_identity`); main wants only the final
  construction. §6's throwaway-work rule applies to the extraction itself.
- **Close #8282 as SUPERSEDED**, not "keep open as an oracle". A 3900-file draft
  that reads as a merge candidate mis-signals to every reader, and a frozen branch
  is valid evidence only about its exact head — an oracle held open indefinitely
  decays as main moves under it. Bank what is needed *now* (the 29/29
  `ReferencePath` controls, the site census, the fixtures), then close it.
- **Run future cuts in temporary worktrees**, not a maintained PR. 222 merges is
  the measurement that argues this.

## 9. Open, and owned elsewhere

- ~~`undefined variable 'v2'` × 95~~ — **CLOSED, and it was never a distinct
  defect** (`crisp-crab-430`, 2026-08-25, four arms by execution). It is the
  **bootstrap trap wearing a resolution diagnostic's clothes**, and the reason
  three sessions could not reproduce it is that **it is not reproducible from
  source at all.**

  The offending construct is a **qualified constructor literal in no-brace
  (condition) position** at `coercion_widening.dag` — this ledger's defect I, the
  parser guard. That guard **exists in the `.dag` authority and was never carried
  into the committed mirror**, because regen was blocked by the very defects the
  branch was fixing. A parse failure at that site degrades the qualified path to a
  bare-variable lookup, so `v2.std.algebra.filter(…)` resolves its first segment
  as a variable named `v2` — 95 identical diagnostics, all in files carrying
  qualified references, none reproducible in a fixture, **because every fixture
  was compiled by a compiler that could parse.** The discriminator is decisive:
  `src/v1/02_parse.dag` carries `qualified_ctor_literal_here` twice and the
  committed mirror `v1_compiler_parse.rs` carries it **zero** times.

  Three consequences, and the second is the one that reaches past this row:

  1. The 95 collapse into **defect I** and are **not separately extractable**.
     Defect I is thereby promoted from a tail item to **the highest-leverage
     remaining one**, because it is the only one of the nine whose absence
     **masks other measurements**.
  2. **Every diagnostic measured on that branch with the committed mirror
     describes a compiler that pre-dates the fixes in its own tree.** That is a
     stronger caveat than "the branch is at 13 errors" and belongs beside that
     figure wherever it is quoted. On any branch where regen is blocked, the
     committed mirror is an *older compiler*, and its output is evidence about
     that compiler rather than about the source.
  3. `deep-ant-102`'s refutation of the cascade hypothesis stands untouched: this
     is not fallout from a broken *provider*, it is fallout from a broken
     **parser**.

  **Why four sessions missed it, including this one.** Everyone varied the
  *source* — named versus positional arguments, with and without imports, one
  versus two source roots, with and without a lambda. This lane's own contribution
  was a source read that narrowed the class to two arms of
  `qualified_value_projection` and correctly eliminated a third; the dichotomy was
  right and **the variable was wrong.** The variable was never the source. It was
  the **compiler**, and specifically `.dag`/mirror skew. This is the **third false
  finding the bootstrap trap produced in this program in one day**, and the most
  expensive.

- **Witness disposition gap** — a missing declaration fail-closes to
  `ReadsLiveTree`, so the witness never runs; 476 of 1514 witness files declare
  none, carrying 3906 test fns, against a floor reporting `declined_live=830`.
  ~3000 test fns are unaccounted for, and **not-discovered is worse than
  declined**. **Owned by `deep-ant-102`, not by this lane** (seam ruling,
  `warm-hawk-909`, 2026-08-25): it is an observation question — witness floor,
  discovery and disposition — not a compiler-semantics one.
- ~~The reconcile name-set dependency~~ — **RETRACTED by its author**, see §3c.
  The operator had left its ordering open; that question is dissolved rather than
  answered, and no work is sequenced behind it. What replaces it as an open item
  is narrower: the 8 anonymous-record ambiguities are real, still block emission,
  and are unclassified — `DeclarationRef` vs `RustItemDeclarationRef` among them
  is plausibly this document's class.
- ~~The FreeText denominator~~ — **ANSWERED**: 8 of 2272, 0.35%
  (→ [keying-census.md](keying-census.md)). Carried into T1 and Track D above.
  What remains open on it is the census's own declared bound: it is
  **text-derived, not Node-derived**, so it cannot see a key type reached through
  an alias, a generic instantiation or a re-export, and it classifies by binding
  name rather than resolved declaration. **Every count is a lower bound on
  identity-keying**, which is the safe direction for this argument but means the
  609 `UNDECIDED` sites are unpriced. The terminal instrument is named: fold
  declaration `Node` trees through the `decl_facts` primitive
  (`coproduct_reflection.rs` `decl_facts_corpus_walk`, which genuinely parses),
  copying `v2.lens.grounding`'s consumption of the sibling `concept_decl_facts`.
  Two traps recorded there: `fact_cardinality_decl_facts` is a **line scanner**,
  so a census built on it is text-derived while looking tree-derived; and
  `decl_facts_corpus_walk` **silently skips unparseable files and excludes
  tests**, reporting `files_scanned` and `files_parsed` so the skew is observable
  only to a consumer that reads both.
- **`module_skips_direct_call_arg_check`** — read by `neat-fox-901` as disabling
  the direct-call argument-type judgment for all of `src/v2`. Code reading only;
  the first harness was non-discriminating and a whole-tree probe is running.
  Compatible with §4b's existing clause, which scopes the exemption to that
  judgment and establishes it does not reach `sole_constructor` — this says the
  judgment it is scoped to is off corpus-wide, not that a new exemption exists.
  **The whole-tree probe was killed at ~40 minutes and is inconclusive**, so this
  stays code-read-only and is not claimed as executed. What would settle it is a
  discriminating pair *inside the tree* under a run that executes body analyses;
  `--source-dir` does not, which a failed red control established.
- ~~Track A has no owner~~ — **OWNED** by `silent-eagle-146` as of 2026-08-25.
  The row was stale within the hour and a peer lane was independently preparing
  to staff the same phase, which is the double-dispatch the implementation /
  observation seam exists to prevent.
- ~~Phase D's target spelling~~ — **DECIDED, not open.** Full qualification;
  authority is §7a. Listed here in error while §7a already recorded the decision,
  which is this document's own §3 violation — one fact, three homes, two of them
  stale.

## 10. What this document does not claim

**The readiness reader lies, and it is this document's own class one layer out.**
`gh pr checks` renders a **cancelled** run as `fail`. Measured on the last 100
`witnesses` runs: **43 cancelled, 17 failure, 25 success, 15 in flight** — so of
the 60 runs the standard readiness command renders as `fail`, **72% are lawful
cancellations**, and a rendered `fail` is wrong more often than it is right.
Cancellation here is `cancel-in-progress` doing its job when a later push lands,
and this tree pushes far faster than the job completes, so supersession is the
normal state rather than an edge case. Read the conclusion instead —
`gh run view <id>`, or
`gh api ".../witnesses.yml/runs?head_sha=<sha>" --jq '.workflow_runs[]|.conclusion'`.
Nothing needs building: the record is intact and only the projection collapses two
states. It is recorded here rather than left as tooling trivia because it is
**execution-provenance loss at the reading layer** — two distinct states rendering
identically, exactly the shape §3c's retraction turned on — and because the cost
lands on merge-readiness calls across six lanes, where the failure mode is
abandoning or churning a good PR on a false red.

**None of the defect instances here was found by searching.** Every one was
tripped over while measuring something else: the pipe-position lowering while
chasing a different diagnostic; `order_typed_call_args` while building a fixture
to check a reviewer's push-back; `build_data_body_index` while confirming an
unrelated emitter defect; the `env` homonym while classifying a census; the
`shared_types` leaf-keying while reviewing a PR expected to be wrong. **That is
the load-bearing reason the population is not five**, and it is a better argument
for the instrument work than any count: a class discovered only by accident has no
measured denominator, so five accidental finds in one day is evidence about the
*rate*, not the *total*. It is also why §9's items are stated as unowned questions
rather than as a remaining backlog.

- **No population effect is claimed for Phase A.** The 50 erasures and the
  constant-`false` predicate are measured; how many of the 13 residual #8282
  errors they close is not, and will not be until a regen, install and full build
  counts it. #8282's own body records one scope claim confirmed at a site and
  disconfirmed across a population; this document does not add a second.
- **The 12-site emitter census is hand-swept** (#8848's own rung note): a newly
  authored bare-name decision moves no number in it and nothing detects the
  omission. Its next-rung trigger is a lens over the emitter's own `Node` tree,
  and Phase C should land that rather than inherit the roster's blind spot.
- **No operator approval is claimed for any phase here.** Phases B and C each
  touch files DESIGN names as load-bearing.
- **Gen 3 stays frozen.** It is an offline differential oracle; nothing here
  merges it, and per the 2026-08-25 ruling main is not merged into it again.

---

## 11. Standing invariants, the seam, and the dispatch protocol

Section 7b divides the work. This section records the rules that hold *across* that division — each one derived from a specific way this program already went wrong, not from general prudence. They are stated here rather than in a lane's brief because a rule that lives in one lane's brief is enforceable by one lane.

### 11.1 The seam: semantic authority vs proof authority

The two roles were previously named *implementation* and *observation*, which mis-describes both. The distinction is not who writes code and who measures; it is **which question a party is entitled to answer**.

- **Semantic authority** — what the program *means*: which bytes the target emits for a fixed semantic input, what a resolution returns, what a key denotes. Changing this changes the answer.
- **Proof authority** — whether a claimed answer *holds*: bootstrap, fixed point, behavioural equivalence, regen provenance. Changing this changes the confidence, never the answer.

The seam matters because the two have opposite failure modes. A semantic change that is wrong is *loud downstream*; a proof that is wrong is *silent*, and it is silent in the direction of appearing green. So a party may hold both roles for different objects, and may never hold both for the *same* object: a proof authored by the party whose answer it validates has no independent referent, which is §5's specification-without-execution wearing a second hat.

Concretely: `#9182` is a semantic object (it changes emitted bytes) held here; its bootstrap and fixed-point proof stay with `deep-ant-102`, who is the required reviewer for regen provenance and is *not* free to also decide what the emission should be. `#9205` is the mirror case — proof authority restoring an instrument to buildable state, explicitly declining to claim its empty map is semantically correct. The moment the question becomes *what should `authored_import_names` contain*, the object crosses the seam.

### 11.2 Four standing invariants

**(1) One producer per production semantic contract.** For each contract, exactly one producer supplies every production consumer, and the producer changes once. This is stronger than per-consumer atomicity, which is what #9088 already falsified: two simultaneously-live consumers can each be internally coherent and disagree with each other, and nothing in either one is detectably wrong.

**(2) No per-consumer straddle across the cutover.** No consumer may be correct under both the old and the new producer during the transition. A consumer that works either way is not a compatibility win — it is the absence of a discriminating input, so the cutover has no observable moment and its failure has no detector.

**(3) No oracle repair by normalization.** When a comparison refuses, the repair is never to normalize both sides until it agrees. That is the rustfmt double-normalization defect the required run already paid for: an identity that holds only if the normalizer is idempotent, whose sole reachable green was the hand-edit the gate exists to refuse. Where two consumers normalize one artifact differently, the artifact is stored in the normalizer's fixed point — the contradiction made unrepresentable rather than detected.

**(4a) Ask whether the subject already publishes its own answer, before
deriving a population from the tree.** A measurement that disagrees with the
producer's own published output is evidence about the *measurement*, not
evidence of a gap. The required floor uploads a per-entry roster on every run
(`gh run download <id> -n required-floor-disposition` → one row per identity
with its disposition and the prefix that matched); the compile phase carries
its own disposition records rather than exit codes. Re-deriving either from
file contents is not independent corroboration — it is a second representation
with no authority, and the producer knows things the files do not (exclusion
rosters, explicit admission, discovery scope).

Two receipts, one in each direction, both from 2026-08-25. A lane read
`floor_discovery_producer.dag` correctly — a missing declaration resolves to
`ReadsLiveTree` — and inferred that ~3,000 witnesses were silently declined.
The source reading was right and the inference was false: files with no
disposition row come back **`planned`**, and the modules actually declined
declare `ReadsLiveTree` **explicitly**. Declines are declared, not defaulted.
Three successive static measurements each disagreed with the floor's own counts
by roughly 4×, and refining the static measurement could not have converged,
because it was measuring the wrong thing more precisely each time. That error
has a name and it is the **mirror** of the one this repository already
documents: *reachability read as occupancy* normally DELETES a quiet guard
because nothing lands in it; this direction reads a quiet guard as a FIRING one
because the arm exists in source. The documented direction removes a real wall;
this one manufactures a defect that does not exist. Same conflation, and only
one half was written down.

The second receipt is this document's own, and it is here because it is worse:
§7a published `sole_constructor` as authored at **176 production sites**, a raw
token count in which **21 occurrences sit inside `data …: String` prose rows**
discussing the feature and most of the rest are references. The declaration
count is **79**. It was written into a committed document, sent to a lane as
the basis for a fan-out estimate, and it was authored *in the same session as
this section*. The discipline is therefore not knowledge — everyone involved
already knew it — it is a habit, and the habit is: **a grep is a hypothesis, and
a count with no producer behind it is not a measurement.**

**(4) No evidence laundering, and no collapsed absence.** A measurement is cited by naming the producer that re-derives it. A row reporting *the check passed* may not share a carrier with *the check did not run* — six distinct inhabitants are in play across this program's measurements (`ran and clean`, `ran and found`, `refused upstream`, `declined by home policy`, `no route at the hermetic boundary`, `not discovered`) and every pair of them has been conflated by something in this repository within the last week. deep-ant's own stale-base catch is invariant (4) applied by hand: a diagnostic measured on a 23-commit-stale base and one measured on main render identically, and only provenance separates them.

Invariant (4) has a live consequence worth recording rather than leaving to be rediscovered. DESIGN.md's 2026-08-25 emit-stage rung-drop row names `gcloud.Auth.ReadADC` as its specimen; the declaration is **gone from `origin/main`** — the only surviving mentions are a retrospective comment in `gunbc.auth.credentials`, stale fixtures under `dag/test/fixture/`, and the row itself. This does **not** falsify the row, which anticipated exactly this and states its own restoration trigger as *a required phase that emits over a closure*, explicitly **not** the specimen's repair. The point is the reverse and it is the stronger one: the specimen was repaired along the read-then-decode line the row itself ruled, and **no mechanism updated the row or noticed**. A rung-drop declaration whose specimens rot silently is a measurement with no producer, which is what invariant (4) exists to forbid. The row's *class* claim stands; its *present-tense specimen* does not.

Invariant (4) also has a coordination-layer form that cost two lanes real time
on the same afternoon, and it is recorded here because the two halves fail in
**opposite** directions, which is what makes either one alone misleading:

- The dashboard summary rendered a **lawfully cancelled** run — superseded by a
  newer push — as a failure.
- The dashboard summary rendered a **succeeded** run as `checks=pending`,
  `mergeable=UNKNOWN`, on a head where the Actions API said
  `conclusion=success` and GitHub said `CLEAN`. Polled three times over sixty
  seconds without converging, so not a race.

So neither surface dominates and neither is the oracle. The split that holds:
**the dashboard for review facts** — counts, distinct approving providers,
request-changes, artifact links, all of which matched exactly — and **the
Actions API keyed to the head sha for the run conclusion.**

With one trap inside the remedy, which is the part worth carrying: the Actions
API **silently accepts an abbreviated sha and returns an empty list** — no
error, no warning, `workflow_runs` length 0. An until-loop waiting on that
result polls forever, and *a loop whose exit condition can never be met is
indistinguishable from a job that has not finished*. Pass the full forty
characters. Replacing a lying reader with a silent one is not an improvement,
and it is invariant (4) reproduced one layer inside its own fix.

One near-miss belongs beside it, because it is the failure mode of *warning
people about instruments*. A lane primed that a reader lies then read a
correct green as a false green, and was minutes from reporting a fabricated
defect against an honest projection. A warning about an unreliable instrument
makes the next ambiguous reading look like the failure that was warned about.
The remedy is the same one this whole section keeps arriving at: name the
producer, key it exactly, and let the reading come from the thing that owns the
fact rather than from a prior about which reader is trustworthy.

**(5) A fix needs a whole-corpus arm, not only its own witness.** A red-on-main
fixture proves the DEFECT is real; it does not prove the FIX is safe, and the
two are separate claims that a red→green witness silently merges.

The receipt is `crisp-crab-430`'s #9200, retracted by its own author against
themselves after two approvals. The fix was correct in direction — it removed a
fabricated `List<Unit>` from an unresolved fold step — and it regressed real
corpus code: `dag/extdeps/render/terminal.dag` goes from 0 blocking on pristine
main to 1 (`no field 'kept' on type 'Unit'`). The mechanism is worth keeping
because it is counter-intuitive: **the fabrication was load-bearing for exactly
the reason it was worth removing.** `List<Unit>` was WRONG BUT CONCRETE, so
`st.kept` type-checked; a fresh type variable is HONEST BUT UNRESOLVED, and the
join across the step's arms cannot close it, so it renders as `Unit` and the
field access fails. Removing it stays right; it has to land with the resolution
path rather than ahead of it.

What makes this an invariant rather than an anecdote is that **no fixture the
author could have written would have caught it** — it took the real corpus — and
**two approvals could not have caught it either, because it is invisible in a
diff.** So the arm is not a review practice or a better-witness practice; it is
a different instrument, and nothing cheaper substitutes.

**THE ARM IS PER-STAGE, NOT ONE INSTRUMENT, AND THE STAGES HAVE DIFFERENT
DENOMINATORS.** `crisp-crab-430` bounded this on their own passing PR rather
than banking the coverage, which is the version to hold:

| a fix in… | its arm | denominator |
|---|---|---|
| resolve / typecheck | floor strict preparation | **3930 modules** — full |
| parse | the parse phase, three source roots | full |
| **emission** | regen's closure + the one v2-emission entry | **135 modules + 1 entry — partial, and the gap is declared** |

So a fix must get the arm *for the stage it changes*, and "green CI" is not the
claim. #9199 is an emitter fix that passed the strongest emitter arm that
currently exists; the honest statement is **"it passed the arm that exists"**,
not "it is corpus-clean". Reading its green as whole-corpus coverage is
reachability-read-as-occupancy again — the phase exists, it ran, and its
denominator is 135 rather than 3930.

This also explains why #9200 *was* caught: it is an INFERENCE fix, so its
regression landed in the one stage that has full corpus coverage. **Had the
same class of mistake been made in the emitter, the arm would have had roughly
a 96% chance of not containing the victim.** That is a stronger argument for
closing DESIGN's declared emit-over-closures gap than anything derivable from
the gap itself — the gap's cost is not "some diagnostics hide", it is "the
invariant above is unenforceable in exactly one stage".

The same requirement arrived independently the same day from the opposite
direction: the coproduct-field widening (§11.2b's neighbour) is admissible only
at zero false positives over `dag/` + `src/v2`, because `declared_type_conformance_note`
records that all four of its predecessor false-positive classes were found by
RUNNING the wall over the corpus and none by reasoning about it. Two lanes, two
subjects, one conclusion: **the corpus is the instrument, and a witness is a
hypothesis about it.**

### 11.2b The dormant repair — three instances in one day

A pattern surfaced three times on 2026-08-25, by three lanes, on three
unrelated subjects. It is recorded as its own class because each lane found it
while looking for something else, and because the first two look like ordinary
incompleteness until the third shows what they have in common.

| repair | state | callers on the fast path |
|---|---|---|
| `rust_nominal_identity_carrier_def` — emits `pub struct Name(pub String)` | written, complete | 0 — `rust_nominal_identity_carrier_type_eligible` returns `false`, 5 call sites waiting |
| `sole_constructor` — parsed, consumed by inference | authored on 79 declarations | 0 in emission — `05_emit.dag` and `05_emit_rust.dag` read it zero times |
| `char_at_ascii_aware` — takes a precomputed `is_ascii` flag | written, self-documenting, names its own defect (`STRING-INDEX-0`) | 0 — the bridge table maps `char_at` → the plain wrapper (`grep -n '"char_at"' src/v1/stage0/src/extdeps_languages_rust_emit.rs`) |

**The shape: the capability exists, is correct, and nothing reaches it.** Each
was priced as a build and each is a routing question. That inverts the estimate
in a way that matters — the `char_at` lane arrived carrying "98 references
across 44 files are quadratic, are they in per-character loops?", and the
answer is that the population is irrelevant, because every one of them lowers
through a single bridge-table row.

**And the third instance is not the same as the other two, which is what makes
the class worth writing down rather than just the observation.** Its doc
comment names the carrier that supplies its precomputed flag: `RcStr`. **`RcStr`
occurs exactly twice in the whole of `src/v1/stage0/src/` — the comment, and
the copy of that comment inside the generator string that emits it**
(`grep -rn RcStr src/v1/stage0/src/`; there is no `struct RcStr` and no type
alias, in `.rs` or `.dag`).** The
carrier does not exist. So this is not an unrouted repair; it is a repair whose
**input has no producer**, and the wrapper that computes `s.is_ascii()` per call
is not laziness but the only callable form. The other two have every input
present and want a routing edge; this one wants a carrier that was never built.

**The buckets were re-sorted on challenge, and the challenge was refuted by
measurement — which is the only reason the sort is worth trusting.**
`warm-hawk-909` objected that instance 1 was mis-filed as a routing case (its
predicate takes a `String`, and C0 — what declaration fact denotes a brand — is
precisely a fact nobody built), and speculated that if `sole_constructor` also
turned out to want a fact, the routing bucket would be **empty**, which would
say the tree does not produce dormant repairs by forgetting an edge but by
shipping capability ahead of the model it consumes. That is a stronger claim
than the class made, so it was checked rather than adopted.

It does not hold, and the disconfirming evidence sits in the same file as the
defect. `05_emit_rust.dag` calls `find_property(props: n.properties, …)` **29
times**, so reading a declaration property at emission is routine and
exercised. `sole_constructor` is stored exactly that way — `04_infer.dag`
consumes it as `decl.properties |> any(p => p.name == "sole_constructor")` —
so emission consuming it is one more call of a form the file already makes 29
times. That is a real routing edge, and the bucket is not empty.

The same measurement re-sorts instance 1 more precisely than either of us had
it. Twelve lines below the broken predicate sits
`rust_nominal_ord_type_decl_ord_eligible(decl: Node, source_indices)` — an
eligibility predicate **that already takes a `Node`**, which is exactly the
shape C0 requires. So the emitter-side mechanism is not missing; the
correctly-shaped sibling is adjacent to the mis-shaped one. What is missing for
brands is upstream, in the model: `type Brand = String` admits
`take_string(s: b)` with zero diagnostics, so there is no distinction for any
predicate to read.

**CLASSIFY BY THE FIRST MISSING PREREQUISITE IN THE PATH.** This section has
now been re-sorted three times — routing-vs-model, then source-vs-target seam,
and now this — and the sequence is kept because it is the evidence: three
people holding progressively better axes still mis-filed the specimens each
time. The partition that survives, from `warm-hawk-909` after external review:

> authorable subject → semantic fact → carrier for that fact → target
> realization rule → consumer routing edge

**The FIRST absent element determines the repair and its cost — not the dormant
function's diff**, which looks identical in all five cases. On that partition:

| instance | first missing prerequisite | why |
|---|---|---|
| nominal identity | **subject classifier** | consumers already exist across item emittability, alias and type-declaration emission, and typed data rendering; the authority itself returns `false` unconditionally |
| `char_at_ascii_aware` | **carrier fact** | routing it today makes the caller fabricate or recompute the very fact whose retention justifies the function |
| `sole_constructor` | **does not classify once** | its source-semantic path is a genuine routing defect (coproduct construction is unwired to an existing fact and an existing refusal producer); its Rust-exposure path lacks a target realization *contract* — nobody has decided whether it realizes as field privacy, constructor privacy, module placement, an opaque factory, or no target restriction at all, so adding an emitter read is speculative routing |

**MY CONCLUSION HELD AND MY SPECIMEN DID NOT.** An earlier revision argued the
pure-missing-edge bucket was not empty, against a proposal that it was, and
offered `sole_constructor` as its member. The conclusion is confirmed — by a
different instance. **gunbc#9018, merged 2026-08-24**, is the real one: the
emitter already knew how to render `Parent::Variant`, and
`collect_value_ref_names` recursed children, params, uses, properties, body,
transport and type_annotation but **never `match_pattern`**, so a nested field
pattern was reachable through no edge and the enum name was rendered without
ever being proposed as an import candidate. One producer edge removed seven
`E0433`s. That is a pure missing call edge with every prerequisite present.

So the permitted statement is narrower than any of the three sorts, and nothing
stronger belongs in this document: **among these three candidates, no
target-emission or runtime repair is yet established as a pure missing call
edge; two are blocked by absent model or carrier facts; the third decomposes
into a known source-semantic routing gap and an unmodeled target-realization
question.** An empty observed bucket is a result, never a theorem about what
this tree produces — and #9018 is why that distinction was worth keeping rather
than hardening into a law twice.

A missing subject classifier is a language change. A missing carrier is a
representation change that closes six entry points at once — `scan_while`,
`skip_horizontal_ws`, `scan_to_eol`, `scan_string_end`, `substring` and
`char_at` all route through the same bridge table to plain entry points whose
bodies open with `if s.is_ascii()`, so sizing that lane by counting `char_at`
references measures the wrong symbol. A missing target realization rule is a
semantic decision nobody has taken. A missing edge is an afternoon. **The four
differ by an order of magnitude and their diffs are indistinguishable from the
outside**, which is the whole reason the partition earns its keep.

The operative rule, and it applies before any lane sizes a population:
**ask whether the mechanism already exists and is merely unrouted — and if it
is, ask whether the thing that would route to it exists, ON BOTH SIDES OF THE
SEAM.** Source-side: is there a fact to read. Target-side: is the emitted form
decided. Either one missing produces the same diff-shaped silence. A dormant
repair looks like missing work and is usually a missing edge; a dormant repair
whose parameter has no producer looks like a missing edge and is a missing
model fact. The three cost estimates differ by an order of magnitude and the
diffs are indistinguishable from the outside.

This is §5's specification-without-execution with the polarity reversed. There,
a thing is claimed done and never runs. Here, a thing is genuinely done,
genuinely correct, and never runs — so it accrues no evidence of its own
absence, and the defect it repairs keeps its full frequency while the repair
sits in the tree being cited as coverage.

### 11.2c RESOLVED — a single candidate resolves from anywhere, and the flip is declared downstream of this cut

**Now established, after being asserted, retracted, and resolved inside two
hours.** The sequence is preserved because each step was wrong in an
instructive way.

**THE PHASE LINES SETTLE IT.** `cool-hawk-324`'s whole-tree log, verbatim
thirteen: `indexed 3928 modules from 2 source roots` · `resolved 2866 sources
(transitive import closure)` · `[census] 1062 indexed modules outside the
closure enter the name census only (not compiled)` · frontend 47s · normalize
3s · reconcile 16min · analyses 1s · **emit 44s** · 2 hard diagnostics.

Every phase completed, emit included. So the corpus-wide zero is **a stage that
ran clean, not a stage that never ran** — the discriminating question §11.2c was
retracted for lacking. `review_codex.dag` was compiled, and `llm`,
`upsert_tagged_cron_tab` and `Step` bind there while an `--entry` compile of the
same file at the same commit refuses all three.

**THE MECHANISM WAS DOCUMENTED IN THE SEAM ALL ALONG.** `v1.04_env`
`global_bare_fallback_invariant`:

> *"Resolution (`global_bare_lookup`): **a single candidate resolves from
> anywhere** (the one-candidate degenerate case of the walk); multiple
> candidates resolve by nearest-ancestor containment under `ImportScoped`, while
> `NamespaceOnlyY` filters to the referencing module's containment chain…"*

**That reconciles both readings, and neither was wrong.** The chain-prefix rule
is real — and it governs the **multi-candidate** case only. A single candidate
short-circuits ahead of it and resolves from anywhere, no prefix relationship
required. Each of the three names is presumably corpus-unique, so each is a
one-candidate case. And the census is built over `graph.modules`, which line 3
shows includes the 1062 modules that are **indexed but never compiled**; under
`--entry` the census covers the closure only, so the same names are absent and
correctly refuse. Same compiler, different census population, opposite answers.

**THE TERMINAL IS DECLARED, AND IT IS THIS PROGRAM'S OWN WORK.** Same note, last
clause: *"The production flip that removes the corpus fallback is **downstream
of reference-derived closure**."* So this is not an undiscovered defect but a
documented interim whose removal is sequenced behind exactly this cut. §11.2c
therefore has a named terminal rather than an open hazard — better news than the
prediction, the retraction, or the confirmation.

**TWO NARROWINGS, BOTH FROM THE MEASURING SIDE.** The corpus-wide zero covers
the **2866 compiled** modules, not 3928; whether the 1062 census-only modules
would produce name-resolution diagnostics is unmeasured and a grep cannot speak
for them. And "whole-tree" compiles the transitive closure of the **first source
root only** — additional roots are dependency pools — so `--source-root dag
--source-root src/v2` made `src/v2` a pool.

**ONE GAP REMAINS AND IT IS THIS DOCUMENT'S FIXTURE.** Two import-free modules
in one pool, provider declaring and consumer referencing, should be a
one-candidate case and should resolve. It refuses — measured across **all four
forms**: bare function call, bare `data` reference, bare type, and bare variant,
under `--source-dir` and `--source-root` alike. The proposed explanation was
`listed_import_required_bare_call_blocked`, which sits directly on that path;
**refuted by reading it** — `bare_free_call_requires_listed_import(name) = name
== "trim"`, a single hardcoded name that no fixture symbol matches. A lead, not
an answer: my runs print no `[census]` line at all, so the census-only
population step appears not to engage at that invocation, which would explain
the divergence but is not established.

The gap is smaller and better named than it was, and it is recorded rather than
closed with a plausible story — the same discipline applied to the
`pr_owner`/`pr_repo` asymmetry, whose bare-name-collision explanation was
likewise refuted by grep (`data owner` and `data repo` each have exactly one
module-scope declaration corpus-wide) and which stays **unexplained**.

### 11.2c-orig The mechanism, as originally traced — `authored_import_names`

Found while unblocking a lane on an unrelated red, and it is the most direct
threat to this program's own premise that has surfaced so far.

`v1.04_lookup` `author_named_visibility` is correctly three-valued —
`AuthorNamedThisName | AuthorNamedNothingForThisName | VisibilityUnobservable` —
and its header is careful about exactly the right distinction: membership in
`source_visible_names` is *necessary* for "the author named this" and not
*sufficient*, because an `is_all` import contributes names the author never
wrote. So it reads `authored_import_names`, the listed-import arm alone.

**But the carrier is `Map<String, bool>`, and the reader synthesizes its third
state from EMPTINESS:**

```
if map_is_empty(m: type_env.authored_import_names) { VisibilityUnobservable }
else { if map_has(…) { AuthorNamedThisName } else { AuthorNamedNothingForThisName } }
```

A two-valued carrier cannot supply a three-valued answer, so *observed, and the
author named nothing* and *nothing populated this* are the same bytes. This is
the identical shape found the same day in the emitter, where `String?` collapsed
AMBIGUOUS and NO-CANDIDATE into one `Absent` and the consumer recomputed the
whole fold to recover what the producer had discarded. Two stages, one defect.

**THE CONSEQUENCE IS NOT SILENCE — IT IS A DIFFERENT ANSWER, PRODUCED
CONFIDENTLY.** An earlier revision of this section said the wall "stops
discriminating, silently, with no diagnostic and no count." `deep-ant-102`
traced the consumer and that framing understates it. `author_named_visibility`
has exactly one caller, `callable_lookup_over_candidates`, where
`VisibilityUnobservable` takes **the same arm as `AuthorNamedThisName`**
(`builtin_admissible = vec![]`). With nothing declared in the parent closure the
lookup then falls through to `func_sig_from_global_bare`, which resolves through
`borrowed_census_decl` — **the whole-tree flat census borrow, last-write-wins
across the entire corpus.**

So deleting imports does not disarm a guard into an unanswered question. It
hands every bare call to a resolver with no import context at all, and that
resolver answers. The failure mode is not a missing refusal but **a wrong
binding that typechecks** — the absorbing fallback with a resolution attached,
which is §5's named trap rather than a gap.

**AND IT IS LIVE TODAY, WHICH REQUIRED CORRECTING A CORRECTION.** The same
trace reported that the whole branch is gated behind
`name_resolution_policy_is_namespace_only()`, whose in-tree comment reads
"default false = production fail-open path" — concluding that this is a
precondition on a future policy flip rather than a present obligation. That is
false. `v1_rt.rs:164` is `Cell::new(true)`, `cli_run.rs` records that the flag
"now defaults to true (§13 unique-on-chain)" as the thing that turned a test
red, and `v1_compiler_infer_env.rs` names the ratification: "step-4 default ON =
NamespaceOnlyY", operator-ratified 2026-07-21. **`false` is the bracket, not the
default.** Namespace-only is the production policy and has been since July.

The quoted comment is real, and it sits immediately after that function's
closing brace while documenting the **next** declaration — the N1a measurement
arm, `type_ref_hit_ne_bind_measure_active`, which its own first words name. So a
careful reader attached prose to the wrong subject because it was adjacent, and
concluded the opposite of the truth about a live production policy. That is
§11.2d's class arriving a second time in one afternoon by a different route:
there a citation named a symbol that does not exist, here a comment named a
neighbour. **In both cases the code was right, the reader was careful, and the
prose was the only thing that lied.**

The immediate corollary for anyone filling a `TypeEnv` initializer: **an empty
`authored_import_names` is a claim that visibility is UNOBSERVABLE, not a claim
that the module has no imports.** Supplying `rc_empty_map()` because a
constructor demands a value is asserting the first while meaning the second.

### 11.2d Two fabricated citations inside a prose row, and how far one travelled

`witty-crane-181` reported, and this section verifies: `declared_type_conformance_note`
cites **`conformance_expansion_depth_note`** and **`conformance_unjudged_live_hole_note`**,
and *neither exists*. Measured with
`grep -rnE '^\s*(data|fn|type)\s+conformance_expansion_depth_note' dag/ src/ --include=*.dag`
→ **0**, same for the second. They occur only inside the prose of the note that
cites them.

**THE INTERESTING PART IS NOT THAT THEY ARE FABRICATED, IT IS THAT NOTHING
COULD HAVE CAUGHT THEM.** DESIGN's 2026-08-23 rung-drop row records that the
cited-symbol census was removed from CI, and it would be easy — and wrong — to
file this as an instance of that declared exposure. The census checked authored
`DeclarationRef` rows. **These citations are bare words inside a
`data …: String` prose row, which that census never read.** So they were
unchecked before the drop and unchecked after it; the drop is not the cause,
and claiming it would be the authority-substitution failure this document
already names — borrowing a declared exposure to explain a defect it does not
cover.

What they actually are is §4c's warning arriving with a bill: *plain source
annotations are modeling debt*, and *an annotation is never evidence that a
machine claim holds*. A citation inside a `String` is invisible to every
mechanism that validates citations, by construction, because it is program data
that happens to read like a reference.

**AND IT TRAVELLED, WHICH IS THE MEASUREMENT.** This document's author read
`conformance_expansion_depth_note` in that prose, believed it, and cited it
onward as the authority for a pre-check — to `warm-hawk-909` ("the place to
check before running") and into `witty-crane-181`'s dispatch brief as a required
step. A lane was directed to consult a symbol that does not exist. It cost
nothing only because that lane checked instead of complying, and answered the
underlying question by construction rather than by lookup.

So the class is not "a stale citation rots quietly." It is: **an unverifiable
citation is laundered into an authority by the act of being repeated**, and each
repetition is harder to challenge than the last, because it now has a
provenance. The repair is the same one §3 already prescribes — cite the symbol,
and a symbol inside a prose string is not a citation — with the addition that
prose rows in load-bearing authorities are exactly where an unresolvable
citation survives longest, since they are read by people and by nothing else.

### 11.2e RULING — anonymous record literals resolve from their expected type

**Decided 2026-08-25. (b), the compiler.** An anonymous record literal is
grounded by its expected nominal type; the author is not obliged to annotate it.

The alternative was to annotate the 8 sites blocking whole-corpus emission. It
is refused, and **not** because annotation is laborious — because the corpus
authority already committed to the opposite. `04_infer.dag`
`declared_type_conformance_note` class (3) IS anonymous record literals, and its
dissolve-on names the terminal verbatim: *"grounds an anonymous literal against
its expected nominal type."* Annotating would author around a defect the tree
has on record as the thing to fix.

Three further findings, each of which corrected something:

**One root, two stages.** Inference sees `Product(<anon>)`, emission sees
`Absent`, for the identical reason — the expected nominal type does not reach
the literal. So (b) does not merely unblock emission, it **retires class (3)**,
one of the four false-positive classes that forced the conformance wall down to
ground kernel scalars. The note is explicit that this is the mechanism: *"Each
numbered class above is a promotion trigger, and each promotion should be
visible as a drop in that count."* With the bound that makes it usable: **that
count is not live on main** — measured at 3005, corrected to 1566, then excluded
entirely (codex 45767), so the receipt needs the measurement re-derived and
cannot be read off the current tree.

**The `count == 0` arm refuses.** A record literal matching no nominal type does
not silently become a positional tuple: `{ a: 1, b: 2 }` emitted as `(1, 2)`
discards field identity and substitutes position, which is §3b's class, and
fabricating at the exact point the compiler has established it knows nothing is
§5's plainest failure. Under (b) that population may fall to zero on its own.
*Not decided here:* whether some target genuinely IS an anonymous tuple, which
would make that emission correct — if the work surfaces one, it comes back
rather than being routed around.

**The diagnostic text is part of the defect.** It ends `— add a nominal type`: a
compiler instructing the author to work around the compiler's own unresolved
obligation. A message surviving a (b) fix keeps teaching authors to annotate for
a reason that no longer exists. And the three-valued carrier is what makes the
replacement possible rather than merely better-worded — ADD A NOMINAL TYPE
(ambiguous) and THIS RECORD MATCHES NO DECLARED TYPE (no candidate) are
different instructions to an author, currently one message.

**The deliverable is the carrier, not the propagation.** `String?` collapsing
AMBIGUOUS and NO-CANDIDATE into one `Absent` is state-space conflation in a
return type, and the consumer recomputing the whole fold over `type_summaries`
is its symptom, not a separate cost defect. Three-valued —
resolved-with-name / ambiguous / no-candidate — and the fold runs once.

**THE POPULATION, AND WHAT IT IS NOT.** The 8 sites are **3 distinct ambiguity
pairs**; 5 of the 8 are one pair repeated in one file. Site count is the wrong
denominator, because what recurs is TYPE PAIRS — the same collision fires
wherever those types are written. On that denominator one pair is test-only, one
is extdeps-only, and one spans both: **`DeclarationRef` vs
`RustItemDeclarationRef`**, where `DeclarationRef` is declared in
`dag/std/decl_ref.dag` and referenced by **573 files, 509 of them outside
`dag/test/`**. Annotating there makes a `std` carrier permanently
annotation-requiring.

That replaces an earlier and wrong version of the same argument, retracted by
its own author: *the sites are in production modules, so annotation is a
treadmill.* **6 of the 8 are in `dag/test/claim/`** — three quarters is
fixture-shaped code, which is the condition under which annotation would have
been cheap and fine. The original was inferred from a truncated log that
happened to show 3 of the 5 repeated sites, making the split look 3-vs-3. The
lesson is the sharper half: **the truncation was declared and then reasoned from
anyway**, which is worse than not declaring it.

And one argument that depends on none of this: `dag/extdeps/git/versioning.dag`
matches **four** candidates, not two. A four-way collision among version-shaped
structs says the field-name heuristic degrades as the corpus accumulates
structurally similar types — so it gets worse with growth regardless of where
the sites live.

*Sequenced after #9209 (Resolution lane), which is in flight and unproven until
regen. No annotation as an interim — not one site: editing the corpus so
whole-corpus emission finishes is mutating the subject to complete the
observation, and the resulting green would mean only "this corpus, after we
changed it, emits."*

### 11.2f BLOCKING PREREQUISITE — v2's grammar cannot parse a qualified record literal

**`a.b.C { f: v }` does not parse.** `fierce-ram-94` established it by execution
against a five-way cause ladder, and it is the single most consequential finding
for this program so far, because **the cut's own target spelling produces this
construct everywhere.**

The dispatched hypothesis — a tail-position `if`/`match` nesting — is
**FALSIFIED**, and cleanly: `port_reading` transcribed exactly into a bare
module, same nesting, every arm a coproduct value, with constructors written
**unqualified**, parses and normalizes clean. Every ablation of the
tail-position axis is green (nested `if` with `Int` arms, no match at all, flat
`if`, match alone at fn tail, no preceding lets).

What discriminates is **qualification CROSSED WITH braces**, and the minimal
case is two lines:

```dag
match arg_value_symbol(arg_capture: arg_capture) {
  Present { value: sym } => a.b.c.PortBareName { name: sym }
  Absent => PortComputedExpression { mentions: mentions }
}
```

Qualified-bare: green. Unqualified-braced: green. **Qualified-braced: red.**
`warm-hawk-909`'s correction was load-bearing — the three earlier fixtures
probed variant *spelling* and the axis is spelling × braces.

*(Cell precision, self-corrected by the lane before this was written up:
qualified-bare is measured green in three positions — if-branch tail, plain fn
tail, and let-bound — but **not yet in a match arm**; that cell sat in a batch
whose entry points were never appended, so six runs returned `NoSuchFunction`
and a truncated grep of the output was read as green. "An absence downstream of
a failure is undefined, not a green" — their words, and the same class this
document keeps recording. It is re-running. **The conclusion does not rest on
it**: the root below is read directly from the grammar and is independent of
every fixture cell.)*

**THE ROOT, verified in `src/v2/extdeps/languages/dag.dag`:**

- `dag_grammar_primary_ident_suffix_expr` — a bare ident's suffix is
  `optional(choice(call_suffix, lbrace field_init_list rbrace))`. **Both
  alternatives.**
- `dag_grammar_postfix_expr_expr` — each dotted `.name` suffix is
  `optional(postfix_call_suffix)`. **Call only. No brace alternative.**

So `a.b.C` consumes as a complete postfix expression and `{ f: v }` is left
over — which is exactly the `parse_g0_tokens_remain` leftover, and it is **a
missing alternative in one production**, not a traversal failing to descend.

**WHY THIS BLOCKS M6.** §7a settles the target spelling as the fully qualified
declaring identity. Every cross-module constructor in a post-cut corpus is
therefore `a.b.C { … }`, and **v2's modeled parser cannot read it.** v1 accepts
the form (measured independently on main), so this is a v1/v2 divergence rather
than a corpus-wide break today — but v2 cannot read a post-cut corpus, and v2 is
the destination. This is a hard prerequisite of the semantic cutover, not
post-cut cleanup, and it moves ahead of everything else in the F set.

**NOT YET MEASURED, and explicitly not asserted:** whether adding the brace
alternative to the dotted suffix is *safe*. It puts `if a.b { … }` into the same
condition-versus-block ambiguity class that `if x { … }` already occupies, for
which the grammar already carries an overlap-residue diagnostic. That
measurement precedes any accept-versus-refuse proposal, and `dag.dag` is not
touched before it.

**A DIAGNOSTIC-READING TRAP FOUND ON THE WAY, worth more than the aside it
arrived as.** On a rejected `parse_module` the HEAD diagnostic is
`parse_grammar_choice_overlap_residue` — a grammar *advisory* that
`rejected_with_pending` prepends ahead of the real refusal. A probe reading
`d.head.reason` gets the advisory, not the parse cause. **Anything keying on the
head diagnostic of a v2 parse rejection is reading the wrong one**, which is
execution-provenance loss wearing a different hat: two distinct states rendering
identically to a consumer that takes the first row.

**AND THE FIVE-WAY LADDER EARNED ITSELF ON THIS ONE.** The same qualified record
literal as a plain fn tail expression is not parse-rejected — it is
`source_normalization_rejected`, because there the orphaned `{ f: v }` re-parses
as a standalone anonymous record statement and dies one stage later. Same
leftover, different rung. **A two-way accepted/rejected probe scores those two
cells identically and reports the wrong construct**, with a green receipt behind
it — which is precisely the failure the ladder was mandated to prevent, caught
by the mandate rather than by luck.

### 11.2g THE CUT'S SUCCESS CONDITION IS ALREADY AN EXECUTABLE TEST, WRITTEN BY SOMEONE ELSE

`dag/test/claim/import_admission_closure_membership_witness_test.dag` measured
this program's central question before this program existed, and its headline is
the answer to it:

> **THE IMPORT LIST DOES NOT GATE A BARE FREE CALL; CLOSURE MEMBERSHIP DOES.**

Four arms, one instrument, one binary, differing only in how the probe reaches
the provider:

| arm | reaches provider via | result |
|---|---|---|
| (1) listed | imports provider AND names the fn | resolves — **the only correct green** |
| (2) selective-unlisted | imports provider, does NOT name the fn | resolves — **pinned defect** |
| (3) pool-coincidence | never imports provider, only a carrier that does | resolves — **pinned defect** |
| (4) absent-from-closure | nothing reaches the provider | **REFUSES** |

Arm (4) is the discriminating control, and it is what makes the other three
readable: the instrument *can* go red on this exact proposition, so three greens
are measurements rather than vacuous passes.

**IT ALSO CLOSES §11.2c's LAST GAP.** This document's two-module fixture — which
refused where the single-candidate rule said it should resolve — **is arm (4)**.
No import edge means the consumer's transitive closure is just the consumer, so
the provider is in the *pool* but never in the *compiled closure*, and the census
only serves what the closure loads.

**The invariant's domain, stated exactly, because the loose form is wrong:** *a
single candidate **in the loaded global-bare census** resolves from anywhere* —
NOT *a single declaration anywhere in the source pool*. Candidate-pool membership
and loaded-closure membership are two facts, and the resolver's cardinality
policy ranges only over the second. **And the corollary matters more than the
correction: pool-only does NOT uniformly refuse.** Arm (3) resolves cleanly
whenever ANY loaded module pulls the provider in, without the consumer naming
it. So the discriminating relation is **reachability into the loaded closure**,
never the presence of an import statement in the consumer's own text — and that
is the relation any visibility-keyed law in this program must key on. Nothing contradicted the single-candidate
rule: the candidate was never in the census the rule runs over. And
`cool-hawk-324`'s whole-tree case is **arm (3)** — `review_codex.dag` is an entry
under the first root, and the modules declaring `Step` and
`upsert_tagged_cron_tab` are dragged into the closure by an unrelated carrier.
The fixture and the corpus differed in exactly one thing, and it was closure
membership.

**ARMS (2) AND (3) FLIPPING TO BLOCKING IS THIS CUT'S SUCCESS CONDITION**, stated
as a pinned executable test with its control already built. The witness says so
itself: *"when the admission wall lands they must FLIP to blocking, and a flip is
the wall landing rather than this witness breaking."* The receipt does not need
authoring.

**AND ITS MOST VALUABLE CONTENT IS NEGATIVE — TWO PLACEMENTS BUILT, MEASURED,
AND WITHDRAWN.** This is where not to put the wall, paid for already:

- **At the whole-pool census seams** (`borrowed_census_decl`,
  `global_bare_callable_node`) it is a **decoration**: the closure regenerated to
  a byte-identical fixed point, 134 of 134, `first_generation_equal=true`, and
  arm (2) still did not fire — because a selectively-imported module's unlisted
  fn is served EARLIER, by the unfiltered parent `func_env` and the
  `ancestry_str_bindings` overlay, so the census fallback is never reached.
  **gunbc#9075's parent-env filter, applied on top and measured, does not create
  that miss either — the ancestry overlay still serves the name.**
- **At the admission seam** (`lookup_func_sig`) it *bites* — three hard
  diagnostics on the same closure — but it bites by returning an unresolved
  signature, degrading into *"if branches resolve to incompatible types:
  Primitive(cost_constant) vs Primitive(CostBound)"*, which names neither the
  unadmitted call nor the import that declined it. **A silent widen wearing a
  type error**, not the typed located refusal §5 requires.

So the machinery was deleted rather than shipped unproven, and the measurement is
what landed. That is the scaffold-admission doctrine executed correctly by
someone with a working wall in hand.

**THE NEXT INCREMENT IS ONE AUTHORITY EXTENDED, NOT A SECOND WALL MINTED.**
`UnlistedImportUse` already covers TYPE positions — `04_resolve.dag`'s
`resolve_node` emits it advisory when masked and the name is outside
`source_visible_names`, with `resolve_node_bounded_masked_boundary` declaring its
promotion to a hard refusal once the corpus burndown reaches zero. **The
value/call position emits nothing**, which is why that half of the class has no
count. Extending the same diagnostic to the call seam — same predicate, same
advisory posture, same burndown — is the increment, and it is the witness's own
declared next-rung trigger.

**SIZING, WITH THE PROVENANCE THAT MAKES IT USABLE.** The selective-unlisted half
is **186 call sites across 118 modules** over `dag`, `src/v2` and `src/v1` — a
static join, an upper bound, blind to local binders that shadow a name. The
pool-coincidence half has **no trustworthy number**: the same join reports 4784,
but arm (4) proves the census only serves what the closure loads, so that figure
counts providers that may never be in any real closure. **It is recorded as an
unbounded class rather than a count** — someone declining to publish a number
they could not defend, which is the discipline this document spent a day
rediscovering from the other direction.

**RUNG: outside the ladder — silent wrongness.** *"A bare call's meaning depends
on which modules the closure happened to load, and no diagnostic reports it at
the value position."* Ceiling: structurally guaranteed, since admission is
decidable from the consumer's own `resolved_imports` joined to the census
candidate's owner module, both already carried.

### 11.2i A resolver policy keyed on spelling — `trim`

`04_env.dag` `bare_free_call_requires_listed_import(name: String) -> Bool` is,
in its entirety, `name == "trim"`. It is a **declared** interim —
`closure_independent_bare_free_call_note` gives it a dissolve-on (the
`PrimitiveDefinition` identity-join) and the closure-membership witness records
what it patches (arm (3), not arm (2)) — so it is not an ad-hoc carve-out, and
that objection was raised and correctly withdrawn.

**The ground that survives is this program's own law.** The carrier takes a
`String`. A legitimate debt admission needs declaration identity, cause, owner,
exact population and retirement condition, and **a `String` cannot express
declaration identity at all** — so the function is structurally incapable of
proving it holds the one canonical `trim` rather than an unrelated homonym. It
keys policy on **spelling**, in a **resolver**, in a corpus whose entire current
defect is that spellings collide.

Read against §5's biconditional: `key_R(x) = key_R(y) ⟺ same_R(x, y)`. The
forward direction is broken — two distinct `trim` declarations receive the same
key — which is **under-keying**, the exact class this program exists to remove,
sitting inside the resolver that decides the question.

And the interim cannot be discharged where it lives: its own dissolve-on is a
`PrimitiveDefinition` identity-join, which is precisely what the `String` carrier
cannot represent. **Not urgent, and not to be cited as part of the namespace
design.** If this program passes near it, it gets an identity-grain admission —
never a wider `String` predicate. A one-line spelling check in a resolver is the
cheapest thing in the tree to widen and the hardest to notice widening.

### 11.2h A scoped guarantee written as a global one — and a call path that cannot see locals

**BELOW FLOOR, dispatched standalone, and explicitly NOT subsumed by this cut.**

`src/v2/compiler/emit_produced.dag` pattern-binds a function-valued field and
calls it:

```dag
ProducedDeclWired { render: render, … } => bind_outcome(o: render(decl), …)
```

`render(decl)` resolved to **`dag/std/layout.dag:59` `fn render(doc, proto)`** —
a module `emit_produced.dag` does not import. Executed evidence (CI run
32882906668): *"call contract mismatch calling `render`: missing required
argument `proto` (1 of 2 required argument(s) supplied)."* `fn render(` is
corpus-globally unique (verified across `dag/`, `src/v2/`, `src/v1/`: exactly
one), so it arrived through the one-candidate `global_bare` case.

**THE SEVERITY IS NOT THE ARITY ERROR.** It failed loudly only because one
argument was supplied against two required. **Had the signatures matched, the
call would have silently invoked an unrelated function from an unimported
module, with wrong output as the only symptom.** So the class is: *a local
binding can be captured by a corpus-global homonym, and whether you find out
depends on an accident of signature shape.* Silent wrongness — outside the
ladder.

**THE MECHANISM IS NOT A PRECEDENCE FAILURE.** `04_lookup.dag`
`callable_lookup_over_candidates` — the production call path — has exactly this
candidate space: `map_get(func_env.local, name)`, then
`parent_closure_callable_candidates`, then `builtin_callable_candidates`, then
`func_sig_from_global_bare`. Grepping its body for
`str_bindings|ancestry_str_bindings|.bindings` returns **0**. The value
environment is not on the call path at all. A pattern-bound field is a *value*
binding, so **the local does not lose a contest — the lookup that resolves calls
cannot see it.**

**AND THE PROSE THAT MISLED BOTH OF US IS THE GENERALISABLE PART.**
`global_bare_fallback_invariant` states: *"lookup_binding_by_name consults
global_bare ONLY AFTER str_bindings/ancestry_str_bindings/intern+bindings all
miss."* That is **true**, and it is about `lookup_binding_by_name`. The call
went through `lookup_func_sig`, a different lookup with no such guarantee and no
reason to have one, since it never touches those maps.

That is **authority substitution**: a guarantee stated for one path, asserted
about another, with no relation claimed by either carrier — and DESIGN's own
entry explains why review reads past it, *both halves check out and only the
arrow between them is missing*. The reporting lane committed the class **while
quoting the document that names it**, in an escalation whose subject was a
resolution defect. The note reads like a global precedence rule; nothing in it
signals that its guarantee is scoped to one lookup among several. **It misled
two people in one day**, so the repair carries a second half: the note names
which lookups it does *not* govern.

**NOT SUBSUMED BY THE CUT, AND THIS IS THE DISPOSITION THAT SOUNDS RIGHT AND IS
WRONG.** `global_bare_fallback_invariant` says the corpus fallback's removal is
downstream of reference-derived closure, which makes "wait for the cut" the
plausible parking spot. **Removing the fallback deletes the WRONG ANSWER without
adding value bindings to the call path.** After the flip, `render(decl)` stops
resolving to `std.layout.render` and starts failing to resolve *at all* — a
different wrong outcome from the same root. The defect survives the cut
untouched, so it is standalone and must not wait on this lane.

The repair that unblocked CI was a **rename of the local**, which is correct as
an unblock and is the workaround shape: it dodges the collision without touching
the rule that allowed it, and the next author who pattern-binds a
corpus-globally-unique name gets the same behaviour with no warning. Population
declined rather than estimated — *any pattern-bound name that happens to be
corpus-globally unique elsewhere is exposed* is the honest statement, and the
fix is a lookup-path change rather than a burndown, so no count is needed.

**The exact mirror of gunbc#9166**, which made an explicit import silently
discarded by a same-name local definition *refuse*. This is a local **binding**
silently discarded in favour of a corpus-global one. Same principle, opposite
direction.

### 11.3 The dispatch protocol

Every dispatch across a lane boundary carries five lines, and a dispatch without them is refused rather than interpreted:

```
AUTHORITY:    semantic | proof — which question this party may answer
CONTRACT:     the one production semantic contract at stake, stated as a sentence
EXACT HEAD:   branch @ sha, and its base
COUNTERPART:  who holds the other half of the seam, and what they owe
RETIRES WHEN: the condition under which this dispatch is complete
```

**At most one active dispatch per (AUTHORITY, CONTRACT) pair.** Two live dispatches against one contract is invariant (1) violated at the coordination layer instead of the code layer, and it produces the same result: two coherent parties, one incoherent contract.

The protocol is not ceremony, and it is not derived from taste. Every field is present because its absence has already cost something in this program: a dispatch went out with a title and no brief and was correctly blocked; a target spelling stated in three places drifted between them and blocked a peer lane; a measurement was taken on a stale head and nearly reported as a live defect. `EXACT HEAD` in particular is what makes the third one detectable by the recipient rather than by luck.

