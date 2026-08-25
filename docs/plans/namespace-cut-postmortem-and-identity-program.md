# Postmortem: gunbc#8282, and the identity program it revealed

**Subject:** why three generations of import deletion failed, what single defect
they were all standing on, and the terminal shape and landing order for each
piece of it.

**Status:** postmortem + direction. **It supersedes nothing.**
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

## 3c. A fifth consumer, at reconcile, and it is live on main

Relayed from `smart-ram-730`, found while confirming an unrelated emitter defect.
Both arms on one host, clean detached worktree at main `4f080fd88ae`, `DIRTY=0`,
same compiler binary, **one variable**.

**Treatment: one rename in one file** —
`dag/test/claim/srv3_path_ownership_witness_test`, `fn owner` →
`fn posix_owner_spec_of`, plus its 23 call sites *in that same file*. Nothing else
touched.

```
BASELINE    reconcile 15m, emit 43s -> 2 hard diagnostics (both emit-stage)
TREATMENT   the 2 disappear, and 8 NEW hard diagnostics appear,
            none of them in the edited file:

  ambiguous anonymous record literal matches 2 structs: FileOwnership, PosixSubject
      dag/test/claim/access_validation_test.dag  (x3)
  ambiguous anonymous record literal matches 4 structs: FirmwareSemanticVersion,
      GitReleaseVersion, MercurialUpstreamVersion, OllamaSemanticVersion
      dag/extdeps/git/versioning.dag
  ambiguous anonymous record literal matches 2 structs: DeclarationRef, RustItemDeclarationRef
      dag/test/claim/cache_retention_axes_witness_test.dag
      dag/extdeps/realization/reconcile_in_process.dag
```

**The 8 cannot be pre-existing-but-unreported, and the argument is what makes
this a finding rather than a curiosity.** Ambiguous-record errors arise at
*reconcile*, which is upstream of emit. The baseline run **reached emit** — it
reports emit done in 43 seconds — and reported exactly 2 diagnostics, both
emit-stage. A baseline whose reconcile held 8 hard errors could not have reached
emit at all. So the 8 were genuinely absent before the rename and present after.

Renaming one function private to one witness module changed **which structs an
anonymous record literal matches in four files that do not import it and were
never opened**. The disambiguation is evidently deciding against a corpus-wide
struct census, so compile outcome is a function of the **whole name set** rather
than of any module's own content. Two consequences belong on the ledger
explicitly:

- **a lane can turn main red by renaming something private to its own file**,
  with no import edge to the affected code and nothing a reviewer could see in
  the diff; and
- **every "the corpus compiles" claim is true only for the exact name-set that
  produced it** — which makes compile-clean receipts contingent on a fact nobody
  records. That reaches DESIGN §4b's evidence rules directly: a green measured
  under one name set is not evidence about another.

This is §3.3(b) generalized past the branch. Ambient whole-pool resolution was
never confined to the bare-reference scanner or to the import-free corpus: the
same *a name's meaning is a function of which other files exist* regime is live on
main today, at reconcile, over anonymous record shape matching. It is the fifth
direction found in one day and **the only one that reaches type resolution** —
the others being positional-vs-named argument binding, `order_typed_call_args`
(§3b), two accepted `contains` functions requiring different emitted authorities,
and `build_data_body_index` indexing every item with a body under its bare name.

**Not established, so it is not inherited as an overclaim:** no read of the
disambiguation code — this is a black-box before/after, not a mechanism. And the
rename *adds and removes a name at once*, so whether the trigger is the removed
name, the added name, or a shift in fold order is undetermined. The separating
experiment — add `posix_owner_spec_of` without removing `owner`, and the converse
— is cheap on the instrument that produced this and is the next step for this row.

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

1. **v1's 2093 `Map<String, _>` are not sloppiness.** They are the model
   faithfully describing what the realization can express. Repointing them to
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

**Answering the sizing question directly: the identity work is one program with
two ends, not one.** The emitter must stop erasing brands before the model-side
repointing can mean anything.

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

**T3 — `KeyedRoster` construction wall.** Success arm reachable only through
`keyed_roster_build`/`insert`. Already declared as
`feature:keyed-roster-construction-wall`.

**T4 — nominal identity survives emission.** `rust_nominal_identity_carrier_type_eligible`
returns a real predicate; a branded scalar emits as a newtype, not a `String`
alias. `Secret` and `String` stop being one Rust type. This subsumes §4b's
`UriValidatedScalar` forgery receipt.

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
deletion is the last step, not the first**. That is not a reversal of delete-first
and the reconciliation is one sentence: **the root being replaced is
import-derived resolution and closure, not import syntax.** DESIGN §3 says
"atomic describes the authority transition, not the amount of implementation
work" — so once nothing reads an import, the grammar is a corpse and deleting a
corpse is bounded cleanup. The invariant that keeps this honest:

> **No PR may leave a consumer that reads an import *and* a consumer that derives
> the same fact from references.** Each repoint moves one consumer all the way
> across. Dual authority is what delete-first forbids; inert syntax is not dual
> authority.

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
| **0** | any of it is measurable | compile outcome is a function of content, not the name set |

Step 0 is §3c, and it is load-bearing in a way that only appears under backward
derivation: **if a rename in an untouched file changes diagnostics elsewhere,
then no ledger is reproducible and no acceptance gate downstream means
anything.** Its placement is the open question in §9 — it is simultaneously what
makes the rest measurable and the only item with no known mechanism.

### Tracks

**Track 0 — independent, no ordering cost.** 0.1 pipe-position lowering (§3a,
fixture red) · 0.2 `order_typed_call_args` (§3b) · 0.3 the reconcile name-set
dependency (§3c) · 0.4 the `coercion_widening.dag:29:49` parse failure.

**Track A — realization.** A.1 `rust_nominal_identity_carrier_type_eligible`
becomes a real predicate; brands emit as newtypes. Independently confirmed by
`neat-fox-901`: the predicate has **5 call sites waiting on it**, and
`rust_nominal_identity_carrier_def` directly above it **already emits
`pub struct Name(pub String)`** — so both halves exist and only the admission
predicate is stubbed. This is a switch, not a build. A.2 the `UriValidatedScalar`
forgery becomes A.1's enrolled regression control (§4b(4)). *Precedes every key
repointing, because it is invisible to model-side checks.*

**Track B — one closure authority (chain step 1).** B.1 route every subject
construction through one authority — ordinary compile, test helpers, floor
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
wall. **Sized in decisions, not sites: ~238, not 2093.** 238 distinct binding
names cover the 1958 named sites; the top name is 37% of them and the top five are
61%. `source_indices` (726) plus its abbreviation `si` (106) is 832 sites that are
**one** `Map<String, NewlineIndex>` threaded through the compiler by parameter
passing — one keying decision replicated by threading, not 832 decisions. 132
names occur exactly once. Pricing this track per site overstates it by more than a
third.

**Track E — the migration (chain steps 4–5).** E.1 the derived projection.
E.2 the acceptance gates. E.3 the perturbation falsifier, which is 0.3's
permanent wall.

**Track F/G — the flip, then the corpse.** F.1 the one motion: apply the rewrite,
flip resolution and closure authority, delete every ambient fallback, regenerate
stage0. F.2 repoint the last import *readers* — `source_visible_names`,
`resolve_module_imports`, v2 `admit_imports`, translate, body-lowering,
`topological_sort`'s ordering. G delete the grammar, AST, keyword and
productions.

F.1 stays one motion because that is the authority transition. F.2 and G are the
trailing steps the operator's framing makes safe: after F.1 no consumer reads an
import, so they are deletions of dead surface rather than a cutover.

### Two decisions taken (operator, 2026-08-25)

- **Target spelling: full qualification.** Chosen as the conservative default.
  Shortest unique suffix stays live evidence from `namespace-resolution-design`
  §8 and can be revisited, but the sweep is written against full qualification so
  the transform has one output format.
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
| **M0** | **Measurable.** Compile outcome is a function of a module's content, not the corpus name set | §3c's perturbation control: renaming a private declaration changes no diagnostic elsewhere |
| **M1** | **Emission preserves identity.** A branded scalar is a distinct Rust type | a `Secret` cannot be passed where a `String` is expected in emitted Rust; `UriValidatedScalar` forgery refused |
| **M2** | **One closure authority.** Every subject is built the same way | no second `extract_imports`; every closure edge names its occurrence and provider |
| **M3** | **Identity survives the pipeline.** Resolution's answer reaches emission intact | the 12 censused emitter sites key on declaration identity; `build_emit_graph_info` has a collision arm |
| **M4** | **Keys are typed.** A spelling is unspellable in key position | `key_eq` is not a parameter anywhere; a content-derived `key_of` does not compile |
| **M5** | **The projection is green.** A binding-preserving import-free corpus exists, derived | `old_resolved_declaration == projected_resolved_declaration`, 0 binding-identity changes |
| **M6** | **Authority flipped.** Containment resolution is production | ambient fallback deleted; stage0 regenerated; acceptance set green |
| **M7** | **Imports unread.** No consumer reads an import | the last readers repointed |
| **M8** | **Grammar deleted.** | `import` refuses at parse; the AST and productions are gone |

M0–M4 are independently valuable on main *with imports*. M5 onward is the cut.

### Wave order

```
WAVE 1   0.1 pipe lowering   0.2 arg ordering   0.4 coercion parse
         A.1 nominal identity switch    A.2 forgery control
         B.1 closure authority          B.2 edge provenance
         D.1 key algebra authoring
         0.3 name-set investigation  (ordering open — see §9)

WAVE 2   B.3 parser-owned reference facts        [needs B.1]
         C.1 delete ambient + proximity fallback [needs B.1]
         C.2 build_emit_graph_info collision arm [needs A.1]
         D.2 impostor separation, two arms       [needs A.1]

WAVE 3   C.3 remaining emitter sites   [needs C.2]
         C.4 OccurrenceBinding ledger  [needs C.1 — a ledger over a heuristic records the guess]
         D.3 KeyedRoster wall          [needs D.2]

WAVE 4   E.1 derived projection  [needs C.4]
         E.2 acceptance gates    [needs E.1]
         E.3 perturbation falsifier — 0.3's permanent wall

WAVE 5   F.1 the flip                       ← one motion, single owner

WAVE 6   F.2 last import readers    G grammar deletion
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
C.4, and F.2. Substrate owns D. E, F.1 and G sit with the integrator, because the
projection and the flip consume all three lanes and belong to none.

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

- **shortest unique suffix** is a *terminal shape* question. It survives as
  evidence, and it is a live input to the Phase D sweep — full qualification is
  not obviously the right target spelling, and §5's own restatement (the key is
  the declared identity carrier, and *spelling* is not identity) is agnostic
  between the two.
- **per-subtree migration** is a *sequencing* question. It is explicitly
  superseded by the ruling above. Big-bang deletion stands.

**What survives from the dissent regardless, and should be adopted:**

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

- **`undefined variable 'v2'` × 95** — the largest unattributed share of the
  branch's 410. Relayed as *not* the pipe case; deep-ant-102 reports testing and
  **refuting** the obvious cascade hypothesis (a broken provider module does not
  degrade a consumer's qualified reference to a variable lookup). The only
  remaining candidate this document knows of that could still be structural. Not
  reproduced here.
- **Witness disposition gap** — a missing declaration fail-closes to
  `ReadsLiveTree`, so the witness never runs; 476 of 1514 witness files declare
  none, carrying 3906 test fns, against a floor reporting `declined_live=830`.
  ~3000 test fns are unaccounted for, and **not-discovered is worse than
  declined**. deep-ant-102 explicitly does not conclude which, and names the first
  check: whether the file glob is wider than the floor's discovery scan. Separate
  lane; it is not import work and should not be folded into this program.
- **The reconcile name-set dependency (§3c)** — the separating add/remove
  experiment, then a read of the disambiguation code. Owned here; the instrument
  is `smart-ram-730`'s. **Its ORDERING is deliberately left open** (operator,
  2026-08-25): the backward chain puts it at step 0 because it is what makes
  everything downstream measurable, and it is simultaneously the only item with
  no known mechanism, so front-loading it is the least predictable choice on the
  board. Decided when the separating experiment says what the trigger is.
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
- **Track A has no owner.** The phase everything else is sequenced behind.
- **Phase D's target spelling** — full qualification vs shortest unique suffix,
  per §8. Needs an operator decision before the sweep is written, not after.

## 10. What this document does not claim

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
