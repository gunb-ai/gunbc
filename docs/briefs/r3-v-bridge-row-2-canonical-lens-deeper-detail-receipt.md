# R3 Verification - Bridge Row 2 Canonical Lens Deeper-Detail Receipt

**Status:** AUDIT RECEIPT - docs-only. This receipt does not flip any
`BridgeLedgerRow.status` and does not close a Debt-Paydown row directly.

**Row:** `bridge_canonical_lens_name_patching_residual`.

**Primary input:** `docs/briefs/bridge-retirement-audit-include-str-family.md`
(`crisp-newt-163` family B packet). This receipt consumes the packet's
canonical-lens subset by BR id: BR-07, BR-08, and Appendix B's
`canonical_lens_bridge_ratchet_test.rs` synthetic `include_str!` harness. BR-06
and BR-17 are named as sibling/blocker edges because the packet explicitly ties
them to the same canonical-lens byte wave, but this row does not absorb their
entire source-text-patching scope.

**Boundary inputs:** `docs/briefs/r3-v-bridge-row-1-sourcespan-deeper-detail-receipt.md`
and `docs/briefs/bridge-retirement-audit-sourcespan-family.md` are boundary
checks only. Row 1 owns file-stamped reflection participation; Row 2 owns
canonical lens byte inclusion, lens-name dispatch, and name-keyed lens-body
selection.

**Canonical disposition input:** `docs/briefs/r2-pb-canonical-lens-bridge-disposition.md`
is the current precise live-state disposition for `test_runner.rs`: canonical
lens `include_str!` bytes, `lens_decl.name == Some("...")` arms, and generic
name-keyed lens lookup remain blocked on PB-Runtime interpreter-as-data or a
typed lens-registry carrier. `docs/briefs/r3-pb-t-lensproducer-sub1-lens-apply-retirement.md`
confirms PB-Runtime/lens-apply retirement is the owner-program path for the
disposition's PB-Runtime option.

## Verification Consumption Rule

Verification should reject a PR that claims to retire a canonical-lens-name
entry unless it demonstrates that **all canonical lens identity surfaces for the
entry are gone or converted to typed carrier consumption**. Removing only one
visible surface is insufficient. In particular, a retirement PR must not leave
behind any of:

1. `include_str!` byte inclusion for canonical lens bodies in production runner
   or canonical-lens gate tests;
2. `lens_decl.name.as_deref() == Some("cost_of")` /
   `Some("named_function_count")` dispatch arms;
3. generic `lens_decl.name.as_deref()` lookup that selects a lens body by name
   from `program_dag`;
4. name-keyed compilation paths that recompile `R1_CANONICAL_*_LENS` bytes to
   obtain a cross-Dag canonical lens body;
5. ratchets that still pin the retired bridge as live rather than shrinking or
   deleting with the same structural replacement.

At review time, each retirement PR must cite the BR id and show that the
replacement carrier is consumed by the runner/lens path in production code. A
fixture-local `DeclarationRef` alone is not enough if the runner still resolves
the executable lens body through a second byte channel or string name.

## Cross-Packet Boundary Discipline

Row 1's receipt pre-coordinated the overlap: Row 1 counts real-path /
logical-file identity stamped into extra Dags and then consumed by
`reflect_program_dag_nodes_in_file` / fold participation. Row 2 counts canonical
lens bytes and lens-name identity. Rows 3 and 4 will own the broader
`include_str!` side-channel and exact-string patching rows.

| Family B entry / surface | Row 2 owns | Row 2 does not own |
| --- | --- | --- |
| BR-07 `test_runner` canonical lens public consts | `R1_CANONICAL_NAMED_FUNCTION_COUNT_LENS` and `R1_CANONICAL_COMPLEXITY_LENS` as canonical lens byte authorities; runner paths that recompile those bytes for lens identity. | Row 1's file-stamped reflection partition after those bytes become a Dag; generic `include_str!` side-channel accounting outside canonical lenses. |
| BR-08 `m1_5_user_authored_lens_gate_test` on-disk vs runner bytes | The byte-equality bridge between on-disk canonical lens text and runner canonical lens constants. | Hermetic fixture halves that are scenario input only; Row 1 file identity if the compiled fixture's virtual path drives participation. |
| Appendix B canonical-lens ratchet synthetic `include_str!` | The ratchet as a reviewer-visible pin for BR-07 counts; it must shrink/delete when BR-07 retires. | The synthetic `include_str!("a.dag")` / `"b.dag"` strings are not independent authorities. |
| `r2-pb-canonical-lens-bridge-disposition.md` name arms | `cost_of`, `named_function_count`, and generic name-keyed lens body selection as Row 2's name-dispatch surface. | Row 1 duplicate-authority/name-preference entries #14/#15/#16, which concern broad declaration preference by source file, not canonical lens dispatch. |
| BR-17 `lens_apply.rs` unit test lens bytes | Optional same-wave cleanup when it is solely canonical lens fixture authority for `named_function_count.dag`. | Full `lens_apply.rs` retirement and reflection semantics; those remain PB-Runtime sub-gate 1 / Row 1 boundary territory. |
| BR-06 R1 gates fixture splice | Consistency pressure when the splice embeds the same canonical lens body bytes as escaped fixture text. | Exact-string/sentinel fixture splicing as a row 4 source-text patching class unless the PR also removes canonical lens identity surfaces. |

This receipt deliberately does not expand Row 2 into BR-14, BR-15, BR-16,
BR-18, BR-19, or BR-A. Those entries remain Row 1 / Row 3 / Row 4 material per
their packet boundaries.

## Per-Entry Receipts

Rows follow the family B packet's leaf-first wave order for the canonical-lens
subset, with the canonical disposition's name-dispatch surfaces included where
they are the live Row 2 bridge rather than a separate BR id.

| Order | Entry | Verification-side reviewer rule | Carrier consumer required | Cross-row impact |
| --- | --- | --- | --- | --- |
| 3 | BR-17 `lens_apply.rs` unit-test canonical lens bytes | If a PR claims canonical-lens byte retirement and touches these tests, the tests must stop treating `../../lenses/named_function_count.dag` as a second canonical byte authority. A shared helper that only moves the string read is not retirement unless the test consumes the same structural lens identity as production. | Ideally the same typed lens registry, `program_dag` lens body identity, or PB-Runtime lens loader used by BR-07. If kept test-only, it remains adjacent cleanup and does not close Row 2 alone. | Optional same-wave cleanup with BR-07. Full `lens_apply.rs` retirement remains PB-Runtime sub-gate 1, not this receipt. |
| 10a | BR-07 `test_runner` canonical lens public consts | Delete `R1_CANONICAL_NAMED_FUNCTION_COUNT_LENS` / `R1_CANONICAL_COMPLEXITY_LENS` as production byte authorities, and remove runner recompilation paths that use those bytes to manufacture canonical lens identity. | PB-Runtime interpreter-as-data, typed lens registry, or structural `program_dag` lens body identity consumed by `LensOutputEquals` / lens application. The replacement must preserve cross-Dag declaration-id coherence without another path/string registry. | Row 1 owns file-stamped reflection participation after compilation. Row 3 may count generic `include_str!` channel mechanics, but Row 2 owns these canonical lens bytes. |
| 10b | Canonical disposition name-dispatch arms | **Retired by R3 gate #33:** `LensOutputEquals` no longer has `lens_decl.name.as_deref() == Some("cost_of")` or `Some("named_function_count")` semantic dispatch arms. | Typed marker declarations in `std.verification` (`CanonicalCostLens`, `CanonicalNamedFunctionCountLens`) select the two legacy runner paths without matching function-name strings. | Remaining Row 2 surface is BR-07/BR-08 byte authority, not name dispatch. |
| 10c | Canonical disposition generic name-keyed lens lookup | **Retired by R3 gate #33:** generic `lens_decl.name.as_deref()` lookup no longer calls `program_dag.declaration_by_name(name)` for id-space or lens-program selection. | Ordinary non-canonical `LensOutputEquals` lenses execute from their fixture `DeclarationRef`; canonical named-function-count still compiles canonical bytes until BR-07 closes. | Adjacent Row 1 broad `declaration_by_name` retirement remains separate. |
| 10d | BR-08 user-authored lens gate on-disk vs runner bytes | Remove byte equality / import coupling between the gate test and `R1_CANONICAL_NAMED_FUNCTION_COUNT_LENS`; the test must consume the same structural canonical lens identity as runner production. | Same carrier as BR-07, visible to the integration test through structural fixture/test-claim identity rather than duplicate on-disk bytes. | BR-08 also has hermetic fixture bytes; those are not Row 2 unless they assert canonical lens byte identity. |
| 10e | Appendix B canonical lens ratchet | When BR-07/BR-08/name-dispatch surfaces shrink, update or delete the ratchet so it pins only live residuals. A retired bridge must not remain documented as an expected positive count. | No new carrier; consumes the same production counts as a verification ratchet. | Ratchet is evidence for Row 2 only, not an independent bridge entry. |
| 11 | BR-06 R1 gates fixture splice, canonical-lens slice only | If a canonical-lens retirement PR edits `emit_r1_gates_fixture`, it must remove duplicate canonical lens text splicing rather than merely renaming sentinels or moving escaped bytes. | Structural `TestClaim` / declaration-ref lens-body carrier or single substrate-owned generator that embeds lens identity without parallel text authority. | Row 4 owns exact-string/sentinel splice retirement generally. Row 2 counts only the canonical-lens byte identity portion if it lands in the same wave as BR-07/BR-08. |

## Substrate/PB Routing Notes

Row 2 routes primarily to PB-Runtime / T-LensProducer-Retirement:

- PB-Runtime interpreter-as-data remains the preferred owner-program path for
  retiring the remaining canonical byte bridge while preserving the P2
  cross-Dag reflection invariant.
- A typed lens-registry carrier remains the Substrate alternative if PB-Runtime
  does not provide enough structural lens identity. That carrier must not be a
  new string/path registry.
- `program_dag` lens body identity is acceptable only when the runner no longer
  recompiles separate canonical bytes; behavior selection through
  `lens_decl.name` was retired by R3 gate #33.
- BR-08 and BR-17 should consume the same production structural lens identity as
  BR-07, or remain adjacent test cleanup rather than row closure.
- BR-06's canonical-lens splice pressure should be coordinated with Row 4
  exact-string/source-text patching so fixture generator cleanup is not counted
  twice.

No STOP+PING is triggered by this receipt. The known substrate/PB alternatives
are already named in the canonical-lens disposition and PB-Runtime sub-gate
brief. Any implementation PR that discovers a third replacement shape must route
that shape through Substrate/PB before claiming Row 2 retirement.

## Per-PR Receipt

Debt found + routed: canonical-lens per-entry retirement roadmap recorded
inline; PB-Runtime carrier asks and any typed lens-registry substrate alternative
routed through T-LensProducer-Retirement / T-Bridge-Retirement.

This receipt does not close a Debt-Paydown row directly. Closure happens only
when PB/Substrate ships the actual retirement PRs and `bridge_ledger.dag` flips
the relevant row from `Open` to `Retired`.

## Test Plan

- `git diff --check`
