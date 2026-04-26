# R2 Impossible-Bugs — effects normalized-view disposition (substrate side)

> **Worker disposition** for the substrate side of the R2 Impossible-Bugs
> "unenumerated effects" worker brief STOP (path (ii) confirmed). Records
> the closed-system effect authority rule, the three-surface split, and
> the dissolution trigger. **No code change carries this disposition** —
> this document is the artifact. Stamping the same disposition into
> `.dag` source comments was attempted in earlier commits on this branch
> and reverted because regen of `src/v3/compiler/src/bootstrap_generated.rs`
> SourceSpan offsets was not runnable from the worker sandbox (cargo
> unavailable). When the substrate next has uncontested host-regen
> capacity, restamping the same wording into `src/v3/std/effects.dag`,
> `dsl/std/behavioral.dag`, and `src/v3/lenses/idempotency.dag` headers
> is straightforward and span-only; this doc is the canonical citable
> source until then.

## Reads first

- Substrate inbox escalation **#856** (forwarded from R2 Impossible-Bugs
  **#857**)
- Design doc **PR #808**
  (`docs/briefs/t-impossiblebugs-unenumerated-effects-design.md`)
- Worker brief STOP path (ii)
  (`docs/briefs/r2-impossible-bugs-unenumerated-effects-worker.md`)
- T-Modeling reply on #856 (authority rule below)
- #856 supplementary audit (three-surface split below)

## Authority rule (per Modeling reply on #856)

**Taxonomy naming is not the issue; authority is.** A named effect
taxonomy is acceptable **only** as a mechanically derived normalized
view over structural signature facts. If a primitive must *declare* the
tag because signature shape cannot derive it, that tag is **parallel
authority** and must retire — or become a structural-coverage-gap
diagnostic — once the signature-shape lens lands.

## Three surfaces (per #856 supplementary audit)

### Surface 1 — Positive signature-shape candidates

Typed primitives whose signatures already meet the closed-system rule:
reads return derived values without modifying receiver/resource;
updates return modified receiver/resource values. The future
signature-shape lens reads these structurally without help.

- `dsl/std/algebra.dag:560-588` — partial-function `get` / `map_get` /
  `lookup` return `Optional<Value>` without modifying ReceiverSelf;
  `map_insert` / `map_merge` / `with` return ReceiverSelf. List ops at
  560-570 follow the same pattern (functional updates).

**Disposition.** No comment change required. These are the lens'
foundation when it lands. Listed so reviewers can see the positive
surface explicitly.

### Surface 2 — Retained normalized transport projection (compatibility bridge)

Mechanically derived normalized view over HTTP method+path transport
facts. **Compatibility bridge territory, not authority.**

**View** (in `src/v3/std/effects.dag`): `OperationEffect`,
`EffectShape`, `IdempotentShape`, `BreakingShape`, `WorkflowEffect`,
`KeySource`, `derive_op_effect`, `derive_effect_shape`,
`compose_effects`, `lane2_workflow_idempotency_report`.

**Consumers (today, on the normalized projection):**

- `src/v3/lenses/idempotency.dag` — Lane 2 Stage 2b dispatch
- `src/v3/lenses/parallelism.dag` — Lane 2 Stage 2e dispatch
- `src/v3/compiler/src/workflow_idempotency.rs` — native Rust mirror
- `src/v3/compiler/src/workflow_parallelism.rs` — native Rust mirror
- Tests: `lane2_stage_2a_effects_smoke`,
  `lane2_stage_2b_db18_test`, `lane2_stage_2e_parallelism_test`

**Why they remain on normalized projection for now.** The
signature-shape effects lens (#808 Slice §1, target path
`src/v3/lenses/effect_enumeration.dag`) does not exist yet; the
re-anchor is its own dedicated worker scope. Same-PR re-anchor would
be too large per the dispatch's bridge allowance. Idempotency /
parallelism / mirrors / tests must keep dispatching against
`WorkflowEffect` / `OperationEffect` until that lens lands and
publishes a structural-fact equivalent.

**No-extension rule.** Coverage gaps (Surface 3 final row) must surface
as **structural-coverage-gap diagnostics** from the future
signature-shape lens — they must NOT be absorbed by extending
`EffectShape` / `IdempotentShape` / `BreakingShape` / `WorkflowEffect`
variants here.

**Dissolution trigger (Surface 2).** Signature-shape effects lens lands
at `src/v3/lenses/effect_enumeration.dag`; the four consumers above
re-anchor onto its structural fact. At that point this view either
retires entirely (if its consumers no longer need method+path-anchored
normalization) or persists narrowly as a projection helper — that
decision belongs to the lens' worker, not this disposition.

### Surface 3 — Declared taxonomy / modifier retirement or gap evidence

Hand-declared parallel authority that **fails the authority rule**
because signature shape cannot derive it today.

| Surface | Where | Today's status |
|---|---|---|
| `side_effects: SideEffects` (`ReadOnly\|WritesState\|WritesExternal`) | `dsl/std/behavioral.dag:9, 87` | Hand-declared, no derivation, **no current downstream consumer reading as derived projection** |
| `idempotent` / `readonly` operation modifiers | extdeps (`gcp/iam.dag:89-126`, `gcp/secret_manager.dag:80-133`, `git.dag:83-116`, etc.) | Declared on operations; cross-checked by `check_modifier_vs_derivation` against the Surface 2 view, not against signature shape |
| Non-REST / resource-contract operations | extdeps + interface contracts | Effect cannot be method+path derived; declared modifier carries the fact |
| JSON / path-extraction extdeps | `dsl/extdeps/llm/openai.dag:92-110`, `dsl/extdeps/llm/anthropic.dag:104-124`, `dsl/extdeps/github/auth.dag:13-24` | `messages: Json` body / discarded modeled facts → signature-shape coverage NOT true today; **path (ii) existence-proof** |

**Classification.** `side_effects` / `SideEffects` is **declared
parallel authority retained only temporarily** — extdeps contract
metadata pending the signature-shape lens. Reviewers/consumers must
NOT read it as effect-proof under any circumstance.

**Cross-surface retirement.** `side_effects` is NOT a local-schema
retirement: the `idempotent` / `readonly` modifiers it pairs with are
declared in extdeps service operations and cross-checked by
`src/v3/std/effects.dag::check_modifier_vs_derivation` against the
Surface 2 method+path normalization. All three retire together
cross-surface at the lens trigger; no fact stored locally in
`behavioral.dag` retires by itself.

**Dissolution trigger (Surface 3).** When the signature-shape lens
lands AND extdeps primitives are reshaped onto resource-threading
discipline (returned-modified-resource): `side_effects` retires;
extdeps `idempotent` / `readonly` declarations retire; the
`check_modifier_vs_derivation` cross-check retires. The
JSON/path-extraction extdeps either become derivable (their primitives
are reshaped to expose modeled facts in their return shape) or surface
as **structural-coverage-gap diagnostics** from the lens — never
absorbed into a new variant on any taxonomy here.

## Path (ii) receipt

Per Surface 3 final row:

- `dsl/extdeps/llm/openai.dag:92-110` — ChatCompletion
  (`messages: Json`, JSON-path output extraction). Method+path
  normalization cannot derive its effect class; signature-shape lens
  has no foundation to read the modeled fact today.
- `dsl/extdeps/llm/anthropic.dag:104-124` — Messages, same JSON-bypass.
- `dsl/extdeps/github/auth.dag:13-24` — `github_token` returns
  `{ token: Secret }`, discards modeled scopes/expires_at.

**Closed in this PR?** No. Closing them is the separate
extdeps-typed-primitive-consumption lane (**#867** scope, intentionally
not absorbed here per manager review of #893).

## No new hand-authored taxonomy

Confirmed: no new variants on `EffectShape` / `IdempotentShape` /
`BreakingShape` / `SideEffects`. No new functions. No new types.

## Out of scope (intentionally)

- Implementing the signature-shape effects lens (#808 Slice §1).
- Re-anchoring the Surface 2 consumers onto the future lens.
- Closing the Surface 3 P1 bypasses (extdeps typed-primitive-consumption
  lane / **#867** territory).
- Retiring the Surface 3 modifier-falsification path or the extdeps
  modifier declarations.

The paused worker (`valiant-lynx-650`) is **not** unpaused by this
disposition; the signature-shape lens is the unblock and that is a
separate dispatch.

## Why this is a doc-only PR (DB-8 / regen note)

Earlier commits on `session/zesty-moth-157` (`4d0e5bfc2`,
`b1419b8f6`, `9f72e51ad`) stamped this disposition into the headers of
`src/v3/std/effects.dag`, `dsl/std/behavioral.dag`, and
`src/v3/lenses/idempotency.dag`. Comment-only edits, no structural
change. They tripped two CI gates:

1. **SG-2 parse-corpus snapshot** — refreshable manually (the
   per-file row is a single line in
   `src/v3/compiler/tests/integration/parse_corpus_manifest.txt`); fix
   landed in `c7e261fa5` and was reverted along with the comment edits.
2. **`regen_bootstrap --verify`** — `bootstrap_generated.rs` records
   per-`Declaration` `SourceSpan` byte ranges, and effects.dag has 114
   such spans in that file. The comment additions shifted those spans
   non-uniformly (multiple insertion points → different deltas per
   declaration). Regen requires `cargo`, which was not available in
   the worker sandbox; manually patching 114 spans without a runnable
   verifier is too risky.

The disposition itself is unchanged. It now lives in this brief
(span-immune, fully reviewable) instead of in `.dag` headers. When
host-regen capacity exists, restamping the same wording into the
`.dag` headers is mechanical and span-only; the brief stays as the
canonical reference.
