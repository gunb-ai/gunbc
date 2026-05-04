# R3 Verification - Bridge Row 4 Exact-String Deeper-Detail Receipt

**Status:** AUDIT RECEIPT - docs-only. This receipt does not flip any
`BridgeLedgerRow.status` and does not close a Debt-Paydown row directly.

**Row:** `bridge_exact_string_patching_residual_retired`.

**Primary inputs:** `docs/briefs/bridge-retirement-audit-include-str-family.md`
for family B BR-19 / BR-09 exact-string boundaries, and
`docs/briefs/r3-v-bridge-row-by-row-retirement-audit.md` for the Row 4 umbrella
rule.

**Boundary inputs:** Rows 1-3 receipts are boundary checks only:
`r3-v-bridge-row-1-sourcespan-deeper-detail-receipt.md` owns SourceSpan/file
identity, `r3-v-bridge-row-2-canonical-lens-deeper-detail-receipt.md` owns
canonical-lens byte/name identity, and
`r3-v-bridge-row-3-include-str-deeper-detail-receipt.md` owns generic
`include_str!` side-channel mechanics.

## Umbrella Rule

Row 4 is structurally different from Rows 1-3. It is an umbrella over exact-
string patching classes, not one directly retireable implementation site.

Verification must reject a PR that claims Row 4 retirement from closure of only
one class. Row 4 may retire only when either:

1. **all underlying exact-string patch classes have structural authority
   replacements and their ratchets/receipts are updated**, or
2. **the umbrella is split into separately tracked per-class rows**, and each
   split row retires independently.

The already-retired lower-helper zero ratchet is evidence for one class only.
It is not umbrella closure.

## Verification Consumption Rule

A Row 4 retirement PR must enumerate every exact-string patch class it touches
and show, per class:

1. the exact string matcher, sentinel replacement, or post-parse mutation is
   deleted or made structural;
2. the replacement carrier is consumed by production code or the relevant
   verification ratchet;
3. related SourceSpan/file, canonical-lens, or `include_str!` side-channel
   surfaces are either untouched or explicitly scoped to Rows 1-3;
4. the class remains under this umbrella only if its retirement trigger is
   bounded and already named; otherwise it must split into its own row before
   Row 4 closure can be claimed.

No PR may close Row 4 by pointing at `bridge_lower_helpers_patch_zero_residual`
alone while BR-19 or any discovered exact-string class remains live.

## Cross-Packet Boundary Discipline

| Family B entry / surface | Row 4 owns | Row 4 does not own |
| --- | --- | --- |
| BR-19 `patch_kernel_bool_boolean_algebra_inhabits` | The exact post-parse Bool inhabits patch and name/file string selection that mutates `Declaration.inhabits`. | Row 1 owns the SourceSpan/file authority gate portion; Row 4 owns the patch class closure. |
| Retired lower-helper class | The receipt records it as already retired by `bridge_lower_helpers_patch_zero_residual_test`; no new closure credit for Row 4. | It is a narrow retired class, not proof that the umbrella is closed. |
| BR-09 SG-0 / infer-helper text mining | Exact-string patching/rewriting classes discovered through `INFER_HELPERS_SOURCE.contains(...)` or similar scans, if they mutate/patch semantics. | Row 3 owns embedded source-text mining as a side-channel ratchet when it only counts or detects text. |
| BR-06 R1 gates fixture splice | Sentinel/exact-string fixture replacement (`R1_*_SPLICE_V1`) outside the canonical-lens byte slice. | Row 2 owns canonical-lens byte identity in the splice; Row 3 owns generic loader/include mechanics. |
| BR-18 raw compiler source embeds | Only a future source-text patch/rewrite class over those embedded Rust sources. | Row 3 owns generic raw source inclusion; Row 1 owns production file participation if changed. |
| BR-07 / BR-08 / Appendix B canonical lens surfaces | No Row 4 ownership unless a PR patches/replaces canonical lens text by exact string rather than retiring byte/name identity structurally. | Row 2 owns canonical bytes, name dispatch, and ratchet counts. |
| BR-01/02/10-16/20-22 side channels | No Row 4 ownership when the surface is text inclusion, scraping, generated Rust diffing, static table embedding, or pipeline anti-bridge. | Row 3 owns these as include/source-text side channels. |
| Appendix A hermetic fixtures | No Row 4 ownership by default. | Scenario inputs are outside the bridge family unless they become exact-string semantic patch authorities. |

## Per-Class Receipts

| Class | Reviewer rule | Carrier consumer required | Cross-row impact | Umbrella / split decision |
| --- | --- | --- | --- | --- |
| BR-19 Bool inhabits post-parse patch | Delete `patch_kernel_bool_boolean_algebra_inhabits` and every call path that mutates `Bool` inhabits after parse; `Bool` authority cannot remain name-plus-file plus host mutation. | Direct authored `Bool inhabits BooleanAlgebra<Bool>` fact in `dsl/std/types.dag` or the v3-authoritative algebra/kernel home, consumed through `Declaration.inhabits` / `TypeConnective::Instantiation` after the v2/parser/bootstrap surface accepts it. | Row 1 owns the file gate; Row 4 owns patch deletion. Coordinate with PB Tier-2 and Substrate/v2 syntax work. | **Remain under umbrella for now.** It has a bounded retirement trigger and is the current load-bearing Row 4 class. Split into its own row only if PB/Substrate wants independent ledger closure before other exact-string classes are resolved. |
| Lower-helper generated-source exact-string patch | Do not count this class as live unless contiguous `patch_lower` + `_helpers` reappears. The ratchet must continue proving zero residual. | Already replaced by generated `lower_helpers` fields/helpers; consumed through `lower_helpers_generated.rs` and live lowerer imports. Ratchet: `bridge_lower_helpers_patch_zero_residual_test.rs`. | Narrow PB Tier-2 slice; separate from BR-19 and any SG/text-mining class. | **Already retired / should remain split as a narrow proof.** It must feed the umbrella ledger as closed evidence, not close Row 4 by itself. |
| BR-09 infer-helper exact-string discovery class | If `INFER_HELPERS_SOURCE.contains(&format!(...))` or sibling scans drive semantic patching/rewriting, the implementation PR must split that class or replace it with structural registry/census facts. Pure count/ratchet mining stays Row 3. | Structural census from `Dag`, generated-file registry, declared table, or typed helper inventory. If the class mutates semantics, it needs its own carrier matching that mutation. | Row 3 owns include/text-mining mechanics. Row 4 owns only exact-string semantic patch classes discovered by that mining. | **Split on discovery.** Do not keep a vague "infer-helper exact-string" bucket under the umbrella; create a per-class row/receipt if it becomes semantic patch debt. |
| BR-06 R1 gates sentinel fixture splice, non-canonical slice | If a PR changes `emit_r1_gates_fixture` sentinel replacement outside Row 2's canonical-lens byte identity, it must remove sentinel/string replacement as authority, not rename sentinels or move escaped bytes. | Structural `TestClaim`, declaration-ref fixture carrier, or single substrate-owned generator that writes fixture structure without exact-string splice authority. | Row 2 owns canonical-lens byte identity in the splice; Row 3 owns loader/include mechanics. | **Split if still load-bearing after Row 2/3 work.** BR-06 mixes canonical-lens and exact-string generator concerns, so any non-canonical residual should get its own row before umbrella closure. |
| Future exact-string patch class discovered by audit/implementation | A PR must identify the exact matcher and semantic effect, then either retire it structurally or add a separately tracked row before Row 4 closure. | Class-specific typed fact, declaration authority, generated table, or structural verifier matching the semantic effect. | Must first classify against Rows 1-3 to avoid double-counting file identity, byte inclusion, or generic text mining. | **Split by default.** Unknown exact-string classes must not be silently absorbed into the umbrella. |

## Substrate/PB Routing Notes

Row 4 routes primarily to PB Tier-2 / PB bootstrap with a Substrate-v2 syntax
dependency for the live Bool patch:

- BR-19 requires source-level Bool inhabits authoring once the parser/bootstrap
  path accepts the syntax. The target fact shape already exists after bootstrap
  as `Declaration.inhabits` / `TypeConnective::Instantiation`; the missing part
  is direct authoring without a host patch.
- The retired lower-helper class remains PB evidence, enforced by
  `bridge_lower_helpers_patch_zero_residual_test.rs`, but it does not close the
  umbrella row.
- BR-09-derived exact-string semantic classes and BR-06 non-canonical sentinel
  residuals should split before implementation if they remain load-bearing.
- Rows 1-3 stay authoritative for file participation, canonical lens identity,
  and generic `include_str!` side-channel mechanics.

No STOP+PING is triggered by this receipt. The only live bounded class is BR-19;
the other potential classes have clear split discipline. If implementation
finds an exact-string class with no clear carrier and no clear split decision,
that is a STOP+PING to Verification Manager before claiming Row 4 progress.

## Per-PR Receipt

Debt found + routed: Row 4 umbrella exact-string patching classes recorded
inline; live BR-19 routed to PB Tier-2 / Substrate v2-syntax work, retired
lower-helper class recorded as narrow evidence, and ambiguous future classes
routed to per-class split rows before umbrella closure.

This receipt does not close a Debt-Paydown row directly. Closure happens only
when PB/Substrate ships the actual retirement PRs and `bridge_ledger.dag` flips
the relevant row from `Open` to `Retired`, or when the umbrella is split into
per-class rows that retire independently.

## Test Plan

- `git diff --check`
