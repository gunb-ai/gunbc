# R3 Debt Dispatch Tracker — 2026-05-09

**Authority**: Director Phase 3 re-task at gunb-ai/gunbc#828 c#4413891498 + PM scope-consolidation re-task at gunbc#846 → gentle-newt-665 inbox c#4413913108. Operator-authorized 2026-05-09 ~20:30Z (lane scope all v3 debt; active posture) + ~23:15Z (orient ALL pending v3 debt work-items through Debt-Paydown Mgr as central tracker).

**Scope** (broadened 2026-05-09 c#4413913108): per-brief dispatch-path classification for ALL pending v3 debt worker briefs under Debt-Paydown central tracker. Currently:
- 6 gpt-5-5-pro novel-finding briefs (F1-F6, PR #2437)
- 4 P0 bug-fix briefs (PR #2373; PM-authored from gpt-5-5-pro reflective review cycle)

= **10 worker-ready briefs** awaiting lane-Mgr absorption + worker spawn.

Per Director ask: each row classified into (a) Mgr-self-author retirement / (b) cross-Mgr surface to lane-owning Mgr / (c) operator-tier ratification needed. Output feeds option-(c)-tightening discipline (PR #2361 §3) with concrete dispatch evidence.

**Scope-boundary note**: Debt-Paydown Mgr is coordinator/tracker, NOT direct-author for substrate carrier changes. Path (a) Mgr-self-author is reserved for cleanup/discipline-shaped rows. Substrate-shape changes route via path (b) to the lane-owning Mgr. Worker spawn authority stays with lane Mgrs.

---

## Triage table

| Brief | Class | Owner Mgr | Capacity | Dispatch path | Rationale | Dep-state | Dispatch surface |
|---|---|---|---|---|---|---|---|
| `r3-bug-u128-grounding-pilot-mirror-sync-worker.md` | C | Substrate (#2068) | WORKING (heavy) | **(b) cross-Mgr** | HIGHEST priority — concrete drift bug; thesis-validating. Rust mirror at `grounding_pilot/src/lib.rs:6-8,256-265` diverged from `.dag` substrate (u128 declared at 3 sites; mirror still says deferred). | dep-clear | this PR consolidated surface |
| `r3-bug-fieldproject-dual-authority-dissolution-worker.md` | B | Substrate (#2068) | WORKING (heavy) | **(b) cross-Mgr** | HIGH — illegal-state-representable. `FieldProject { field_label: String, field_child: Option<DeclarationId> }` admits inconsistent pairs. | dep-clear | this PR consolidated surface |
| `r3-bug-resolve-producer-opt-typed-return-worker.md` | B | Substrate (#2068) | WORKING (heavy) | **(b) cross-Mgr** | HIGH — P3 fail-closed violation; 4-state collapse to None. (Originally surfaced to Verification at c#4413712491; PM scope-consolidation 2026-05-09 ~23:15Z reclassifies as Substrate-owner per brief header.) | dep-clear | this PR consolidated surface |
| `r3-bug-callgraph-forward-only-authority-worker.md` | B | Substrate (#2068) | WORKING (heavy) | **(b) cross-Mgr** | MEDIUM — clean refactor; illegal-state-representable in product. | dep-clear | this PR consolidated surface |
| `r3-novel-f1-missingemissionpath-typed-axes-worker.md` | C | Substrate (#2068) | WORKING (heavy) | **(b) cross-Mgr** | Substrate fix: type axes (`connective`/`behavior`/`target`) against `TypeConnective`/`Behavior`/`ShapeATarget` carriers. Rust mirror trivially aligns. | dep-clear | gunbc#2068 c#4413712338 |
| `r3-novel-f2-shapeatarget-vs-languagespec-worker.md` | F | Grounding (sunny-koi-893 / #2063) | IDLE (capacity) | **(b) cross-Mgr** | Grounding-side: closed `ShapeATarget` enum vs `LanguageSpec` data extensibility. Coordinate with Substrate on `LanguageSpec`-ref carrier shape if Option A pursued. | dep-clear | gunbc#2063 c#4413712632 |
| `r3-novel-f3-map-string-bool-as-set-worker.md` | F | Substrate (#2068) + PB (#2074) coord | WORKING (heavy) | **(b) cross-Mgr** | Substrate-led migration `Map<String,Bool>` → `Set<String>`. v2-side surfaces coordinate with PB Mgr on v2-retirement timing. | dep-clear | gunbc#2068 c#4413712338 |
| `r3-novel-f4-partitionresult-anonymous-bypass-worker.md` | G | Substrate (#2068) | WORKING (heavy) | **(b) cross-Mgr; small-scope opportunistic** | Single-option fix; could fold into next Substrate worker that touches `dsl/std/filesystem.dag`. | dep-clear; opportunistic | gunbc#2068 c#4413712338 |
| `r3-novel-f5-composedeffect-illegal-product-worker.md` | C | Substrate (#2068) + PB (#2074) coord | WORKING (heavy) | **(b) cross-Mgr** | Substrate-side: `ComposedEffect` redesign sum-vs-product. v2-side consumer; PB coord on v2-retirement timing. STOP-and-PING gate: `OperationEffect` shape co-evolution. | dep-clear (Option A); v2 coord | gunbc#2068 c#4413712338 |
| `r3-novel-f6-derive-op-effect-string-parser-worker.md` | C | Substrate (#2068) + PB (#2074) coord | WORKING (heavy); implementation receipt in progress via issue #2470 | **(b) cross-Mgr** | Substrate-side: typed `HttpMethod`/`Path` carrier consumption replaces string parsing at structural boundary. | dep-clear (substrate-prereq audited 2026-05-09: `HttpMethod` carrier exists at `std/types.dag`; worker migrated `derive_op_effect` to typed `HttpMethod` + `PathTemplate` consumption with parsing left at surface helpers) | gunbc#2068 c#4413712338 |

---

## Summary

| Path | Count | Notes |
|---|---|---|
| (a) Mgr-self-author | 0 | None — all 10 are substrate-shape changes or substrate-Mgr-owned bug-fixes; lane-owner-Mgr authority. |
| (b) cross-Mgr surface | 10 | Substrate 9 + Grounding 1; PB v2-retirement coord on 3 (F3/F5/F6). |
| (c) operator-tier ratification | 0 | None — all carrier shapes have ratified Director or substrate precedents. |

**Cross-Mgr capacity (per PM 2026-05-09 c#4413913108)**:
- **Substrate Mgr (warm-wolf-698 / gunbc#2068)**: WORKING; heavy queue (PR #2400 active + 3 open assignments + 3 active worker children). 9 of 10 briefs in this lane. **No escalation pressure**; absorbs at natural cycle pace.
- **Grounding Mgr (sunny-koi-893 / gunbc#2063)**: IDLE with capacity (1 assignment, 3 workers, 1 ACTIVE Cursor). 1 brief (F2).
- **PB Mgr (gunbc#2074)**: coord on F3/F5/F6 v2-retirement timing.

**Velocity expectations**: F1/F4 + 4 P0 bugs are dep-clear-and-substantively-small (~1 PR each). F2/F3 cross-cutting (1-2 PRs). F5/F6 require v2-retirement coord; sizing depends on PB Mgr's bulk-dissolution timing. Substrate Mgr's heavy-queue state means absorption rate is ≤2 briefs/cycle — expect Substrate-side cycle-1 = u128 + FieldProject (HIGHEST + HIGH).

**Open prereqs requiring substrate-grep before dispatch**:
- (none) — F6 substrate-prereq audited 2026-05-09; all rows dep-clear.

**Dispatch authority**: per PM standing-authority confirmation (gunbc#846 c#4413701937 + scope-broadening c#4413913108), Debt-Paydown Mgr cross-surfaces to lane Mgrs. Lane Mgrs hold dispatch authority on absorption.

**Option-(c) tightening evidence (per PR #2361 §3)**: zero (c)-tier items across all 10 rows.

**PM tracking commitment**: deep-wolf-155 watches absorption-receipts at gunbc#2068 + #2063; escalates via PM-tier surface if Substrate Mgr stalls >24h on any individual brief. Debt-Paydown Mgr holds central-tracker authority on aggregate progress reporting.

---

## Cross-references

- Sweep doc rows: `docs/audit/r3-debt-sweep-2026-05-06.md` Class B rows 8 + Class C rows 1/2/3/4/5/6 + Class F rows 3/4 + Class G row 14.
- Novel-finding brief files: `docs/briefs/r3-novel-f{1..6}-*-worker.md` (PR #2437 merge `060c6154c`).
- P0 bug-fix brief files: `docs/briefs/r3-bug-{u128-grounding-pilot-mirror-sync,fieldproject-dual-authority-dissolution,resolve-producer-opt-typed-return,callgraph-forward-only-authority}-worker.md` (PR #2373).
- gpt-5-5-pro source: PR #2358 §8 meta-finding cycle (3 sha windows) + earlier reflective review cycle (PM dispatch authority gunbc#846 c#4413207527).

---

**End of dispatch tracker.**
