# SHELL-DAG-CENSUS-0A — dispatch brief (derived shell execution census)

Status: dispatch brief for the next shell→dag cut. Authored 2026-08-20 against `4cec10f6`, corrected twice before dispatch (see "Corrections that produced this brief" at the end). Not a design authority — the carriers it describes become the authority when they land.

# Revised call

The 0A contract changes in four material ways:

1. **`36/1 at 4cec10f…` is a historical calibration receipt, not an oracle and not a permanent expected count.**
2. **Every fact states its evidence grain.** Declared type, static reachability, interpreter semantics, and runtime observation are separate claims.
3. **The argv representation-ambiguity repair is removed from scope.** #8549 already closed it at the seam.
4. **Instrument validity includes formatting, subject identity, exact-token discrimination, and message integrity—not merely detector-logic mutations.**

The dispatch precondition remains unmet. `main` is still `4cec10f66a3d84cbde631e20ca9feaa31457d88c`; #8652 remains open, while #8653 is closed unmerged. Since your message, #8652’s body has been updated to claim that a clean two-generation fixed-point protocol was run, but its CI checkbox remains open and the repair is not on `main`. The prerequisite is therefore unchanged: the merged `main` SHA itself must pass required regen, fixed point, and the full witnesses workflow. fileciteturn121file0L1-L7 fileciteturn114file0L4-L13 fileciteturn115file0L4-L13

# The corrected historical calibration

At the exact historical subject:

```text
subject SHA:
4cec10f66a3d84cbde631e20ca9feaa31457d88c

EmitArtifactThenThinRun occurrences:
  1  variant declaration
 36  production match arms across 14 modules
  4  dag/test/claim occurrences
----
 41  total occurrences

Production-arm body partition:
  1  static shell-realizing route
 35  no static shell route
  0  unresolved/unknown
```

The sole static realizer is:

```text
gunbc.host_effect_realize.realize_converge_on_host
  -> EmitArtifactThenThinRun
  -> ExistingHostQuiescentReload
  -> realize_converge_in_process
  -> retained script execution
```

The number `36` must **not** be compiled into the detector as its expected current population. Instead, 0A must run the completed detector against a worktree pinned to that exact SHA and produce a dated calibration receipt. It must separately run against its own branch head and publish the current population.

A disagreement has this disposition:

```text
HistoricalCalibrationDisagrees {
  historical_receipt
  detector_result
}
```

It is neither “the detector is wrong” nor “update the expected count.” It stops acceptance until the discrepancy is explained.

The omitted Codex arm is now an explicit control. Its `EmitArtifactThenThinRun` branch returns `CodexSupervisedSessionApplyRefused`, but only a body read establishes that it performs no execution. The return variant’s name is not the evidence. fileciteturn120file0L2-L7

# Evidence grain is part of every fact

The prior brief used “reach” too loosely. Given the current refinement and coproduct construction gaps, 0A must not imply that a statically visible route is a successfully executable runtime route.

I would rename the central result from:

```text
MayReachShell
```

to:

```text
StaticShellRoute
```

and make the basis explicit.

## Required grain vocabulary

```dag
type EvidenceGrain
  = AuthoredSyntax
  | DeclaredType
  | StaticControlFlow
  | StaticValueFlow
  | InterpreterSemantics
  | RuntimeObservation
  | ConstructionGuarantee
```

A fact may cite more than one grain, but it may claim only what those grains establish.

| Fact class | Evidence grain | What it may claim | What it must not claim |
|---|---|---|---|
| File/declaration/body inventory | `AuthoredSyntax` | This body or arm exists at this SHA | It executes in production |
| Declared argument or return type | `DeclaredType` | The author declared `List<String>`, `String`, etc. | The runtime `Value` has that representation |
| Call edge | `StaticControlFlow` | The body contains or transitively selects this call | The call is dynamically reached |
| Value origin/path | `StaticValueFlow` | This authored expression can feed this parameter or channel | The runtime value satisfies its refinement or coproduct shape |
| Shell-sink meaning | `InterpreterSemantics` | This operation/channel interprets bytes as a shell program when evaluated successfully | A valid carrier necessarily reaches the interpreter |
| Shell route | `StaticControlFlow + StaticValueFlow + InterpreterSemantics` | An authored path can attempt shell interpretation | A shell process was spawned |
| Runtime carrier shape | `RuntimeObservation` or `ConstructionGuarantee` | The observed/guaranteed runtime representation is this shape | Anything inferred solely from the declared type |
| Runtime process execution | `RuntimeObservation` | This process actually executed with these channels | Whole-corpus coverage |
| Final migration disposition | Reviewed classification in 0B | This static route belongs to runtime, foreign, bootstrap, or pending work | Completeness of the derived population |

## Consequence for the detector

The detector must not prune a route because:

```text
the declaration says List<String>
the declaration says NonEmptyStr
the declaration says a particular coproduct
a refinement should hold
an arm appears unconstructible from the declared type
```

At 0A grain, every syntactically authored branch remains part of the control-flow population unless a stronger construction wall is separately cited.

The detector may report both:

```dag
type DeclaredCarrierShapeFact {
  occurrence: OccurrenceId
  declared_type: String
  basis: DeclaredType
}

type RuntimeCarrierShapeReading
  = RuntimeShapeUnobserved
  | RuntimeShapeObserved {
      shape: RuntimeValueShape
      receipt: DeclarationRef
    }
  | RuntimeShapeGuaranteed {
      shape: RuntimeValueShape
      wall: DeclarationRef
    }
```

Most 0A rows should carry:

```text
runtime_shape = RuntimeShapeUnobserved
```

That is not a defect in 0A. It is an honest statement of its static grain.

The sharp-ant `258 operations / five declared argv shapes` measurement can be retained as a **declared-type side receipt**, but it is not an 0A acceptance condition and cannot establish that a runtime `Value` is a list, string, or `ProcessArgvExpansion`.

# Class 1 is struck completely

The reissued brief must contain no work item to repair `push_shell_argv_tokens`’ former list-versus-concatenated-string ambiguity.

#8549 already changed that position to refuse with a typed, located diagnostic when the same free monoid admits both readings. It preserved native list expansion, codepoint-monoid decoding, and the explicit `ProcessArgvExpansion` route. That is a completed prerequisite, not remaining SHELL-DAG work. fileciteturn119file0L4-L14

For 0A, direct argv execution is simply a negative discrimination control:

```text
shell.Exec.RunArgv
    -> direct process carrier
    -> NOT a POSIX/Bash program sink
```

A malformed or unexpected runtime representation may cause a refusal. It does not transform direct process execution into shell interpretation.

# Instrument robustness contract

The five failures you identified should become five independent controls. They are not one generic “bad grep” class because each has a different remedy.

| Failure observed | Required 0A control |
|---|---|
| Too narrow to contain the answer | Recursive full-body traversal fixture with the real sink more than ten nested nodes below the arm root |
| Pointed at the wrong subject | Subject identity carrier: exact commit, roots, inventory digest, and an anchor file that must occur in the selected corpus |
| Format-induced false zero | Surface-format invariance pair plus a nonzero control partition in the same run |
| Over-broad token false positive | Exact parsed identity control: `words` must not match `keywords` |
| Messenger altered the result | Structured receipt file with content digest and read-back; no count transmitted through shell interpolation or hand-transcribed prose |

## 1. No source-text grep as the authority

The production detector must consume parsed bodies or a typed body-fact projection.

It must not classify routes using:

```text
grep
regular expressions over source text
line windows
substring search
file-name patterns
variant-name patterns
comment or prose matching
```

A text search may be used during development to locate candidates. It may not produce the committed census.

If current body projections cannot preserve callee identity, branch identity, and argument edges, the worker must stop and open a separate substrate projection increment. It must not fill the gap with a source-text approximation.

## 2. Format-equivalence fixture

Two fixture files should express the same semantic route with deliberately different formatting:

```text
fixture A:
  ordinary one-line call and string layout

fixture B:
  calls split over lines
  nested indentation
  comments between syntax nodes
  long strings using continuation/layout forms
  arguments placed many lines below the callee
```

After normalizing fixture-specific path and occurrence identity, their derived facts must be equal:

```text
normalize(census(fixture_a))
==
normalize(census(fixture_b))
```

This is the direct control against the wrapped diagnostic-string false zero.

## 3. Same-run dirty control

Every production census invocation must also scan an isolated control partition containing exactly:

```text
1 known shell route
1 direct-argv no-shell control
1 substring near miss
1 deeply nested route
```

The output remains partitioned:

```text
production_result
control_result
```

A production result of zero is admitted only when the control partition reports its exact expected population in the **same process, with the same detector implementation and settings**.

This encodes:

> When a scan returns zero, the first hypothesis is the scan.

## 4. Orthogonal observation

At least one positive observation must not flow through the census fold.

A suitable control is:

```text
directly parse one planted fixture
-> locate its exact callee node
-> locate the exact program-channel argument edge
-> assert both exist
```

This control may reuse the parser, but it must not call:

```text
shell_route_facts()
shell_sink_facts()
shell_producer_facts()
```

or their shared selection predicate.

The historical `36/1` audit is a second, human-produced orthogonal calibration. Its earlier `24` error demonstrates why it cannot be the authority, but the corrected measurement remains useful as an independent comparison.

## 5. Exact identity, never token substrings

All operation, function, variant, and field matching must use parsed declaration or symbol identity.

The fixture matrix must include:

```text
words       -> intended exact symbol hit
keywords    -> no hit
swordsmith  -> no hit
```

Likewise, `Run` must not match `RunArgv`, and `ShellCommand` in prose must not match a `ShellCommand` constructor.

## 6. Receipt integrity

The detector should emit a structured artifact, for example:

```text
target/shell-dag-census/<subject-sha>/facts.tsv
target/shell-dag-census/<subject-sha>/summary.json
```

The summary must carry:

```text
subject commit
source roots
source inventory digest
detector declaration or binary identity
schema version
files attempted
files parsed
parse refusals
declarations
bodies
match arms
sink occurrences
static routes
unknown routes
facts artifact digest
```

A witness must read the artifact back and prove:

```text
reported row count == parsed artifact row count
reported digest == content digest of artifact
reported subject == requested subject
```

The PR body may quote that artifact. It must not be the only home of the result.

No command should place report text containing backticks, dollar signs, braces, or shell metacharacters inside an interpolated shell string. The artifact path—not its content—is the handoff.

# Revised result types

```dag
type StaticShellReachability
  = NoStaticShellRoute
  | StaticShellRoute {
      route: ShellRouteFact
    }
  | StaticShellRouteUnknown {
      cause: ShellCensusRefusal
    }

type ShellRouteFact {
  root_consumer: DeclarationRef
  owner: DeclarationRef
  arm: BodyArmIdentity?
  sink: ShellSinkFact
  call_path: List<DeclarationRef>
  branch_path: List<BodyBranchFact>
  source: ShellProgramSourceReading
  declared_carrier: DeclaredCarrierShapeFact?
  runtime_carrier: RuntimeCarrierShapeReading
  basis: List<EvidenceGrain>
}

type ShellSinkFact {
  owner: DeclarationRef
  occurrence: OccurrenceId
  language: ShellLanguage
  channel: ShellProgramChannel
  interpreter_path: List<InterpreterStep>
  basis: List<EvidenceGrain>
}

type ShellProgramSourceReading
  = RawLiteralSource
  | StaticStringCompositionSource {
      producers: List<DeclarationRef>
    }
  | GrammarEmittedSource {
      emitter: DeclarationRef
    }
  | MaterializedFileSource {
      writers: List<DeclarationRef>
      path_identity: FilePathIdentity
    }
  | ExternalSource
  | MixedSource {
      parts: List<ShellProgramSourceReading>
    }
  | SourceUnknown {
      cause: ShellCensusRefusal
    }
```

The names make the ceiling visible:

```text
StaticShellRoute
```

does not claim:

```text
RuntimeShellExecuted
```

A later wet receipt may add:

```dag
type RuntimeShellExecutionReading
  = RuntimeShellExecutionUnobserved
  | RuntimeShellExecutionObserved {
      route_key: ShellRouteKey
      receipt: DeclarationRef
    }
```

That is not required to complete 0A.

# Reissued dispatch brief

> ## SHELL-DAG-CENSUS-0A — derive static shell-program routes from parsed bodies
>
> **Precondition**
>
> Do not branch until #8652 is merged and the resulting exact `main` SHA passes:
>
> ```text
> claim_executor --required-regen
> claim_executor --required-regen-fixed-point
> full witnesses workflow
> ```
>
> A branch-local claim or first-generation result is not sufficient. Record the exact first green `main` SHA as the census subject.
>
> **Goal**
>
> Derive, from parsed `.dag` bodies, every authored path that can attempt POSIX/Bash program interpretation. The population must be derived from the corpus and interpreter semantics, never from remembered bridge names, filenames, imports, variant names, refusal types, line windows, or source-text grep.
>
> This is a **static may-route census**, not a runtime-execution guarantee.
>
> **Required evidence grain**
>
> Every fact carries one or more of:
>
> ```text
> AuthoredSyntax
> DeclaredType
> StaticControlFlow
> StaticValueFlow
> InterpreterSemantics
> RuntimeObservation
> ConstructionGuarantee
> ```
>
> A declared type may be recorded, but no runtime representation may be inferred from it. Do not prune routes using refinements, declared coproduct shape, or expected interpreter representation.
>
> **Required corpus partitions**
>
> Scan all authored `.dag` files under the declared production roots. Scan tests and fixtures too, but report them in separate partitions:
>
> ```text
> Production
> Test
> Fixture
> ```
>
> No file may disappear through exclusion. Each attempted file becomes either:
>
> ```text
> Parsed
> ParseRefused { path, cause }
> ```
>
> **Required body population**
>
> Enumerate:
>
> ```text
> fn bodies
> func bodies
> service operation bodies
> data initializers that feed executable plans or function values
> every nested if/match/loop/closure body
> every match arm at occurrence identity
> every direct call and service invocation
> ```
>
> A later realizing sub-arm makes the containing arm `StaticShellRoute` even when an earlier sibling or first branch refuses.
>
> **Required shell channels**
>
> Recognize shell interpretation through:
>
> ```text
> TransportScript/direct script arguments
> sh or bash -c command arguments
> sh or bash -s standard-input programs
> wrapped forms such as sudo -S sh -s
> remote command strings with modeled shell interpretation
> materialized script files later executed
> GitHub Actions run bodies
> cron command bodies
> git-hook programs
> grammar-emitted Bash artifacts
> ```
>
> The semantic seed may identify what interprets bytes as shell. It may not enumerate current call sites.
>
> **Required value flow**
>
> Trace program values backward through:
>
> ```text
> bindings
> parameters and returns
> record fields and variant payloads
> list/fold/map constructions
> concat/join/interpolation
> renderer calls
> file write/read joins where identity is structural
> transitive helper calls
> recursive call-graph fixed points
> ```
>
> An unresolved relevant call or unjoinable file flow becomes `StaticShellRouteUnknown`, never `NoStaticShellRoute`.
>
> **Required body law**
>
> Return shape, variant name, reason-field name, comments, and prose are irrelevant.
>
> The result is decided only from transitive body behavior:
>
> ```text
> NoStaticShellRoute
> StaticShellRoute
> StaticShellRouteUnknown
> ```
>
> Explicit controls must cover:
>
> - a refusal represented as a success-valued record;
> - a refusal represented as `HostnameSetCasApplyFailed { stderr: ... }`;
> - `CodexSupervisedSessionApplyRefused` under `EmitArtifactThenThinRun`;
> - a realizer behind an earlier refusing branch;
> - a classifier returning `Bool` or a variant label with no execution.
>
> **Historical calibration—not authority**
>
> Run the finished detector against:
>
> ```text
> 4cec10f66a3d84cbde631e20ca9feaa31457d88c
> ```
>
> and publish a dated receipt reconciling:
>
> ```text
> 41 total EmitArtifactThenThinRun occurrences
>   1 declaration
>  36 production match arms across 14 modules
>   4 test occurrences
>
> 36 production arms:
>   1 StaticShellRoute
>  35 NoStaticShellRoute
>   0 StaticShellRouteUnknown
> ```
>
> This measurement must not become a permanent hard-coded count. A disagreement blocks acceptance for investigation; it does not automatically indict the detector.
>
> **Required live discoveries**
>
> Without using their names as seeds, the detector must find:
>
> - the one historical converge realizer above;
> - all four `spark_managed_access_apply` script producers flowing through `stdin_payload` to `sudo -S sh -s`;
> - every remaining `retained_*` route;
> - direct `RunArgv` routes as no-shell controls;
> - the Codex and hostname refusal disguises as no-shell arms.
>
> **Instrument controls**
>
> The same production implementation must pass:
>
> 1. **Deep-body control:** the sink sits more than ten nested nodes below the matched arm.
> 2. **Subject control:** output carries exact SHA, root set, inventory digest, and reaches a planted anchor under every intended root.
> 3. **Format-invariance control:** semantically identical, differently formatted fixtures produce equivalent facts.
> 4. **Exact-identity control:** `words` matches; `keywords` and other substring near misses do not.
> 5. **Messenger-integrity control:** structured output round-trips with row count and digest unchanged.
> 6. **Orthogonal anchor:** a direct parsed-node assertion sees one planted sink without invoking the census fold.
> 7. **Same-run dirty control:** the control partition reports its exact nonzero population in every production census invocation.
>
> A zero production count is inadmissible unless all seven hold in the same subject run.
>
> **Class 1 status**
>
> Do not modify `push_shell_argv_tokens` for the former free-monoid list/string ambiguity. #8549 already made that ambiguity a typed located refusal. Preserve it and use direct argv as a negative shell control.
>
> **Deliverables**
>
> ```text
> typed sink, route, producer, arm, evidence-grain, and refusal carriers
> corpus-derived producer
> deterministic structured facts artifact
> deterministic summary artifact
> historical 4cec10f calibration receipt
> current-head receipt
> fixture and mutation-control suite
> reconciliation predicates
> ```
>
> **Non-goals**
>
> ```text
> no shell migration
> no bridge deletion
> no route disposition table
> no runtime-value guarantee from declared types
> no CLI-AUTHORITY tool/argv census
> no srv* survival decision
> no build-cache wiring decision
> no stage0 mirror edit
> no grep-generated production population
> ```
>
> **Stop condition**
>
> If the available parsed-body facts cannot preserve exact callee identity, argument edges, branch identity, or interprocedural value flow, stop. File the missing typed projection as a separate substrate increment. Do not substitute text scanning.
>
> **Merge bar**
>
> ```text
> exact-base main green before branch
> zero production parse refusals
> zero unexplained StaticShellRouteUnknown rows
> all corpus/count reconciliation laws hold
> all instrument controls discriminate under mutation
> historical 36/1 calibration reconciled or discrepancy explained
> current-head receipt published
> facts artifact deterministic on repeat
> receipt row count and digest round-trip
> full CI green
> ```

# Trust predicate

The 0A result should expose a single predicate with no runtime overclaim:

```text
static_shell_census_trusted =
    subject_identity_holds
    && corpus_denominator_nonvacuous
    && production_parse_refusals == 0
    && unexplained_static_route_unknowns == 0
    && control_partition_matches
    && deep_body_control_holds
    && format_invariance_holds
    && exact_identity_control_holds
    && orthogonal_anchor_holds
    && messenger_integrity_holds
    && route_sink_producer_arm_reconciliation_holds
    && repeated_run_digest_equal
```

It should **not** be named:

```text
shell_execution_guaranteed
shell_runtime_population_complete
shell_migration_complete
```

The strongest honest conclusion from 0A is:

> At this exact source subject, the parsed authored corpus contains this complete, non-vacuously derived population of static paths capable of attempting shell interpretation; every row states whether its carrier shape is merely declared, statically flowed, construction-guaranteed, or runtime-observed.

---

## Corrections that produced this brief

Recorded because each one falsified an input the brief had already been written against, and a reader who finds only the final text will re-derive them.

1. **The acceptance oracle was wrong.** An earlier revision required the detector to reconcile against a manual 24-arm `EmitArtifactThenThinRun` audit. Measuring that audit in order to pin it falsified it: at `4cec10f66a3d84cbde631e20ca9feaa31457d88c` the tree carries 41 occurrences — 1 variant declaration, **36 production match arms across 14 modules**, 4 under `dag/test/claim/`. Neither of the two manual audits reached 36, and neither named `gunbc.host_effect_codex_supervised_turn`. The conclusion was unaffected — exactly one arm realizes — but had 24 been pinned as the oracle, the first *correct* detector would have reported 36 and the reconciliation step would have read as a detector bug. Hence `HistoricalCalibrationDisagrees` stops acceptance for investigation rather than indicting either side.

2. **Declared types constrain less than the brief assumed.** The argv reachability probe found 258 shell-transport operations whose argv elements resolve to five shapes, none typed `Map`, `Set`, `Record` or nullable. That is a *declared-type* result, and declared types are not construction-enforced here: 3939 `WhereRefinementUnenforced` across 612 files, and separately any coproduct in this tree can be constructed by naming the type with no variant tag, refused only at runtime by the interpreter. Hence the `EvidenceGrain` vocabulary, and hence the rule that the detector may not prune a route because a declaration says `List<String>`.

3. **A scoped repair was already closed.** The brief's original Class 1 item — the argv representation ambiguity — was closed on main before dispatch, by the same lane that was about to be assigned it. It survives only as a negative control.

4. **Prior art exists with the exact blind spot this census is built to avoid.** `v2.lens.effect_reach` classifies host-effect sinks and carries a `ShellExecRunSink` variant, but `sink_kind_for_callee` selects by name equality against a remembered callee-text list, everything else falling to `UnknownHostEffectSink`. It therefore cannot see `sh -c`, `sh -s` on stdin, or a `sudo -S sh -s` wrapper — the same hole the hand census carried before `gunbc.spark_managed_access_apply` was found. Disposition is **supersede, not extend**: extending it inherits its selection principle.

## Where the instrument controls came from

The seven controls are not theorized. Each is a failure that actually occurred while producing this brief, and they are **not one class**:

| control | the failure it encodes |
| --- | --- |
| deep-body | an instrument too narrow to contain the answer — a realizing sub-arm sat ten lines below anything the command could print |
| subject | an instrument pointed at the wrong subject — a stale head, a mirror rather than its authority |
| format-invariance | a **confident false zero**: a grep for a diagnostic string returned 0 on both refs while the author was looking at that sentence, because it wraps across a backslash continuation |
| exact-identity | a **confident false positive**: a pattern containing `words` matched inside `keywords` and returned seven irrelevant maps |
| messenger-integrity | the report of the previous item reached its recipient with the words deleted, because backticks inside a shell command were substituted away |
| orthogonal anchor | every one of the above was caught only because an observation existed that did not come through the instrument |
| same-run dirty partition | a zero is admissible only when something non-zero is proven visible in the same run |

The generalization, which is the reason a zero production count is inadmissible unless all seven hold: **when a scan returns zero, the first hypothesis is the scan.**
