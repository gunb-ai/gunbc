# The floor's semantic output: a retained ordered artifact, and the Actions log demoted to diagnostic

**Status: design note. No code lands from this document.**

Doctrine anchors: DESIGN §5 (fail-closed; no fabricated plausible output; the absorbing fallback), §4b (guarantee ladder), §3 (single authority; cite the symbol), §4c (annotations are not evidence), §6 (denominate the benefit in displaced cost).

## 1. The displaced cost, measured

`gunbc.witness_floor_workflow` emits `.github/workflows/witnesses.yml`: checkout, toolchain, build, then one step invoking `claim_executor --required-floor` over two source roots. Read on main, four properties hold and all four are load-bearing:

- **There is no artifact step.** The floor's typed, located, counted diagnostics exist only as bytes inside a foreign executor's job log. Semantic output has no durable carrier at all.
- **`CARGO_TERM_COLOR` is `always` and the build step shares the stream**, so semantic output travels one unframed byte stream with build decoration and ANSI escapes. Two channel roles fused at the byte level.
- **Concurrency cancels in progress on pull requests.** A superseding push kills a run mid-fold, leaving no partial prefix, no terminal marker and no record of which rows had folded.
- **The job timeout is 180 minutes** and the floor cut removed the per-witness eval deadline, so a hung fold also ends with whatever the executor happened to capture.

Together, **no observation distinguishes three states with opposite remedies**:

```
the floor ran to completion and refused with N diagnostics
the floor was cancelled after diagnostic k
the reader truncated the log at diagnostic k
```

That is the state-space conflation, not a presentation inconvenience. The measured symptom that surfaced it: the `gh` CLI and the API job-log endpoint return different prefixes for one long step, so the terminal refusal that decided a run can be absent from the surface a reader reaches for. **Precision, because the weaker claim is the true one:** this establishes that the retrieval surfaces share no dependable completeness contract. It does not establish that GitHub's stored log discards bytes. The conclusion is unchanged — a channel whose consumers can receive different prefixes with no completeness verdict is unfit to carry semantic authority.

## 2. What the vertical is

Not "add streaming to CI logs." Stated as the construction added rather than the debt removed:

```
typed floor outcome
  -> ordered semantic channel
  -> complete retained artifact
  -> materialization receipt
  -> verified terminal verdict
```

GitHub Actions executes and stores the first instance. It does not define the channel. This is deliberately chosen ahead of live tailing because it proves the strongest invariant first: **a complete semantic answer either arrives as a verified whole, or refuses.**

## 3. Channel roles

Roles are content of the Work's output contract, reached through the existing `OutputContractRef`. They are not a new field on `Work` and they are **not** a Demand fact: a Demand may choose a channel, a destination and a completeness requirement; it may never relabel an output from diagnostic to semantic.

```
SemanticOutput          losing it means the promised answer was not delivered
DiagnosticObservation   losing it costs inspectability, not the verdict
ProgressObservation     losing it makes a live view stale, nothing more
```

For this vertical:

```
required-floor-semantic   SemanticOutput        OrderedEmission
required-floor-console    DiagnosticObservation
```

Because role changes what a loss means, the channel roster, role and frame contract participate in output-contract identity and therefore in the work key. Concrete destination and transport do not.

## 4. Cursors: boundary offsets, scoped ordering

A cursor is the count of frames already consumed, equivalently the index of the next frame wanted — **not** the sequence number of the last frame seen. Continuity is then `retained.end_exclusive == live.start` rather than successor arithmetic, which removes an off-by-one policy from every handler and gives cursor zero a meaning (before the first frame) without a special case.

Ordering is **total within one emission log and incomparable across logs**. Comparison is three-valued — before / equal / after / incomparable-carrying-both-log-refs — for the reason `std.content_hash` `compare_content_hash` already gives for cross-family digests: two cursors from different logs are not known-different, they are incomparable, and collapsing that into a definite answer reports a comparison that never happened.

Offset ordering, range validity, containment, continuity, gap and overlap live **once** in the ordered-emission authority. Handlers consume an already-decided range; they never define cursor arithmetic. Were the cursor an opaque brand, each handler would supply its own comparison and two would eventually disagree about whether a gap exists — surfacing as a silently missing frame, which is the one property this model exists to prevent.

## 5. Production before rendering

`claim_executor` must not write text and reparse or split it. At the point where the floor holds the typed row outcome it emits an ordered semantic frame, and **independently** projects a human line to stdout. `std.observation` already carries typed process events and outcomes as distinct model states rather than glyphs; reuse that vocabulary where it preserves the real facts, and wrap the observation transition around the exact floor result carrier rather than flattening richer refusal coordinates into a prose field.

## 6. Coverage is an identity join, not a tail of failures

The artifact must prove which population was planned and what happened to every row: observed subject identity, exact claim-roster digest, every row identity, every row terminal outcome, typed refusal and failure detail, final counts, terminal run outcome. **Ordering detects delivery corruption; identity proves coverage. Neither substitutes for the other.**

## 7. The manifest is written last

An events file plus a terminal manifest, the manifest written after everything else and carrying schema, subject identity, channel identity, roster digest, the range `[0, N)`, the events blob digest, terminal outcome and counts.

Written last, each failure becomes a distinct typed refusal instead of a plausible prefix:

| Failure | Detected by |
|---|---|
| crash mid-run | no terminal manifest |
| truncated events | digest or count mismatch |
| dropped record | offset gap |
| duplicated record | repeated offset |
| reordered record | non-monotonic offset |
| wrong population | roster-digest mismatch |
| plausible prefix | no valid terminal envelope |

The verifier returns a typed refusal rather than handing a prefix to a consumer.

## 8. The artifact determines the check

Uploading a sidecar while the job result still comes from the process exit code leaves the artifact observational rather than authoritative. Capture the host termination, verify locally, upload, then derive the conclusion:

| Observation | Result |
|---|---|
| complete artifact says pass; host exit agrees | pass |
| complete artifact carries a floor refusal or failure | fail, with that semantic result |
| artifact absent or missing its terminal manifest | refuse: semantic output incomplete |
| digest, range or roster fails | refuse: semantic artifact invalid |
| artifact says pass, host exit says crash | refuse: termination mismatch |
| host exit says success, artifact incomplete | refuse — never trust the exit over the artifact |

**A cancelled run is no verdict, not a pass.** That is the `#8059` failure — a PR merged on a cancelled run, main left with 99 type errors — stated as a rule the mechanism enforces rather than a thing to remember.

And the wall that makes the retrieval defect merely inconvenient: **no gate, publisher, deduplicator, retry classifier or result reader may parse the Actions log to recover the floor verdict.**

## 9. Two implementation constraints

- **`witnesses.yml` is emitted, not authored.** Its authority is `gunbc.witness_floor_workflow`. The added verify, upload and conclusion steps are modeled there and regenerated. Hand-editing the YAML is out-of-band actuation — hand-writing an actuator is hand-applying it with a commit wrapped around it (DESIGN §5), and it is a hard reject regardless of the diff's other merits.
- **Both formats already have cited authorities, which shrinks the work.** `extdeps.github.actions` `upload_artifact_action` exists at ref v4 (its historical double-binding defect is fixed), so the artifact transport needs no new upstream modeling. `extdeps.languages.json` carries grammar, parse, emit and subject modules, so a JSON-lines artifact is emitted and read through the substrate rather than a host parser.

## 10. Owning the format does not require owning the storage

What must be ours: the semantic frame schema, sequencing, terminalization, the completeness decision, content identity and verification. The first storage and transfer binding may be GitHub's artifact service. That makes GitHub Actions a foreign executor plus a materialization provider plus a diagnostic console — and **not** the semantic-output authority. The same artifact later moves to a content-addressed store with no change to Work, Demand, cursor, frame or verification semantics.

## 11. Boundaries

**One Work, one Attempt.** No witness becomes fabric Work; no batch, worker or coordinator concept returns on this path. One preparation, one claim fold, one verdict — the current cut's central construction, preserved.

**No fabricated Grant.** Today's run does not exercise a real fabric grant, because GitHub still chooses the runner. Do not mint a pre-execution grant after the fact to claim the fencing kernel has a production consumer; the artifact format can be terminal now, and a later owned-execution receipt wraps it with a grant-bound reference.

**Live tailing is later, and additive.** It adds an output-source-ready result, a live handler, and live-from-attachment / resume-at delivery — changing no channel role, cursor ordering, artifact range, delivery requirement, work identity, frame schema or fencing rule. Late join then has three honest outcomes, never two: replay from a complete retained prefix, an explicitly requested live tail returning its exact starting cursor, or a typed refusal naming the unavailable range. A silently-partial attachment is the fabricated output; a *requested* suffix whose start is located is not.

## 12. What this note does not claim

No code lands from it. Nothing here is measured except the four workflow properties in §1 and the retrieval-surface disagreement, and the latter is a retrieval fact rather than a storage fact. No frame schema is specified. The relationship to the fabric's delivery carrier is a reference, not an implementation.
