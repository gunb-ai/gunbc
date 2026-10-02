# PLAN — one nominal type-declaration model; delete refinements

**Status: PLAN. No implementation lands until the operator rules on the decisions in §7.**
Operator direction 2026-10-02 (work item adhoc-7e1dbee1-8e0). Governing sections: DESIGN §2, §3
(replacement migrations), §4 (coercion law, text identity), §4b, §5.

## 0. What the corpus holds today (provisional — grep, not the instrument)

These figures come from a line grep over `dag/` and `src/v2/` at main `dd0614551b5`. They are
**not** the census; §1 names the instrument that replaces them, and its first run supersedes every
number here. Per DESIGN §6 they are not to be quoted onward.

| form | grep count | mechanism today |
|---|---|---|
| `type X = Y` (no `where`, no modifier) | ~556 lines | transparent alias |
| `type X = Y where brand("…")` | ~334 | nominal by brand literal; v1 seed recognises `brand` by spelling (`v1.compiler.parse`, `v1.compiler.infer` `where_refinement_predicates_equivalent`) |
| `type X nominal_opaque = Y` | 2 declarations (`std.types` `Secret`, `extdeps.access.posix_effective_principal`) | second nominal mechanism |
| `type X = Y where <pred>` (real refinements) | 31 lines, ~27 distinct types | `range`, `string_non_empty`, `gt_zero`, `lower_hex_16/40/64/128`, `oci_path_component_syntax`, `oci_tag_syntax`, `oci_other_digest_algorithm`, `oci_other_digest_encoded`, `is_text_readable`, `c_translation_unit_basename_safe` (and `unicode_scalar` per the carry-over list) |
| single-field records | — | already nominal in v2 (`v2.compiler.infer` `infer_record_construct_type`) |

The brief's alias figure (~324) and mine (~556) disagree by a factor that a line grep cannot
explain; that disagreement is itself the reason M0 is an instrument and not a grep.

## 1. The use census (instrument, M0)

**Instrument:** `gunbc.instruments.type_declaration_use_census`, rostered as
`//gunbc/instruments:type-declaration-use-census` (a row in `instrument_registry`, DESIGN
"Building & checks": a new measurement is a row, not a flag). It does not exist yet; it is the first
landing of this program.

It walks the annotation-erased declaration tree of the closure `dag` + `src/v2` (not text) and, for
every type declaration of form alias / brand / `nominal_opaque` / refinement, joins it against its
**use sites** from the resolved reference index:

- **(a) nickname / instantiation name** — every use site is interchangeable with the body: no `as`
  into or out of it, no site whose acceptance would change if the name were replaced by its body.
- **(b) distinct type over a representation** — some site relies on the name *not* unifying with its
  body or a sibling: an `as` cast into it (construction), a cast out of it (projection), or a
  signature where a sibling over the same body would be a defect (the `HostIdentity` collision the
  brand witness records).
- **(c) discriminant** — the name's identity is *branched on*: it appears as a coproduct variant
  payload that a `match` eliminates, as a key/column of a dispatch table, or as a type argument
  that selects a projection (`Vendor<Domain>`-style; `std.os` per-vendor rows).

Output: one row per declaration `{module, symbol, form, class, witness_site}` where `witness_site`
is the reference that put it in (b) or (c) — a classification without its evidence site is a
§5 fabricated output. A declaration with no use site is reported as **unconsumed** (DESIGN §3c),
not folded into (a). The instrument refuses (exit 2) if the reference index is incomplete for any
module in the closure — an unindexed module is could-not-tell, never (a).

**What I can say without the instrument, and stake as a falsifiable prediction:** the language has
no typecase, so (c) cannot exist today as "switch on which named type a value is"; wherever it is
real, the corpus already spells it as a coproduct whose variants carry the named types, or as a
type argument (`Vendor<Domain>`). If M0 finds a (c) site that is neither — a brand literal compared
as a string, a dispatch keyed on a type's spelling — that is a finding against this plan's §2.3 and
reopens it.

## 2. One declaration model

There are exactly **two** type-declaration forms after this program, and they are not two identity
mechanisms — one introduces no identity, the other is the only thing that does:

### 2.1 Default: `type X = Y` is a transparent name (class a)

Unchanged in meaning: an alias, identity coercion, no construction, no projection. This is the
default because the operator ruled the nickname use intended and because it is the overwhelming
majority (§0). It carries no `where` and no modifier, ever.

It is **not** a nominal type and makes no safety claim (DESIGN §4b: a richer name is not safety).
A nickname for a concept already named elsewhere is still DESIGN §3's nicknaming; that is a review
matter, not a type-system one, and this plan does not change it.

### 2.2 Explicit: a record is the only nominal identity (class b)

`type X { value: Y }` — a single-field record. Records are already nominal in v2, so this adds no
mechanism; it deletes two (`brand`, `nominal_opaque`).

- **In (construction):** `X { value: y }`. For a type with an invariant, the **checked constructor**
  `fn x_of(y: Y) -> X?` is the only intended entry: it builds `X` from the layer below or answers
  `Absent` (the existing `std.types` `http_status_of` shape — assembly line: a value that is not what
  we need is rejected one layer down, never produced and then narrowed).
- **Out (projection):** `x.value`. Total, exact, explicit.
- **No `as` between nominal types.** `as` is no longer how a brand is asserted; a cast from `X` to a
  sibling record over the same body has no declared route and refuses (§6).

Where a type composes another nominal type (`SecretValue` over `Secret`, `PositiveInt` over `Nat`,
`CTranslationUnitBasename` over `NonEmptyStr`), the checked constructor takes the *lower nominal
type*, not its representation: `secret_value_of(s: Secret) -> SecretValue?`. The crossing between
carriers is an inhabitance judgment (`v2.std.inhabitance`), not type equality (sharp-raven-357
carry-over (1)).

### 2.3 Discriminant (class c) is a coproduct, not a type-identity match

"Switch on which named type a value is" is written as a closed coproduct whose variants carry the
nominal records: `type Payload = Http(HttpStatus) | Port(Port)`, eliminated by `match`. Reasons:

- A match over *open* type identity (typecase) is not exhaustive, so it cannot sit on the floor's
  "closed variants eliminate exhaustively" (DESIGN §4b); a coproduct is closed by construction.
- Typecase makes the alias/record distinction observable at runtime, so changing a nickname into a
  record would silently change control flow — a meaning fork (§3).
- Where the discrimination is at the type level (`Vendor<Domain>`), it is already a type argument
  and needs nothing new.

So no third form exists. If M0 refutes the §1 prediction this subsection reopens before M2.

## 3. Refinement deletion

### 3.1 Each refinement becomes a record plus a checked constructor

| type(s) | new body | checked constructor (one layer down) |
|---|---|---|
| `Octet`, `PrefixLength`, `Hextet`, `IpmiChannelNumber`, `IpmiUserId`, `RetryCount`, `HttpStatus`, `Port`, `GitTreeEntryName{Before,After}SlashValue`, `EpochSecs`, `EpochMs`, `Duration`, `PositiveFanCount` | `{ value: Int }` | `<t>_of(n: Int) -> T?` with bounds from data rows (as `http_status_min/max` today — the refinement's duplicated literals disappear) |
| `PositiveInt` | `{ value: Nat }` | `positive_int_of(n: Nat) -> PositiveInt?` |
| `NonEmptyStr`, `LanguageId`, `SecretName`, `FilePath`, `GitRef`, `GcpProjectId` | `{ value: String }` | `<t>_of(s: String) -> T?` |
| `SecretValue` | `{ value: Secret }` | `secret_value_of(s: Secret) -> SecretValue?` |
| `Sha1/Sha256/Sha512/Fnv1a64StructuralDigestHex` | `{ value: String }` | `<t>_of(s: String) -> T?` in `std.content_hash` |
| `OciPathComponent`, `OciTag`, `OciOtherDigestAlgorithm`, `OciOtherDigestEncoded` | `{ value: String }` | in the OCI extdeps module, beside the grammar they cite |
| `ReadableEntry` | `{ value: FileEntry }` | `readable_entry_of(e: FileEntry) -> ReadableEntry?` |
| `CTranslationUnitBasename` | `{ value: NonEmptyStr }` | takes `NonEmptyStr` |
| `unicode_scalar` users | per M0 | per M0 |

`std.types` currently declares `Port`, `SecretValue`, `NonEmptyStr` twice; the rewrite keeps one.
Some `EpochSecs`/`Duration`-style ranges with only a lower bound may be (a) in practice — M0 decides
whether each keeps a constructor or becomes a plain alias (dropping a check nothing enforced is not a
rung drop: DESIGN §4b says the brand/refinement was cosmetic until construction enforced it; M0 must
show which sites the seed's literal-only check actually refused).

### 3.2 Consumers of the where-machinery, deleted with it

v2:
- `src/v2/extdeps/languages/dag.dag` — the `where` clause productions and the `nominal_opaque` modifier.
- `src/v2/compiler/03_resolve.dag` — the where-call resolution path.
- `src/v2/compiler/04_infer.dag` — the `RefinementDeclaration` frontier.
- `src/v2/compiler/body_lowering_fold.dag`, `src/v2/compiler/emit_produced.dag` — where lowering.
- `src/v2/workflow/floor_cost_debt.dag`, `floor_grandfathered_roster.dag`, `floor_pure_producer_share.dag` — rows naming where witnesses.
- Tests: `src/v2/test/claim/parse/where_clause_required_comma_test.dag`, the where arms of `d1_declaration_grammar_parse_test.dag`, `type_decl_modifier_g0_parse_probe_test.dag`, and the where references in `edge_label/structural_label_readers_test.dag`, `namespace_xl0/reference_conservation_test.dag`.

dag:
- `std.types` `brand`, `range`, `string_non_empty`, `gt_zero`, `WherePredicateMarker`; `std.content_hash` `lower_hex_*`; `unicode_scalar`; the OCI and `is_text_readable` / `c_translation_unit_basename_safe` predicate fns.
- `gunbc.where_refinement_predicate_vocabulary` whole (`WherePredicateGrounding`, `WherePredicateEnforcement`, the v1 hand tables) — deleted with the refinements, never left as a second authority (carry-over (2)).
- `test.claim.where_refinement_predicate_vocabulary_witness_test` (its subject is gone).
- `dag/extdeps/languages/rust/types.dag` where rows; `gunbc.plans.p1_where_clause_lowering`; `gunbc.plan_registry_batch_e` row; `gunbc.census_closure_frontier` row; `gunbc.rung_drop.g0_type_decl_modifier_parse_without_sealing_property`; `gunbc.rust_source_type_bindings` Secret row.
- Recurring-failure-mode rows (`refinement_predicate_enforced_only_where_the_value_is_a_literal`, `parameter_refinement_has_no_carrier`, `authored_lexeme_read_as_a_grammar_marker`, `kernel_type_secret_has_no_v2_value_type_node`, `behavior_named_edge_label_validated_not_constructed`) are **updated, not deleted**: the class is climbed by construction, and each records the dissolution.
- The #12506 parked where-predicate frontier row is **retired by the deletion**, not flipped to a judgment (carry-over (3)).

v1 (§4): `v1.compiler.parse` (`"brand" =>` and the where grammar), `v1.compiler.infer`
(`where_refinement_predicates_equivalent`, `where_predicate_literal_string_args_match`,
`decidable_where_string_predicate_holds`, the brand arm), `v1.compiler.emit_rust` where handling.

## 4. The v1 seed

Purpose test (`gunbc.v1_maintenance_standing` `v1_seed_standing`): the seed compiles `src/v2` and
the corpus the floor reads; once no source contains `where` or `nominal_opaque`, the seed's where
machinery serves nothing on the self-host path, and **deleting** it shrinks the seed — admitted as
the opposite of growth. What the seed must *gain* is at most this, and M1 measures it before any
seed edit:

1. Single-field-record nominality: the seed must refuse `X { value: … }` where `Y` is expected for two
   distinct records over the same body. If v1 already types record construction nominally, the gain
   is zero. If not, the edit is admitted under the purpose test because the brand witness's
   COLLISION/MISMATCH refusals would otherwise be lost on the seed path — a §4b silent regression.
2. Nothing else: checked constructors are ordinary functions returning `T?`.

Stage0 mirrors of `src/v1/*` are regenerated in the same PR as each seed edit (regen-fixed-point on
the `build` lane); a mirror is never hand-edited.

## 5. Replacement-migration order (delete-first)

**M0 — census instrument** (§1). Read-only; lands first because §2.3 rests on its prediction and §3.1's
alias-vs-constructor split rests on its classes. Its own controls: a fixture closure with one
declaration of each class must classify exactly; an unindexed module must refuse exit 2.

**M1 — controls, expecting red** (lands before the cutover, in one PR with M2 or immediately ahead):
the brand witness's six arms rewritten over records, each first run against current main:

| arm | new fixture | expected after cutover | today |
|---|---|---|---|
| COLLISION | `type A { value: String }`, `type B { value: String }`, `fn f(a: A) -> B { a }` | refuse TypeMismatch | records already refuse — run to confirm; if green-before, it is a control, not a red |
| MISMATCH | same, spelled through a cast `a as B` | refuse (no declared route) | **red expected** if `as` admits record-to-record |
| CONSTRUCT | `fn f(s: String) -> A { A { value: s } }` | accept | accept |
| DUAL | one declaration, two use sites | accept | accept |
| STRIP | `fn f(a: A) -> B { B { value: a.value } }` | accept | accept |
| UNDECLARED_SITE | `fn f(a: String where brand("x")) -> …` | refuse at **parse** (`where` no longer exists) | **red**: parses today |
| NO_WHERE_ON_TYPE | `type A = String where range(min: 0)` | refuse at parse | **red** |
| NO_OPAQUE | `type A nominal_opaque = String` | refuse at parse | **red** |
| CHECKED_CTOR | `http_status_of(n: 600)` is `Absent`, `http_status_of(n: 200)` Present, both by execution | green | green (positive control) |

Per DESIGN §4b(4) none retires; the reds flip to permanent controls. The pre-wall-red /
post-wall-green pair is the discriminator, run on both the v2 route and the seed route.

**M2 — the cutover, one motion.** Delete the `where` grammar and `nominal_opaque` modifier in v2
`dag.dag` and v1 parse together, *then* fix forward every site the compile refuses — the refusals are
the census of load-bearing dependents (DESIGN §3). Brand sites become records; refinement sites become
§3.1. No intermediate state where both a brand and a record answer for one type.

Consumers **outside the required gate** (`v2.workflow.required_floor required_gate_prefixes`) do not
refuse loudly and are enumerated by name before deletion: the M0 instrument emits that list (every
declaring/consuming module not under a gate prefix) and M2's PR body carries it; any not repaired in
M2 is a declared §4b(3) rung drop with population and trigger, never silence. Known now:
`dag/extdeps/languages/rust/types.dag`, `gunbc.rust_source_type_bindings`,
`gunbc.plans.p1_where_clause_lowering`, the recurring-failure-mode rows in §3.2.

M2 is large (~360 declaration sites plus every brand `as` site). The only admissible staging is by
the gap-intolerant carve-out — and it does not apply: nothing outside this repo consumes these types.
So the operator decision is size, not staging (§7 D4).

**M3 — delete the machinery** that the cutover left unreferenced (§3.2 list, seed tables, vocabulary
module, #12506 row) — in M2's PR if it fits; it must not outlive M2 by more than one landing.

Does not touch the N7 lane (calm-boar-904): the 8 native tests need none of these forms; if M0 finds
one of them uses a brand or refinement, M2 sequences after N7 lands.

## 6. Coercion (§4) and cosmetic brands (§4b)

- **Alias:** identity is total, exact, unique, semantics-preserving — the only implicit route this
  plan keeps.
- **Record construction / projection** are not coercions: they are explicit terms, so §4's law is not
  asked of them. Record `X` to sibling `Z` over the same body has **no** declared route; `as` refuses.
  This makes STRIP an explicit `.value` projection instead of a cast down, which is the honest spelling
  of what the brand witness already admitted.
- **Checked constructor** is a partial conversion; it returns `T?` and is never implicit (§4: a phase
  that requires proof refuses implicit coercion).
- **§4b, "a brand is cosmetic until construction enforces it":** today a brand is enforced only at
  literal sites by the seed table — cosmetic everywhere else. After M2 a record's identity is enforced
  by the typechecker everywhere, which climbs (b) from mitigation to structurally guaranteed. The
  *invariant* (`Port ∈ 1..65535`) climbs only as far as construction is controlled: while
  `Port { value: 0 }` is writable outside `port_of`, the invariant is mechanically preventable at best
  (review), not guaranteed. Reaching structural impossibility needs **constructor visibility** —
  record literal admitted only inside the declaring module — which is §7 D2. Without D2 this plan
  must say so beside each checked constructor rather than claim the invariant.

## 7. Decisions the operator owns

- **D1 — default form.** Plain `type X = Y` stays a transparent alias (default); nominal identity
  only by single-field record (explicit). Recommend: yes.
- **D2 — constructor visibility.** Admit a record literal for an invariant-carrying type only in its
  declaring module, so `x_of` is the sole entry and the invariant is structural (also what
  `nominal_opaque` was reaching for). Recommend: yes, as a separate landing after M2, with the
  invariant's rung stated honestly until then.
- **D3 — discriminant = coproduct; no typecase.** Recommend: yes, contingent on M0's prediction.
- **D4 — M2 as one cutover PR** (~360 declarations), not split by module. Recommend: one PR, with
  M3 included if it fits review.
- **D5 — seed:** delete where machinery from v1 under the purpose test; add record nominality to v1
  only if M1 shows the seed lacks it. Recommend: yes.
