# PRE-REGISTRATION — alias-typed `data` initializer repair (registered 2026-08-23, before the repair was built)

Committed **before** the repair commit and before any after-arm exists. The commit that introduces
this file changes no emitter code — that is the point of it being separate.

## Subject and before-arm

| field | value |
|---|---|
| subject | `907f19c2cc7cf31d2525236c93d3c92332182cde` — first main sha whose floor reports `failed=0` |
| before-arm log | `docs/probes/board_907f19c2cc7/03_ingest.cargo.log`, 234163 bytes |
| sha256 | `b39d6f428920eccb8629cc731a35c6c1555eb548b20e0d13edc65c66ab8d40fb` — **verified before classifying** |
| before-arm E0308 | **128** |
| before-arm coded rows | **316** |
| entry | `src/v2/compiler/03_ingest.dag`, M=1 |

The before-arm is not re-measured: it is the retained board log, and this branch is based on the
sha that produced it. Verifying the sha256 is what makes reuse legitimate rather than a stale read.

## The defect, and a correction to my own first statement of it

Reported to `smart-ram-730` as *"a `data` row whose declared type is an alias to a **generic**
instantiation loses its outer record wrapper."* **That was too narrow, and the fixture refuted it
before the repair was written.** A non-generic alias fails identically. The 2×2:

| declared type | emitted initializer | correct? |
|---|---|---|
| `WrapKeyAlias = Wrap<Key>` (alias, generic) | `Key { lens: true }` | ✗ wrapper dropped |
| `PlainAlias = Plain` (alias, non-generic) | `Key { lens: true }` | ✗ wrapper dropped |
| `Wrap<Key>` (inline, generic) | `serde_json::from_value(json!({"concern": …}))` | ✓ |
| `Plain` (inline, non-generic) | `serde_json::from_value(json!({"concern": …}))` | ✓ |

**The axis is aliasing, not genericity.** Genericity was a property of the first instance found, not
of the defect. Recorded here rather than quietly narrowed later, because a pre-registration that
gets its own mechanism wrong and then corrects it after the result is worthless.

The two arms take **different emit paths** — the alias arm builds a literal, the inline arm goes
through `serde_json`. So the alias *selects* a path rather than mislabelling a shared one, and a
repair that only corrected the rendered type name would go green while emitting the same wrong bytes.

## Prediction

- **E0308 128 → 115** (−13), **coded board 316 → 303**.
- The 13 are the `v2_lens_coverage.rs` rows: `expected Coverage<Rc<CoverageDefectAcceptanceKey>>,
  found CoverageDefectAcceptanceKey`, one per `data coverage_defect_*` row sharing
  `type CoverageDefectAcceptance = Coverage<CoverageDefectAcceptanceKey>`.
- **Minimum removable unit is 13, not 1.** One decision emits all 13; the repair removes 13 or 0.

### Predeclared risk that the repair OVER-delivers

Because non-generic aliases share the defect (row 2 of the 2×2), **any** alias-typed `data` row in
the closure is a candidate, and I have not enumerated the corpus. So the outcome may be *more* than
−13. That is registered **now, before the after-arm runs**, so that an over-delivery is a
predeclared possibility rather than a result narrated into a success. If it exceeds −13 I report the
extra rows individually and say they were not in the predicted set.

**Rows I explicitly do NOT predict:** the 3 `std_realization_schedule.rs` `Measure<…>` rows my
classifier keys to the same `D` root. They are a different defect sharing a root label. Not folded
in to make the number larger.

## Refusal conditions — what counts as this having failed

1. The after-arm does not reproduce the before-arm's non-target rows (anything other than the 13
   moving means the repair is not single-axis and the claim is withdrawn).
2. `via_direct` / `via_plain_direct` stop being green — the positive controls must not move.
3. The board falls by a number I cannot attribute row-by-row.

A fall to exactly 115 with the 13 named rows gone and nothing else changed is the only clean pass.

## Evidence enrolled (both arms, per DESIGN §4b(4))

The fixture is enrolled as executing evidence, not only the red arm: `via_alias` / `via_plain_alias`
are the discriminating REDs that must go green, and `via_direct` / `via_plain_direct` are the
positive controls that must **stay** green. A climb deletes the production machinery it obsoletes
and keeps the evidence; deleting the controls would recreate specification-without-execution one
rung up.

## v1 admission

This edits the v1 seed. Admitted under the **purpose test** in `gunbc.v1_maintenance_standing`
`v1_seed_standing`: the wrong bytes are in v2's own emitted mirror, so the change serves the v2
self-host program directly. The freeze is not repealed by this — v1 stays closed to growth for its
own sake, and this is a defect repair on the self-host path.
