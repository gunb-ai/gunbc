---
status: decomposition (Mgr-tier scope-breakdown; not a worker brief)
authority parent: R3 Substrate Manager (warm-wolf-698 lane)
authoring session: bold-heron-632 (adhoc-e48c09a4-8d9)
date: 2026-05-12
gate: §1.8 row #60 `substrate_gap_parser_grammar_closed`
ratification anchor: Q-MachineConstraint-Carrier RATIFIED at gunbc#828 #issuecomment-4385530115
note: PB Mgr role unfilled at authoring time — PB-specific slices (E, F) require PM resourcing decision before dispatch
---

# Gate #60 `substrate_gap_parser_grammar_closed` — scope decomposition

## Purpose

This document decomposes gate-60 into named, dispatchable work-slices.
It is **not** a worker brief; each slice points to (or names) the brief
that would author the change.

The gate's Pass condition is **concept faithfully modeled** (per Q-MC
sub-decision 5, Brian directive 2026-05-06): substrate carries algebra
and machine-constraint as independent composing axes with `.dag`-generated
interaction substrate; the 3-pair set `Int<64>` / `Real<64>` / `Nat<8>`
lowering through stage0 without v2-fallback is **minimum existence-proof
evidence**, not the closure target. §1.4's conjunctive form still binds
(representative gap-test executes AND class-bridge enumeration = 0).

## Substrate state at decomposition

What is already on `main`:

| Substrate fact | Location | PR |
|---|---|---|
| `MachineWidth<bits>` carrier (phantom unary) | `dsl/std/machine_constraints.dag:45` | #1856 |
| `Compose<Algebra, MachineConstraint> = Phantom` | `dsl/std/machine_constraints.dag:112` | #1856 |
| `Int = AbelianGroup<GroupCompletion<Nat>>` algebraic concept | `dsl/std/integer.dag:147` | #1466 |
| `UInt = Nat` canonical-instance form | `dsl/std/integer.dag:148` | #1818 |
| `Real = ApproximateField<FieldOfFractions<Int>>` | `dsl/std/float.dag` | #2397 / #2409 |
| `IntW32/64/128`, `UIntW32/64/128` (substrate aliases) | `dsl/std/integer.dag:46-56` | #2161 |
| `Real32/Real64` = `Compose<Real, MachineWidth<Word32\|Word64>>` | `dsl/std/float.dag` | #2570 |
| Width carriers `Word16/Word32/Word64` | `dsl/std/bit.dag:29-31` | (pre-R3) |
| Grounding G2 primitive-row consumer evidence (`i32/i64/u64/f32/f64`) | `dsl/extdeps/languages/rust/primitives.dag` | #2570 |

What is **not** on `main` (load-bearing for gate-60):

1. `Word8` width carrier (`Nat<8>` evidence-pair requires it; `bit.dag`
   defines `Byte` but not `Word8`).
2. User-surface `Int<N>` / `Real<N>` / `Nat<N>` parser syntax that
   desugars to `Compose<Algebra, MachineWidth<WordN>>`. Today, source
   must write the substrate spelling (or use the `IntW32`-style
   alias); parser-grammar surface is still substrate-bare.
3. End-to-end demonstration that the user-surface 3-pair set lowers
   through stage0 without invoking v2-fallback for type-resolution.
4. v2-oracle parity on the same source program (Class 1 5-criteria
   final row per `docs/briefs/r3-substrate-s3-machine-constraint-carrier-worker.md:32-41`).
5. Class-bridge enumeration receipt = 0 (§1.4 conjunctive rule).

## Decomposition

The five outstanding facts above map onto six slices. Slices A-D are
substrate-lane (Substrate Mgr authority); E-F are demonstration/parity
and route through T-Numeric-Construction + PB lanes.

### Slice A — `Word8` width-carrier slice

**Scope.** Add `type Word8 { bytes: List<Byte> }` to `dsl/std/bit.dag`
(or equivalent unary-byte form per existing record convention). No
parser change.

**Why load-bearing.** `Nat<8>` evidence-pair requires
`MachineWidth<Word8>` as slot-2 of `Compose<Nat, MachineWidth<W>>`;
`bit.dag` currently starts at `Word16`. Without `Word8`, the minimum
existence-proof set cannot be spelled even in substrate form.

**Brief author.** Substrate Mgr (small enough to absorb into a worker
brief alongside the Slice B substrate aliases below; combined PR is
fine).

**Owner role.** Substrate worker (no PB-tier dependency).

**Gate signal.** Adds a unit-shape carrier; no §1.8 receipt of its own.
Feeds B/C.

### Slice B — Substrate aliases `Nat8` / `Real8`-pair completion

**Scope.** Author `type Nat8 = Compose<Nat, MachineWidth<Word8>>` (and
the corresponding `UInt8` alias if not already covered by
`UIntW8`-shaped form) under `dsl/std/integer.dag`. No parser change.

**Why load-bearing.** Completes the substrate-side evidence triple
(`IntW64` / `Real64` / `Nat8`) used by Slice E for the minimum
existence-proof demonstration.

**Brief author.** Substrate Mgr; bundle with Slice A.

**Owner role.** Substrate worker.

**Gate signal.** Updates §1.8 row #18 receipt detail (no new gate).

### Slice C — User-surface parser desugaring `T<N>` → `Compose<T, MachineWidth<WordN>>`

**Scope.** Implement the parser-grammar surface authored at
`docs/briefs/r3-substrate-s3-phase-2-parser-grammar-worker.md`:
user writes `Int<64>`, parser elaborates parametrically to
`Compose<Int, MachineWidth<Word64>>`. Numeric literal `N` in the
slot-2 position desugars to the matching `Word<N>` carrier from
`dsl/std/bit.dag`.

**Why load-bearing.** This is the *parser-grammar* in the gate name.
Substrate already carries the shape (Slice A/B); without parser
desugaring, gate-60's "parser handles generic `Compose<...>`
interaction syntax" predicate cannot be discharged.

**Open design knob (do not silently pre-decide).** Whether numeric
literal `64` in slot-2 desugars only when slot-1 is a known
algebraic-concept carrier (`Int`/`Nat`/`Real`/`UInt`), or whether
it desugars universally. Brief at
`docs/briefs/r3-substrate-s3-phase-2-parser-grammar-worker.md`
preserves this as a clarifying question for Substrate Mgr at
dispatch.

**Brief author.** Already authored; status = draft, worker pin
valiant-ant-72 (freed-pool post #1856).

**Owner role.** Substrate worker. Touches v3-compiler parser
(`src/v3/compiler/src/parse_generated.rs`) — operator BLOCKING risk
on `SELF_HOSTING.md` authority-audit per
`feedback_self_hosting_md_authority_audit_before_naming.md`; brief
must include the grep-before-name discipline.

**Gate signal.** Discharges criterion 2 of Class 1 5-criteria
Pass (parser handles generic interaction syntax) — see
`docs/briefs/r3-substrate-s3-machine-constraint-carrier-worker.md:32-41`.

### Slice D — Class-bridge enumeration zero receipt

**Scope.** Author or extend the existing class-bridge enumeration
test/audit so that gate-60's class produces `0` outstanding bridges
once Slice C lands. Receipt format follows the
`*_residual_census_receipt` naming convention per
`feedback_substrate_plumbing_receipt_naming.md`.

**Why load-bearing.** §1.4 conjunctive rule (`r3-program-plan.md` §1.4):
"concept faithfully modeled" replaces pair-counting but does NOT
relax the conjunctive form — representative gap-test executes AND
class-bridge enumeration = 0. This is the second conjunct.

**Brief author.** Substrate Mgr (small follow-on to Slice C).

**Owner role.** Verification-tier worker (read-only audit + ratchet
predicate).

**Gate signal.** Directly required by §1.4; receipt cited in gate
close PR.

### Slice E — Minimum existence-proof demonstration (`Int<64>` / `Real<64>` / `Nat<8>` lower without v2-fallback)

**Scope.** Author a substrate integration test (model:
`m1_substrate_test` / `m2_substrate_inhabitance_test` family) that:
1. Parses source containing the three user-surface spellings.
2. Verifies elaboration to `Compose<…, MachineWidth<Word…>>`.
3. Lowers through stage0 to target Rust `i64` / `f64` / `u8` rows
   from `dsl/extdeps/languages/rust/primitives.dag`.
4. Confirms NO v2-refinement-syntax path is taken (per
   T-V2-Retirement supersession captured in
   `docs/briefs/r3-substrate-s3-phase-2-parser-grammar-worker.md:18-33`).

**Why load-bearing.** This IS the §1.8 row #60 evidence artifact
("minimum existence-proof evidence"). Without it the gate cannot
close even with Slices A-D in.

**Brief author.** T-Numeric-Construction lane (cross-Mgr with
Substrate) — coordinates with existing Grounding G2 primitive-row
machinery already PASSING for `i32/i64/u64/f32/f64` per §1.8 row #18.
`u8` row may need extension; verify against
`dsl/extdeps/languages/rust/primitives.dag` before authoring.

**Owner role.** T-Numeric-Construction worker. Routes through PM
if no T-Numeric-Construction Mgr session is active at dispatch
time.

**Gate signal.** First conjunct of §1.4 (representative gap-test
executes); also closes §1.8 row #67
`numeric_construction_demonstration` for the `Int<64>` / `Real<64>`
subset (row #67's full close additionally needs `Int<32>` round-trip
per `docs/r3-design-schedule-2026-05-06.md:177`).

### Slice F — v2-oracle parity check

**Scope.** Run the same source program through the legacy v2
emit chain (`dsl/extdeps/languages/{rust,python,go}/emit.dag` per
`docs/audit/t-v2-retirement-audit.md:105`) and through the v3
stage0 path; assert equivalent target-language output.

**Why load-bearing.** Criterion 5 of the Class 1 5-criteria Pass
(`docs/briefs/r3-substrate-s3-machine-constraint-carrier-worker.md:40-41`).
Confirms the v3 surface produces output equivalent to v2's, with
NO v2-fallback invoked on the v3 leg.

**Brief author.** Substrate Mgr + PB Mgr (parity-harness machinery
historically lives under PB lane).

**Owner role.** PB worker — **PB Mgr role currently unfilled**.
If PB-tier dispatch is the right venue, escalate to PM for PB Mgr
spin-up; if Substrate-tier absorption is acceptable (Substrate Mgr
authors a one-off parity ratchet next to the Slice E demonstration),
this collapses into Slice E. **Decision required.**

**Gate signal.** Closes criterion 5 of Class 1 Pass.

## Dependency graph

```
A (Word8)        ─┐
                  ├──► C (parser desugar) ──► D (class-bridge=0 receipt)
B (Nat8 alias)   ─┘                       └─► E (existence-proof demo) ──► F (v2 parity)
```

A + B are independent and can land in a single PR. C depends on
A + B for the `Nat<8>` arm of the user-surface; the `Int<64>` /
`Real<64>` arms could in principle land first (substrate already
present), but bundling C against the full 3-pair surface avoids
a second parser-touching PR. D and E both depend on C; D and E
are independent of each other. F depends on E.

## Routing summary

| Slice | Owner role | Mgr | Brief status | Notes |
|---|---|---|---|---|
| A | Substrate worker | Substrate Mgr | needs authoring (small) | bundle with B |
| B | Substrate worker | Substrate Mgr | needs authoring (small) | bundle with A |
| C | Substrate worker (parser) | Substrate Mgr | **AUTHORED** at `docs/briefs/r3-substrate-s3-phase-2-parser-grammar-worker.md` | ready to dispatch post-A/B |
| D | Verification worker | Substrate Mgr | needs authoring | small follow-on to C |
| E | T-Numeric-Construction worker | T-Numeric-Construction lane (or PM-routed) | needs authoring | demonstration brief |
| F | PB worker | **PB Mgr (unfilled)** | needs authoring + role decision | escalate to PM, OR collapse into E |

## Open items / risks

1. **PB Mgr role unfilled.** Slice F's natural venue is PB-tier
   v2-parity harness. Decision needed: (a) escalate to PM to spin
   up PB Mgr for this and other PB-tier resourcing, or
   (b) collapse F into Slice E as a one-off Substrate-tier parity
   ratchet. Recommendation: route to PM for option (a) since
   T-FixedPoint #2087 and other PB-tier lanes are also pending PB
   Mgr resourcing (per `project_t_lensproducer_retirement_posture.md`
   neighbourhood).
2. **Open parser design knob in C.** Restrict `N`-literal desugaring
   to known algebraic-concept carriers, or allow universally? Brief
   defers to Substrate Mgr at dispatch; flagging here so the
   decision is not silent.
3. **Authority-audit risk in C.** Parser touches
   `parse_generated.rs`; per
   `feedback_self_hosting_md_authority_audit_before_naming.md`,
   any new sum/struct names must grep against
   `src/v3/SELF_HOSTING.md` before authoring. Brief must surface
   the discipline.
4. **`u8` primitive-row coverage.** Slice E assumes
   `dsl/extdeps/languages/rust/primitives.dag` has an exposed
   `u8` row consumable by Grounding G2. Verify at brief authoring
   time; if absent, prepend a small primitives-row slice.
5. **Pair-counting trap.** §1.8 row #60 Pass criterion is "concept
   faithfully modeled", not "≥3 pairs land". Slice E exists as
   evidence-of-faithfulness, not as a target. Reviewers should
   not gate on additional pairs beyond the demonstration set.

## What this decomposition does NOT do

- Author the slice briefs themselves (only Slice C is already
  authored; A/B/D/E/F brief-authoring is the next step).
- Decide the parser design knob in Slice C.
- Resolve the PB Mgr resourcing question.
- Dispatch workers — dispatch follows brief authoring +
  Substrate Mgr ratification per usual lane protocol.
