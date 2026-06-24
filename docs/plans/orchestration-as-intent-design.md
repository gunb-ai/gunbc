# Orchestration-as-intent (gap B) — design proposal

> **Status: DESIGN-ONLY, sign-ready draft.** No implementation. Realizes item **§4(B)** of
> [emission-ingestion-inverse.md](emission-ingestion-inverse.md) — *"a `Pipeline`/`Step`/`Run`/`Check`
> vocabulary so transports author intent and `emit(intent, Bash)` renders shell."*
> Lane: **ShellProgram → DAG de-fork** (neat-fox-547). DESIGN refs: §2 (decompose, don't re-coin),
> §3 (intent / transport / policy are three facts), §4 (`emit = serialize_target ∘ translate`, one
> grammar read both directions, N+M not N×M), §5 (fail-closed; construction over validation; "never" =
> decidability), §6 (idea→idea; price in displaced cost).
>
> Every place I am unsure is tagged **⚠ FLAG**. The brief asks me to flag rather than commit on
> load-bearing modeling; I have done so liberally — §9 collects them.

---

## 0. The one-sentence claim

> Control flow (if/else, for, while, retry) becomes a small, closed, **medium-agnostic intent
> coproduct in `std`**, and shell control flow falls out of `emit(intent, Bash)` over **bash grammar
> rows at the realization edge** — so the doomed `extdeps/languages/bash/program.dag` AST is **never
> extended**, it is **bypassed and then deleted**.

The whole design is the consequence of one constraint (the brief's central one): you cannot solve
control flow by adding `Else`/`For`/`While`/`Case` arms to `program.dag`'s `ShellStmt`, because
`program.dag` is the forward-emitter scaffold being *deleted* at lane-end (§5 of
emission-ingestion-inverse — the containment guard's roster empties → `program.dag` deletable). Every
arm you add to it is debt you have promised to throw away. So the new expressive power must live in
exactly two legitimate homes:

1. **the intent coproduct** (`std`, medium-agnostic — depends on no language AST), and
2. **bash grammar productions** (`v2.extdeps.languages.bash`, the realization edge the §5 guard
   *permits* to author target syntax).

Nothing new lands in `program.dag`. That is the design.

---

## 1. The gap, shown by the two hardest consumers

Both consumers the brief named author orchestration as **strings**, two different ways. Together they
define the real requirement surface.

### 1.1 `ci_spec.ci_cargo_eagain_retry_core` — nested-concat bash string

`dsl/gunbc/ci_spec.dag:77` builds, by a 14-deep `concat` tree, this shape (elided):

```bash
BUILD_LOG=$(mktemp)
if ! ( <command> ) 2>&1 | tee "$BUILD_LOG"; then
  if grep -qiE '<infra_retry_grep_alternation>' "$BUILD_LOG"; then
    echo "::warning::cargo/sccache EAGAIN ...; cold retry CARGO_BUILD_JOBS=1 (keep sccache)"
    RETRY_LOG=$(mktemp)
    if ! ( CARGO_BUILD_JOBS=1 <command> ) 2>&1 | tee "$RETRY_LOG"; then
      if grep -qiE '<infra_retry_grep_alternation>' "$RETRY_LOG"; then
        echo "::warning::...; retry without RUSTC_WRAPPER"
        env -u RUSTC_WRAPPER CARGO_BUILD_JOBS=1 <command> || exit 1
      else
        exit 1
      fi
    fi
    rm -f "$RETRY_LOG"
  else
    exit 1
  fi
fi
rm -f "$BUILD_LOG"
```

The intent is **one idea**: *"run `<command>`; if it fails for an infra reason, retry it twice with
escalating mitigations (drop parallelism, then drop the wrapper); a non-infra failure is fatal."* That
is a **bounded retry policy with a failure classifier** — not 40 lines of nested `if`. The features
exercised: nested `if/else`, command-substitution (`$(mktemp)`), pipe-to-`tee`, `grep` *as a
condition*, env-prefix (`CARGO_BUILD_JOBS=1`) and env-unset (`env -u RUSTC_WRAPPER`), `2>&1` redirect,
`exit 1` / `|| exit 1`, and a GHA `::warning::` annotation (which is **gap A** — `EmitDirective`).

### 1.2 `fleet_converge_emit` — bash-as-string-literal `Doc` projection

`dsl/gunbc/fleet_converge_emit.dag` is worse: bash authored as a `List<String>` of raw lines
(`converge_script_header`, `host_converge_doc`, …), never even reaching `program.dag`'s AST. It
defines **shell functions** (`decide_verdict`), uses **for-loops** (`for unit in $(systemctl
list-units … | awk '{print $1}'); do … done`), **while-read loops** (`… | while IFS= read -r unit;
do … done`), **elif chains**, **arithmetic** (`drifted=$((drifted + 1))`), **positional params**
(`$1`…`$7`), **ignore-failure** (`|| true`), and **discard redirects** (`>/dev/null 2>&1`). The
`# comments` live *inside the strings* — a `MediumStructureLeak` (emission-ingestion-inverse §5.1),
invisible to the `program.dag` import guard because there is no AST import to catch.

**What the two consumers prove:** consumer (1) is the **control-flow skeleton** (if/else + retry);
consumer (2) adds **bindings, named procedures, arithmetic, and iteration over command output**. A
vocabulary that renders (1) fully and (2) mostly is the right MVP; the residue in (2) (functions +
arithmetic) is a named **second tier** (§3.2), not a reason to widen tier-1.

---

## 2. Grounding the vocabulary (§2/§3 — DFS the concept DAG *before* coining)

The brief offers `Pipeline`/`Step`/`Run`/`Check` "or whatever the modeling justifies." DESIGN §2/§3
**require** the DFS: a fresh enum for a concept that already exists is a failed decomposition
(nicknaming, the §3 violation). Here is the DFS. **The headline finding: two of the four names ground
entirely into existing carriers, and control flow grounds into the substrate's own behaviors — so the
genuinely *new* vocabulary is small.**

| proposed name | DFS verdict | grounds into (existing carrier) |
| --- | --- | --- |
| **`Run`** | a command **effect** + its exit + its argv | `std.effects.EffectShape` (read/upsert/…) + `std.process.ProcessExit` + a `CommandRef` (argv = the §3 *transport* modeling of a CLI, e.g. `git diff` argv models git). Env-prefix / redirect / pipe are **Run modifiers**, not new statements. |
| **`Check`** | assert a predicate; on failure emit a diagnostic + non-zero exit | **`std.witness.Witness` (`Holds \| Violates`) + gap-A `EmitDirective`/`Diagnostic` on the `Violates` arm + `ProcessExit`.** *Check is not new — it is `Witness` lifted to orchestration, reusing gap A's diagnostic realization.* The `grep -qiE … "$LOG"` is a `Check` over a `LogMatches` predicate. |
| **control flow** (`If`/`For`/`While`/`Retry`) | selection + iteration | the substrate's **own** `Behavior` coproduct — `src/v2/std/node.dag:26` is `… \| Branch \| Loop`. `If` = `Branch`; `For`/`While`/`Retry` = `Loop` with a **`DescentEvidence`** bound (`std.graph` / `std.computation` — `Strict \| NonIncreasing \| DescentUnknown`). Control flow is **not invented** — it is the substrate execution model (§4: "recursion is sugar over `Loop`; cyclic relations via acyclic encodings") *made addressable as data* so `emit` can render it. |
| **`Pipeline`** | an ordered sequence of steps + a failure-propagation policy | `FreeMonoid<Step>` (ordered composition, `std.algebra`) + an `Outcome`-style failure policy (`fail-fast` ≈ `set -e`/`&&` vs `continue`). Not a bare `List` — the *policy* is the orchestration fact. |
| **`Step`** | the recursive node | a coproduct whose arms are `Run \| Check \| If \| For \| While \| Retry \| Seq(Pipeline) \| …`. This **is** new — it is the union that makes the others composable. |

**So the net-new concepts are `Step` (the recursive union) and `Pipeline` (sequence + failure
policy). Everything else is a binding onto an existing authority.** That is the §2 test passing: net
concepts do not grow by re-invention.

**⚠ FLAG 2a — is `Pipeline` distinct from `Seq`?** I model `Pipeline = Seq + failure-policy`. One
could instead fold the policy into each `Step` and make `Pipeline` a pure `FreeMonoid<Step>`. I lean
to a thin `Pipeline { steps, on_failure }` record because the failure policy is a *whole-sequence*
fact (`set -e` is script-scoped), but flag it for sign — it is the one place the vocabulary could be
one concept smaller or one larger.

**⚠ FLAG 2b — `Check` predicate algebra vs `Run` exit-status.** A `Check` and "a `Run` whose
non-zero exit is fatal" overlap. I keep them distinct because a `Check` carries a **`Diagnostic`**
(gap A) and asserts a *predicate* (filesystem/log/string), whereas a `Run` *is* the effect. But the
boundary (`grep` is a `Run` that the surrounding `if` turns into a `Check`) is real — see §9.

---

## 3. The intent vocabulary (the coproduct)

Medium-agnostic. New module **`std/orchestration.dag`** (⚠ FLAG 3a: name/home — it is structured-
program control flow, a *universal framework* per Böhm–Jacopini = sequence/selection/iteration, so
`std` is justified; but if the operator reads orchestration as a *domain* model it belongs downstream
of `std`. I lean `std` because it grounds into the substrate's own `Behavior`, which is maximally
universal). **It imports no language AST** — that is what keeps `program.dag` out of the intent layer.

### 3.1 Tier 1 — the control-flow skeleton (renders consumer (1) fully)

```
# sketch, not final syntax — sign the SHAPE, not the spelling
type Predicate                              # grounds Check + If conditions; a closed algebra
  = ExitZero    { run: Run }                #   `<run>` succeeded            -> `if <run>; then`
  | FileExists  { path: PathIntent }        #   std.filesystem predicate     -> `[ -e <path> ]`
  | LogMatches  { source: Run, pattern: GrepAlternation }   # grep -qiE '<pat>' <file>
  | StrEq       { lhs: Word, rhs: Word }    #                                -> `[ "<lhs>" = "<rhs>" ]`
  | Not         { inner: Predicate }
  | And         { lhs: Predicate, rhs: Predicate }
  | Or          { lhs: Predicate, rhs: Predicate }

type Run {                                  # one command effect + its modifiers
  command: CommandRef                       #   argv; §3-transport modeling of a CLI (opaque-fenced ok)
  env:     List<EnvBinding>                 #   CARGO_BUILD_JOBS=1   (EnvSet | EnvUnset)
  redirect: RedirectSpec?                   #   2>&1 | >/dev/null 2>&1   (closed enum, NOT a string)
  capture:  CaptureSpec?                    #   `| tee <file>` / `$(...)` command-substitution
}

type Step
  = Do      { run: Run }
  | Assert  { check: Check }                #   Check = Witness + Diagnostic-on-Violates (gap A)
  | If      { cond: Predicate, then: Pipeline, else_: Pipeline? }   # else OPTIONAL
  | For     { binder: Symbol, over: ValueSource, body: Pipeline }
  | While   { cond: Predicate, body: Pipeline, bound: DescentEvidence }   # bound REQUIRED (fail-closed)
  | Retry   { attempts: Int, body: Pipeline,                         # bounded -> terminating
              classify: FailureClassifier, on_exhausted: Pipeline }

type Pipeline { steps: List<Step>, on_failure: FailurePolicy }      # FailFast | Continue
type FailurePolicy = FailFast | Continue
```

Notes that matter for sign:

- **`While.bound: DescentEvidence` is required, not optional.** An unbounded `while true` is exactly
  the §5 "bounded-forever ≠ unknown" fail-open. By making the bound a **required field grounded in
  `DescentEvidence`** (`Strict | NonIncreasing | DescentUnknown`), an un-terminating loop is
  **unwritable** (construction, not validation — §5). `DescentUnknown` is the honest fail-closed
  bottom. This is a *strictly stronger* guarantee than the hand-rolled bash, which has no
  machine-checkable bound at all.
- **`Retry.attempts: Int` is the bound** — `Retry` is the common bounded-loop case made first-class
  (consumer (1)'s whole shape). `classify: FailureClassifier` grounds directly into the **already
  existing** `gunbc.ci_failure_class` (`InfraSignature` / `infra_retry_grep_alternation` /
  `classify_failure_reason`). No new classifier concept.
- **`If.else_: Pipeline?`** — optional else. Crucially, *adding else does not add a `Step` arm*; it is
  a field. At the **emit** layer (§4) `else` present vs absent selects a **different grammar
  production**, not a different intent node and **never** a `program.dag` change.
- **`RedirectSpec` / `EnvBinding` / `CaptureSpec` are closed enums**, not strings — `2>&1` is
  `StdoutAndStderr`, `>/dev/null 2>&1` is `DiscardAll`, `env -u X` is `EnvUnset{X}`. This is the
  §5.1 "the medium stays a `Node`, string only at the edge" rule applied to shell *operators*.

### 3.2 Tier 2 — bindings + procedures + expressions (named, for consumer (2))

Consumer (2) (`fleet_converge`) needs three more things. I propose them but **scope them explicitly as
tier 2** so tier-1 can ship and prove the architecture first:

```
  | Let   { name: Symbol, value: Expr }                    # drifted=0 ; verdict=converged
  | Call  { procedure: Symbol, args: List<Word> }          # decide_verdict "$eff" "$want"
type Procedure { name: Symbol, params: List<Symbol>, body: Pipeline }   # shell function
type Expr                                                  # the value algebra
  = Lit | VarRef { name } | Arith { op, lhs, rhs }         # $((drifted + 1))
  | CmdSubstLines { run: Run }                              # $(systemctl ... | awk ...)
```

**⚠ FLAG 3b — tier 2 is real scope, not polish.** `fleet_converge` cannot be rendered *fully*
without `Procedure`/`Let`/`Arith`. Tier 1 renders its control-flow skeleton (the for/while/elif), but
the function definitions and `$((...))` accumulators need tier 2. I am **not** hiding this: the
honest claim is *"tier 1 renders consumer (1) end-to-end and consumer (2)'s control flow; tier 2
completes consumer (2)."* See §6 for exactly which lines need tier 2.

**⚠ FLAG 3c — `ValueSource` / `CmdSubstLines` is the soft spot.** `for unit in $(systemctl … | awk
…)` iterates over the *lines of a command's stdout*. Modeling that faithfully means the `over` is a
`Run` whose capture is "split on newlines." This is sound but it drags pipe+awk into the model. I
propose `ValueSource = CmdSubstLines{run} | Glob{pattern} | ModeledList{items}` and **fence the awk
pipeline as an opaque `CommandRef`** (the §3 honest boundary: the orchestration layer composes Runs;
whether each Run's argv is fully modeled is a *separate* extdeps axis — see §7).

---

## 4. The emit mechanism — `emit(intent, Bash)` over grammar rows (NOT over `program.dag`)

This is the load-bearing half. It **reuses gap A's exact realized pattern** (the `EmitDirective` →
bash rows in `src/v2/extdeps/languages/bash.dag:1305-1389`) and **the bash grammar productions that
already exist** for compound forms.

### 4.1 What already exists and proves the mechanism

`v2.extdeps.languages.bash` already models, as bidirectional grammar rows (each with
`*_concrete_tokens` / `*_emitted` Node / `*_source_text` / `*_lex` / `*_translation_rules_node` /
`*_target_model`):

- sequencing: `bash_and_then` (`&&`), `bash_or_else` (`||`), `bash_pipe` (`|`)
- grouping: `bash_subshell_true`, `bash_pipe_and_or_brace_grouped`
- **control flow: `bash_if_true_then_false`** — an `if … then … fi` production
- **nesting: `bash_subshell_and_or_nested`** — proves a production's bound slot can hold a
  *recursively-emitted sub-Node*, not just a leaf spelling.
- effects/modifiers: `bash_env_prefixed_cargo` (`VAR=v cmd`), `bash_assign_cmdsubst_mktemp`
  (`x=$(mktemp)`), `bash_with_redir_stderr_null` (`2>/dev/null`), `bash_exit_code_1` (`exit 1`),
  `bash_heredoc_simple`.

**The mechanism is therefore already realized — for fixed literals.** `bash_if_true_then_false`
emits `if true; then false; fi` where `true`/`false` are *baked in*. The gap B work is to lift these
from **fixed-literal** productions to **parameterized** productions whose bound slots are filled by the
recursive emit of a child intent.

### 4.2 The lift: parameterized control-flow productions

An `If{cond, then, else_=None}` emits via a bash production whose token list is:

```
[ FixedToken kw_if , BoundToken(cond) , FixedToken kw_semicolon_then ,
  BoundToken(then) , FixedToken kw_fi ]
```

— and the binding for `cond` resolves to `emit(cond, Bash)` (itself a production — `[ <run> ]` or a
`test` expression), and `then` resolves to `emit(then_pipeline, Bash)`. This is **exactly**
`bash_conj_emitted_from_tokens` (bash.dag:48) — already used by `bash_subshell_and_or_nested` to
splice a sub-Node into a BoundToken position. **No new emit kernel; a `BoundToken` binding whose
spelling is a sub-emission instead of a leaf.**

`else_ = Some(p)` selects a **different production** —
`[ kw_if, cond, kw_then, then, kw_else, BoundToken(else_), kw_fi ]` — chosen by `emit` the way ingest
selects a production forward (§4 of DESIGN, one grammar both directions). `elif` is the same: an
`If` whose `else_` is a `Pipeline` containing a single `If` renders as `elif` via a production that
recognizes that shape (⚠ FLAG 4a: elif-vs-nested-if-fi is an emit *optimization* — `else { if … }`
and `elif` are semantically identical; I propose emitting `elif` when the else-body is exactly one
`If`, but flag that the first cut may emit nested `if…fi` and the `elif` sugar is a follow-on row).

`For`, `While`, `Retry` are three more production families:

```
For    -> for <binder> in <over>; do <body> done
While  -> while <cond>; do <body> done          (bound is checked at MODEL time, not emitted)
Retry  -> emitted as a bounded unrolling OR a `for _ in $(seq 1 N)` loop with a classify+break body
```

**⚠ FLAG 4b — `Retry` lowering is the least-settled emit.** Two options: (i) **unroll** to the
nested-`if` cascade consumer (1) hand-writes today (faithful to the current output, attempts is a
small literal), or (ii) emit a real `for i in $(seq 1 N); do … classify … break/continue; done`
loop. (i) round-trips trivially and reproduces today's bytes; (ii) is smaller shell but needs loop+
break productions. I lean **(i) for the first cut** (it makes the retry PR a *byte-preserving*
re-expression — the strongest possible correctness proof, see §7) and (ii) as a follow-on. Sign
needs to pick.

### 4.3 Why this never touches `program.dag` (the central constraint, discharged)

| where control flow could live | this design |
| --- | --- |
| `program.dag` `ShellStmt` arms (`Else`/`For`/`While`/`Case`) | **never added** — `program.dag` stays frozen, roster shrinks, it is deleted at lane-end |
| a richer hand-rolled `serialize_bash` | **never** — `serialize_bash` is the forward emitter being dissolved |
| **intent coproduct** (`std/orchestration.dag`) | the control-flow *meaning* (medium-agnostic) |
| **bash grammar productions** (`v2.extdeps.languages.bash`) | the control-flow *shell spelling* (the realization edge the §5 guard permits) |

The §0 containment guard (emission-ingestion-inverse §5) **enforces** this: any module that imports
`program.dag`'s `ShellStmt`/`serialize_bash` outside the realization-edge allow-set is a
`RealizationVocabularyLeak`. The intent layer imports *no* bash AST, so it is clean by construction.
As each consumer migrates from hand-authored shell to `Pipeline` intent, its `program.dag` import
drops off the frozen roster; when the roster empties, the guard flips ratchet→wall and `program.dag`
is deletable. **This design is the thing that empties the roster.**

---

## 5. Worked rendering — consumer (1) `ci_cargo_eagain_retry_core`

The whole 40-line nested-`if` is **one `Retry` intent** (`<command>` is the existing `command`
parameter):

```
Retry {
  attempts: 3,                                  # original + 2 escalations
  body: Pipeline [ Do { run: Run {              # attempt N
          command: <command>,
          env:     [ <escalation env for this attempt> ],   # [] | CARGO_BUILD_JOBS=1 | +EnvUnset RUSTC_WRAPPER
          capture: TeeTo { log: Fresh(BUILD_LOG) } } } ],    # $(mktemp) + 2>&1 | tee
  classify: FailureClassifier {                 # grounds into gunbc.ci_failure_class — EXISTING
     infra:  LogMatches { pattern: infra_retry_grep_alternation() },  # grep -qiE '<alt>' "$LOG"
     escalations: [ EnvSet(CARGO_BUILD_JOBS=1),                       # attempt 2 mitigation
                    EnvUnset(RUSTC_WRAPPER) ],                        # attempt 3 mitigation
     on_infra_warning: EmitDirective { severity: Warning, message: "…cold retry…" } },  # gap A!
  on_exhausted: Pipeline [ Do { run: Run { command: Exit(1) } } ]     # non-infra OR attempts spent
}
```

**Tier coverage:** this is **tier-1-complete**. Every feature maps:

| bash feature | intent carrier |
| --- | --- |
| `$(mktemp)` + `2>&1 \| tee "$LOG"` | `Run.capture = TeeTo { log: Fresh(…) }` |
| `grep -qiE '<alt>' "$LOG"` as condition | `Predicate.LogMatches` (classifier `infra` arm) |
| `env -u RUSTC_WRAPPER CARGO_BUILD_JOBS=1` | `Run.env = [EnvUnset(RUSTC_WRAPPER), EnvSet(CARGO_BUILD_JOBS, 1)]` |
| `echo "::warning::…"` | `EmitDirective{Warning}` — **gap A row, already landed (#5505)** |
| `exit 1` / `\|\| exit 1` | `on_exhausted` / `FailFast` |
| the nested-`if` cascade itself | the `Retry` lowering (§4.2, option (i) reproduces it byte-for-byte) |

`emit(this, Bash)` walks the `Retry` production → unrolls `attempts` → for each attempt emits a
`Run` production (env-prefix rows + capture rows, already in bash.dag) wrapped in an `if !( … ); then`
production whose body is the classify `if grep …; then <warning> <next attempt> else exit 1 fi`. The
diagnostic emits via the gap-A `emit_directive` rows. **Output is the same shell; the source is one
`Retry` value.**

---

## 6. Worked rendering — consumer (2) `fleet_converge_emit`

This is the honest stress test. The control-flow skeleton is **tier-1**; the function/arith residue is
**tier-2**.

`decide_verdict` (a shell **function**, fleet_converge_emit.dag:77-80):

```bash
decide_verdict() {
  if [ -z "$1" ]; then verdict=absent
  elif [ "$1" = "$2" ]; then verdict=converged
  else verdict=drifted; fi
}
```

renders as a **tier-2** `Procedure`:

```
Procedure { name: decide_verdict, params: [eff, want], body: Pipeline [
  If { cond: StrEq{ VarRef(eff), Lit("") },               # [ -z "$1" ]   ⚠ FLAG 6a: -z vs StrEq""
       then: [ Let { verdict, Lit(absent) } ],
       else_: [ If { cond: StrEq{ VarRef(eff), VarRef(want) },           # elif
                     then: [ Let { verdict, Lit(converged) } ],
                     else_: [ Let { verdict, Lit(drifted) } ] } ] } ] }
```

The `for`-loop over `systemctl` output (fleet_converge_emit.dag:99-105):

```bash
for unit in $(systemctl list-units … "$5" 2>/dev/null | awk '{print $1}'); do
  systemctl set-property "$unit" "$6=$7" >/dev/null 2>&1 || true
  eff="$(systemctl show "$unit" --property="$6" --value 2>/dev/null || true)"
  if [ "$eff" != "$7" ]; then all_ok=0; fi
done
```

renders as a **tier-1 `For`** whose `over` is a `CmdSubstLines{run}` (§3.2) and whose body is `Do` +
`Let` + `If`:

```
For { binder: unit,
      over: CmdSubstLines { run: Run { command: <systemctl list-units … | awk …>,   # ⚠ FLAG 6b
                                       redirect: DiscardErr } },
      body: Pipeline [
        Do { run: Run { command: <systemctl set-property …>, redirect: DiscardAll, capture: IgnoreFail } }, # || true
        Let { eff, CmdSubstLines… },
        If { cond: Not{StrEq{VarRef(eff), VarRef(want)}}, then: [ Let { all_ok, Lit(0) } ] } ] }
```

The `while IFS= read -r unit; do … done` drain loop (fleet_converge_emit.dag:119-122) is a `For`
over `CmdSubstLines` (a `while read` is iteration over stdin lines — **⚠ FLAG 6c**: I model
`while read` as `For` over a line source, *not* as a `While`, because it has no predicate; flag in
case the operator wants `while-read` as its own constructor).

**Honest tally for consumer (2):**

| feature | tier | covered? |
| --- | --- | --- |
| for / while-read / if / elif | 1 | ✅ |
| `\|\| true`, `>/dev/null 2>&1`, `2>&1` | 1 | ✅ (`IgnoreFail`, `DiscardAll`, closed enums) |
| `decide_verdict` etc. shell **functions** | 2 | ✅ via `Procedure`/`Call` (tier 2) |
| `drifted=$((drifted + 1))` **arithmetic** | 2 | ✅ via `Let`+`Arith` (tier 2) |
| positional params `$1`…`$7` | 2 | `Procedure.params` (tier 2) |
| `systemctl … \| awk '{print $1}' \| sort -r \| head` pipelines | **fenced** | ⚠ opaque `CommandRef` (§7) |

So consumer (2) is **tier-1 for its control flow, tier-2 for its functions/arith, and fences the
awk/sort/systemctl argv as opaque commands**. I flag this as the realistic scope rather than claiming
a clean one-shot.

---

## 7. Correctness story — fail-closed + round-trip

### 7.1 Fail-closed (construction over validation, §5)

- **Termination is unwritable-if-unbounded.** `While.bound: DescentEvidence` and `Retry.attempts: Int`
  are *required fields*. A non-terminating loop cannot be constructed — strictly stronger than the
  hand-rolled bash, which has no checkable bound. `DescentUnknown` is the honest fail-closed bottom.
- **`Check` cannot fail open.** A `Check`'s `Violates` arm *is* a `Diagnostic` + non-zero
  `ProcessExit` by construction (it reuses the gap-A `Outcome`/`Rejected` fail-closed monoid). The
  bash bug class "forgot `|| exit 1`" becomes **unwritable** — the failure exit is part of the
  `Assert` production, not something the author can drop.
- **Operators are closed enums, not strings.** `RedirectSpec`/`EnvBinding` cannot express an
  un-modeled redirect; an unanticipated shell operator is a *typed rejection at authoring*, not a
  silent pass (the §5.1 `MediumStructureLeak` it replaces).
- **Discriminating RED witnesses** (mirroring gap A's `emit_directive_…_to_stdout_wrong`): each
  control-flow production ships a `*_wrong_*` twin — `for` without `done`, `if` without `fi`, an
  `Assert` whose emitted shell drops the non-zero exit (the fail-open it forbids), a `Retry` whose
  classifier never matches (would loop-or-fall-through). Each goes **RED by execution**, so the row
  is proven the §5-of-DESIGN way (a real consumer green + a discriminating input red), not by grep.

### 7.2 Round-trip (the §5.2 oracle)

The bash control-flow productions are **bidirectional grammar rows** (the `if_true_then_false`
production already has an ingest side). So the §5.2 oracle — *`ingest ∘ emit = id` over the same
rows* — gates them: a parameterized `If` production is *done* only when emitting `If{…}` to shell and
re-ingesting that shell yields the same `If` Node. `DecodeFidelity` marks the boundary:

- **`Lossless`** for the closed tier-1/tier-2 constructor set (sequence/selection/iteration/let/call
  over modeled commands) — these round-trip.
- **`lossy`-fenced** for the opaque `CommandRef` residue (an un-modeled `awk '{print $1}'` argv): the
  orchestration layer round-trips the *control flow*; the opaque command body is fenced and marked,
  **never fake-gated** (§5.2 face ③). This is the §3 honest split: orchestration owns the *composition*;
  whether each leaf command's argv is fully modeled is a separate extdeps axis (gap-A-adjacent) that
  this design **deliberately does not block on**.

**⚠ FLAG 7a — the strongest first proof is byte-preservation.** For consumer (1), `Retry` lowering
option (i) (§4.2) reproduces today's exact shell. So the migration PR can assert
`emit(retry_intent, Bash) == ci_cargo_eagain_retry_core(command)` **byte-for-byte** — the most
discriminating possible witness (any emit drift goes RED against the known-good string). I strongly
recommend the first consumer migration carry this byte-equality witness; it is worth more than the
round-trip law for the *migration* step (the round-trip law is the *medium-completeness* gate, a later
step — emission-ingestion-inverse §5.2(A)).

---

## 8. The N+M payoff and how `RunStep` dissolves

`emit(intent, Bash)` is one direction of `emit = serialize_target ∘ translate`. The **same** `If`/
`For`/`Retry` intent renders to other media by **rows, not new emitters** (§4 of DESIGN, N+M not
N×M):

- **GitHub Actions YAML** — `extdeps/github/actions.dag`'s `RunStep { run: String, if_condition:
  String }` is the live fork: orchestration-as-string *inside* a YAML model. Under this design,
  `run` becomes a `Pipeline` and `if_condition` becomes an `If` intent — and the GHA realization rows
  render `if_condition` to the YAML `if:` key and a single `Run` to `run:`. The two string fields
  dissolve into one intent + two target row-sets (bash + GHA).
- **Make / PowerShell / etc.** — each is a row-set binding the same intent constructors to that
  medium's control-flow syntax. Authoring the intent once; M media = M row-sets, never M emitters.

This is the displaced cost (§6): today, every new orchestration target or every new control-flow shape
is hand-written bash (consumer (1)/(2)) *plus* a parallel GHA `if_condition` string *plus* the
`MediumStructureLeak` grep-checks that try to police them. The intent layer makes the shell fall out
of one authored value and makes "no runner env var accessed" a **model walk**, not a grep.

---

## 9. Open questions (sign-blocking vs follow-on)

**Sign-blocking (need a decision before build):**

1. **⚠ 2a** — `Pipeline = Seq + failure-policy` (a record) vs `Pipeline = FreeMonoid<Step>` with
   policy on each `Step`. (I lean record.)
2. **⚠ 4b** — `Retry` lowering: unroll (byte-preserving, my lean) vs `for … seq … break` loop.
3. **⚠ 3a** — home: `std/orchestration.dag` (my lean, grounds into substrate `Behavior`) vs a
   downstream domain module.
4. **⚠ 2b / 9-Check-vs-Run** — the `Check`/`Run`-with-fatal-exit boundary. Where exactly does a
   `grep` stop being a `Run` and become a `Check`? Proposal: a bare command is a `Run`; a command
   *used as a condition* (inside `If.cond`/`While.cond`/`classify`) is wrapped as `ExitZero{run}` or
   `LogMatches`. Confirm this is the rule.

**Follow-on (does not block tier-1 build):**

5. **⚠ 3b/3c, 6b, 7b (opaque-argv fence)** — how much of each leaf command (`systemctl`, `awk`,
   `sort`) to model as an extdeps shape vs fence as opaque `CommandRef`. The design *works* with full
   fencing; modeling them is the gap-A-adjacent extdeps axis, separately prioritized.
6. **⚠ 4a** — `elif` sugar vs nested `if…fi` (emit optimization).
7. **⚠ 6c** — `while read` as `For`-over-lines vs its own constructor.
8. **Tier-2 sequencing** — `Procedure`/`Let`/`Arith` land after tier-1 proves the architecture on
   consumer (1). Consumer (2) is the tier-2 acceptance case.

---

## 10. Proposed sequencing (build order, once signed — NOT this task)

1. **Tier-1 vocab** in `std/orchestration.dag` (`Predicate`/`Run`/`Step`/`Pipeline`), grounding
   `Check`→`Witness`+gap-A, control flow→`Behavior`+`DescentEvidence`, `Retry.classify`→
   `gunbc.ci_failure_class`. No emit yet.
2. **Parameterized bash control-flow productions** in `v2.extdeps.languages.bash` (`If`/`For`/`While`/
   `Retry` + the env/redirect/capture modifier rows), each with a `*_wrong_*` RED twin.
3. **Migrate consumer (1)** `ci_cargo_eagain_retry_core` → a `Retry` value, with the **byte-equality
   witness** (⚠ 7a). This is the first paying consumer and the strongest proof.
4. **Round-trip law** over the new productions (medium-completeness gate, §5.2(A)).
5. **Tier-2** (`Procedure`/`Let`/`Arith`) → **migrate consumer (2)** `fleet_converge_emit`.
6. Each migrated consumer drops off the §0 containment-guard roster; when it empties + the
   `MediumStructureLeak` rosters empty, `program.dag` is **deleted** (lane-end).

## Dissolution trigger (DESIGN §6)

Delete this doc when the tier-1+tier-2 vocabulary is built, both named consumers are migrated to
`Pipeline` intent under a green round-trip law, and `program.dag` is deleted — at which point the
design lives in the executable rows + the guard, and this prose is redundant (it folds into
[emission-ingestion-inverse.md](emission-ingestion-inverse.md)'s own dissolution).
