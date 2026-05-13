# R3 Gate-87-D Ratchet Inventory Fail-Closed Audit — 2026-05-13

**Gate:** `lens_cementing_test_discipline_complete` (gate-#87, status `CONSUMER_LANDED + PASSING`
per [`docs/r3-program-plan.md`](../r3-program-plan.md) row #87).

**Scope:** verify each merge-visible single-authority surface listed in
[`docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md`](../briefs/r3-cementing-discipline-pattern-2026-05-12.md)
§1 is fail-closed against drift in every direction — i.e. a worker adding, deleting, or
mutating any one surface without the matching surfaces causes a deterministic test
failure rather than silent fail-open.

**Not in scope:** the broader Band-C dispatch (`cementing_dispatch.dag`) beyond what
the gate-#87 `regen.dag` registry ∩ register-v2-complete projection requires; #84
cementing-class bulk-port disposition (covered separately by
[`r3-cementing-discipline-pattern-2026-05-12.md`](../briefs/r3-cementing-discipline-pattern-2026-05-12.md) §3).

---

## §1. Surfaces under audit

| ID | Surface | Authority |
|---|---|---|
| A | `LensRegistryEntry` rows in `src/v3/compiler/regen.dag` | DSL data — sole inventory of compiler-generated lenses |
| B | `R3_GATE_87_CEMENTING_REGEN_SUITES` in `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs` | Single-authority T-PB-B-1 runner table |
| C | `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_<lens>.dag` harness files | On-disk Band-C receipt artifacts |
| D | `cementing_band_c_v2_complete_receipts` list in `src/v3/compiler/tests/dag/cementing_dispatch.dag` | Band-C dispatch enumeration |
| E | `tests/integration.rs` `#[path = "integration/cementing/<stem>.rs"] mod <stem>;` bindings | `temporary-rust` Band-C receipts |

Per `INVARIANTS.md` P2 (single authority), B is the only inventory for which
harnesses `t_pb_b_1_dag_runner_test` executes; A is the only inventory of compiler-generated
lenses; D is the only dispatch list. C is enforced by name correspondence to B (stem
match) and on-disk existence; E is enforced as a peer of C for the `temporary-rust` kind.

## §2. Direction-by-direction ratchet matrix

For each ordered pair `(X → Y)` — "X shipped without matching Y" — list the test or
compile-time check that fail-closes.

| From → To | Failure scenario | Fail-closed mechanism |
|---|---|---|
| A → B | New `LensRegistryEntry` lands without runner-suite row | `r3_gate_87_regen_lens_registry_names_match_fixture_inventory` (BTreeSet `assert_eq!`) — `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs:134-144` |
| B → A | New runner-suite row lands without `regen.dag` registry entry | Same `assert_eq!` (set equality is symmetric) |
| B → C | Runner-suite row points at missing `.dag` file | `include_str!` in `R3_GATE_87_CEMENTING_REGEN_SUITES` literal (`r3_gate_87_cementing_regen_runner_suites.rs:63-124`) — compile-time `error[E0464]` if file missing |
| C → B (orphan dag file in scope, with regen entry) | Worker adds `tests/dag/t_r3_gate_87_cementing_regen_<NEW>.dag` plus a matching `LensRegistryEntry` but forgets the suite row | Caught indirectly: registry name `NEW` appears in A but not in B → `assert_eq!` from row 1 fails |
| **C → B (orphan dag file out of scope)** | Worker adds `tests/dag/t_r3_gate_87_cementing_regen_<orphan>.dag` with no matching `LensRegistryEntry` and no suite row | **NOT FAIL-CLOSED.** No filesystem scan of the prefix exists. See §3. |
| D → A | Dispatch row references a registry name not in `regen.dag` | `evaluate_cementing_dispatch_projection` — `cementing_dispatch.rs:391-397` ("`lens_capability_register` escalates v2 cementing for lens basenames … but no `LensRegistryEntry` …") |
| D → B | Dispatch `kind=="dag"` row references a `module_stem` not in `R3_GATE_87_CEMENTING_REGEN_SUITES` | `cementing_dispatch.rs:464-471` ("stem is not wired in `R3_GATE_87_CEMENTING_REGEN_SUITES`") |
| D → C | Dispatch `kind=="dag"` row references a missing harness file | `cementing_dispatch.rs:476-482` (`path.is_file()` check) |
| A ∩ register-v2-complete → D | New `regen.dag` row whose basename is V2-complete in `lens_capability_register_rows`, but no matching `(registry_name, module_stem, kind)` triple in `cementing_dispatch.dag` | `cementing_dispatch.rs:428-435` (`receipt_triples != expected_triples`); also `expected_cementing_receipt_triples` returns explicit `Err` (line 240-247) when its expansion table is missing |
| D → E | Dispatch row with `kind=="temporary-rust"` whose stem is missing as a `tests/integration/cementing/<stem>.rs` file or whose `#[path]` binding is absent in `tests/integration.rs` | `cementing_dispatch.rs:484-507` (`path.is_file()` and `integration_rs_cementing_path_attr_binds_mod_stem` checks) |
| Receipt duplication | Two `cementing_receipts` rows share `(registry_name, module_stem, kind)` | `cementing_dispatch.rs:420-425` (`!receipt_triples.insert(triple.clone())` branch) |
| Dispatch declaration leak | A claim of shape `CementingDispatchMatchesProjection` declared in any file other than `**/cementing_dispatch.dag` | `cementing_dispatch.rs:361-366` (declaration-file gate) |

All checks above except the marked row are fail-closed by either (a) a compile-time
`include_str!` resolution, (b) a `BTreeSet` `assert_eq!`, or (c) an explicit `Err`
return in `evaluate_cementing_dispatch_projection` that maps to `ClaimResult::Fail`.

## §3. Residual fail-open gap

**Finding: orphan harness file under the `tests/dag/t_r3_gate_87_cementing_regen_` prefix is undetected.**

A worker can land a file matching the gate-87 prefix without either (i) a runner-suite
row in `R3_GATE_87_CEMENTING_REGEN_SUITES`, or (ii) a `LensRegistryEntry` row whose
`name` matches the prefix stem. Such a file is silently dead:

- `r3_gate_87_regen_lens_registry_names_match_fixture_inventory` compares A↔B only;
  neither set sees the orphan file.
- `cementing_dispatch.rs` checks A→D and D→{A,B,C,E}; no surface walks the C
  prefix outside of stems referenced by D or B.
- `t_pb_b_1_dag_runner_test` iterates `R3_GATE_87_CEMENTING_REGEN_SUITES` only.

**Blast radius:** narrow. The orphan file is not executed (no runner picks it up) and
not registered in any single-authority surface, so its behavior cannot regress live
gate-87 semantics. The harm is shape (a) substrate-name reservation drift — a future
worker may pick the same stem for a real receipt and silently shadow the orphan, and
(b) reviewer load — diffs may carry dead `.dag` text.

**Severity:** Mgr-tier follow-up, not gate-#87 status-changing. Gate-87 remains
`CONSUMER_LANDED + PASSING`: every *live-on-main* `LensRegistryEntry` carries its
paired Band-C receipt, and every receipt enumerated in D resolves to a real file +
runner-suite row. The gap is at the orphan-file boundary, which by construction
cannot be referenced by any other surface.

**Proposed closure (not implemented in this audit):**

Add a single integration test (sibling of
`r3_gate_87_regen_lens_registry_names_match_fixture_inventory`) that

1. `fs::read_dir`s `src/v3/compiler/tests/dag/`,
2. filters entries whose filename starts with `t_r3_gate_87_cementing_regen_` and
   ends with `.dag`, strips prefix + suffix to get the lens-name stem,
3. asserts the set equals
   `r3_gate_87_cementing_regen_lens_names_for_runner_table()` from
   `r3_gate_87_cementing_regen_runner_suites`.

Once landed, this closes C→B in both directions and removes the last fail-open
surface from the gate-#87 ratchet. The follow-up is a single hand-written `#[test]`
function with no substrate change — it consumes the existing single-authority
`R3_GATE_87_CEMENTING_REGEN_SUITES` and the existing prefix/suffix constants
already exported from `r3_gate_87_cementing_regen_runner_suites`.

## §4. Cross-references touched but not reaudited

- `EXPECTED_HAND_AUTHORED_TEST` in `src/v3/compiler/tests/integration/sg0_census_test.rs` —
  ratchet of SG-0 hand-Rust test census. Its own ratchet is gate-#84 scope per
  `r3-cementing-discipline-pattern-2026-05-12.md` §3; outside gate-#87 fail-closed
  surface (the cross-coupling is "PR that ports any Rust receipt in this lane
  decrements the census in the same PR" — a worker discipline statement, not a
  structural ratchet).
- `lens_register_correspondence_test` — ratchets `docs/v3-lens-capability-register.md`
  prose table against `lens_capability_register_rows` structural authority; feeds D
  via `lens_capability_register_v2_cementing_basenames`. In scope of gate-#87
  via the V2-complete projection but the ratchet itself is upstream — not audited
  here.
- `t_pb_b_1_dag_runner_test` — executes the runner; failure modes are receipt-body
  rather than inventory drift. Not in fail-closed-inventory scope.

## §5. Conclusion

Gate-#87 inventory ratchet is **fail-closed in every direction except orphan-file
C→{A,B} for files matching the prefix without any registry or suite peer.** The
residual gap is small-radius and structurally cannot affect live gate-#87 behavior;
closure is a one-test follow-up.

No status change recommended for `lens_cementing_test_discipline_complete`.
Recommend filing the §3 follow-up as a Mgr-tier work item bound to Phase 3 /
#84 sequencing.
