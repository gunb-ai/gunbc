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
expression syntax (`${{ ... }}`) at the five identified expression sites.
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
| `if` condition | `if: github.event.pull_request.draft != true` | `Job.if_condition: String?` — opaque (existing precedent) |
| Action input | `with: { ref: ${{ github.event.pull_request.head.sha }} }` | `Step.with: Map<String, String>` — opaque per value |
| Env value | `env: { TOKEN: ${{ secrets.X }} }` | `Step.env: Map<String, String>` — opaque per value |

The five sites currently form a hidden parallel authority: at each site, an
expression *could* live behind a string, and gunbc has no structural way to
know which strings are expressions versus literals.

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

All five expression sites consume `Expression`. Cost-of-change after
introduction: low — new expression-bearing GH Actions field costs one type
swap. **Up-front authoring cost is high**: full GH Actions expression grammar
must be modeled (operators, precedence, contexts: `github.*`, `vars.*`,
`secrets.*`, `inputs.*`, `needs.*`, `steps.*`, `job.*`, `runner.*`, `env.*`,
`hashFiles`, `toJSON`, `format`, `contains`, `startsWith`, `endsWith`,
`join`, `fromJSON`, etc.).

### Option (b) — String-opaque per-site (extend per field as needed)

Keep `Job.if_condition: String?` precedent; extend `RunnerSpec` to admit
`Expression(String)` arm; `Step.with[k]` / `env[k]` / `ConcurrencySpec.group`
stay String. Cost-of-change: **linear** with each new expression-bearing
field — every new site needs its own carrier extension.

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

All five sites migrate to `Expression`:

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

type Step {
  ...
  with: Map<String, Expression>,  // was: Map<String, String>
  env: Map<String, Expression>,   // was: Map<String, String>
  ...
}
```

Cost-of-change post-introduction: **low at every site** (new
expression-bearing field carries `Expression`). Cost of dissolution from
scaffold to typed AST: **single carrier edit + 5 site-already-migrated**
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

1. **Single-authority preserved without premature commitment.** All five
   expression sites name the same substrate concept (`Expression`); no
   future consumer has to discover that `Job.if_condition` and
   `Step.with[k]` were both "expressions" at different sites — the type
   says so.
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
sum type and the substrate cost is one carrier name + five
site-already-migrated.

---

## §4. Dissolution path preview

When dissolution fires, the second variant follows the GH Actions
expression grammar at https://docs.github.com/en/actions/learn-github-actions/expressions
The likely target shape (not authoritative until dissolution):

```dag
type Expression
  = OpaqueString(String)        // dissolves to specific arms
  | Literal(LiteralValue)
  | Var(VarPath)                // github.event.pull_request.draft
  | BinOp(Expression, BinOpKind, Expression)
  | Func(FunctionName, List<Expression>)
  | Index(Expression, String)
```

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
- Slice 4 acceptance gate: emitter output for the five expression-site
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

## §6. Open questions surfaced (not pre-authored)

1. **Sum-vs-alias declaration form.** Should `Expression` land as
   `type Expression = OpaqueString(String)` (sum with one arm) or
   `type Expression { value: String }` (record)? Either preserves the
   structural pattern-match property; sum form is closer to the
   dissolution target shape. Director call if both are admissible.
2. **Migration sequencing across the five sites.** Do all five sites
   migrate to `Expression` in one PR (substrate-prereq PR before Slice 4)
   or incrementally (per-site as Slice 4 lands each emitter arm)?
   Recommendation: one PR (cohesive substrate change; per
   `feedback_single_bundle_ratification_uniform_substrate_cause` — the
   substrate-cause is uniform across sites).
3. **`Job.if_condition: String?` precedent retirement.** The existing
   opaque-string precedent on `Job.if_condition` becomes
   `Job.if_condition: Expression?` under option (c). This is a real
   substrate edit, not just an addition; any current consumer reading
   `if_condition` must accept the Expression type. Audit at migration PR
   authoring time.

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
- No conflict with (c-refined) PR #2749 ratification: `EmissionTarget`
  axis = gunbc emission policy (gunbc namespace); `Expression` axis = GH
  Actions value-language (extdeps namespace). Orthogonal axes.
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
4. **5-site uniform migration** (`RunnerSpec.ExpressionRunner` variant /
   `Job.if_condition` / `ConcurrencySpec.group` / `Step.with[k]` /
   `Step.env[k]`): RATIFIED. Single-authority for expression substrate.

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
