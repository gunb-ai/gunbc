# R3 Novel-Findings (F1–F6) Dispatch Tracker — 2026-05-09

**Authority**: Director Phase 3 re-task at gunb-ai/gunbc#828 → gentle-newt-665 inbox c#4413891498. Operator-authorized 2026-05-09 ~20:30Z (lane scope all v3 debt; active posture).

**Scope**: per-brief dispatch-path classification for the 6 gpt-5-5-pro novel-finding worker briefs that landed via PR #2437. Per Director ask: classify each into (a) Mgr-self-author retirement / (b) cross-Mgr surface to lane-owning Mgr / (c) operator-tier ratification needed. Output feeds option-(c)-tightening discipline (PR #2361 §3) with concrete dispatch evidence.

**Scope-boundary note**: Debt-Paydown Mgr is NOT direct-author for substrate carrier changes. Path (a) Mgr-self-author is reserved for cleanup/discipline-shaped rows. Substrate-shape changes route via path (b) to the lane-owning Mgr.

---

## Triage table

| Brief | Class | Owner Mgr | Dispatch path | Rationale | Dep-state | Dispatch surface |
|---|---|---|---|---|---|---|
| `r3-novel-f1-missingemissionpath-typed-axes-worker.md` | C | Substrate (warm-wolf-698 / #2068) | **(b) cross-Mgr** | Substrate fix: type axes (`connective`/`behavior`/`target`) against `TypeConnective`/`Behavior`/`ShapeATarget` carriers. Rust mirror trivially aligns. No operator ratification needed; carrier names already exist. | dep-clear | gunbc#2068 c#4413712338 |
| `r3-novel-f2-shapeatarget-vs-languagespec-worker.md` | F | Grounding (sunny-koi-893 / #2063) | **(b) cross-Mgr** | Grounding-side: closed `ShapeATarget` enum vs `LanguageSpec` data extensibility. Coordinate with Substrate on `LanguageSpec`-ref carrier shape if Option A pursued. | dep-clear | gunbc#2063 c#4413712632 |
| `r3-novel-f3-map-string-bool-as-set-worker.md` | F | Substrate (#2068) + PB (#2074) coord | **(b) cross-Mgr** | Substrate-led migration `Map<String,Bool>` → `Set<String>` across graph/syntax/node files. v2-side surfaces (`src/v2/04_infer.dag`) coordinate with PB Mgr on v2-retirement timing. | dep-clear (no R3-blocking prereq) | surfaced this PR (cross-cuts #2068 + #2074) |
| `r3-novel-f4-partitionresult-anonymous-bypass-worker.md` | G | Substrate (#2068) | **(b) cross-Mgr; small-scope opportunistic** | Single-option fix (`partition_entries` returns `PartitionResult` named type). Could fold into next Substrate worker that touches `dsl/std/filesystem.dag`; no separate-PR dispatch needed. | dep-clear; opportunistic | gunbc#2068 c#4413712338 |
| `r3-novel-f5-composedeffect-illegal-product-worker.md` | C | Substrate (#2068) + PB (#2074) coord | **(b) cross-Mgr** | Substrate-side: `ComposedEffect` redesign sum-vs-product (Option A preferred per `feedback_load_bearing_ratchet_preservation`). v2-side consumer at `src/v2/effect_derivation.dag` + `stage0/.../effect_derivation.rs` coordinates with PB Mgr on v2-retirement timing. STOP-and-PING gate: `OperationEffect` shape co-evolution. | dep-clear (Option A); v2 coord required | gunbc#2068 c#4413712338 |
| `r3-novel-f6-derive-op-effect-string-parser-worker.md` | C | Substrate (#2068) + PB (#2074) coord | **(b) cross-Mgr** | Substrate-side: typed `HttpMethod`/`Path` carrier consumption replaces string parsing at structural boundary. v2-side surface; PB coord. | dep-clear (substrate-prereq audited 2026-05-09: `HttpMethod` carrier exists at `std/types.dag`; imported by `src/v2/effect_derivation.dag:8` + `src/v3/std/services.dag:3`) | gunbc#2068 c#4413712338 |

---

## Summary

| Path | Count | Notes |
|---|---|---|
| (a) Mgr-self-author | 0 | None — all 6 are substrate-shape changes; lane-owner-Mgr authority. |
| (b) cross-Mgr surface | 6 | All routed; Substrate-owned (5) + Grounding-owned (1); PB v2-retirement coord on 3. |
| (c) operator-tier ratification | 0 | None — all carrier shapes have ratified Director or substrate precedents. |

**Velocity expectations**: F1/F4 are dep-clear-and-substantively-small (~1 PR each in lane Mgr's backlog). F2/F3 are larger (cross-cutting consumer touch; expect 1-2 PRs). F5/F6 require v2-retirement coordination; sizing depends on PB Mgr's bulk-dissolution timing.

**Open prereqs requiring substrate-grep before dispatch**:
- (none) — F6 substrate-prereq audited 2026-05-09: `HttpMethod` carrier exists at `std/types.dag`; consumed by `src/v2/effect_derivation.dag:8` + `src/v3/std/services.dag:3`. F6 dispatch is dep-clear.

**Dispatch authority**: per PM standing-authority confirmation (gunb-ai/gunbc#846 c#4413701937), Debt-Paydown Mgr surfaced these to lane Mgrs at gunbc#2068 c#4413712338, gunbc#2063 c#4413712632. Lane Mgrs hold dispatch authority on absorption.

**Option-(c) tightening evidence (per PR #2361 §3)**: zero (c)-tier items; option-(c) discipline holds for these rows — no operator allocation needed.

---

## Cross-references

- Sweep doc rows: `docs/audit/r3-debt-sweep-2026-05-06.md` Class C rows 4/5/6 + Class F rows 3/4 + Class G row 14.
- Brief files: `docs/briefs/r3-novel-f{1..6}-*-worker.md` (PR #2437 merge `060c6154c`).
- gpt-5-5-pro source: PR #2358 §8 meta-finding cycle (3 sha windows).

---

**End of dispatch tracker.**
