# B4 — Identity-Carrier Substrate Pass `(M; Tier 1; primary recommendation)`

> **Program brief.** Director (`zesty-bear-812`) coordinates; sub-briefs
> dispatched as workers. Frames the §0 class from
> [`docs/briefs/debt-paydown-synthesis-2026-04-25.md`](debt-paydown-synthesis-2026-04-25.md)
> as **one M-scope substrate program**, not eight item-by-item paydowns.

## Read first

- **[`docs/briefs/debt-paydown-synthesis-2026-04-25.md`](debt-paydown-synthesis-2026-04-25.md) §0** — the headline finding: dominant scaffold pattern in 2026-04-25 analyses is a single class of identity-bridge sentinels with a shared upstream root cause.
- **`feedback_groundedness_gates_lenses` (revised 2026-04-25)** — *"the language has no vocabulary other than primitives + composition/namespacing; 'leaving the stack' means writing a different compiler, not an in-language feature; lenses apply to every program because there's no way to author one outside the kernel."* Load-bearing for this brief's framing.
- **`feedback_compiler_is_dag_processor`** — compiler knows only `Node / Conj / Disj / Cardinality / Bit`.
- **`feedback_lenses_not_passes`** — analyses are lenses over physics; zero heuristics.
- **`feedback_construction_over_ratchets`** — model first; violations dissolve.
- **`feedback_no_metadata_markers`** — no `__is_X` string markers; model concepts structurally in `std/`.
- **[`THESIS.md`](../../THESIS.md)** + **[`MODELING.md`](../../MODELING.md)** + **[`INVARIANTS.md`](../../INVARIANTS.md)**.

## Frame — groundedness gates lenses (revised)

The language vocabulary is **primitives + namespacing/composition only**:
- 5 behaviors (`Value | Transform | Branch | Loop | Bind`)
- typed substrate carriers
- 4 type connectives (`Conjunction | Disjunction | Cardinality | Bit`)

There is no user-defined-primitive feature, no escape syntax, no annotation the compiler can't see through. **Consequence: there is no "ungrounded user program" category.** A user "leaving the stack" inside the language means composing primitives into named patterns (namespacing) — the compiler sees through; lenses apply for free. "Leaving the stack" outside the language means literally writing a different compiler — not our concern until they ground their outputs back into our primitives by emitting `.dag`.

**The lens contract is: applies to every program by construction.** If a lens needs an ungrounded-output path for user programs, the design has a leak — there's no syntax for a user to author an ungrounded program.

### Sharpened diagnosis of the §0 class

The eight surface sentinels — `PROGRAM_INPUT_SENTINEL`, fixture-filename routing, `span.file ==` checks, `include_str!` lens side-channels, file-preference rank, `bind.span.file` emission special cases, `EXTDEPS_BOOTSTRAP_FIXTURES`, the `lens_apply.rs` `algebra.dag` fold-skip — are **NOT "ungrounded fallbacks the compiler labels"** (earlier framing). They are:

> **The compiler itself failing to use the language's primitives + namespacing internally.**

The compiler reaches for sentinel strings instead of structural carriers because the substrate doesn't yet carry the structural identity end-to-end through lowering. The fix is to **make the compiler hold itself to the language's own vocabulary**: where today it asks `bind.span.file == "match_emit_helper.v3"`, it should ask a structural question against a typed carrier (`DeclarationRef`, structural template-formal edge, explicit input-value carrier, structural emit-helper carrier).

This is the **highest-leverage paydown in the entire 2026-04-25 inventory** because eight item-by-item PRs will fight the same upstream every quarter; one substrate pass dissolves the class.

## Eight surface dissolution sites

| # | Site | File:line | Today | Carrier needed |
|---|------|-----------|-------|----------------|
| §0.1 | `PROGRAM_INPUT_SENTINEL` (5 sites) | `test_runner.rs:1594, 1617, 1642, 1709, 1855` | sentinel-string `"r1_lens_output_input_from_program"` | `DeclarationRef` for the program-input-binding role |
| §0.2 | Fixture-filename → bind-name routing | `test_runner.rs:47-48` | filename string lookup | `DeclarationRef` (same carrier as §0.1) |
| §0.3 | `include_str!` canonical lens side-channel | `test_runner.rs:23, :33` | `include_str!("../lenses/named_function_count.dag")` | `DeclarationRef` to the canonical lens declaration |
| §0.4 | `span.file ends_with "std/algebra.dag"` fold-skip | `lens_apply.rs:38, :372-383` | path-suffix string check | structural fold-shape carrier |
| §0.5 | `span.file == "dsl/std/types.dag"` type-alias bridge | `lower.rs:836` | path-equality string check | structural type-alias carrier |
| §0.6 | `bind.span.file == "named_alias_emit_helper.v3"` / `"match_emit_helper.v3"` | `emit.rs:3181, 3206` | path-equality string check | structural emit-helper carrier (typed role marker on the binding) |
| §0.7 | `declaration_name_preference_rank(&span.file)` | `dag.rs:2735-2764` + `lower.rs:1451-1452, 1546-1547` | file-suffix preference table | structural declaration-source carrier |
| §0.8 | `EXTDEPS_BOOTSTRAP_FIXTURES` manual fixture list | `bootstrap.rs::std_fixtures()` | hardcoded list of fixture paths | structural extdeps-fixture-set carrier |

Each site has its own dissolution comment today. **What's been missing is a single substrate pass that lands the carriers; the eight sites then dissolve as consumers of the new substrate.**

## Program shape — substrate-first, sites-after

This program is **not** "go fix eight files." That ordering would patch the symptoms while the substrate stays absent, leaking new sentinel-string sites in the next quarter.

### Phase 1 — substrate carriers (M-scope, sequential)

Land the typed carriers into `src/v3/std/` and lower them end-to-end:

1. **`DeclarationRef`** — typed reference to a declaration by structural identity, not by name-string. Consumers: §0.1, §0.2, §0.3, §0.5, §0.7. Likely the largest sub-deliverable; substrate-port + lowering wiring + consumer migration.
2. **Structural fold-shape carrier** (template-formal edge) — typed carrier on fold-template instantiation that records which formal binds the step. Consumers: §0.4 + B3's underlying invariant.
3. **Structural emit-helper carrier** — typed role marker on `Bind` / `Branch` nodes that participate in match/named-alias emission, attached at lowering time, not inferred from `span.file`. Consumers: §0.6.
4. **Structural extdeps-fixture-set carrier** — typed extdeps-bootstrap-set declaration, consumed by `bootstrap.rs::std_fixtures()`. Consumers: §0.8.

Each of these is a sub-brief. Author and dispatch sequentially or in parallel where dependencies allow (DeclarationRef likely first; emit-helper carrier likely second since it's a smaller targeted shape; fold-shape carrier independent; fixture-set carrier independent of the other three).

### Phase 2 — site dissolution (S-scope each, parallel after Phase 1)

Once each Phase 1 carrier lands, the corresponding sites dissolve mechanically:
- §0.1-§0.3, §0.5, §0.7 dissolve with `DeclarationRef`.
- §0.4 dissolves with the fold-shape carrier.
- §0.6 dissolves with the emit-helper carrier.
- §0.8 dissolves with the fixture-set carrier.

These are mechanical follow-up PRs once substrate is in place; not the substantive work.

### Phase 3 — discipline ratchet (one-time, after Phase 1)

Add a per-PR gate (and CI lint, if structural): **no new `span.file ==` / `span.file.ends_with` / sentinel-string checks in `src/v3/compiler/src/`.** The `feedback_no_textual_enforcement_bridges` discipline says no grep enforcement of "be structural"; this gate is reviewer-side discipline + the PR-template-line addition from synthesis §5.4, not an automated regex check. The structural prevention is that the carriers exist and the right answer is structurally available; reviewers just need to refuse the sentinel-string path.

## Acceptance — program-level

- [ ] All four Phase 1 carriers land in `src/v3/std/` with substrate-port + lowering + consumer migration.
- [ ] All eight Phase 2 sites dissolve; sentinel strings deleted.
- [ ] Phase 3 reviewer-discipline addition lands (PR-template line, brief checklist line).
- [ ] DB-8 fixed-point converges bit-identically across every Phase 1 + Phase 2 PR.
- [ ] No new `span.file == "..."` / `span.file.ends_with("...")` / sentinel-string sites introduced anywhere in `src/v3/compiler/src/`.
- [ ] Lens contract holds: every program (well-formed + structurally complete) has lens output without sentinel-string fabrication.

## Sub-brief dispatch order

Director authors and dispatches per Phase 1 ordering:
1. **B4.1 — `DeclarationRef` carrier** (largest; most consumers; gate for §0.1, §0.2, §0.3, §0.5, §0.7).
2. **B4.2 — Structural fold-shape carrier** (independent; can run in parallel with B4.1).
3. **B4.3 — Structural emit-helper carrier** (independent; can run in parallel with B4.1).
4. **B4.4 — Structural extdeps-fixture-set carrier** (independent; can run in parallel with B4.1).
5. **B4.5–B4.12 — Eight Phase 2 site-dissolution PRs** (mechanical; dispatch as each Phase 1 carrier lands).

Each sub-brief gets the substrate-principle audit per `feedback_substrate_principle_audit` before authoring. Each sub-brief lands SG-0 census deltas + REGEN_OUTPUTS partition updates + DB-8 fixed-point check.

## STOP-AND-ESCALATE — program-level

- **A Phase 1 carrier proves to require substrate work outside this program's scope** (e.g., DeclarationRef requires resolving an InternTable + node-name interaction beyond the carrier itself) — STOP. Surface for re-scoping.
- **A Phase 2 site refuses to dissolve cleanly with the landed carrier** (e.g., consumer needs information the carrier doesn't yet provide) — STOP. Indicates the carrier was under-specified; revise Phase 1 sub-brief.
- **A new sentinel-string site is introduced during Phase 1 work** (e.g., a workaround in the substrate landing PR itself) — STOP. The discipline is being violated by the dissolution work; surface for design call.
- **DB-8 drifts on any sub-PR** — STOP immediately.
- **A site turns out NOT to belong to the §0 class** (e.g., a sentinel that's actually correct boundary contract per `feedback_fail_closed_is_boundary`) — STOP. Re-classify; remove from Phase 2 list.

## Non-goals

- Not addressing P3 fail-closed leaks (B1, B2, B3 — Tier 0).
- Not addressing host-Rust mirrors of std `.dag` carriers (Tier 3 #10 — Zero-Floor + R2 Grounding territory).
- Not addressing lossy lens reflection (Tier 3 #8 — adjacent but distinct; defer behind this program).
- Not extending lens taxonomy.
- Not adding analyzer-local heuristics or whitelists (per `feedback_construction_over_ratchets`).

## Cross-manager note

- **Zero-Floor Manager (`stern-swift-335`)** — heads-up. Phase 1 carriers touch substrate-adjacent territory; coordinate at sub-brief authoring (especially B4.1 DeclarationRef interaction with InternTable / node-name work).
- **Grounding Manager (`crisp-seal-366`)** — heads-up. §0.7 file-preference rank and §0.8 extdeps-fixture-set may touch Grounding-adjacent territory; coordinate if needed.
- **PM** — synthesis framing (one program, not eight paydowns) is the load-bearing reframe; primary recommendation per `debt-paydown-synthesis-2026-04-25.md` §0.

## Reporting

Program-level brief; no single PR. Sub-briefs (B4.1 – B4.12) report individually as authored.

On Phase 1 close: Director signals PM + cross-managers; Phase 2 dispatch begins.
On program close: SG-0 census reflects the dissolution; ROADMAP debt rows for the §0 class retire.
