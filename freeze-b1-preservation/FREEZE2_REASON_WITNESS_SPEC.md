# FREEZE-2 SPEC INPUT — reason-bearing witnesses for the rtp acceptance set (claims-only, zero Rust change)

Written 2026-11-07 by sleek-seal-226, pre-run (no freeze-2 output informed it). Companion to
PRE_REG_ROW_15695_freeze2_rev3.md (sha c89ad90e...), obligation 2. Scope note: this touches a CLAIMS FILE, so
it changes the seed and therefore requires a NEW frozen packet through the normal review chain. No retarget of
a spent packet, no auto-apply. Owner of the go/no-go: tidy-ant-251 (packet) with keen-otter (bytes).

## The measured problem (from the two v17 logs only; shas 5d485d14... / 515596e7...)
- Entry resolution is NOT the failure: `claims-rtp.log:27` `[resolve] ...return_tail_position.dag: 301070ms
  (245 modules, 11489 resolved items in closure)`; `:29` `running 17 witness(es)`; every witness has its own
  `subject=` hash. Same shape in ce (`245 modules, 10032 resolved items`, 13 witnesses).
- The witness output cannot carry a reason: a FAIL is one bare line `FAIL <name>` plus
  `[witness] <name>: cpu=... eval_steps=32146xx subject=<hash> eval_self=0.000ms`. `eval_self=0.000ms` for all
  30 witnesses; per-witness `eval_steps` vary only in the hundreds out of 3,214,6xx (the measured work is the
  shared fill, not the claim body); the single reason-bearing line in the log,
  `[floor-reach-probe] ... reach=surface_unarmed ... last_builtin=rtp_refuses_as`, is emitted ONCE, not per
  witness. No `expected`/`actual` is ever printed.
- Therefore 11 bare FAILs cannot distinguish `RtpFileRefused { fatal_reason }` from `RtpRefused { reason, at }`
  from `RtpAbsent`. A FAIL of that shape is NO-OBSERVATION-class evidence, not support for any hypothesis.

## The verdict already carries the reason; only the witness throws it away
`src/v2/test/claim/namespace_xl0/return_tail_position.dag` (seed 495da688), `rtp_verdict_of` :192-206:
`NativeCensusModuleFileRefused => RtpFileRefused { fatal_reason: rtp_file_refusal_reason(...) }`,
resolve-refusal => `RtpRefused { reason: d.head.reason, at_atom: ... }`, and `rtp_verdict_in` :208 returns
`RtpAbsent` when the module is not in the census. The acceptance claims then collapse all of that to a Bool
(`rtp_is_resolved(v: v)`, e.g. `a_tail_return_of_a_fn_body_is_accepted`), which is why nothing is legible.

## Existing idiom to copy (no new machinery, same claims tree)
`src/v2/test/claim/unary_operator_arm.dag:54-63`:

```
fn uoa_reason(src: String, subject: List<Symbol>) -> String {
  match uoa_assemble(src: src, subject: subject) {
    Rejected { diagnostics: d } => concat("assemble:", symbol_lexeme(sym: diagnostics_fatal_reason(d: d)))
    Accepted { value: resolved, diagnostics: _ } =>
      match infer(tree: resolved) {
        Accepted { value: _, diagnostics: _ } => "ACCEPTED"
        Rejected { diagnostics: d } => concat("infer:", symbol_lexeme(sym: diagnostics_fatal_reason(d: d)))
      }
  }
}
```
and claims of the form `uoa_reason(src: ..., subject: ...) == "ACCEPTED"` / `== "infer:<reason>"`
(:73, :85, :89, :93). String-typed claims already work under `claim_batch`, and reason-spelling comparison is
the established way these claims discriminate.

## Proposed shape (freeze-2 seed change, claims-only)
Add one helper to the rtp claims file, mirroring `uoa_reason` over the existing verdict type:

```
fn rtp_reason_spelling(context: NativeTestContext, module_name: String, unit: Symbol) -> String {
  match rtp_verdict_in(context: context, module_name: module_name, unit: unit) {
    RtpResolved                        => "RESOLVED"
    RtpAbsent                          => "ABSENT"
    RtpFileRefused { fatal_reason: fr } => concat("file_refused:", match fr {
      optional_present(value: s) => symbol_lexeme(sym: s)
      optional_absent()          => "<none>"
    })
    RtpRefused { reason: r, at_atom: _ } => concat("resolve_refused:", symbol_lexeme(sym: r))
  }
}
```

Then, for the CONTROL fixture only (`rtp_tail_ctl_source`, `fn rtp_probe(i: Int) -> Int { return i }` —
grammar-neutral, so the #15695 fix cannot reach it), add a MUTUALLY EXCLUSIVE SPELLING PROBE SET, one claim per
reachable spelling:

| claim | asserts |
|---|---|
| `rtp_ctl_spelling_is_resolved` | `rtp_reason_spelling(...) == "RESOLVED"` |
| `rtp_ctl_spelling_is_absent` | `... == "ABSENT"` |
| `rtp_ctl_spelling_is_file_refused` | `... == "file_refused:<literal>"` (the exact lexeme frozen pre-run) |
| `rtp_ctl_spelling_is_resolve_refused` | `... == "resolve_refused:<literal>"` |

Law, stated and pre-registered BEFORE the run: at most one of the four can pass (the spellings are
pairwise-distinct literals), and the acceptance claim `a_tail_return_of_a_fn_body_is_accepted` must pass with
`rtp_ctl_spelling_is_resolved`. That gives freeze-2 three outcomes instead of two: PASS/PASS (candidate P),
FAIL + `resolve_refused:<r>` (dead rung, with its reason named), FAIL + `ABSENT` (the fixture never entered the
census — a third, distinct cause). All four predictions are written before any run; the exact `<literal>`
strings are frozen with the packet; a flip outside the four is a FINDING reported verbatim, never re-expacted.

## The seed base is a commit sha, not a branch (binding; tidy-ant-251 msg_c94b95b6, 2026-11-07)

**The freeze-2 seed is cut from a base named as a COMMIT SHA, never a branch name.** The base is the `#15999`
repair head merged with the then-green main head — exactly how the v17 seed `495da688` was built (repair + main
merge). At the time of writing the repair head is `a4a21da46284e64a31efff08585c42aaf589089a`, and main is
`7d062bda1b6dab67bc72e0c49d0eb635c45ca7f9` with its `v2-bootstrap` job still in progress, so the base sha is
**fixed and recorded at cut time in the packet**, not here.

**Plain main is not a valid base, and that is measured, not inferred.** Running
`prerun_checks_freeze2.sh 7d062bda1b6dab67bc72e0c49d0eb635c45ca7f9` gives `STATUS: FAIL`: `git apply --check` of
the registered re-anchor (`2852848f…`) refuses, the rtp claims file declares **17** `test fn` instead of 21, and
all four probe names are absent. The cause is that the probe diff's context is the repair's own edit of the same
import block. The failure is silent in the dangerous direction: a main-only seed still prints 17 well-formed
witnesses and only the four probe lines are missing — precisely the state law (1) must be able to distinguish from
"the probe ran and said `ABSENT`".

**Required attachment for any grant:** the receipt printed by

```
bash /tmp/packet/prerun_checks_freeze2.sh <BASE_SHA>
```

run at that exact base sha, hash-pinned to the registered artifact `2852848f…`, printing `STATUS: PASS` together
with the receipt's own sha256. A missing receipt, or any status other than `PASS`, means **no grant may be
requested**. The receipt covers: the diff sha; `git apply --check` with **no fuzz**; the applied seed's
`grep -c '^test fn '` == **21** with every probe name the artifact declares actually present; the applied probe
block digesting to `af6f1747…` under the same canonical function that froze that value; and `claim_batch.rs`
still containing `total_witnesses`, `for function in &group.functions` and `validate_explicit_functions` — the
source facts that make the 21-name grouped executor form load-bearing rather than cosmetic.

**Pointer note, adding nothing to any line above:** the companion pre-reg is now **REVISION 4**, sha
`b77f206d…`, which supersedes the rev3 `c89ad90e…` named in the header. rev4 keeps rev3's 30 rows byte-identical
and adds the four probe rows plus the 21-name grouped form. Any change to a line above this section, other than
the addition of this section, is a content change and comes back for contract review first.

## Cost and risk, honestly
- Cost is one helper + 4 claims in ONE claims file, reusing `symbol_lexeme`/`concat`/`diagnostics_fatal_reason`
  idioms already in the tree. No Rust, no workflow, no lane change.
- It does NOT weaken any existing expectation: the 17 rtp + 13 ce rows in rev3 stay as they are; the probes are
  ADDITIVE witnesses.
- It requires a NEW seed and therefore a NEW packet — which is already the plan (v17 is spent), so this adds no
  new packet beyond what freeze-2 needs anyway.
- If the parent decides freeze-2 must not touch claims files, then freeze-2's FAILs must be recorded as
  NO-OBSERVATION-class, and the discriminator cannot be settled by this packet either. That is a decision for
  tidy-ant-251, not me; I state it so the choice is explicit rather than silent.
