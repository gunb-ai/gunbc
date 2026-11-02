# T4 A2/A4 unbound-name attribution

Analyst: valiant-deer-27. Census input: `emit.json` at
`~/runs/cc2/cc2-15637t2/out/` on srv2 (sha256 4687104a…, 63,670 B) over tree `d77e2d8a`, entry
`v2.cli.compile_cli`, gen-0 native emit: 230 members, 224 refused.
Rulings quoted below are tidy-ant-251's, 2026-11-02. Reading only: no host run, no corpus change.

## Basis and method

Of the 224 refusals, **96 reported sites across 44 names** carry
`reason: resolve_reason_unbound_symbol`; the other 128 refusals carry other reasons
(inhabitation, lowering, emit-not-realized) and are outside A2/A4.
**A "site" here is a module, not a line**: `emit.json` carries one diagnostic per refused module
(and `file/start/end` are null), so a count is the number of modules whose first refusal names that
symbol. Inside those modules the occurrences are many — `discriminant` is refused in 8 modules and
appears 77 times in non-comment position across them.

Method: every `.dag` at `d77e2d8a` indexed once for (i) module-level `fn|type|data` declarations,
(ii) sum alternatives (`type X = A | B {…}`, including the `= A` first arm), (iii) that module's own
`import <mod> { names }` rows. Each refusal was then classified against **the refusing module**, not
against the corpus.

## The measured shape (all 96 sites)

| class | sites | meaning |
|---|---|---|
| NO_DECLARED_HOME | **77** | refusing module has no same-module declaration and no import row naming the symbol |
| DECLARING_MODULE_IMPORTED_NAME_NOT | **19** | the declaring module IS imported; its import row does not name the symbol |
| IMPORT_NAMED_BUT_REFUSED | **0** | no site has a name row and still refuses |
| SAME_MODULE_DECL | **0** | no same-module binding defect appears in A2/A4 |

The native import door is faithful. Every A2/A4 unbound refusal is a reference whose module carries
no name row for that symbol — the repair cannot be more import rows, which are retiring.

## A2 — constructor tags and type-name heads

| name | modules | the declaration it should bind to | cause class |
|---|---|---|---|
| discriminant | 8 | no `.dag` declaration; claimed by the `std.primitives` hand enumeration, **no `PrimitiveContract` row** | modeling obligation (see below) |
| Value | 2 | constructor of `type Behavior` in **v2.std.node** | authored constructor tag |
| TypeNode | 2 | constructor of `type NodeKind` in **v2.std.node** (`v2.std.list_introduction` imports the module, names `Node Transform Value Branch Loop Bind Match`, not `TypeNode`) | authored constructor tag |
| Found | 1 | constructor of `type NamedEdgeTargetLookup` in **v2.std.node** | authored constructor tag |
| None | 1 | constructor of `type Diagnostics` in **v2.std.diagnostic** | authored constructor tag |
| Accepted | 1 | constructor of `type Outcome` in **v2.std.diagnostic** | authored constructor tag |
| Equal | 1 | constructor of `type Ordering` in **std.algebra** | authored constructor tag |
| Present | 1 | constructor of `std.optional.Optional<T>`; **`std.list` carries no `import std.optional` row at all** | no declared home |
| ContentHashDifferent | 1 | constructor of `type ContentHashComparison` in **std.content_hash** | authored constructor tag |
| RuntimeAggregate | 1 | constructor of `type RuntimeValue` in **v2.std.runtime** | authored constructor tag |
| ReceiverElementOrSelf | 1 | constructor of `type AlgebraTypeTemplate` in **std.algebra** | authored constructor tag |
| Filesystem.Write | 1 | constructor `Write` of `type Verb` in **std.effect_grant**, written with a qualified head | qualified head not read |
| Optional | 1 | constructor of `type GrammarExpr` in **v2.std.grammar** — the spelling also names `std.optional.Optional` | collision |
| Node | 1 | `v2.std.node.Node` is a type, and the same spelling is a constructor in several sums | collision |
| Map | 5 | type `Map<K,V>` in **std.types**, referenced in **annotation position** (`exports: Map<ModuleId, List<Node>>` in v2.std.compilers.compilation_unit, `keyword_literals: Map<String, LiteralValue>` in std.syntax); those modules' `import std.types { … }` rows omit it | no declared home for the TYPE name (membership lane) |
| PartialFunction | 2 | record type `type PartialFunction<K,V> { lookup: fn(K) -> V? }` in **std.algebra** (module imported, name not named); used both as an annotation and as a construct head | no declared home for the TYPE name (membership lane) |
| List | 11 | type in **std.types** (4 of 11 modules import `std.types` without naming `List`) | no declared home |
| Disposition | 1 | type in **std.disposition** | no declared home |
| TargetEffectApplyShape | 1 | type in **v2.std.compilers.target_model** (module imported, name not named) | no declared home |

## A4 — declared cross-module functions

| name | module refusing | declaring module (measured) | class |
|---|---|---|---|
| sequence_outcome | v2.compiler.translate | `fn sequence_outcome` in **v2.std.diagnostic** — the import row names `Accepted None Outcome outcome_accepted …` but not this | declaring module imported, name not named |
| outcome_accepted | v2.compiler.emit_produced | **v2.std.diagnostic** | same |
| interpretation_algebra_slot_identities_equal | v2.compiler.eval | **v2.std.runtime** | same |
| runtime_value_equality | v2.extdeps.runtimes.v2_effect_io_pure | **v2.std.runtime** | same |
| optional_present | v2.extdeps.languages.swift | **std.optional** | same |
| list_map | v2.compiler.resolve | **std.algebra** | same |
| chars_to_string | std.host_text | no `.dag` declaration; in the `std.primitives` hand enumeration, no `PrimitiveContract` row | text-port disposition |
| atom_identity_hash | std.content_hash | no `.dag` declaration; a `PrimitiveContract` row `name: "atom_identity_hash"` EXISTS | primitive tier not reached |
| map_keys | v2.std.provenance | no `.dag` declaration; `PrimitiveContract` row exists | primitive tier not reached |
| flat_map | std.roster_frontier | no `.dag` declaration; `PrimitiveContract` row exists | primitive tier not reached |

## Where the repair goes, and the earliest producer

`v2.compiler.resolve`'s `OccurrenceResolution` states the native ladder: a bare name is answered by
(1) a lexical binder, (2) a parameter or type parameter, (3) a **declared reference** — own module, an
import row that names it, or a qualified path — and (4) four stopgap tables (kernel type spelling,
kernel canonical symbols, `std.primitives` reached through `v2.compiler.primitive_authority`, the
collection roster). Candidates come from `v2.std.symbol_index`'s `symbol_index_global_unique_lookup`;
an empty candidate set is exactly `resolve_reason_unbound_symbol`.

Nothing in that ladder can produce a binding for (a) a cross-module declaration whose module is in the
closure but is not named by an import row, or (b) a **constructor tag at all**, since a tag is not a
module-level declaration and no name row can ever carry it. Those two facts are the whole of A2/A4.

**Corrected framing (tidy-ant-251, 2026-11-02).** The v1 occurrence-binding route
(`std.occurrence_binding_resolve`, `std.dissolution`/`unbound_dissolution`, the silent-pick policy
primitives) is the authority **being retired**; v2's native canonical binder is the authority to
extend. v1 may serve only as an offline differential oracle (DESIGN §3, replacement migrations). An
earlier draft of this note said the root was that the native route "does not consume v1's verdict";
that framing is withdrawn. What survives measurement is narrower and local: **the native binder has no
constructor arm and no namespace-membership arm**, and its elided-construct arm
(`resolve_elided_construct_walk`, `ConstructTagElided`, `BoundByExpectedType`) reads the expected type
only for headless `{ … }` literals, never for an authored tag in a nested pattern or record head.
`dag/gunbc/rung_drop/variant_fields_unlowered_on_the_native_route.dag` is the same producer in the
opposite direction (its retracted `resolve_pattern_node_is_constructor` harm sentence, and the
standing statement that the global-bare Unbound arm refuses rather than silently picking).

## Routed by the ruling, so this table is not a to-do list for one lane

1. **A4 plus `List`, `Disposition`, `TargetEffectApplyShape`** (declaring module in the closure, no
   name row): the wave lane — wise-moth-707, KeepHead rows with `home =` the declaring module above.
2. **The constructor arm** (the A2 tags, and the elided `Map`/`PartialFunction` heads): valiant-deer-27,
   extending v2's existing elided-construct machinery so an **authored** tag resolves through the
   declared constructor set of its expected type; refusals stay for ambiguity and for a tag outside
   that set. First deliverable is a design note naming the exact producer and the RED list.
3. **Primitives: no `PrimitiveContract` rows will be added** — that tier is a stopgap the namespace cut
   shrinks. `length` is ruled KeepHead `std.algebra length`; `chars_to_string` goes to the text-port
   disposition.
4. **`discriminant` has no `.dag` declaration**, so it is a modeling obligation, not a binding defect.
   Measured use across the 8 modules (77 non-comment occurrences): **47 tag comparisons**
   (`if id == discriminant(v: OrderingComparison {})`, `if variant == discriminant(…)`), **16 identity
   atoms** (`Atom { identity: discriminant(v: AlgebraPrimitive {}) }`), 11 other argument positions,
   and **3 occurrences where `discriminant` is a binder of a function parameter** in
   `v2.std.compilers.target_model` (`f: fn(discriminant) { if discriminant == ^conditional_… }`), which
   shadow the spelling locally. So the use is a total map from a sum value to its tag, consumed as an
   Int for dispatch and as an identity atom — the shape `match` already provides for the dispatch half.
   The corpus also has a tag-literal spelling (`^name`) used 2083 times, which covers the "I want the
   tag as a value" half without a primitive. For the decision: keep it as a modeled operation
   (declared where sums live, over a sum to its tag) or rewrite the 47+16 sites to `match` and `^tag`.
5. **The 17 spellings that have `PrimitiveContract` rows and still refuse** (join, concat, first, fold,
   get, reverse, starts_with, substring, to_string, map_get, code_point, string_length, contains,
   map_keys, flat_map, atom_identity_hash, string_contains): not A2/A4, routed to the wave lane; a
   missing roster row does **not** explain them.

## Not established by this reading

* Whether the last (primitive) tier is consulted at a call head versus a pattern atom — the reason the
  17 rows above still refuse. Needs a one-module host probe (`extdeps.uri` for `join` would show it).
* The 128 non-`unbound_symbol` refusals in the same table.
* Per-line locations for any of the 96 sites: the census carries no spans.

## OPEN — the identity half of `discriminant` (decider: tidy-ant-251, 2026-11-02)

The ruling was: no primitive and no new declaration for `discriminant`; its uses split onto what the
language already declares — the 47 tag comparisons become `match` on the value, and the 16 identity
atoms take the tag as a value (`^name`) **only if** `Atom.identity`'s type accepts a tag value with the
same identity semantics. I stopped, as instructed, and measured what that field is typed as:

* `type Atom = Atom { identity: Symbol }` — v2.std.node. The field is a **Symbol**, not an Int, so the
  Int-representation worry does not arise; a different one does.
* `type Symbol` is an **opaque kernel type** (v2.std.node declares it with no body, beside `type String`
  and `type Hash = FinitelySupportedFunction`-style kernel rows). `^name` is a **symbol literal**, not a
  sum tag value — v2.std.node's own comment beside `symbol_type_node`: "THE VALUE TYPE OF A SYMBOL, as a
  node: what a symbol literal (`^x`) is typed as…". An earlier sentence in this note ("the corpus
  already spells a tag as a value 2083 times as `^name`") is **withdrawn**: those are symbol literals.
* The builtin's own signature returns **text**: the `discriminant` entry in `src/v1/04_method.dag` is
  `BuiltinSignature { params: [v: AlgebraTypeVariable "discriminant_subject"], returns: string_type }`.
  So today those 16 atoms feed a String (the tag NAME as text) into a field typed Symbol. Either an
  unstated String→Symbol crossing sits at that position, or those sites are an inhabitant fact native
  typing will refuse independently.
* No `^algebra_primitive` / `^ordering_comparison` / `^equality_comparison` style symbol is authored
  anywhere in the corpus, so the swap would also invent the identity vocabulary rather than reuse it.

So the identity half is **not a mechanical migration**. The three shapes a decision could take:

1. **Model the map**: one declared operation from a sum value to its tag as a Symbol. One new authority,
   16 sites respelled to call it; the crossing is declared and typed once.
2. **Respell to authored symbols**: each of the 16 atoms names a Symbol that is declared for the concept
   (`^ordering_comparison` etc.), so the tag-as-text disappears. Cheapest at the call sites, but it makes
   16 new symbol names the identity vocabulary, and the correspondence to the sum's alternatives is then
   convention, not a checked fact.
3. **Change the identity**: make `Atom.identity` carry the tag text as String. That is a representation
   change to identity itself (DESIGN §4), with every consumer of atom identity downstream.

Not chosen here; `discriminant` stays unbound in T4 and counted as such until the migration is
scheduled. The comparison half (47 sites) is unaffected and lands on `match`.

## Addendum 2026-11-02 -- the bounded set, a second closure, and one diagnostic gap

### The bounded set, read off the receipt (not off a guess)

The set the constructor-arm evidence run is bounded to is defined by the receipt, not by eye: the
**30 modules whose FIRST refusal in the gen-0 emit of `v2.cli.compile_cli` is
`resolve_reason_unbound_symbol` on an A2 constructor name**, taken from
`srv2:~/runs/cc2/cc2-15637t2/out/emit.json` (sha256 `4687104a9d...`, tree `d77e2d8a`). All 30 are
present in a current checkout. Their (module, first-refusal name) pairs:

| module | unbound_ref | module | unbound_ref |
|---|---|---|---|
| std.compiler_entry | Disposition | v2.compiler.program_partition | Value |
| std.algebra | DiscriminantValue | v2.std.compilers.semantic_decl_emission | TypeNode |
| v2.std.node_query | TargetEffectApplyShape | v2.std.verification | TypeNode |
| extdeps.languages.json.emit | Value | v2.std.dependency | TypeNode |
| extdeps.languages.json.grammar | Value | v2.lens.reference_deps | Present |
| v2.std.list_introduction | Value | std.bytes | Present |
| extdeps.uri_path | Value | std.list | Present |
| std.occurrence_binding_resolve | Value | v2.std.compilers.compilation_unit | Present |
| std.unicode.scalar | None | extdeps.languages.rust.capabilities | None |
| v2.std.data_initializer_identity | None | std.trait_derive_shape | List |
| v2.std.decl_index | Optional | extdeps.languages.rust.derive_contracts | List |
| v2.std.compilers.sugar | Accepted | extdeps.languages.rust.emit | Map |
| std.graph | Accepted | v2.std.compilers.target_model | Node |
| v2.std.cardinality | Accepted | v2.std.syntax | Map |
| v2.compiler.infer | Found | | |

96 modules refuse first on SOME unbound name; 30 of them on an A2 name. A census that counts "modules
with unbound names" without naming the reference cannot separate those two numbers, which is why the
A2/A4 grouping in the table above states the name per module.

### A second closure, from clever-crane-830's char_at fixture (cc2-charat)

Head `8f601cb914a9` (PR #15656), gen-1 harness `a5981454`, `emit.json` sha256 `ae7cf447`, out at
`srv2:~/runs/cc2/cc2-charat/out/`. A TWO-module fixture (an entry plus one member that calls
`std.host_text char_at`) closes to **28 modules**, and the refusal histogram is:

| refusal | modules |
|---|---|
| `resolve_reason_unbound_symbol` | 11 |
| `target_value_expr_reason_transform_shape_invalid` | 4 |
| `target_diagnostic_reason_payload_shape_invalid` | 4 |
| `emit_module_member_generic_not_realized` | 3 |
| `infer_grounding_not_derived` | 2 |
| `infer_grounding_witness_not_derived` | 2 |
| `infer_present_arm_unbound` | 2 |
| `infer_grounding_transform_operand_unbound` | 2 |
| one each | `resolve_module_not_found`, `infer_reason_pattern_payload_formal_uninstantiated`, `emit_module_member_opaque_not_realized`, `resolve_reason_reference_category_mismatch`, `infer_transform_operand_type_mismatch` |

The 11 unbound rows are `std.compiler_entry`, `std.decl_ref`, `std.types`, `std.algebra`,
`std.content_hash`, `std.host_text`, `std.roster_frontier`, `extdeps.uri`, `std.decimal`,
`std.render_repeat_string_bootstrap`, `std.checked_arithmetic`. So the A2 class is not specific to
`compile_cli`'s closure: a two-module fixture hits the same wall, and `std.host_text` itself is one of
the refusing modules.

### FINDING: the unbound refusal does not always carry the name it refused on

In cc2-charat **every one of the 11 `resolve_reason_unbound_symbol` rows has `unbound_ref: null`**, while
in cc2-15637t2 the same reason carries the name (`Value`, `Filesystem.Write`, ...). So a census or an
admission check keyed on the unbound NAME silently drops rows depending on the site, and "N modules show
cause X" is not comparable between two runs unless each states which field it read. Whoever routes the
wave lane should treat a null `unbound_ref` as "the refusal is real, the reference is not reported" --
not as "no unbound reference".

### Status of the one-root repair for A2's constructor class

Draft PR gunb-ai/gunbc#15671, head `49553c1c7e8b`: the expected-type constructor-set arm is coded in
`v2.compiler.resolve` (door order and refusals exactly as ruled), the new cause is enrolled in the
closed `ResolveRefusalReason` vocabulary and in `known_frontier_causes` in the same branch, the REDs and
controls are in `src/v2/test/claim/name_resolve/construct_tag_expected_type.dag`, and the bounded
identity diff for it is the 30-module set above run through the emitted driver's `census-scoped` verb
with `--against`. A local AST pass over that set found **zero** sites where a spelling bound today by
the name door is not the expected type's arm -- with the instrument's coverage limit stated in the PR
(422 of 7,754 construct sites have an expectation a syntactic pass can read; `resolve` carries no
expectation into a call argument at all).

