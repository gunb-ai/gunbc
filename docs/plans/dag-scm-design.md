# Source-intent integration — a first-principles SCM design

**Status:** operator-directed design seed, revised 2026-07-27. This revision supersedes this
file's 2026-07-25 premise that “merge = keyed diff over node identity.” It preserves the signed
visibility-first sequence and the landed economics corpus, but treats nodes, files, branches, and
three-way comparison as possible evidence or realizations—not as the foundation.

“Source-intent integration” is a descriptive name in this plan, not a proposed substrate type.
No carrier lands from this document alone. The first implementation step remains concept-DFS in
`std/` and a discriminating scenario corpus; the plan must not create an SCM-local nickname for a
concept the substrate already owns.

Roadmap carrier: ROADMAP §2, “SCM — source-intent integration, visibility-first.” Economics
grounding: DESIGN open thread “SCM economics — the GitLab 10-K corpus.” Visibility authority:
[node/subtree visibility grants](node-subtree-visibility-grants.md). Storage/surface authority:
[module identity vs storage](module-identity-storage-binding-design.md).

## 1. Product objective — minimize judgment without buying silent errors

The product is not “Git with smaller conflict markers.” It is:

> **Minimize the total human/LLM judgment needed to accept concurrent source changes, subject to
> never automatically asserting a result the available model and evidence do not ground.**

Humans and LLMs are both expensive, lossy judgment providers. They should receive only the
irreducible normative choices. Everything else—alignment, replay, independence, invariant
checking, affected-set calculation, receipt reuse, and projection—should be mechanical.

The displaced cost has four terms, all denominated in time:

1. integration compute;
2. CI work invalidated or repeated because main advanced;
3. human/LLM attention and tokens spent reconstructing context and resolving conflicts; and
4. recovery from a clean-looking but wrong automatic integration, paid later at interest.

The motivating workload is an explicit **stress profile**, not a universal constant: 50 agents per
developer, 10 proposals touching the same modeled region, and a 30-minute CI path. Every proposal
invalidated after it was green spends at least another affected 30-minute validation interval;
serial invalidations also add wall-clock queue delay. The first model must carry these as workload
inputs so the benefit is measured rather than asserted.

A lower textual-conflict count is not itself success. An aggressive integrator can improve that
number by silently choosing wrong results. The product metric is **judgment displaced at a fixed
safety contract**, with false automatic integrations priced most heavily.

## 2. Separate the five concerns

Git's native interface necessarily exposes commits, trees, paths, blobs, and textual merge
drivers. Those are useful storage and compatibility concepts. They are not the program.

This lane keeps five layers separate:

| concern | authority | never allowed to decide |
|---|---|---|
| program model | the actual `Node` + `Edge` graph, grounded facts, claims, effects, and bounds | file layout, line position, or formatter output |
| proposed change | a transformation over modeled facts plus its explicit obligations and provenance | an inferred story about a user's unexpressed preference |
| integration | the joint-result calculation and its evidence/refusal | queue arrival order, path overlap, or a tie-break heuristic |
| history | grounded transition and validation receipts over accepted states | a second copy of the program |
| medium / transport | `.dag`, Rust, markdown, files, Git trees, CLI/REST, remote storage | semantic correctness beyond the medium's declared decode fidelity |

Text and files have two downstream jobs, already named by the storage-binding design:

- **capture:** lift an edit made through a surface into a proposed graph transformation, with an
  explicit fidelity verdict; and
- **projection:** render the accepted graph through the declared medium.

A path move, import deletion, formatting pass, or source reorder may therefore be a semantic no-op
even though its Git tree changes. Conversely, two disjoint textual hunks may jointly violate one
modeled invariant.

## 3. Grain comes from groups of units; SCM does not choose one

The recent group-of-units lane is the starting authority. Its current concrete carrier is
`gunbc.roster_registry.GroupMembership`, which distinguishes membership known:

- **by containment**;
- **by derivation**;
- by a declared, counted **frontier** with a reason and dissolution trigger; or
- by an ungrounded nickname, which is the violation.

This is the useful content behind the working shorthand `Group<U>`; this plan must not mint a
competing generic group type merely because the shorthand is convenient.

A module is one important example: a containment-derived group of program units. An affected
closure is a derived group. A temporarily hand-curated migration set is a frontier group. Files
are storage realizations of such groups, not group authorities.

The integration algorithm is therefore **unit- and group-polymorphic**:

- a unit may be a declaration, binding edge, argument edge, claim, grammar row, effect, or another
  grounded fact;
- a proposal may affect one unit or a derived group of units;
- the evidence needed to align two observations depends on the unit's actual authority; and
- “same node,” “same module,” and “same file” are observations at different grains, never universal
  conflict definitions.

Identity remains involved, but only as detective evidence. A key can align two facts through time;
it does not decide what their joint result should be. No universal durable `ScmEntityId` is assumed.
Containment position, declaration reference, content identity, normalization, bounded extensional
identity, provenance, and an authored transformation can each supply different grades of evidence.
Unknown alignment stays unknown.

## 4. The native transaction is a proposed transformation, not an endpoint diff

A native proposal should eventually carry, by reuse of existing substrate concepts:

- the authoritative target it proposes to advance;
- the exact accepted parent state it was authored against;
- the modeled transformation—what facts or relations it proposes to change;
- explicit preconditions and postconditions the author relies on;
- claims, effects, resource bounds, and observable behavior relevant to admission;
- a derivable affected group and dependency closure;
- provenance, including the authoring surface if one captured it; and
- capture/decode fidelity.

These are **roles**, not a proposed `Intent` record. `EffectAttemptIntent`, `StandingIntent`, graph
intent in the orchestration lane, `ChangeSet`, temporal snapshots, and other neighboring concepts
must be DFS'd before any new carrier is named.

An endpoint diff remains valuable, but it answers only “what observations differ?” A native
proposal answers “what transformation was requested, against which accepted state, and under
which obligations?” Capturing the operation before flattening it to two snapshots removes much of
the detective work a later three-way merge must redo.

### Native history

Authority is **target-scoped**, not a property of a branch name. A project may have a primary
target, supported-release targets, or another explicitly admitted target, but each target has one
current accepted state and each proposal names exactly one target. `main` is one possible Git
projection of that role; checking out or receiving another ref does not make it an authority.

The authoritative history for one target can be a linear sequence of accepted transitions:

```text
target T at accepted state S_n
  + explicitly selected transformations authored against known parents
  + current evidence and admission policy
  -> accepted transition receipt R_(n+1)
  -> target T at accepted state S_(n+1)
```

Workspaces may be concurrent, but branches and merge commits are not fundamental. A workspace is a
sandbox containing pending proposals. Its construction history is evidence, not accepted history.
Acceptance records the target, exact accepted predecessor, each proposal's authored parent, the
applied transformation set, the resulting state hash, and the evidence/receipts that grounded it.
Git commits and trees are a third thing: compatibility projections of those facts.

This is not “last writer wins with a nicer log.” Before a stale-parent proposal can advance current
state, the system re-evaluates its transformation against current modeled state. Arrival order is
not intent. If two compatible proposals commute, validation may run in parallel and either
serialization must yield the same declared-equivalent result. If order changes meaning and no
policy declares an order, the system asks rather than letting queue timing choose.

Native transactions therefore do not require three endpoint snapshots as their primitive. The
authored parent remains necessary causal evidence; the operation is replayed and verified on the
new state. Three-way comparison remains necessary at a **lower-fidelity import/capture boundary**
where only base, left endpoint, and right endpoint survived.

No wall-clock fact supplies semantic order. Commit time, authoring time, upload time, observation
time, queue position, and “objectively arrived first” may be retained as audit or performance
evidence, but cannot rank proposals or recover intent. A transition precedes another because an
authorized receipt names its accepted predecessor; one proposal depends on another only through an
explicit parent/dependency edge. A declared domain-time contract may decide whether evidence is
fresh enough to use, but never turns chronology into proposal priority.

### Deduction contract — one fold, not an interaction catalog

Native integration is one general reconciliation operation over modeled facts, not a growing
dispatch table for “rename versus call,” “move versus edit,” or every future pair of language
features. Conceptually:

```text
obligations =
  fold(selected proposal set, empty, compose grounded obligations)

candidates =
  derive(accepted target state, current observations, obligations, admission contract)

outcome =
  classify(minimal admissible candidates)
```

The proposal fold accumulates transformations and obligations; it does **not** apply a list in
arrival order. Its compatible region must compose associatively and commutatively. Failure to
compose is evidence for `Contradictory`, `Ambiguous`, or `Unknown`, never a reason to add a
first-arrival branch.

Safety, explicit obligations, grounding, and the required evidence grade are admission conditions,
not weighted preferences. A cheaper result cannot buy permission to violate one. Among candidates
that meet those conditions, preserve every fact whose change is not grounded, then use the
declared cost order to eliminate only strictly dominated realizations. If several materially
different candidates remain equivalent or incomparable, no hidden weight represents “what users
usually mean”: the answer is `Ambiguous`.

New models participate by contributing their ordinary facts, relations, claims, effects,
equivalence, bounds, and costs to this fold. They do not add cases to a central SCM switch. The
worked examples below are discriminating inputs for the same operation, not its implementation
branches. If adding a language concept requires editing the integrator rather than supplying facts
through a shared authority, the missing work is a model/algebra seam to ground—not a twentieth-year
special case to preserve.

### Closedness and humility — more information is not omniscience

Daglang's advantage is information density: types, bindings, containment, effects, claims,
resource bounds, dependency closure, and execution receipts can all be premises that Git does not
have. That supports more deductions; it does not grant permission to infer an unexpressed premise.

> A modeled fact `X` licenses consequences of `X`. It does not prove that no relevant, unmodeled
> fact `Y` exists.

The change set is closed in one useful sense: every accepted result change must be grounded in an
admitted proposal or declared canonicalization. The world of relevant meaning is not automatically
closed merely because the program typechecks. A closed coproduct closes its declared axis; a
derived affected/dependency closure closes only the relation and bound its evidence names; a
lossless surface closes only its declared construct set. `Applied` therefore requires a
closedness/completeness witness for the **specific admission contract**, not a claim that all user
intent or all program behavior has been modeled. If a potentially relevant axis, dependency,
effect, capture fragment, or proof bound is missing, the result is `Unknown` at that evidence
grade.

This is also how the model can evolve without making old confidence implicit. An admission receipt
names the model/contract inputs and closedness evidence it relied on. Adding a relevant model fact
invalidates the affected receipt and causes re-evaluation; absence in the older model never becomes
negative evidence by inertia.

Two live repo precedents carry the same epistemic rule:

- `gunbc.os_install_deduction` carries an attested `observed_at` timestamp, but the timestamp does
  not strengthen the runtime verdict. Even a visible login prompt yields
  `KvmSuggestsOsBooted`, not `RuntimeOsInstalled`; the stronger conclusion is reserved for
  independent read-back evidence.
- `gunbc.membership_reconcile` turns a removed member with absent ownership evidence into
  `MemberTeardownRefused { cause: OwnershipUnknown }`. There is no effect arm that can reinterpret
  “ownership was not modeled” as permission to destroy the member.

These are precedents for the evidence discipline, not claims that source integration is an
OS-install or membership-diff algorithm.

## 5. The integration result — four honest outcomes

Illustrative result shape (roles only):

```text
IntegrationOutcome<Result, Evidence, Question>
  = Applied {
      result,
      grounding_evidence,
      scope_and_closedness_evidence,
      no_extra_change_evidence,
      reusable_validation_receipts
    }
  | Contradictory {
      minimal_incompatible_proposals,
      counterexample
    }
  | Ambiguous {
      materially_distinct_valid_results,
      smallest_required_question
    }
  | Unknown {
      missing_model_or_evidence
    }
```

- **Applied** means the relevant candidate/evidence scope is closed for the admission contract and
  exactly one result equivalence class satisfies every admitted proposal and required obligation,
  adds no ungrounded change except declared canonicalization, and survives the declared cost order.
- **Contradictory** means a closed admission problem proves that no result can satisfy the joint
  obligations. Return a minimal incompatible core and a witness, not a broad conflict region.
- **Ambiguous** means more than one materially distinct result is valid and the model contains no
  authority for choosing among them. Return the alternatives and the smallest normative choice.
- **Unknown** means candidate closedness, model, alignment, fidelity, observation, or bound is
  insufficient to establish another outcome. Finding one plausible candidate without proving the
  relevant scope closed is still `Unknown`; it is neither “probably safe” nor “everything
  conflicts.”

The admission policy states the evidence grade it requires. A structurally universal result must
not be presented as behaviorally safe; a behaviorally bounded result must name its bound. If the
policy requires a proof the system cannot produce, `Unknown` blocks automatic admission.

Humans or LLMs may answer an `Ambiguous` question by submitting another explicit proposal or
claim. They may help model an `Unknown`. Their answer does not retroactively turn a guess into
evidence.

### Laws for the compatible region

For proposals admitted as jointly compatible, integration should be:

- **idempotent:** submitting the same proposal twice has the same result as once;
- **commutative:** arrival order does not change the result;
- **associative:** compatible batches compose without batch-boundary semantics;
- **grounded:** every result change traces to an admitted proposal or declared canonicalization;
- **minimal/universal:** every admitted change appears exactly once and no other semantic change
  appears;
- **closedness-qualified:** uniqueness is claimed only over the candidate/evidence scope proven
  complete for the declared admission contract;
- **exact-parent checked:** replay states what changed since authoring and never applies against an
  assumed parent;
- **metadata-inert:** erasing or permuting incidental timestamps, arrival metadata, queue order,
  or object identifiers cannot change the semantic outcome;
- **order-honest:** order dependence yields `Ambiguous`, `Contradictory`, or `Unknown` unless an
  explicit policy makes that order part of intent; and
- **fail-closed:** missing alignment, fidelity, or proof never widens into an automatic result.

## 6. Worked examples

### 6.1 Different call arguments — often compatible, never assumed compatible

Accepted source:

```dag
foo(a, b)
```

Proposal P changes the argument bound to `foo`'s first parameter:

```dag
foo(c, b)
```

Proposal Q changes the argument bound to its second parameter:

```dag
foo(a, d)
```

A text merge sees two edits to one line and will ordinarily refuse. A node-key rule might say the
call node overlaps and also refuse. Neither answer uses all the available model.

The program graph can instead observe two distinct argument-binding edges. If it also establishes
that:

- both edges still target the same resolved `foo`;
- each replacement inhabits its parameter type/refinement;
- the combined call satisfies cross-argument preconditions;
- effects and resource claims remain admissible; and
- every required behavioral claim holds within its declared bound,

then P and Q commute and the sole grounded result is:

```dag
foo(c, d)
```

If `c` changes overload resolution, or `foo` requires the two arguments to share a region, the
result may be contradictory. If both rebinding interpretations are valid and materially different,
it is ambiguous. If the relevant relationship is not modeled, it is unknown. “Different argument
keys” is evidence that starts the check, not permission to skip it.

The reverse is equally important: two transformations touching the **same** coarse key are not
automatically contradictory. They may be identical, act on independent substructure, or normalize
to one transformation. Key intersection identifies where deeper reconciliation is required.

### 6.2 Rename plus a new reference — textual clean merge, semantic question

P renames declaration `foo` to `bar`. Q, authored concurrently, adds a call spelled `foo()` in a
different line or file. Git can merge this cleanly while leaving a broken reference.

With binding provenance, the native possibilities are explicit:

- Q targeted the same declaration and the rename transports the reference → `Applied`, emitting
  `bar()`;
- Q explicitly requires a distinct/public name `foo` to remain → `Contradictory`; or
- endpoint import cannot establish which declaration Q meant → `Ambiguous` or `Unknown`.

The system does not infer “they probably meant the renamed function” from spelling similarity.

### 6.3 Disjoint edits with an emergent invariant failure

P lowers one authorization threshold. Q broadens a separately stored audience grant. The edits
touch different units and files, but together violate a claim such as “untrusted callers can never
reach destructive effect E.” Integration returns `Contradictory` with that claim's counterexample.
A clean textual merge is not safety evidence.

### 6.4 Two valid results, no authority to choose

P changes a representation. Q inserts an adapter whose placement before or after that change
produces two valid but observably different programs. If both satisfy the current claims and no
modeled policy defines placement, the result is `Ambiguous`, not a deterministic tie-break.

### 6.5 Surface-only change

A formatter reorders declarations, a module moves between files, or namespace-derived imports
disappear while the graph stays identical. The semantic transformation is a no-op; storage binding
or projection provenance changes separately. No semantic conflict exists.

## 7. The safety quadrants and the real bar

“Compatible” below means compatible relative to the explicitly required model, claims, fidelity,
and bounds—not unknowable private thoughts:

| underlying joint result | textual Git says clean | textual Git says conflict |
|---|---|---|
| compatible | correct automatic merge | **false conflict**: unnecessary judgment and CI delay |
| incompatible | **silent wrong merge**: highest-cost failure | correct refusal, usually imprecise |

Daglang's opportunity is not merely moving cases from the right column to the left. It is:

- prove more compatible cases and return `Applied`;
- prove more incompatible cases and return a small `Contradictory` witness;
- expose genuinely normative choices as `Ambiguous`; and
- keep incomplete modeling visible as `Unknown`.

The strongest honest “true final intent” guarantee is:

> The accepted result is the unique result, up to declared equivalence, that satisfies all explicit
> proposals and required obligations, and every result change is grounded in those proposals.

No SCM can guarantee an unexpressed mental preference. Claiming that would convert missing
information into a silent decision.

### Evidence ladder

Evidence composes from weaker to stronger scopes:

1. **capture fidelity:** exact authored transformation, lossless structural recovery, ambiguous
   recovery, or unknown;
2. **structural preservation:** well-formed graph; every captured edit represented once; no
   invented structural edit;
3. **resolution and typing:** bindings, inhabitance, refinements, and namespace claims hold;
4. **cross-unit claims:** invariants over derived groups and dependency closures hold;
5. **effects/resources:** interference, grants, resource bounds, and temporal preconditions hold;
6. **behavioral evidence:** required observables agree within an explicit bound; and
7. **normative intent:** only what the author explicitly supplied—never inferred from plausibility.

An admission surface may require a particular rung. The receipt must name the rung; lower evidence
cannot masquerade as higher.

### Comparison evidence, not design authority

Git's documented default merge strategy combines non-overlapping textual changes and exposes
content conflicts when it cannot. Rename/copy pairing is a similarity calculation. A clean result
therefore means the configured merge machinery found no conflict; it is not a language-semantic
proof ([Git merge](https://git-scm.com/docs/git-merge),
[diffcore rename detection](https://git-scm.com/docs/gitdiffcore#_diffcore_rename_for_detecting_renames_and_copies)).

The empirical baseline reinforces the safety constraint:

- The ASE 2024
  [evaluation of version-control merge tools](https://doi.org/10.1145/3691620.3695075) reports,
  over its corpus, Git `ort` at 2,748 correct / 3,078 unhandled / 157 incorrect scenarios; roughly
  5.4% of Git's handled outputs in that experiment were incorrect. Its structural Java baseline
  handled more cases but also produced more incorrect results—evidence that “fewer conflicts”
  alone is the wrong objective.
- The 2026 preprint
  [On Correctness of Software Merge](https://arxiv.org/abs/2607.07987) gives a useful structural
  floor—parsability plus universal edit preservation—and checks it over 43,774 Java file
  scenarios. Its guarantee is deliberately syntactic/structural and depends on AST differencing;
  it does not establish behavioral or normative intent.

These baselines belong in the discriminating corpus. They do not define daglang's model.

## 8. Existing concept DFS — reuse, do not fork

The first modeling pass must try to compose these authorities:

| existing carrier | contribution | limit for this lane |
|---|---|---|
| `gunbc.roster_registry.GroupMembership` | how membership in a group of units is known | provenance of membership, not yet the whole integration value |
| `std.change.keyed_two_way_diff` | exact keyed endpoint observation | observes change; does not infer or reconcile peer proposals |
| `std.change.keyed_three_way_fold` | conservative base/observed/desired reconciliation | asymmetric desired-state/infrastructure shape; key overlap currently collapses to conflict |
| `gunbc.membership_reconcile` | one generic desired-vs-observed fold with stable member identity and an un-emittable ownership-unknown arm | infrastructure convergence and an epistemic precedent, not concurrent author intent |
| `v2.std.find_witness.CandidateSet` + `UniqueOnly` | finite closed candidate selection with no/unique/ambiguous outcomes | supplied structural candidates only; does not generate the SCM candidate space or prove behavioral completeness |
| `v2.std.constraints.ConstraintGraph` + `solve_constraints` | existing structural “find what satisfies” authority | extend/compose after DFS; do not mint an SCM-local solver nickname |
| `std.realization_reconcile` | apply → read-back → grounding evidence | receipt shape, not source integration semantics |
| `std.temporal_effect` | exact snapshots, intent hashes, idempotency, prior receipts, generations | effect-attempt vocabulary; do not rename it into generic source intent |
| `std.computation_identity` | structural/normalized/bounded-extensional evidence plus typed unknown | identity evidence, not user-intent identity |
| `std.perturbation` | response to changed inputs | a building block for bounded noninterference evidence |
| `std.realization.Independence` | `Independent | Dependent | Unknown` | currently effect-shape-specific and deliberately coarse |
| `std.pareto` + `std.realization_schedule.CostAccount` | dominance without hidden scalar weights; grounded time/space/power accounting | may prune strictly dominated safe candidates; cannot trade cost for safety or choose between incomparable intents |
| `gunbc.os_install_deduction` | evidence-graded deduction where timestamps remain provenance and weak observations stay weak | domain-specific precedent, not an SCM outcome or solver |
| `v2.compiler.source_authority` + `DecodeFidelity` | ingest/emit authority and honesty boundary | only lossless fragments may recover exact proposals |
| affected-set and materialization lanes | dependency-scoped validation and content-keyed receipt reuse | selection/caching must not decide semantic compatibility |
| `extdeps.git` + `extdeps.git.object_store` | cited Git operation, object, tree, commit, ref, and diff interface shapes | external compatibility authority; Git transport/policy and source integration stay separate |
| GitLab/Atlassian/Microsoft SEC, pricing, and `gunbc.econ.scm_*` carriers | grounded distribution/serving/agentic-stress economics | price the product and store; they do not imply integration semantics |

The existing keyed diff remains useful beneath the new model. Its meaning changes from “the merge
algorithm” to “one observation/capture engine.” In particular, the storage-binding plan's current
same-key refusal is the safe adapter available **before** semantic integration lands; it is not a
proof that same-key proposals are contradictory.

The same anti-fork rule applies to “deduction.” The first implementation must determine how the
existing finite-candidate, structural-constraint, Pareto, closure, and diagnostic authorities
compose. It must not land a generic `ScmSolver`, `ReconciliationConstraint`, or interaction-rule
registry beside them merely because the SCM is the first demanding consumer.

## 9. Admission and CI — linear history without serial work

The native fast path:

1. Read target `T` at accepted state `S_n`, its admission contract, and validation receipts.
2. Observe required current facts with explicit authority/fidelity; missing observation stays
   missing.
3. Author a modeled transformation directly, or capture one from a declared lossless surface.
4. Select and record the proposal set at an explicit admission frontier. An authorized actor or
   declared policy may select it; transport arrival, timestamps, and scheduler iteration may not.
5. Derive the affected group and required claims, then fold the selected proposals into grounded
   obligations.
6. Derive and classify minimal admissible candidates against `S_n`.
7. Reuse receipts whose content-addressed model/contract/dependency inputs are unchanged; validate
   only newly
   affected obligations.
8. If `Applied`, atomically advance `T` only while its current state is still `S_n`; otherwise
   restart from step 1. For any other outcome, return the smallest
   `Contradictory`/`Ambiguous`/`Unknown` handoff.
9. Project the new state to all declared surfaces, including Git.

Linear accepted history does not imply serial validation. Compatible proposals and their affected
claims can evaluate in parallel; the acceptance log serializes receipts only after their
commutativity and current-parent conditions are established.

The frontier solves a finite coordination question without pretending to know the future. A
proposal not yet observed cannot block progress. But if two known proposals are order-dependent,
automatically accepting whichever the scheduler happened to visit first would itself be a
normative choice. Once an authorized acceptance advances `T`, its receipt—not its timestamp—makes
the result the authoritative prior for later proposals.

This is the direct answer to green-then-main-advanced CI waste. A receipt should be keyed by the
actual modeled inputs of the claim, including any closedness evidence, not by “the whole branch is
still at this SHA.” If the target
advances outside those inputs, the green receipt remains grounded. If it advances inside them, the
system revalidates the affected claim before acceptance. A blanket rerun is an `Unknown`/modeling
deficit, not a silent “fail-closed” success.

## 10. Git is a compatibility realization, not the semantic authority

Compatibility is non-negotiable for adoption, but it is downstream:

### Export

- Each accepted native transition can emit an ordinary one-parent/squash Git commit and source
  tree. A team using squash-to-main sees the history shape it already expects.
- Native proposal/evidence receipts may travel as optional metadata; a normal Git client can ignore
  them and still clone/build the projected tree.
- Files, paths, formatting, opaque blobs, and derived artifacts follow the storage-binding and
  materialization policies; they are not smuggled into the semantic model.

### Import

External Git work supplies base and endpoint snapshots. Ingest recovers graph transformations with
an explicit fidelity result:

```text
ExactAuthored | StructurallyRecovered | AmbiguousRecovery | UnknownRecovery
```

Those labels are illustrative roles pending DFS. Rename similarity, matching content, and
three-way ancestry are evidence. They never become invented identity. A textual merge driver may
remain as a compatibility fallback, but its clean result carries only textual evidence and cannot
claim the native safety grade.

The first product can therefore be a Git-compatible semantic admission tool rather than a new
hosted object store. It proves the integration advantage while keeping clone, editor, CI, and forge
workflows intact. A native store is justified only after the semantic consumer displaces measured
cost.

## 11. Visibility, locked realizations, and customer trust

The [visibility-grants authority](node-subtree-visibility-grants.md) §11 owns the publication
rungs, ciphertext/interface cut, admission/refusal rules, hole residues, placement limits, churn
blinding, and crypto-shred semantics. This plan neither abbreviates nor redefines them.

The only integration consequences carried here are:

- the signed order remains: implement `Reference`/`Publish` first over today's two Git storage
  roots; and
- the same grant interface later constrains native history, storage, projection, and remote
  execution without becoming an integration/conflict predicate.

No `Applied` integration outcome can widen a publication grant, and no secrecy or publication
claim can be inferred from structural compatibility. The concerns compose through explicit grants
and realization bindings only.

Vertical integration can bring the product closer to real compiler/use-case failures. It must not
turn “visibility into customer code” into an undeclared surveillance business model. Raw source
stays governed by `Publish`/effect grants; telemetry is derived, consented, and minimal—fidelity
bucket, outcome bucket, cost, and anonymized mechanism gap where possible. Trust is part of the
safety product, not a growth shortcut.

## 12. Product thesis and falsification

The idea is interesting because agent concurrency changes the economics of source control. A
30-minute validation path and many simultaneous proposals make integration quality a direct,
compounding compute/latency/token cost. Daglang has unusually rich evidence available because the
program, claims, compiler, affected set, and execution model can share one substrate.

The product advantage is therefore not “the language knows what the user meant.” It is:

> **More modeled information produces more mechanical deductions, while every missing closure,
> fidelity, or normative fact remains visible as `Unknown` or `Ambiguous`.**

The user-facing operation stays simple: reconcile proposals and observations against the accepted
target, apply the unique safe minimal result when it is grounded, and otherwise ask the smallest
question. The internal model can deepen for decades without turning that operation into a catalog
of language-feature interactions.

The initial wedge is narrow and credible:

- teams running many coding agents against the same modeled monorepo;
- long or expensive CI;
- a lossless daglang source model;
- Git-compatible input/output; and
- a dashboard showing judgment and validation work actually displaced.

The GitLab corpus supports the “cheap to serve” premise, not “distribution is free.” GitLab's
serving cost is a minority of revenue while selling and R&D dominate. Word of mouth is plausible
only if the product makes savings obvious in the existing workflow. Compatibility and receipts are
therefore part of the product, not marketing afterthoughts.

Track at least:

- automatic `Applied` rate by evidence rung;
- `Contradictory`, `Ambiguous`, and `Unknown` rates and causes;
- judgment requests, human minutes, and LLM tokens per accepted proposal;
- false-conflict rate relative to the required contract;
- detected and escaped wrong integrations;
- CI minutes and wall-clock delay invalidated, rerun, and reused;
- proposal queue latency under the 50-agent/10-overlap stress profile;
- Git-import fidelity distribution and round-trip fidelity; and
- storage operations/bytes/egress under the landed packing and provider models.

The “unsafe automatic integration” target is not hand-waved as absolute. Every claim is scoped to
the declared model, fidelity, and bound, and the falsifier corpus must actively search for escaped
wrong results.

## 13. Discriminating scenario corpus

Before storage or forge work, land a model-level corpus whose expected outcomes separate the
designs:

| scenario | textual/key-overlap baseline | required native outcome |
|---|---|---|
| `foo` arguments changed independently | textual conflict / call-key overlap | `Applied foo(c,d)` only with binding + joint-obligation evidence |
| rename plus concurrent old-spelling call | often textually clean | transported binding, `Contradictory`, `Ambiguous`, or `Unknown`; never broken-clean |
| disjoint policy changes violate one claim | textually clean | `Contradictory` with minimal claim counterexample |
| two valid order-dependent transformations | deterministic queue result | `Ambiguous` with the two material alternatives |
| duplicate proposal delivery | duplicate patch/application risk | idempotent `Applied` |
| formatting/reorder/file move, same graph | large textual diff | semantic no-op; storage/projection change only |
| lossy Git import | plausible inferred delta | `Unknown`/`AmbiguousRecovery`, never exact-intent claim |
| absent required behavioral bound | structurally clean | structural evidence only; admission `Unknown` if behavior is required |
| identical facts/proposals with timestamps and arrival order permuted | first/last arrival often becomes operational order | identical native outcome and result evidence |
| one plausible result but relevant candidate/dependency closure unproved | clean-looking candidate | `Unknown`, never “unique because nothing else was modeled” |
| new modeled invariant contributed through the shared claim relation | new merge-driver case | outcome changes through the same fold; no integrator branch changes |

Every `Applied` case needs a perturbation that makes one obligation false and turns the outcome
red. Every refusal needs a nearby compatible control, so “always refuse” cannot satisfy the suite.
The timestamp case must permute and erase incidental time metadata. The model-extension case must
add its invariant as data while holding the reconciliation implementation fixed; otherwise the
claimed general algorithm has become a disguised interaction registry.

Git's default merge, the current keyed-diff adapter, and at least one structural merge baseline
should run on the same corpus. The comparison is evidence, not the design authority.

## 14. Sequencing and acceptance

The previously signed visibility sequence stays intact. Integration work starts model-first and
does not wait for a native store:

1. **Visibility Stage 0 (already first):** `Publish`/`Reference` model, public/private Git roots,
   file-grain declarations, push guard, and existing-public-corpus stamp.
2. **P0 — cost and scenario model:** carry the agent/CI stress profile, the four outcome roles,
   target scope, candidate closedness, evidence rungs, timestamp non-authority, and the scenario
   corpus as `.dag` facts/witnesses after concept DFS.
   **Accept T1:** every scenario produces its named outcome; compatible/incompatible RED controls
   prevent always-apply and always-refuse; timestamp/arrival permutations cannot change a semantic
   verdict, while removal of required closedness evidence changes `Applied` to `Unknown`.
3. **P1 — pure source integration fold:** one generic fold integrates directly authored
   transformations over a small modeled program, using group-membership provenance and existing
   constraint/witness/identity/change/evidence/Pareto carriers. Scenarios supply facts; they do not
   add fold arms.
   **Accept T1:** idempotence, compatible commutativity/associativity, grounding,
   order-dependence, and no-extra-change witnesses execute; adding a new invariant through the
   shared claim relation changes its scenario outcome without editing the fold.
4. **P2 — lossless authoring capture:** one daglang surface recovers the same transformation as
   direct submission.
   **Accept T2:** edit → ingest → proposal → integrate → emit round-trips; a lossy edit refuses
   without mutation.
5. **P3 — target-scoped atomic admission + receipt reuse:** pending proposals re-evaluate against
   the target's current accepted state; an explicit frontier selects known proposals; affected
   validation receipts survive unrelated advances.
   **Accept T2/T3:** two real concurrent proposals accept without full CI replay; a changed required
   input or closedness/model authority invalidates the receipt and reds before acceptance; a
   compare-and-advance race re-evaluates rather than mutating a stale target.
6. **P4 — Git compatibility realization:** import an ordinary Git branch/PR with fidelity, export
   accepted transitions as ordinary squash/one-parent commits.
   **Accept T3:** unmodified Git clone/build works; native metadata removal lowers evidence but
   never changes the emitted program; ambiguous recovery refuses.
7. **P5 — publication ladder and remote realization:** after visibility Stage 0, land the ladder
   and execute withheld nodes through declared interfaces/effect grants where a consumer prices it.
8. **P6 — native store/serving:** only after the semantic path is a named consumer; use the landed
   object-storage, packing, reliability, and regional-compute carriers. Never one stored object per
   semantic node by default.

The publication ladder may advance independently after visibility Stage 0; it is not a semantic
integration prerequisite. No phase is complete at T0 algebra alone: each names an executing
consumer and a discriminating red.

## 15. Non-goals

- Mind-reading or claiming certainty about unexpressed intent.
- Reducing conflict count by silently choosing a plausible result.
- A weighted “likely intent” score, timestamp/arrival tie-break, or pairwise interaction catalog
  standing where the general reconciliation fold and an honest ambiguity belong.
- Claiming the modeled program is globally complete; every automatic result is scoped to explicit
  closedness, fidelity, equivalence, and proof bounds.
- Making node identity, module identity, a file path, or a universal durable entity ID the
  definition of compatibility.
- Replacing Git hosting, object storage, or forge UI before the integration consumer proves value.
- One semantic node per billable storage object.
- General arbitrary-language semantic merge outside a declared `DecodeFidelity` boundary.
- Unbounded behavioral equivalence; every such claim names a bound.
- Centralized access to customer source as an assumed business advantage.

## Dissolution trigger (DESIGN §6)

This document is the one design seed for the SCM lane, not a second status ledger. Its sections
dissolve as follows:

- outcome/evidence/group roles → the DFS-selected `std/` and plan carriers plus executing witnesses;
- capture/storage content → `v2.compiler.source_authority` and the module-storage design;
- visibility/ladder content → the visibility-grants authority;
- admission/receipt content → the temporal-effect, affected-set, realization, and roadmap rows;
- Git shapes → `extdeps.git` interface operations plus peripheral handlers; and
- economics → the already-landed cited extdeps/econ carriers.

Delete this plan when the native admission consumer reaches P4/T3 and the registered roadmap/carrier
graph contains the remaining P5/P6 work. Until then, update this file rather than opening a parallel
SCM design.
