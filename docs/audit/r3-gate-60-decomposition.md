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
| `Int = AbelianGroup<GroupCompletion<Nat>>` algebraic concept | `dsl/std/integer.dag:123` | #1466 |
| `UInt = Nat` canonical-instance form | `dsl/std/integer.dag:124` | #1818 |
| `Real = ApproximateField<FieldOfFractions<Int>>` | `dsl/std/float.dag` | #2397 / #2409 |
| `Int8…Int128`, `UInt8…UInt128` (canonical fixed-width rows) | `dsl/std/integer.dag:45-56` | #2161 |
| `IntW32/64/128`, `UIntW32/64/128` (historical substrate aliases) | `dsl/std/integer.dag:79-84` | (pre-#2161 hold-overs) |
| `Real32/Real64` = `Compose<Real, MachineWidth<Word32\|Word64>>` | `dsl/std/float.dag` | #2570 |
| Width carriers `Word16/Word32/Word64` | `dsl/std/bit.dag:29-31` | (pre-R3) |
| Grounding G2 primitive-row consumer evidence (`i32/i64/u64/f32/f64`) | `dsl/extdeps/languages/rust/primitives.dag` | #2570 |

Additional substrate facts on `main` relevant to the 8-bit arm:

| Substrate fact | Location | PR |
|---|---|---|
| `Byte` carrier (8-bit width unit) | `dsl/std/bit.dag:26` | (pre-R3) |
| `Int8 = Compose<Int, MachineWidth<Byte>>` | `dsl/std/integer.dag:45` | landed |
| `UInt8 = Compose<UInt, MachineWidth<Byte>>` (≡ `Nat<8>` via `UInt = Nat`) | `dsl/std/integer.dag:52` | landed |
| Named dissolution trigger: `MachineWidth<WordN>` → `MachineWidth<N>` literal-Nat when parser lands | `dsl/std/integer.dag:71-78` | (in-substrate) |

What is **not** on `main` (load-bearing for gate-60):

1. User-surface `Int<N>` / `Real<N>` / `Nat<N>` parser syntax that
   desugars to `Compose<Algebra, MachineWidth<N>>` per Q-MC
   sub-decision 3 ratified spelling (gunbc#828
   #issuecomment-4385530115: *"`Int<64>` parses/elaborates as
   `Compose<Int, MachineWidth<64>>` parametrically"*) — note
   **literal-Nat `N`**, NOT `WordN` carrier. Current `Word16/32/64`
   spelling in `dsl/std/integer.dag:45-56` is the pre-parser
   workaround carrying its own dissolution trigger.
2. End-to-end demonstration that the user-surface 3-pair set lowers
   through stage0 without invoking v2-fallback for type-resolution.
3. v2-oracle parity on the same source program (Class 1 5-criteria
   final row per `docs/briefs/r3-substrate-s3-machine-constraint-carrier-worker.md:32-41`).
4. Class-bridge enumeration receipt = 0 (§1.4 conjunctive rule).
5. Workaround dissolution: retire `IntW*` / `UIntW*` aliases and
   collapse `MachineWidth<WordN>` slot-2 spellings to `MachineWidth<N>`
   once parser lands, per the named dissolution trigger.

## Decomposition

The four outstanding facts above map onto four slices, plus one
workaround-dissolution follow-on. Slices C, D, and Z are substrate-lane
(Substrate Mgr authority); E and F are demonstration / parity and route
through T-Numeric-Construction + PB lanes.

**Reviewer-corrected note (per BLOCKING findings 2026-05-12 on this
PR).** Earlier drafts of this decomposition proposed adding a `Word8`
record (Slice A) and a `Nat8` substrate alias (Slice B). Both are
withdrawn:

- `dsl/std/bit.dag:26` already defines `Byte` as the 8-bit carrier;
  introducing `Word8` would create parallel authority (INVARIANTS P1).
- `Int8 = Compose<Int, MachineWidth<Byte>>` and
  `UInt8 = Compose<UInt, MachineWidth<Byte>>` already exist at
  `dsl/std/integer.dag:45,52`; with `UInt = Nat` (line 124),
  `Nat<8>` substrate ≡ `UInt8`. No new substrate alias is needed.

The 8-bit arm is therefore parser-only work; Slice C is the entire
substrate-side surface change.

### Slice C — User-surface parser desugaring `T<N>` → `Compose<T, MachineWidth<N>>`

**Scope.** Implement the parser-grammar surface authored at
`docs/briefs/r3-substrate-s3-phase-2-parser-grammar-worker.md`:
user writes `Int<64>`, parser elaborates parametrically to
`Compose<Int, MachineWidth<64>>` per Q-MC sub-decision 3 ratified
spelling (gunbc#828 #issuecomment-4385530115). Numeric literal `N`
in the slot-2 position desugars to **literal-Nat `N`** as the
`MachineWidth<…>` argument — NOT to a `Word<N>` carrier. The
`MachineWidth<Word*>` slot-2 form currently used at
`dsl/std/integer.dag:45-56` is the pre-parser workaround that
dissolves under this slice (see Slice Z).

For the `Nat<8>` arm: literal `8` maps to `MachineWidth<8>`; the
existing `Byte` carrier is the substrate-side 8-bit grounding and
remains under the hood at emission time, but is **not** the slot-2
spelling the parser produces.

**Why load-bearing.** This is the *parser-grammar* in the gate name.
Substrate already carries the algebraic concepts and the
`MachineWidth<bits>` carrier; without parser desugaring producing
the ratified `MachineWidth<N>` literal-Nat slot-2 spelling, gate-60's
"parser handles generic `Compose<...>` interaction syntax" predicate
cannot be discharged without re-validating the workaround as the
target.

**Open design knob (do not silently pre-decide).** Whether numeric
literal `N` in slot-2 desugars only when slot-1 is a known
algebraic-concept carrier (`Int`/`Nat`/`Real`/`UInt`), or whether
it desugars universally. Brief at
`docs/briefs/r3-substrate-s3-phase-2-parser-grammar-worker.md`
preserves this as a clarifying question for Substrate Mgr at
dispatch.

**Brief author.** Already authored; status = draft, worker pin
valiant-ant-72 (freed-pool post #1856). **Brief must be amended**
to specify literal-Nat slot-2 spelling (not `Word<N>`) before
dispatch — current Phase-2 brief text at lines 70-74 references
`Compose<Int, MachineWidth<N>>` correctly; verify no residual
`Word<N>` slot-2 wording before worker pickup.

**Owner role.** Substrate worker. Touches v3-compiler parser
(`src/v3/compiler/src/parse_generated.rs`) — operator BLOCKING risk
on `SELF_HOSTING.md` authority-audit per
`feedback_self_hosting_md_authority_audit_before_naming.md`; brief
must include the grep-before-name discipline.

**Gate signal.** Discharges criterion 2 of Class 1 5-criteria
Pass (parser handles generic interaction syntax) — see
`docs/briefs/r3-substrate-s3-machine-constraint-carrier-worker.md:32-41`.

### Slice Z — Pre-parser workaround dissolution (`MachineWidth<WordN>` → `MachineWidth<N>`; retire `IntW*` / `UIntW*` aliases)

**Scope.** Collapse `Int16/Int32/Int64/Int128` (and the `UInt*` /
`Real*` family) from `Compose<…, MachineWidth<WordN>>` to
`Compose<…, MachineWidth<N>>` literal-Nat spelling. Retire the
historical `IntW32/64/128` and `UIntW32/64/128` aliases at
`dsl/std/integer.dag:79-84` per the named dissolution trigger
recorded at `dsl/std/integer.dag:71-78`.

**Why load-bearing.** Without dissolution, the gate-60 receipt
would Pass against a substrate that still carries two slot-2
spellings — closure would be "concept faithfully modeled modulo a
pre-parser scaffold", which is not faithful representation
(INVARIANTS P2 dual-authority). The dissolution trigger is
already in-substrate and is a one-shot drop once Slice C lands.

**Brief author.** Substrate Mgr (small follow-on to Slice C; can
bundle in the same PR if churn is manageable, but a separate PR is
preferable to keep the parser-surface PR scope-tight).

**Owner role.** Substrate worker.

**Gate signal.** Required for gate-60 close per INVARIANTS P2; also
discharges the pre-parser-scaffold callout at
`dsl/std/integer.dag:71-78`.

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
1. Parses source containing the three user-surface spellings
   (`Int<64>` / `Real<64>` / `Nat<8>`).
2. Verifies elaboration to `Compose<…, MachineWidth<N>>` literal-Nat
   slot-2 spelling per Q-MC sub-decision 3 (NOT `MachineWidth<WordN>`).
3. Lowers through stage0 to target Rust `i64` / `f64` / `u8` rows
   from `dsl/extdeps/languages/rust/primitives.dag` (where `Byte`
   is the in-substrate 8-bit grounding for the `u8` row).
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
C (parser desugar, literal-Nat slot-2) ──► D (class-bridge=0 receipt)
                                       └─► E (existence-proof demo) ──► F (v2 parity)
                                       └─► Z (Word*-workaround dissolution)
```

Substrate carriers for all three arms (`Int<64>`, `Real<64>`, `Nat<8>`)
are already on `main` — the 8-bit arm uses the existing `Byte` carrier
via `UInt8 = Compose<UInt, MachineWidth<Byte>>` and `UInt = Nat`.
Slice C is therefore the single entry-point; D, E, Z fan out from C.
F depends on E. D and Z are independent of each other.

## Routing summary

| Slice | Owner role | Mgr | Brief status | Notes |
|---|---|---|---|---|
| C | Substrate worker (parser) | Substrate Mgr | **AUTHORED** at `docs/briefs/r3-substrate-s3-phase-2-parser-grammar-worker.md` | verify brief specifies literal-Nat slot-2, not `Word<N>`, before dispatch |
| D | Verification worker | Substrate Mgr | needs authoring | small follow-on to C |
| Z | Substrate worker | Substrate Mgr | needs authoring | retire `IntW*/UIntW*` aliases + `MachineWidth<WordN>` slot-2; one-shot |
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
  authored; D/Z/E/F brief-authoring is the next step).
- Decide the parser design knob in Slice C.
- Resolve the PB Mgr resourcing question.
- Dispatch workers — dispatch follows brief authoring +
  Substrate Mgr ratification per usual lane protocol.
