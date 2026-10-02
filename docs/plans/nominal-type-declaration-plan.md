# PLAN — one nominal type-declaration model; delete refinements

**Status: PLAN, revision 2. Design only; nothing lands until the operator rules on §9.**
Operator direction 2026-10-02 (work item adhoc-7e1dbee1-8e0); revision 2 recuts the plan around the
operator's ruling on gunbc#13024 rev 1, relayed by sharp-raven-357:

> **Alias is equality, constructor is progress, visibility is authority.**

Governing sections: DESIGN §2, §3 (replacement migrations), §4 (operations from inhabitance; the
coercion law), §4b, §5.

Every count in this document is **provisional** — line greps at main `dd0614551b5`, not the census.
They are not a size and are not to be quoted onward (DESIGN §6). The only admissible size is the
output of the resolved census (§2).

## 1. The principle, applied

| concern | after this program | replaces |
|---|---|---|
| **equality** | `type X = Y` — X and Y are the same type; no construction, no projection, no identity | (unchanged) |
| **progress** | a new type is introduced by a **data constructor**; a value of `X` exists only because a constructor ran | `where brand(...)`, `where <pred>`, `nominal_opaque` |
| **authority** | who may **construct** and who may **view** the representation are two independent visibility axes, governed by the declaring module | the three declaration modes |

Brand, refinement and opacity stop being declaration modes. They become **points in the
(constructor visibility × view visibility) plane** of one declaration form:

| old mode | raw constructor | checked constructor | representation view |
|---|---|---|---|
| brand (plain nominal tag) | public | — | public |
| refinement (validated type) | **private** | public | public |
| `nominal_opaque` (opaque) | private | as the module chooses | private or restricted |

A one-field nominal type **may lower** to the existing single-field record machinery (v2 records are
already nominal: `v2.compiler.infer` `infer_record_construct_type`). That is a realization choice: the
constructor is the semantic authority, not the record layout, and nothing downstream may read
nominality off the layout.

## 2. The resolved semantic census (M0)

The grep counts in circulation (~16.6k `"…" as T`, ~9.3k `as String|Int|Nat`, ~334 brand lines, ~31
refinement lines, 2 `nominal_opaque` declarations) are **not a size** — a spelling match cannot tell an
alias no-op from a nominal introduction. M2 is sized from this census only.

**Instrument:** `gunbc.instruments.type_declaration_use_census`, rostered as
`//gunbc/instruments:type-declaration-use-census` (a row in `instrument_registry`; DESIGN
"Building & checks"). It reads the **resolved** tree of the closure `dag` + `src/v2` — the declaration
each reference resolves to, never the leaf spelling (DESIGN §4: compatibility keys on the exact
declaration).

**Declaration classes** (every type declaration the program affects):

| class | definition |
|---|---|
| `TrueAlias` | `type X = Y` with no nominal use: every `as` into/out of it is an alias no-op |
| `UnconstrainedNominal` | brand-like: some site relies on X ≠ its body; no predicate |
| `CheckedConstructionStage` | carries a predicate today (refinement) or an `_of` checked constructor |
| `Opaque` | `nominal_opaque`, or a nominal whose view no consumer outside its module reads |
| `MultiFieldRecord` | record with >1 field (already nominal; listed so the census is total) |
| `Sum` | closed coproduct |

A declaration that matches no class is a **census refusal** (exit 2), not a default bucket.

**`as`-site classes** (every `as`, target resolved to its declaration):

| class | definition | M3 rewrite |
|---|---|---|
| `AliasNoOp` | target is a `TrueAlias` of the source | delete the cast |
| `NominalIntroduction` | base → nominal | constructor call (checked where the type is a stage) |
| `NominalProjection` | nominal → base, consumer genuinely needs the raw value | explicit view (`.value`) |
| `OperationWorkaround` | nominal → base only to compare, hash, encode, concatenate | the derived operation on the nominal directly (§5) |
| `GenuineConversion` | a declared conversion between distinct representations | unchanged (DESIGN §4 named conversion) |
| `Unrelated` | `as` in another grammar role (import rename, …) | unchanged |

Each row carries `{module, symbol | site, class, evidence_site}`; a classification without its
evidence site is a §5 fabricated output. Output also lists, by name, every affected module **outside**
`v2.workflow.required_floor required_gate_prefixes` (DESIGN §3: those do not refuse loudly).

**Controls:** a fixture closure with one declaration of each class and one `as` of each class must
classify exactly; a module the reference index did not cover must refuse exit 2 (could-not-tell is
never `TrueAlias` or `Unrelated`); and a mutation control — an introduction rewritten as a projection
— must change the classification.

## 3. One declaration form

**Equality.** `type X = Y`. Unchanged; the default.

**Progress.** A nominal declaration names its constructor(s); a one-field nominal over `Y` lowers to a
single-field record. Concrete surface syntax for the visibility axes is a §9 decision (S1); this plan
fixes the semantics:

- **In:** only by a constructor whose visibility admits the call site. A **raw** constructor is total
  over the representation; a **checked** constructor is an ordinary function in the declaring module
  that is the only public entry when the raw one is private.
- **Out:** only by the representation view, where its visibility admits the site. Opaque means there is
  no view outside the module.
- **Matching:** on constructors (destructure a nominal value where its view is visible) and on closed
  sums. **There is no typecase**, and no design for one.
- **No `as` between nominal types**, and no `as` that introduces or strips nominality — those are
  constructor calls and views. `as` remains only for declared conversions (DESIGN §4).

Composed stages build from the layer below: `SecretValue` is checked from `Secret`, `PositiveInt` from
`Nat`, `CTranslationUnitBasename` from `NonEmptyStr`. The crossing is an inhabitance judgment
(`v2.std.inhabitance`), never type equality.

## 4. Checked construction (D7)

A checked constructor is an ordinary function boundary. Its definition is "success advances the value
one stage; refusal leaves it where it was, with a reason" — `Optional` is a convenience shape, not the
definition. Vocabulary is existing (DFS'd, nothing minted):

- residue plus reason: `std.error_primitives` `Result<ok, err>` with `err` carrying the **original
  value** and a typed cause (`port_of(n: Int) -> Result<Port, PortRefusal>`, `PortRefusal { value: Int,
  cause: … }`);
- stage-level diagnostics where the boundary is a compiler stage: `v2.std.diagnostic` `Outcome<T>`;
- `T?` where no consumer needs the reason (the existing `std.types` `http_status_of`).

Whether evidence beyond the reason is carried (a witness that the predicate held) is §9 decision S3.

Candidates, per the provisional list (M0 decides each one's class and may demote a lower-bound-only
range nobody enforced to `TrueAlias`): `Octet`, `PrefixLength`, `Hextet`, `IpmiChannelNumber`,
`IpmiUserId`, `RetryCount`, `HttpStatus`, `Port`, `GitTreeEntryName{Before,After}SlashValue`,
`EpochSecs`, `EpochMs`, `Duration`, `PositiveFanCount`, `PositiveInt`, `NonEmptyStr`, `LanguageId`,
`SecretName`, `FilePath`, `GitRef`, `GcpProjectId`, `SecretValue`, `Sha1/Sha256/Sha512DigestHex`,
`Fnv1a64StructuralDigestHex`, the four `Oci*` syntax types, `ReadableEntry`,
`CTranslationUnitBasename`, and `unicode_scalar`'s users. Bounds come from data rows (as
`http_status_min/max` already do); the refinement's duplicated literals disappear with it.

## 5. Derived capabilities (D6)

A nominal type **derives or exports selected capabilities** from its representation — equality, hash,
ordering where meaningful, encoding where intentionally exposed. This is DESIGN §4's
"operations come from inhabitance" applied to a nominal: the type inhabits the structure its
representation inhabits, **by the declaring module's selection**.

- **Not subtyping, not coercion.** `RoadmapNodeId` has equality because its representation does; it
  is never usable where a `String` is expected. No implicit route is added (DESIGN §4 law untouched).
- **Selective.** A `Secret` does not acquire display or JSON because its field is a `String`. The
  default is **nothing derived**; each capability is opted into at the declaration.
- **One vocabulary.** Equality and ordering are `std.algebra` (`AlgebraProfile`,
  `AlgebraCarrier`, `algebra_profile_equality_extensional`, `Ordering`); hashing is `std.content_hash`
  `HashFamily`. Encoding has no single consumed home I could find in the DFS — §9 decision S2 (DFS
  further vs. name the home), never a second capability vocabulary.
- **Consequence for the census:** every `OperationWorkaround` cast (strip to compare/hash/encode) is
  replaced by the derived operation, which is why M0 must separate it from `NominalProjection`.

## 6. Deletion (D5, atomic, in the cutover)

Deleted in the same semantic cutover as the migration — no intermediate state where a brand and a
constructor both answer for one type:

- **Grammar:** `where` clause and `nominal_opaque` modifier — `src/v2/extdeps/languages/dag.dag`;
  `v1.compiler.parse` including the `"brand" =>` spelling match.
- **v2 stages:** the where-call path in `src/v2/compiler/03_resolve.dag`; the `RefinementDeclaration`
  frontier in `src/v2/compiler/04_infer.dag`; where lowering in `body_lowering_fold.dag`,
  `emit_produced.dag`.
- **v1 seed:** `v1.compiler.infer` `where_refinement_predicates_equivalent`,
  `where_predicate_literal_string_args_match`, `decidable_where_string_predicate_holds`, the brand arm;
  `v1.compiler.emit_rust` where handling; the hand tables.
- **Vocabulary:** `std.types` `brand`, `range`, `string_non_empty`, `gt_zero`, `WherePredicateMarker`;
  `std.content_hash` `lower_hex_*`; `unicode_scalar`; the OCI, `is_text_readable` and
  `c_translation_unit_basename_safe` predicate fns; **`gunbc.where_refinement_predicate_vocabulary`
  whole, with `WherePredicateGrounding`** — never left as a second authority.
- **Frontier:** #12506's parked where-predicate row is **retired**, not flipped to a judgment.
- **Rows naming the deleted machinery:** `dag/extdeps/languages/rust/types.dag` where rows,
  `gunbc.plans.p1_where_clause_lowering`, `gunbc.plan_registry_batch_e`, `gunbc.census_closure_frontier`,
  `gunbc.rung_drop.g0_type_decl_modifier_parse_without_sealing_property`,
  `gunbc.rust_source_type_bindings` Secret row, the where rows in `src/v2/workflow/floor_cost_debt.dag`,
  `floor_grandfathered_roster.dag`, `floor_pure_producer_share.dag`.
- **Tests whose subject is gone:** `test.claim.where_refinement_predicate_vocabulary_witness_test`,
  `where_clause_required_comma_test`, the where arms of `d1_declaration_grammar_parse_test`,
  `type_decl_modifier_g0_parse_probe_test`, where references in
  `edge_label/structural_label_readers_test`, `namespace_xl0/reference_conservation_test`.
- **Recurring-failure-mode rows** (`refinement_predicate_enforced_only_where_the_value_is_a_literal`,
  `parameter_refinement_has_no_carrier`, `authored_lexeme_read_as_a_grammar_marker`,
  `kernel_type_secret_has_no_v2_value_type_node`, `behavior_named_edge_label_validated_not_constructed`)
  are **updated, not deleted**: each records its climb.

This list is the known set; M0's out-of-gate enumeration completes it, and anything it names that the
cutover does not repair is a declared §4b(3) drop with population and trigger.

**v1 seed under `gunbc.v1_maintenance_standing` `v1_seed_standing`:** deletion of seed machinery the
self-host path no longer reaches is shrink, admitted. The seed must also **gain** constructor/view
visibility enforcement for the sources it compiles, or the refusals of §7 are lost on the seed route —
a silent regression; that gain serves v2 self-host and is admitted under the purpose test, sized by M1.
Stage0 mirrors are regenerated in each PR that edits `src/v1` (regen-fixed-point, `build` lane), never
hand-edited.

## 7. Controls (written expecting-red first)

The brand witness's six arms (`test.claim.brand_nominal_identity_witness_test`) are rewritten over
constructors and every refusal is preserved; each is run against current main before the wall lands.
Per DESIGN §4b(4) none retires.

| arm | fixture (semantic) | after | before |
|---|---|---|---|
| COLLISION | two nominals over `String`, value of A where B is declared | refuse | control (records already refuse) |
| MISMATCH | same, through a cast `a as B` | refuse | **red** if `as` admits it |
| CONSTRUCT | public raw ctor from `String` | accept | accept |
| DUAL | one declaration, two use sites | accept | accept |
| STRIP | `B` constructed from `A`'s view | accept | accept |
| UNDECLARED_SITE | `where brand(...)` on a parameter | refuse at parse | **red** |
| PRIVATE_RAW_CTOR | raw ctor of a validated type called outside its module | refuse | **red** (no visibility yet) |
| PRIVATE_VIEW | view of an opaque type read outside its module | refuse | **red** |
| NO_IMPLICIT_CAPABILITY | `RoadmapNodeId` where `String` is declared, equality derived | refuse | control |
| UNDERIVED_CAPABILITY | display/JSON of `Secret` with no opt-in | refuse | **red** |
| CHECKED_CTOR | `port_of(n: 0)` refuses with residue `0` and a cause; `port_of(n: 80)` advances; by execution | green | positive control |
| NO_WHERE / NO_OPAQUE | `type A = String where range(...)`, `type A nominal_opaque = String` | refuse at parse | **red** |

Each red is run on the v2 route and the seed route (rung is the minimum across paths, §4b(1)).

## 8. Migration order

1. **M0 — resolved census** (§2). Read-only.
2. **M1 — constructor/view visibility and capability derivation semantics**, with §7's controls, in v2
   and the seed. Lands with its reds; legacy forms still parse.
3. **M2 — mechanical rewrite, generated from the census**, never hand-edited:
   brand → public ctor + public view; refinement → private raw ctor + checked ctor + public view;
   `nominal_opaque` → private ctor/view; aliases untouched; `NominalIntroduction` → ctor calls;
   `OperationWorkaround` → derived operations; `NominalProjection` → explicit view; `AliasNoOp` →
   cast deleted. The generator is a fold over the census rows.
4. **M3 — exception review**: every site the generator could not classify or rewrite, listed by M0,
   reviewed by hand. The generator refuses rather than guessing.
5. **M4 — one semantic cutover PR**: the generated migration, the reviewed exceptions, and §6's
   deletion together. The deletion is the census's confirmation: anything that still refuses was
   missed by M0, and M0 is corrected rather than the site patched.

The N7 lane (calm-boar-904) is not touched; if M0 finds a brand or refinement in its 8 native tests,
M4 sequences after N7.

## 9. Residual operator decisions

Ruled: D1 (alias is equality), D2 (constructor + independent visibility axes), D3 (no typecase),
D4 (resolved census sizes M2), D5 (atomic deletion), D6 (selective derived capabilities), D7 (checked
construction may return residue). Left open:

- **S1 — surface syntax** for constructor visibility, view visibility and capability opt-in. Recommend:
  annotations on the declaration only (no new declaration keyword), so the one form stays one form;
  the operator picks the spelling.
- **S2 — encoding's home.** The DFS found equality/ordering in `std.algebra` and hashing in
  `std.content_hash`, but no consumed encoding capability authority. Recommend: a bounded DFS in M1
  before anything is named; if none exists, an operator ruling on the home before M1 lands.
- **S3 — evidence in checked construction.** Recommend: residue + typed cause only (`Result`); a
  carried proof witness waits for a consumer (DESIGN §3c).
- **S4 — default view visibility** for a brand-like nominal. Recommend: public (it is the old brand's
  behavior; the census will show how many already only project).
- **S5 — PR size.** M4 is one cutover; its size is unknown until M0 runs. Recommend: accept it as one
  generated PR whatever the number, since staging would make both forms observable.
