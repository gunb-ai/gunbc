# GitHub Actions Expression Substrate — Mgr Canvas

**Status:** **RATIFIED** by Director (zesty-bear-812) 2026-05-12T07:39:44Z
per PR #2751 comment (Director session `msg_168005e1` to PM deep-wolf-155).
Option **(c)** is the ratified substrate-shape: `Expression` sum-type at
`dsl/extdeps/github/actions.dag` with single `OpaqueString(String)`
variant + 🟡 YELLOW classification + three-condition dissolution trigger
(§3). All 5 expression sites migrate uniformly. See §7 below for
ratification dispositions and cascade implications.

**Authority:** Director (zesty-bear-812) `msg_2a68a4b5` routing to canvas
authority via PM (deep-wolf-155) `msg_e79d1a50` per
`feedback_substrate_shape_belongs_in_mgr_canvas`. Director ratifies surfaced
shape; does not pre-author. Authoring discipline cites
`feedback_extdeps_header_discriminator_before_field_placement` — the
expression-AST decision is an extdeps-fidelity question (modeling what GH
Actions provides), not a gunbc-policy decision.

**Scope:** Decide how `dsl/extdeps/github/actions.dag` models GitHub Actions
expression syntax (`${{ ... }}`) at the seven identified expression sites
(`Job.if_condition`, `RunStep.if_condition`, `UsesStep.if_condition`,
`RunnerSpec`, `ConcurrencySpec.group`, `Step.with[k]` on `UsesStep`,
and `env[k]` on both `RunStep`/`UsesStep` — see §1 table).
Out-of-scope: gunbc-side evaluator implementation; provider-neutral
expression carriers beyond GH Actions (deferred until a second CI provider
lands).

---

## §1. The gap

Current `.github/workflows/ci.yml` uses GH Actions expression syntax that
the carriers in `dsl/extdeps/github/actions.dag` cannot represent without
falling back to opaque strings:

| Expression site | ci.yml example | Current carrier surface |
|---|---|---|
| Runner selection | `runs-on: ${{ vars.CI_RUNNER \|\| ubuntu-latest }}` | `RunnerSpec = HostedRunner \| SelfHosted` — enum literal only |
| Concurrency group | `concurrency.group: ${{ github.workflow }}-...` | `ConcurrencySpec.group: String` — opaque |
| `if` condition (3 sites) | `if: github.event.pull_request.draft != true` | `Job.if_condition: String?` (`:117`), `RunStep.if_condition: String?` (`:154`), `UsesStep.if_condition: String?` (`:163`) — opaque (existing precedent at 3 sites) |
| Action input | `with: { ref: ${{ github.event.pull_request.head.sha }} }` | `Step.with: Map<String, String>` — opaque per value |
| Env value (2 sites) | `env: { TOKEN: ${{ secrets.X }} }` | `RunStep.env: Map<String, String>` (`:151`), `UsesStep.env: Map<String, String>` (`:162`) — opaque per value at 2 sites |

The **seven** expression sites (3× `if_condition` on `Job`/`RunStep`/`UsesStep`;
`RunnerSpec`; `ConcurrencySpec.group`; `Step.with[k]` on `UsesStep`;
2× `env` on `RunStep`/`UsesStep`) currently form a hidden parallel
authority: at each site, an expression *could* live behind a string, and
gunbc has no structural way to know which strings are expressions versus
literals.

**Extdeps-fidelity finding** (per the
`feedback_extdeps_header_discriminator_before_field_placement` discriminator):
GH Actions expression syntax is **what the platform provides** — the
runtime parses and evaluates `${{ ... }}` at workflow execution time. It is
a platform fact, not gunbc policy. Modeling it belongs in
`dsl/extdeps/github/actions.dag`, not in gunbc namespace.

---

## §2. Three candidate shapes

### Option (a) — Full typed expression AST

```dag
type Expression
  = Literal(String)
  | Var(String)            // github.x, vars.x, secrets.x, inputs.x
  | BinOp(Expression, BinOpKind, Expression)
  | Func(String, List<Expression>)
  | Index(Expression, String)
  | ...
```

All seven expression sites consume `Expression`. Cost-of-change after
introduction: low — new expression-bearing GH Actions field costs one type
swap. **Up-front authoring cost is high**: full GH Actions expression grammar
must be modeled (operators, precedence, contexts: `github.*`, `vars.*`,
`secrets.*`, `inputs.*`, `needs.*`, `steps.*`, `job.*`, `runner.*`, `env.*`,
`hashFiles`, `toJSON`, `format`, `contains`, `startsWith`, `endsWith`,
`join`, `fromJSON`, etc.).

### Option (b) — String-opaque per-site (extend per field as needed)

Keep `Job.if_condition: String?` / `RunStep.if_condition` /
`UsesStep.if_condition` precedent; extend `RunnerSpec` to admit
`Expression(String)` arm; `Step.with[k]` / `RunStep.env[k]` /
`UsesStep.env[k]` / `ConcurrencySpec.group` stay String. Cost-of-change:
**linear** with each new expression-bearing field — every new site needs
its own carrier extension.

### Option (c) — `Expression` sum type with single initial variant + 🟡 YELLOW classification

```dag
// 🟡 YELLOW (scaffold) — see §3 dissolution trigger.
// Models GitHub Actions ${{ ... }} expression syntax as a single
// substrate fact across all expression sites. Currently scaffolded
// at OpaqueString while no consumer needs to evaluate expression
// content; the carrier exists to give every expression site one
// substrate name to migrate to when typed-AST dissolution fires.
type Expression
  = OpaqueString(String)
```

All seven sites migrate to `Expression`:

```dag
type RunnerSpec
  = HostedRunner { label: RunnerLabel }
  | SelfHosted { labels: List<String> }
  | ExpressionRunner { expr: Expression }

type Job {
  ...
  if_condition: Expression?,    // was: String?
  ...
}

type ConcurrencySpec {
  group: Expression,            // was: String
  cancel_in_progress: Bool,
}

type Step
  = RunStep {
      ...
      env: Map<String, Expression>,    // was: Map<String, String>
      if_condition: Expression?,        // was: String?
      ...
    }
  | UsesStep {
      ...
      with: Map<String, Expression>,    // was: Map<String, String>
      env: Map<String, Expression>,     // was: Map<String, String>
      if_condition: Expression?,        // was: String?
      ...
    }
```

Cost-of-change post-introduction: **low at every site** (new
expression-bearing field carries `Expression`). Cost of dissolution from
scaffold to typed AST: **single carrier edit + 7 sites-already-migrated**
(versus option (b) where each site has its own carrier surface to
re-flatten).

---

## §3. Comparison

| Axis | (a) Full typed AST | (b) String-opaque per-site | **(c) Single-variant scaffold** |
|---|---|---|---|
| Up-front authoring cost | high (full grammar) | low | **low** |
| Cost-of-change: new expression site | low (Expression already exists) | high (per-site decision) | **low** (Expression already exists) |
| Single-authority for "what is an expression" | ✓ Expression carrier | ✗ scattered across sites | **✓ Expression carrier** |
| Extdeps fidelity | ✓ models platform | ✓ models platform | **✓ models platform** |
| Practice 4 (coproduct dissolution) discipline | terminal (🟢 or 🔴) | N/A (no sum) | **🟡 YELLOW with named trigger** |
| Slice 4 (YamlStatic emit) cost | high (grammar engine) | low (string pass-through) | **low** (variant unwraps to String) |
| Slice 5+ (BinaryShim eval) cost | covered (typed AST) | uncovered (no parser) | covered when dissolution fires |
| Premature-modeling risk | high (axes chosen before consumer pressure) | low | **low** (scaffold defers axis choice) |

**Recommendation: option (c).**

**Reasoning**:

1. **Single-authority preserved without premature commitment.** All seven
   expression sites name the same substrate concept (`Expression`); no
   future consumer has to discover that `Job.if_condition`,
   `RunStep.if_condition`, `UsesStep.if_condition`, and `Step.with[k]`
   were all "expressions" at different sites — the type says so. This
   directly addresses the P2/P5 hidden-parallel-authority pattern: if
   `Job.if_condition` adopts `Expression` while `RunStep.if_condition`
   and `UsesStep.if_condition` remain `String?`, the "what is an
   expression" fact lives in two places (typed at one site, opaque
   string at two others), violating P2 and leaving P5 dissolution
   blocked at the un-migrated sites.
2. **Cost-of-change is correct for the current consumer set.** YamlStatic
   emit (Slice 4) is the only ratified consumer per `(c-refined)` in PR #2749;
   it needs to render expressions verbatim. `Expression::OpaqueString`
   unwraps to the string the emitter needs. No grammar engine required.
3. **Practice 4 discipline is satisfied.** Per `docs/modeling-discipline.md`,
   every coproduct with N ≥ 2 variants must classify. A single-variant
   sum type is technically not yet a coproduct, but introducing the carrier
   as a sum (rather than a type alias) makes the dissolution path explicit
   — adding the second variant is a single declaration edit, not a
   carrier-shape change.
4. **Pre-empts the type-alias trap.** If `Expression` were introduced as
   `type Expression = String`, downstream consumers would treat it
   interchangeably with `String`, and the eventual second-variant
   introduction would require type-coercion work at every site. Sum-form
   from the start forces consumers through a pattern match (even if only
   one arm exists), making future additions structural.

**🟡 YELLOW classification** (per modeling-discipline.md Practice 4):

- **GREEN rejected**: a richer source exists. GH Actions expressions have
  a documented grammar (operators, functions, contexts); typed AST is the
  terminal shape. Declaring `Expression` GREEN would falsely close the door.
- **RED rejected**: dissolving to a typed AST is not cheap-now. Full GH
  Actions expression grammar covers contexts (`github.*`, `vars.*`,
  `secrets.*`, etc.), operators, functions (`hashFiles`, `toJSON`, etc.),
  and context-dependent variable resolution. No current consumer needs to
  evaluate — only render. Dissolving now is premature per
  "no parallel authority that 'picks up slack' when the substrate is
  incomplete; if the substrate under-determines, the substrate is
  incomplete and the right response is to extend the substrate, not bolt
  on a decider" inverted: don't extend further than the consumer pressure
  justifies.

**Named dissolution trigger**: dissolve when **either**:

(a) A non-YamlStatic emission target requires evaluating expression
content rather than passing it through verbatim. The candidate trigger
event is the first BinaryShim or InlineGunbc emitter (per PR #2749 (c-refined))
that needs to resolve, e.g., `runs-on: ${{ vars.CI_RUNNER || ubuntu-latest }}`
to a concrete runner at gunbc runtime (cannot pass through verbatim because
the binary runtime, not GH Actions runtime, must select).

(b) A gunbc-side consumer (cost lens, complexity lens, affected-set lens)
needs to read expression content structurally — e.g., affected-set lens
walking `if: contains(steps.changes.outputs.files, 'src/')` to determine
which CI gates depend on which paths.

(c) A second CI provider (GitLab, Buildkite) lands and a provider-neutral
expression substrate is needed — at which point the per-provider scaffold
shape forces the question of common AST.

Until one of (a)/(b)/(c) fires, `Expression` remains a single-variant
sum type and the substrate cost is one carrier name + seven
site-already-migrated.

---

## §4. Dissolution path preview

When dissolution fires, the second variant follows the GH Actions
expression grammar at https://docs.github.com/en/actions/learn-github-actions/expressions.

**Important extdeps-fidelity correction (per operator BLOCKING at PR #2751:214,
2026-05-12)**: GH Actions expression-bearing scalars are **template strings**,
not pure expression ASTs. A scalar field like
`concurrency.group: ${{ github.workflow }}-${{ github.event.pull_request.number || github.run_id }}`
is a *sequence* of alternating literal-text segments and `${{ ... }}`
expression segments, not a single expression. A dissolution target
modeling each scalar as one pure AST node would mis-model what the
platform actually parses (text-with-interpolated-expressions), violating
INVARIANTS P1 (modeling faithfulness). The corrected dissolution target
distinguishes the template-string layer (sequence of segments) from the
expression-AST layer (the content of one `${{ ... }}` segment):

```dag
// Top-level: an expression-bearing field holds a template string,
// which is a sequence of text segments and interpolated expression
// segments. A pure-literal scalar is a single TextSegment; a pure-
// expression scalar is a single ExpressionSegment.
type Expression
  = OpaqueString(String)              // 🟡 SCAFFOLD until dissolution
  | Template(List<TemplateSegment>)   // dissolved form

type TemplateSegment
  = TextSegment(String)
  | ExpressionSegment(ExpressionAst)

// The inner expression grammar (per docs.github.com expression docs):
type ExpressionAst
  = Literal(LiteralValue)
  | Var(VarPath)                      // github.event.pull_request.draft
  | BinOp(ExpressionAst, BinOpKind, ExpressionAst)
  | Func(FunctionName, List<ExpressionAst>)
  | Index(ExpressionAst, String)
```

This shape is extdeps-faithful: the carrier mirrors the platform's
actual parse structure (template string → segment list → per-segment
either literal text or an expression AST). Pure literal scalars
(e.g., `runs-on: ubuntu-latest`) collapse to `Template([TextSegment("ubuntu-latest")])`;
pure expression scalars collapse to `Template([ExpressionSegment(...)])`;
mixed scalars carry the full segment list. This corrects the original
sketch above, which collapsed template and expression layers into one
type and would have under-modeled the platform.

(Note: the original sketch is preserved here as authoring-evolution
record for the BLOCKING-finding audit trail; the corrected shape
immediately above supersedes it.)

`OpaqueString` may persist as a SCAFFOLD arm during the migration window
(per `INVARIANTS.md` P5: Progress Is Dissolution; scaffold arms must
declare a sunset milestone). Per the migration discipline at
`feedback_pattern_a_scaffold_sentinel_per_instance_ratification`,
scaffold-arm presence requires per-instance Director ratification at
dissolution time; this canvas does not pre-author that decision.

---

## §5. Slice 4 implementation implications

Under option (c):

- Slice 4 (YamlStatic emitter) emit logic: pattern-match on `Expression`,
  unwrap `OpaqueString(s)` to `s`, emit `s` verbatim. Single-arm
  pattern match; trivial.
- Slice 4 acceptance gate: emitter output for the seven expression-site
  fields must match current ci.yml byte-for-byte (per the
  feedback_extdeps_header_discriminator_before_field_placement carry-
  forward acceptance gates from PR #2744).
- WI-2 declaration-only (cool-carp-720) NOT affected — WI-2 declares the
  projection function signature; expression-site carrier types are
  consumed in Slice 4 per-arm body, not declaration.

If Director ratifies a different option (a/b), Slice 4 brief authoring
adjusts accordingly. Option (a) would require expression-grammar parser
work before Slice 4 dispatches; option (b) would require per-site carrier
extensions.

---

## §5.5 Inventory completeness — audit against actions.dag schema + GH Actions docs

**Trigger**: codex BLOCKING review on PR #2751 sha 4f41aebb (2026-05-12
~09:12Z) + operator BLOCKING inline at :315 (same timestamp):

> The site inventory is keyed to selected ci.yml examples instead of the
> official GitHub Actions context-availability/workflow-syntax tables
> plus every carrier in actions.dag → audit all expression-capable
> workflow keys and either migrate them uniformly or explicitly stage a
> bounded, triggered partial migration.

**Finding accepted as substantive scope correction.** Per
[GH Actions context availability](https://docs.github.com/en/actions/reference/workflows-and-actions/contexts)
+ [workflow-syntax](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax),
expression-capable fields in actions.dag carriers (audited 2026-05-12):

| Carrier | Field | Type at HEAD | Expression-capable per GH Actions docs |
|---|---|---|---|
| `Workflow` | `name: String` (`:22`) | String | ✓ |
| `Workflow` | `env: Map<String, String>` | Map values | ✓ values |
| `Job` | `name: String?` (`:112`) | String? | ✓ (per jobs.<job_id>.name context) |
| `Job` | `if_condition: String?` | String? | ✓ (already in 7-site) |
| `Job` | `env: Map<String, String>` | Map values | ✓ values |
| `Job` | `runner: RunnerSpec` | enum | ✓ via new `ExpressionRunner` (in 7-site) |
| `Job` | `timeout_minutes: Int?` | Int? | ✓ (string-coerced to int) |
| `Job` | `continue_on_error: Bool` | Bool | ✓ (string-coerced to bool) |
| `Job` | `concurrency.group: String` | String | ✓ (already in 7-site via ConcurrencySpec) |
| `Job` | `concurrency.cancel_in_progress: Bool` | Bool | ✓ (string-coerced to bool) |
| `RunStep` | `name: String?` | String? | ✓ |
| `RunStep` | `run: String` | String | ✓ |
| `RunStep` | `env: Map<String, String>` | Map values | ✓ (already in 7-site) |
| `RunStep` | `working_directory: String?` | String? | ✓ |
| `RunStep` | `if_condition: String?` | String? | ✓ (already in 7-site) |
| `RunStep` | `continue_on_error: Bool` | Bool | ✓ |
| `RunStep` | `timeout_minutes: Int?` | Int? | ✓ |
| `UsesStep` | `name: String?` | String? | ✓ |
| ~~`UsesStep`~~ | ~~`uses: ActionRef`~~ | ~~ActionRef~~ | ✗ **literal-only** per GH Actions workflow-syntax docs (no `jobs.<job_id>.steps.uses` in context-availability table); removed from inventory per operator BLOCKING at :381 (2026-05-12T10:12:15Z) — modeling-faithfulness P1 violation to invent platform capability |
| `UsesStep` | `with: Map<String, String>` | Map values | ✓ (already in 7-site) |
| `UsesStep` | `env: Map<String, String>` | Map values | ✓ (already in 7-site) |
| `UsesStep` | `if_condition: String?` | String? | ✓ (already in 7-site) |
| `UsesStep` | `continue_on_error: Bool` | Bool | ✓ |
| `UsesStep` | `timeout_minutes: Int?` | Int? | ✓ |
| `MatrixStrategy` | `dimensions: Map<String, List<String>>` (`:223`) | Map values | ✓ (matrix values per workflow-syntax) |
| `MatrixStrategy` | `include: List<Map<String, String>>` (`:224`) | List of Map | ✓ (matrix includes per workflow-syntax) |
| `MatrixStrategy` | `exclude: List<Map<String, String>>` (`:225`) | List of Map | ✓ (matrix excludes per workflow-syntax) |
| `MatrixStrategy` | `fail_fast: Bool` (`:226`) | Bool | ✓ (typed; string-coerced expression) |
| `MatrixStrategy` | `max_parallel: Int?` (`:227`) | Int? | ✓ (typed; string-coerced expression) |

Total expression-capable surface: **28 fields** across `Workflow`, `Job`,
`RunStep`, `UsesStep`, `ConcurrencySpec`, `RunnerSpec`, `MatrixStrategy`
(post-corrections: `UsesStep.uses` removed per :381 — literal-only;
`MatrixStrategy` carriers added 2026-05-12T11:35Z per codex BLOCKING
review 10128 / sha 248f2cf3 — full-carrier sweep cross-reference against
GH Actions workflow-syntax). The 7-site enumeration was keyed to ci.yml
usage, not actions.dag schema — under-modeling the platform surface by
21 sites.

**Audit methodology note** (per codex BLOCKING review 10128): the
audit above is now the cross-product of (every current carrier in
`dsl/extdeps/github/actions.dag`) × (the GH Actions context-availability
+ workflow-syntax tables). Carriers whose every field is literal-only
(e.g., `LogAnnotation`, `Artifact`, `WorkflowPermissions` with enum
levels, `ActionRef`, `CheckConclusion`) are omitted by construction.
The implementing PR (per §7.5 ask #4 / Slice 4 brief) re-runs this
audit against `actions.dag` HEAD at implementation time and migrates
any additional expression-capable fields surfaced — the canvas-level
audit is the authority-at-ratification-time; the implementing-PR audit
is the authority-at-migration-time, per `feedback_grep_substrate_before_naming_ratification`
discipline expanded to substrate-shape audits.

### §5.5.1 Migration rule (revised — string-typed sites only; typed-field sites HOLD per §5.5.2)

The 23 expression-capable sites split into three classes by HEAD-type:

- **String-typed sites (18)**: fields already typed `String` / `String?` /
  `Map<String, String>` at HEAD — `Workflow.name`, `Workflow.env`,
  `Job.name`, `Job.if_condition`, `Job.env`, `Job.concurrency.group`,
  `RunStep.name`, `RunStep.run`, `RunStep.env`,
  `RunStep.working_directory`, `RunStep.if_condition`,
  `UsesStep.name`, `UsesStep.with`, `UsesStep.env`,
  `UsesStep.if_condition`. Migration shape under (c) is uniform:
  every site → `Expression` / `Expression?` / `Map<String, Expression>`.
- **Typed-field sites (9)**: `Job.timeout_minutes: Int?`,
  `Job.continue_on_error: Bool`,
  `Job.concurrency.cancel_in_progress: Bool`,
  `RunStep.continue_on_error: Bool`, `RunStep.timeout_minutes: Int?`,
  `UsesStep.continue_on_error: Bool`, `UsesStep.timeout_minutes: Int?`.
  GH Actions string-coerces expressions at these sites at runtime.
  The shape that captures this (wrap vs `TypedOrExpression<T>` sum vs
  defer) is an **open Director-tier question** per §5.5.2 / §6 Q#4.
  **HOLD migration** until §6 Q#4 ratifies.
- **Enum-extension sites (1)**: `Job.runner: RunnerSpec`. Migration adds
  a new variant (`ExpressionRunner { expr: Expression }`) to the existing
  sum rather than swapping the type. Per §2 (c) sketch for `RunnerSpec`.
  This site is in scope for the §7.5 ask #4 prereq PR alongside the
  15 string-typed sites — same uniform-string-expression class at the
  substrate level (the variant carries `Expression`).

  **Note**: `UsesStep.uses: ActionRef` was previously listed here as a
  second enum-extension site (with a proposed `ExpressionActionRef`
  variant) but is **removed** per operator BLOCKING at PR #2751 :381
  (2026-05-12T10:12:15Z): GH Actions workflow-syntax treats `uses:` as
  a literal action location (no entry in the context-availability
  table at `jobs.<job_id>.steps.uses`); modeling it as
  expression-capable would invent platform capability and violate
  INVARIANTS P1 modeling faithfulness.

Total in-scope for the substrate-prereq PR under §7.5 ask #4 / Slice 4
brief: **18 string-typed + 1 enum-extension = 19 sites**. The 7
typed-field sites sequence as a follow-on substrate-prereq PR after
§6 Q#4 Director ratification resolves the wrap/sum/defer choice.

**Why uniform-not-partial within the string-typed class**: leaving any
string-typed expression-capable field as opaque `String` while migrating
others creates the hidden-parallel-authority pattern operator BLOCKING
flags — P2/P5 violation by structural split. Either gunbc structurally
recognizes "this field can carry a GH Actions expression" via
`Expression`, or it doesn't.

**Why typed-field sites can HOLD without violating the same discipline**:
typed-field sites carry a different question (how to model
string-coerced-to-typed expressions), not the same question. The 5 sites
in the typed-field class form a separate uniform migration class once
§6 Q#4 ratifies; deferring the class as a whole pending the typed-
question resolution preserves single-authority for "what is a
string-typed expression-bearing field" while leaving the typed-coerce
class as a distinct decision. Mixing the two classes in §5.5.1 would
create the contradiction codex REQUEST_CHANGES review 10083 flagged
("ALL 22 migrate" + "typed shape unresolved" cannot both hold).

**Implementing-PR scope**: §7.5 ask #4 / Slice 4 brief migrates the
18 string-typed + 1 enum-extension sites (19 total) uniformly. The 7
typed-field sites sequence as a follow-on substrate-prereq PR after
§6 Q#4 Director ratification resolves the wrap/sum/defer choice.

### §5.5.2 Typed-field expression semantics (new §6 question)

`timeout_minutes: Int?`, `continue_on_error: Bool`, and
`concurrency.cancel_in_progress: Bool` are **typed** fields where GH
Actions coerces expression output to the typed value. Two candidate
shapes for typed fields:

(i) **Wrap**: `timeout_minutes: Expression?` — expression
content evaluates to a numeric string at runtime; gunbc emitter renders
verbatim; gunbc evaluator (Slice 5+) parses to Int. Loses the typed
literal case (must wrap `5` as `Expression(OpaqueString("5"))`).

(ii) **Sum**: `timeout_minutes: TypedOrExpression<Int>?` where
`TypedOrExpression<T> = Literal(T) | Expression(Expression)`. Preserves
typed literals; cost: one extra carrier per typed expression-capable
field.

(iii) **Defer**: keep `Int?`/`Bool` literal-only for now; add `Expression`
variant in implementing PR if ci.yml actually uses expressions at those
sites. **Risk**: future ci.yml authoring that adds an expression at
`timeout-minutes` requires retroactive substrate work.

This canvas does not pre-author the choice (per §6 framing); routing to
§6 question #4 below for Director.

### §5.5.3 7-site framing retained as §1/§2 reference

The §1 table and §2 (a/b/c) code sketches retain their 7-site framing as
the ci.yml-keyed reference set, with §5.5 as the actions.dag-keyed audit
extending to 23 sites. The substrate-shape ratification covers the
expanded scope per §5.5.1 migration rule (all expression-capable fields);
the §1/§2 7-site enumeration is the minimum subset proven by current
ci.yml usage, not the migration ceiling.

---

## §6. Open questions surfaced (not pre-authored)

1. **Declaration form — RESOLVED: sum, not record.** Earlier draft
   framed this as "either preserves the structural pattern-match
   property; Director call if both admissible." Operator BLOCKING on
   PR #2751 at :258 (2026-05-12) corrected this: a record form
   (`type Expression { value: String }`) does **not** preserve the
   single-arm pattern-match obligation, and adding a second variant
   later (a `Template` carrier per §4 dissolution) becomes a carrier-
   shape change (record → sum), not a single-declaration edit. That
   breaks both the Practice 4 single-edit dissolution path claimed
   in §3 AND the consumer-side pattern-match property (record consumers
   project to `.value` as a string, not pattern-match on a tag — so
   the addition of `Template` would force every consumer to be
   rewritten, not just the carrier).
   
   **Resolution**: `Expression` lands as a one-arm sum (`type Expression
   = OpaqueString(String)`), not a record. Forces consumers through a
   single-arm pattern match from day one; dissolution to add `Template`
   is a single-line edit at the declaration site and a single new
   pattern-match arm in each consumer. This was the substantive
   ratification under (c) and is no longer a Director-tier open
   question; it was implied by §3 reasoning point #4 ("Pre-empts the
   type-alias trap") which applies equally to record-form aliases.

2. **Migration sequencing across the 23 expression-capable fields**
   (revised from "seven sites" per §5.5 audit). Do all sites migrate
   to `Expression` in one PR (substrate-prereq PR before Slice 4) or
   incrementally? Recommendation: one PR (cohesive substrate change;
   per `feedback_single_bundle_ratification_uniform_substrate_cause` —
   the substrate-cause is uniform across sites).
3. **`Job.if_condition: String?` precedent retirement.** The existing
   opaque-string precedent on `Job.if_condition` becomes
   `Job.if_condition: Expression?` under option (c). This is a real
   substrate edit, not just an addition; any current consumer reading
   `if_condition` must accept the Expression type. Audit at migration PR
   authoring time.
4. **Typed-field expression semantics (per §5.5.2).** How do
   `timeout_minutes: Int?`, `continue_on_error: Bool`, and
   `concurrency.cancel_in_progress: Bool` migrate? Three candidate
   shapes: (i) wrap to `Expression?` losing typed-literal case,
   (ii) sum `TypedOrExpression<T> = Literal(T) | Expression(Expression)`,
   (iii) defer typed-field migration until ci.yml uses expressions at
   those sites. Director-tier choice; substrate-shape implication for
   how gunbc models GH Actions' string-coerced expression semantics.

---

**Authored by**: warm-wolf-698 (R3 Substrate Mgr) per Director
(zesty-bear-812) routing via PM (deep-wolf-155) `msg_e79d1a50` 2026-05-12
~07:30Z citing Director routing message `msg_2a68a4b5`.

**Canvas readiness for Director ratification**: **RATIFIED** by Director
2026-05-12T07:39:44Z. See §7 below for ratification dispositions.

---

## §7. Ratification dispositions (2026-05-12T07:39:44Z)

Director (zesty-bear-812) ratified option **(c)** per PR #2751 comment
quoting session `msg_168005e1` to PM `deep-wolf-155`. Source-verification
items per ratification:

- `dsl/extdeps/github/actions.dag` header lines 1-6 confirm "platform
  constraints — what GH Actions provides and requires" boundary. GH
  Actions expressions are platform-provided value-shapes, not gunbc
  emission policy → extdeps placement respects the discriminator
  (`feedback_extdeps_header_discriminator_before_field_placement`).
- No conflict with (c-refined) PR #2749 ratification: `WorkflowRuntime`
  axis (renamed from `EmissionTarget` per PR #2749 §7.3.3 P2 name-
  collision fix; `src/v3/SELF_HOSTING.md:609` owns the Shape-A
  `EmissionTarget` name) = gunbc CI realization-mode policy (gunbc
  namespace); `Expression` axis = GH Actions value-language (extdeps
  namespace). Orthogonal axes.
- Authority audit clean: only `Job.if_condition: String?` precedent;
  canvas extends uniformly; no sibling carrier; no naming collision.
- YELLOW classification justified per Practice 4 — §3 + §4 name the
  trigger explicitly + preview typed-AST dissolution shape.

**Ratifications**:

1. **`Expression` sum-type placement at `dsl/extdeps/github/actions.dag`**:
   RATIFIED.
2. **Single `OpaqueString(String)` variant + 🟡 YELLOW classification**:
   RATIFIED. Pre-emptive over-modeling is the wrong default per
   `feedback_construction_over_ratchets` +
   `feedback_checkpoint_dissolution_default`.
3. **Three-condition dissolution trigger**: RATIFIED. Trigger framing
   makes the dissolution path explicit.
4. **Uniform migration of all expression-capable fields in
   `actions.dag`** (post-correction; see scope-evolution note below):
   RATIFIED. Single-authority for expression substrate. The
   implementing-PR scope per §7.5 ask #4 / Slice 4 brief covers the
   §5.5 audit set: **19 sites in scope** (18 string-typed + 1
   enum-extension `RunnerSpec.ExpressionRunner`). **9 typed-field
   sites HOLD** pending §6 Q#4 Director ratification of the
   wrap/sum/defer choice. Total expression-capable surface in
   `actions.dag`: 23 sites; the (c) substrate-shape ratification covers
   all 23 per the "single-authority for expression substrate" principle,
   but the implementing-PR migrates 16 immediately + 7 deferred class
   sequences as a follow-on after §6 Q#4 resolves.

> **Site-count correction evolution (cumulative)**: Director ratification
> at 2026-05-12T07:39:44Z referenced a "5-site" framing. Three
> subsequent BLOCKING reviews on PR #2751 expanded the audit:
> 
> 1. 2026-05-12T07:45:24Z (:34): `actions.dag` has 3× `if_condition` +
>    2× `env` (not 1+1). Corrected 5 → 7 sites.
> 2. 2026-05-12T09:12Z (:315) + codex (4f41aebb): inventory was
>    ci.yml-keyed not actions.dag-schema-keyed. Audited actions.dag
>    against GH Actions context-availability + workflow-syntax docs;
>    corrected 7 → 22 sites via §5.5.
> 3. 2026-05-12T10:12:15Z (:365): missed `Workflow.name` + `Job.name`.
>    Corrected 22 → 24 sites.
> 4. 2026-05-12T10:12:15Z (:381): `UsesStep.uses` is literal-only per
>    GH workflow-syntax (no entry in context-availability table);
>    removed. Corrected 24 → 23 sites.
> 
> Net audit set: 23 expression-capable sites in `actions.dag` (15
> string-typed + 1 enum-extension + 9 typed-field-HOLD). This is a
> cumulative site-count correction sequence, not substrate-shape
> changes — the (c) ratification ("single-authority for expression
> substrate") covers all 23 uniformly. Leaving any expression-capable
> site (within the migrating class) un-migrated would violate P2
> (hidden parallel authority: one site typed, others opaque) and block
> P5 dissolution at the un-migrated sites. The typed-field class HOLD
> is bounded with a named trigger (§6 Q#4) per Practice 4 YELLOW
> discipline, not arbitrary deferral.
> 
> §1 + §2 + §5.5 + §6 + §7 ratification ask #4 all reflect the
> corrected counts; §1/§2 retain the original 7-site ci.yml-keyed
> framing as the minimum-proven-subset reference per §5.5.3.

**Cascade implications** (per Director directive):

- cool-carp-720 (WI-2 PR #2745): post-land, `project_github_actions`
  constructs `Expression(OpaqueString("..."))` wrapping gunbc-side values
- stern-stag-854 (Slice 4-5 briefs): YamlStatic emission emits
  `OpaqueString` variant verbatim; BinaryShim/PythonShim
  evaluate at runtime (opaque-passthrough until dissolution fires);
  Slice 4 emit logic stays trivial
- PR #2746 (still-heron-763 emitter-dispatch canvas): can reference
  "ratified `Expression` substrate per PR #2751" once both land
- Not blocking PR #2744 / #2750 / #2746 (independent axes)

PR #2751 standalone-mergeable per normal dashboard review cycle; no
operator-tier bypass.
