# Plan — v2 self-hosting

**Status:** planning tracker for the **Weak Self Host -> Strong Self Host** program (operator-signed 2026-07-11, manager: sharp-bee-290) · **DESIGN.md + the carriers remain the authority** (DESIGN §6). A task's real state is its branch/PR, not this file. Linked from `ROADMAP.md` §1 *Get off v1*. **THE 27-ROW ROSTER CARRIER IS DELETED** (`src/v2/compiler/self_host/frontier.dag`, cut 2026-08-17): it was a hand-authored denominator that no consumer derived, and DESIGN §3's delete-first ruling says a superseded authority comes out at the root rather than being maintained while it dies. This doc is the wave sequencer only; **it no longer has a roster carrier to point at**, and the per-module dispositions it used to defer to must be re-derived from the emitter rather than re-authored. Surviving self-host authorities: `v2.compiler.self_host.emitter_producer_provenance` (which producer emitted a module), `v2.compiler.self_host.stage0_crate_layout` (crate partition), `v2.compiler.self_host.v2_emitter_direct_rust_door_contract` (the emit door). Related: [namespace-resolution-design.md](namespace-resolution-design.md) · [general-body-producer-design.md](general-body-producer-design.md) · [s2-v2-self-emit-direction.md](s2-v2-self-emit-direction.md) · [decl-emission-defork-design.md](decl-emission-defork-design.md) · [dag-v2-defork-audit.md](dag-v2-defork-audit.md) · [seed-shrink-census.md](seed-shrink-census.md) · [typescript-gap-census.md](typescript-gap-census.md) · [emitted-crate-partition-design.md](emitted-crate-partition-design.md) · [c-linkage-unit-realization-design.md](c-linkage-unit-realization-design.md) (the C realization of the same target-agnostic `CompilationUnit` shape self-host emission is built on — the 2nd realization, staffed to stress-test that the shape doesn't cement Rust's acyclic-linkage assumption).

> **END GOAL (decided 2026-06-21 — anchored in ROADMAP §1, do not re-litigate).** `.dag` is the authority/truth; **v2 emits BOTH Rust AND TypeScript as first-class realizations** (not one-or-the-other). Rust is the active seed language today; TypeScript joins the fixed point **after Rust self-host lands** (ROADMAP `5-ts-first-class` — deferred, not dropped). Each realization is proven **by execution**: self-emitted modules compile cargo-green and are **behaviorally equivalent** to the v1 seed on a discriminating corpus (green-by-execution + a discriminating RED, DESIGN §5). **Byte-identity with the seed is explicitly NOT the goal** (operator, 2026-07-08). The seed shrinks across a **typed self-host frontier** (DESIGN §7): each of the 27 compiler modules is *self-emitted* or *seed-retained* with a reason + migration trigger — countable, prioritizable, never a silent escape hatch. **Terminal (Wave 4):** hand-written compiler logic -> 0; a pinned, content-addressed, v2-emitted bootstrap kernel survives (~8-15k LOC).

Historical receipts through 2026-06-23 (Track A/B/T/Z, session-era drift findings) live in git history; superseded by the four-wave structure below.

---

## 0. State of play

- **No compiler module is self-emitted with a green behavioral-equivalence receipt today.** That statement survives the roster cut; the way it used to be *counted* does not. `compiler_frontier_self_emitted_baseline = 0` on a hand-authored 27-row roster (#6445) is deleted along with the carrier — the denominator was authored, not discovered, so `0/27` measured the roster's opinion of how many modules exist rather than the corpus. Do not restore a numerator/denominator here: the honest replacement is a derived one, and until it exists this row is a qualitative claim, stated as such.
- **Emit surface is the dominant blocker — FIRM (qualitative).** The long pole is Rust emit-surface completeness (body producer, value-expression wiring, decl emit), not front-end parse/resolve. Per-module class breakdown is **not** transcribed here — see §2.
- **Wave 1 — IN FLIGHT.** Emit-surface + naming foundation; highest risk; no v1 deleted in this wave.
- **Front-end + emit infrastructure — background facts (not active tracks).** `src/v2/` is 100% `.dag`; full pipeline present. Route-A cargo-green landed (#5777/#5873); CI `dag_compile_clean` + `regen --verify` (#5873). Multi-target emitter: one `fold_node` catamorphism, 14+ targets.

## 1. The four-wave program

Operator-signed 2026-07-11. Each wave is a **dependency-gated exit** (green-by-execution receipt); parallel width is **within** a wave, not across waves. ~**4 waves total** to delete `src/v1`; ~**3 remaining** from now. **`src/v1` deletion happens only at the END of Wave 4.**

### Wave 1 — emit-surface + naming foundation (IN FLIGHT)

The old 2-wave plan collapsed here: "Wave 2 = the sweep" hid the emit-surface keystone. Exit gate — **all** must be green-by-execution before Wave 2 opens:

1. **General body producer emits REAL ingested fn bodies** — RECEIPT LANDED (#6558 body producer + #6526/#6523 fast+long witnesses). Residual explicitly NOT proven by those witnesses: Stage D MVP subsumption, FLAG D binding-key re-grounding, FLAG E body_lowering_fold dissolution. (A fourth residual, `frontier probe per-module receipts`, is retired rather than outstanding: the probe survey is deleted with the roster cut.)
2. **Namespace SymbolIndex / tree-resolution lands** — RECEIPT LANDED (#6523 gate-2 witnesses + #6538 scaling receipt). Residual: 03_name_resolve/03_resolve end-to-end ingest wire. (The `frontier probe blocker_class rows` half is retired with the roster cut, not outstanding.)
3. **FLAG D binder identity grounded** — RECEIPT LANDED (#6575). EnvironmentBindingKey re-grounding on identity; `bind_eval_occurrence_identity_defect` witnesses green; conform-now protocol closed ([general-body-producer-design.md](general-body-producer-design.md) §9).
4. **Weak self-host behavioral receipt green** — RECEIPT LANDED (#6578). `dag/tools/self_host_logic_behavioral_transport.dag` emit→compile→run→equals-seed chain green-by-execution for `dag/std/logic.dag` (plain + `--inject-fault` RED control).

**NO v1 deleted in Wave 1.**

### Wave 2 — first real flips

The two big emit-surface tracks (~11 modules) **self-emit + behaviorally verify + REPLACE their v1 counterparts** -> **first v1 deletions**.

### Wave 3 — remaining module drain

Every remaining compiler module flips to self-emitted **or** is declared a **pinned-kernel** row with an honest reason + migration trigger. The exit used to read `27/27 self-emitted-or-declared-kernel`; that fraction is retired with the roster carrier, because both halves came from the same hand-authored row list. **The exit condition is unchanged in substance and now needs a derived denominator to state it against** — an emitter-side census of what the compiler actually consists of, not a re-authored list. Naming that derivation is the first Wave 3 obligation.

### Wave 4 — collapse to kernel + delete v1

1. Pipeline runs on **emitted Rust** (not v1 seed mirror).
2. `src/v1` collapses to **~8-15k LOC pinned bootstrap kernel** ([seed-shrink-census.md](seed-shrink-census.md)).
3. **Import grammar deleted** (import-deletion ladder B4 — see §3).
4. **CI compile cone gone** — regen cutover complete; HAND queue drained ([invert-hand-maintained.md](invert-hand-maintained.md)).
5. TypeScript fixed-point work **opens** post-Rust (END GOAL unchanged; [typescript-gap-census.md](typescript-gap-census.md)).

## 2. Census honesty — what is firm vs carrier-held

- **FIRM (qualitative):** emit surface is the dominant blocker class. This is the one census claim that survived the cut, because it never depended on the roster's row count.
- **WAS carrier-held, now unheld.** Per-module gap classification lived on the deleted `frontier.dag` as 27 `SeedRetained { reason, migration_trigger }` rows plus `compiler_frontier_census_attribution: ExecutionMeasured`. The pointing-at-a-carrier discipline (DESIGN §2/§6: no parallel-ledger doc) is unchanged and is *why* nothing was transcribed back into this doc when the carrier went — a plan that answers a question its authority no longer answers is the parallel ledger that rule forbids. The gap is declared, not filled.
- **RETIRED mechanism (was #6464):** the structural census, its fail-closed classifier, and the `frontier_probe_emit_from_ingest` / `frontier_probe_survey` host-transport receipts are deleted with the roster. The survey never produced a receipt on any host, including a dedicated runner, so what is lost is machinery rather than evidence — read emitter refusals by driving emission directly.

## 3. Import-deletion ladder (parallel track — NOT a wave gate)

Namespace resolution and import deletion are **separate timelines**. Tree-resolution can work while imports still exist.

1. **B1 merged + B2 (#6462)** — reference-deps + equivalence = Wave 1-era **resolution** (naming authority without deleting `import`).
2. **B3** — ~1854-file migration; **parallel to Waves 2-3**.
3. **B4** — delete `import` grammar = **Wave 4** terminal step.

## 4. Retired framing

The June 2026 **Track A/B/T/Z** structure, Purity/Forced-precondition-order as active sequencing, and byte-digest Stage C as the milestone oracle are **retired**. Absorbed facts: Route-A cargo-green DONE (background); byte-oracle superseded by behavioral-equivalence (2026-07-08); Track Z terminal -> **Wave 4**; Track T -> END GOAL **sequenced post-Rust**, not shelved. `regen --verify` (#5873) and #5639 drift receipts remain cautionary background for Wave 4 cutover.

## 5. Prerequisites (unchanged gates)

1. **De-fork / single std authority** — [dag-v2-defork-audit.md](dag-v2-defork-audit.md); cross-tree import wired (#5473).
2. **Seed-only parallel representations** — [seed-debt-bundle-item-2.md](seed-debt-bundle-item-2.md).
3. **HAND kernel D (interpreter pure-eval)** — [interpreter-kernel-d.md](interpreter-kernel-d.md).
4. **Emit-on-demand execution** — parallel track (ROADMAP §1); not a Wave 1 blocker.

Related: [S2 — v2 emits v2: strategic direction & decomposition](s2-v2-self-emit-direction.md) — the active lane decomposition toward the Track Z fixed point (B-rungs, whole-module harness lead).

## Dissolution trigger (DESIGN §6)

Delete this doc when Wave 4 lands: pipeline on emitted Rust, import grammar deleted, v1 collapsed to pinned kernel, CI compile cone gone. Absent v1 compiler logic is the authority at that point; this tracker is redundant. (The frontier carrier this condition used to name was deleted 2026-08-17 — it is not a retirement input any more.)
