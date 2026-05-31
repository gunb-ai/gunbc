# v4 SG-1b Worksheet — Function-boundary signature realization (`TargetFunctionSignatureRealization`)

> **Status:** WORKSHEET APPROVED — Modeling DFS Manager §8 sign-off 2026-05-31 (`proud-pike-680`; msg_a2db9528). **READY-FOR-IMPLEMENTATION-DISPATCH** (Target Realization / keen-heron TR lane).
> **Date:** 2026-05-31
> **Dispatch anchor:** `docs/audit/v4-rustc-error-catalog-2026-05-31.md` §3 (SG-1 partial dissolution / SG-1-FOLLOWON); `#4086` P3 routing row **SG-1b function-signature String↔Symbol**; `docs/planning/v4-predicate-dependency-graph-2026-05-31.md` §3.3.
> **Sibling discipline:** sharp-otter-407 classification — **separate §10.0 worksheet**, NOT an amendment to `docs/planning/v4-sg1-target-atom-realization-worksheet-2026-05-30.md` (SG-1 atom authority closed via #3956).
> **Canonical home:** `src/v4/std/target_model.dag` (`v4.std.target_model`) — same module as SG-1/SG-2 carriers per `docs/design-target-realization-canonical-home.md` §1 Option A.
> **Prerequisite:** SG-1 worker PR **landed** (#3956); SG-2 worker PR **landed** (`signature_type_form` is `TargetTypeExpression` under SG-2 projection).

---

## Mechanical dispatch rule

> **No SG-1b implementation worker may land until this worksheet is complete and Modeling DFS Manager–approved.**

Acceptance is boundary-scoped falsification (§10.6 on signature sites only), not E0308 count reduction.

---

## §10.0-adapted worksheet

```text
SG class: SG-1b (sibling to SG-1; catalog label SG-1-FOLLOWON)
Representative emitted failure:
  pub fn loop_bound_edge() -> String {
      Symbol("loop_bound_edge".to_string())
  }
  // rustc E0308: expected `String`, found `Symbol` (~1317 instances post-#3956)
Immediate local patch:
  (a) Patch Arrow return serialization to always emit `-> String` when the
      substrate carrier is kernel Symbol; OR
  (b) Patch value path to emit `"x".to_string()` instead of Symbol(...) so
      the body matches the existing `-> String` annotation.
Why that patch is forbidden:
  (a) Cements binding_spellings / legacy atom-spelling as parallel authority
      at fn boundaries while SG-1 value path already consumes
      TargetAtomRealization — INVARIANTS P2 violation persists at the
      signature site only.
  (b) Reintroduces the SG-1-forbidden value-only template patch; leaves
      signature and value derived from different facts; blocks falsification
      that changing one realization row moves both together.
DFS path:
  std/ authority:
    - type Symbol bare at src/v4/std/node.dag:10 (kernel-ambient)
    - Arrow connective at src/v4/std/node.dag (fn signatures are Arrow-shaped)
  extdeps/language authority:
    - rust.dag: TargetAtomRealization rows for Symbol/Bool/Char (#3956)
    - rust.dag: NO function-boundary signature realization rows (2026-05-31)
  compiler stage consuming it:
    - 06_translate: value path uses translate_target_atom_realization_for_carrier
      + target_atom_value_expression (SG-1 consumer)
    - 06_translate: type-expression *serialization* for TargetTypeExprAtom uses
      type_expr_spelling_for_atom → target.binding_spellings (bypasses atom row
      ship type); Arrow codomain/domain sites inherit that path
      (serialize_type_expr_arrow_bounded → serialize_type_expr_emitted_bounded)
  existing scaffold/dissolution notes:
    - SG-1 worksheet + #3956 — atom catalog authority; do not reopen
    - node.dag:84-85 — Symbol-tagged Loop bound dissolution (T-12); no new
      Symbol-tag consumers in generated Rust
Deepest unsound boundary:
  Missing per-boundary signature realization fact. SG-1 row supplies
  type_form (decl-site) and value_form (value ship) but fn entry/exit sites
  need an explicit signature_type_form keyed by the same source_carrier,
  without duplicating the atom catalog.
Systemic fix:
  TargetFunctionSignatureRealization {
    source_carrier: Node,              // same Node key as SG-1 row
    target_model: TargetModel,
    boundary_site: FunctionBoundarySite // Param | Return (fn entry/exit)
    signature_type_form: TargetTypeExpression  // SG-2 vocabulary instance
  }
  Carrier ONCE in v4.std.target_model; per-language rows in
  extdeps/languages/<lang>.dag scoped to fn-boundary sites only.
  Rows MUST reference SG-1 catalog via source_carrier lookup — FORBIDDEN to
  duplicate parallel rust_target_atom_realization_* tables.
  Consumer: 06_translate Arrow serialization + fn-boundary atom type sites
  consult signature realization before binding_spellings fallback.
  Invariant: signature_type_form must be the type rustc expects for values
  produced by the paired SG-1 value_form at that boundary (String↔Symbol
  alignment for Symbol kernel atom on Rust).
Non-goals:
  - Amending v4-sg1-target-atom-realization-worksheet or reopening SG-1 scope.
  - Duplicating TargetAtomRealization rows under a second catalog name.
  - Global replacement of binding_spellings (non-boundary decl sites may still
    use binding_spellings until a separate ratified tranche says otherwise).
  - SG-RC-LAYERING, SG-2 generic arity, loop_bound_edge / T-12 dissolution.
  - E0308 / ~1317 count as acceptance.
Falsification probe:
  (1) Change rust TargetFunctionSignatureRealization Symbol return row
      (e.g. flip signature_type_form spelling/structure); verify emitted
      `pub fn ... -> T` changes WITHOUT mutating SG-1 type_form used at
      non-boundary type-alias decl sites.
  (2) Change SG-1 value_form for Symbol; verify fn return annotation AND
      body still agree (signature consumer must remain coupled via shared
      source_carrier lookup — if only body changes, boundary authority broken).
  (3) Grep 06_translate for fn-boundary atom spelling that bypasses
      target_function_signature_realization_lookup — expect zero after refactor.
Metric allowed only as secondary:
  ~1317 E0308 `expected String, found Symbol` — evidence only.
```

---

## Single-authority fact (dispatch gate)

| Fact | Authority home | Consumes | Forbidden parallel |
| ---- | -------------- | -------- | ------------------ |
| `TargetFunctionSignatureRealization` | `v4.std.target_model` (carrier once) | `TargetAtomRealization` lookup by `source_carrier` | Second atom catalog; binding_spellings at fn boundary; name-keyed `if symbol → String` |

**String↔Symbol scope (Pareto subset).** Post-#3956 probe (`docs/audit/v4-rustc-error-catalog-2026-05-31.md` §3): the class is the **function-boundary** residue where SG-1 aligned value construction but signature serialization still projects kernel `Symbol` through the pre-SG-1 spelling path. Rows are scoped to `FunctionBoundarySite` **Param** and **Return** only — not a second global atom table.

---

## Tightened worker brief (Target Realization — handoff to `keen-heron-687` post-§8)

```text
Implement TargetFunctionSignatureRealization for Rust fn boundaries.

Canonical carrier home:
  Define TargetFunctionSignatureRealization + FunctionBoundarySite ONCE in
  src/v4/std/target_model.dag. Per-language rows in extdeps/languages/rust.dag.

SG-1 coupling (hard):
  Rows keyed by source_carrier Node identity shared with SG-1 catalog.
  Lookup SG-1 TargetAtomRealization to validate pairing; FORBIDDEN to copy
  type_form/value_form fields into duplicate data lines.

Rows (initial Rust MVP1):
  At minimum: Symbol kernel atom × { Param, Return } boundary sites where
  post-#3956 probe shows String↔Symbol mismatch. Bool/Char only if probe
  shows the same boundary mismatch after SG-1 (do not speculative-expand).

Consumers:
  Refactor src/v4/compiler/06_translate.dag so Arrow/domain/codomain
  serialization at fn boundaries consults signature realization before
  type_expr_spelling_for_atom / binding_spellings.
  Non-boundary type-alias / decl-site paths keep SG-1 type_form authority.

Falsification:
  Deliver (1)–(3) from worksheet. Add or extend LeafModelClaim in
  src/v4/test/claim/language_model/rust.dag for fn-boundary coupling
  (signature_must_track_value_form) — PLANNED until this PR lands.

Non-goals:
  - Touch v4-sg1-* worksheet files.
  - Amend rust_target_atom_realization_* catalog membership.
  - Error-count ratchet as acceptance.
```

---

## §8 Manager approval checklist (`proud-pike-680`) — CLOSED 2026-05-31

- [x] Single-authority fact: `TargetFunctionSignatureRealization` in `v4.std.target_model`
- [x] Sibling discipline: SG-1 atom worksheet untouched; no implementation in worksheet PR
- [x] SG-1 consumption: rows lookup atom catalog by `source_carrier`; no duplicate catalog
- [x] Boundary scope: `FunctionBoundarySite` limits authority to fn entry/exit
- [x] Cross-section: `signature_type_form` uses SG-2 `TargetTypeExpression` only
- [x] Spot-fix forbidden: global `-> String` coercion; value-only Symbol strip
- [x] Falsification (1)–(3) accepted for worker dispatch
- [x] Worker dispatch — **authorized** to Target Realization Manager (keen-heron TR lane)

---

## Related artifacts

- `docs/audit/v4-rustc-error-catalog-2026-05-31.md` §3, §5 (SG-1-FOLLOWON routing)
- `docs/planning/v4-sg1-target-atom-realization-worksheet-2026-05-30.md` — atom authority (closed; not amended)
- `docs/planning/v4-sg2-type-expression-projection-worksheet-2026-05-30.md` — `signature_type_form` vocabulary
- `docs/planning/v4-correctness-ladder-2026-05-30.md` §10.0, §10.6, §11.1
- `docs/design-target-realization-canonical-home.md` §1–§2
- `src/v4/extdeps/languages/rust.dag` — `rust_target_atom_realization_*` (consume, do not duplicate)
- `src/v4/compiler/06_translate.dag` — `type_expr_spelling_for_atom`, `serialize_type_expr_arrow_bounded`
