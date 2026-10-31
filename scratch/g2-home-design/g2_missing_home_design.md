# G2 missing-home design: 15 no-home heads (NAMESPACE-XL)

wise-moth-707, 2026-10-31, for tidy-ant-251 and sleek-owl-397. **Design only. Nothing here is applied, and no home is final until it is ruled.**

**Population.** These are the class-b rows ("primitive contract, NO home") of `scratch/n7-readahead/n7_readahead_v1.tsv` at 7f1302cf2b94, minus four heads that already have a home or a wave in flight: `map_get`, `join`, `sort_by` and `map_keys` (#15164). Fifteen heads remain.

**Sources.**
- Seed signatures: `v1.compiler.method` `builtin_function_registry` and the `std.algebra` method templates.
- Realization: the `v1_rt` and `v1_interpreter` dispatch arms.
- Consumer counts: the readahead table.

**Governing rules.**
- Strict first: an unresolved or ambiguous home refuses.
- No identity-to-spelling fallback.
- Every text crossing goes through the declared unfold (`gunbc.design_ruling text_crossings_go_through_the_unfold`).
- Search the existing std declarations before minting a new one.

**Rulings not found.** The grouped table marks `replace`, `split`, `starts_with` (std.host_text), `parse_int` (std.integer) and `to_string` (zesty-badger-599's wave) as RULED. No `gunbc.design_ruling` row names any of them. I take those rulings as given from the dashboard, and they should land as `design_ruling` rows.

## A. Already homed in std.algebra: a sweep row, not a new declaration (4 heads)

The table filed these as `G1?`/`G3?`. Each already has a declaration with the same meaning, so the work is a `RenameHead` row in `v2.workflow.roster_import_sweep`, and nothing new is minted.

| head | sites | home (exists) | rewrite | open check |
|---|---|---|---|---|
| `first` | 5 in 2 files (v2.std.node_query 4, std.source_annotation 1) | `std.algebra` `list_head<T>(xs) -> Optional<T>` | `xs.first()` → `list_head(xs: xs)` | The seed answers Null on an empty list; check that every consumer matches `Present`/`Absent` (the companion-tag path covers this). |
| `get` | 4 in 3 files (std.content_hash 2, std.power_of_two 1, std.computation 1) | `std.algebra` `list_at_optional<T>(xs, index) -> Optional<T>`. The projection rows (`primitive_get`) already exist. | `get(xs, i)` → `list_at_optional(xs: xs, index: i)` | One seed name covers **two meanings**: a list index and a map lookup (a Str or map receiver goes through `raw_map_lookup`). Classify each site by receiver type. A map-receiver site goes to `map_lookup`; a site whose receiver type cannot be read refuses. **No receiver-blind row.** |
| `reverse` | 3 in 3 files (std.occurrence_identity, std.occurrence_binding, std.induction) | `std.algebra` `list_reverse<T>(xs) -> FreeMonoid<T>` | `reverse(xs)` → `list_reverse(xs: xs)` | The seed has no string reverse, so a Str receiver refuses. |
| `flat_map` | 3 in 3 files (std.roster_frontier, std.graph, std.algebra) | `std.algebra` `list_flat_map<T,U>(xs, f) -> FreeMonoid<U>` | method or free call → `list_flat_map(xs:, f:)` | `list_flat_map` is a `fold_list` with `list_push`, so it keeps order only if `fold_list` folds left to right. Add an order RED (`[1,2]` with `f(x)=[x,x+10]` must give `[1,11,2,12]`) before the row. |

## B. Ruled homes: declare and realize (5 heads)

**`std.host_text`** declarations are host text (kernel `String`). Each is a `HostRealizedSeam` with one `std.primitive_projection` row, the same pattern as `chars`.

| head | sites | home | proposed signature | realization / notes |
|---|---|---|---|---|
| `starts_with` | 28 in 4 files (extdeps.uri 20, v2.std.anonymous_binder 4, std.content_hash 2, extdeps.languages.dag.syntax 2) | `std.host_text` | `starts_with(text: String, prefix: String) -> Bool` | Rust `str::starts_with`; pure and total. |
| `split` | 27 in 6 files (extdeps.uri_path 11, v2.compiler.source_authority 10, std.unicode.scalar 2, std.content_hash 2, extdeps.uri 1, extdeps.languages.dag.syntax 1) | `std.host_text` | `split(text: String, separator: NonEmptyStr) -> List<String>` | **Stricter than the seed.** With an empty separator the seed splits around every scalar, which is the meaning of `chars`. With a `NonEmptyStr` separator that call is unwritable rather than meaning a second thing. Precondition: the census shows no site passes an empty separator; a site that does refuses and is named. |
| `replace` | 18 in 3 files (std.types 16, v2.compiler.source_authority 1, extdeps.ietf.http_semantics 1) | `std.host_text` | `replace(text: String, needle: NonEmptyStr, replacement: String) -> String` | Replaces every match (`str::replace`). The empty-needle case is excluded for the same reason as `split`. The seed's `format!` coercion of non-text arguments is **not** carried over: arguments are typed. |
| `string_length` | 20 in 11 files (v2.compiler.source_authority 6, and 2 or 1 each in 10 others) | `std.host_text` | `string_length(text: String) -> Int` | **Ruling needed (see D)**: this has no RULED home in the grouped table. The unit is Unicode scalars, and the declaration's comment says so. |
| `parse_int` | 1 (extdeps.ietf.http_semantics) | `std.integer` (in flight, #15472, draft) | `parse_int(text: String) -> Int?` | Rust `str::parse::<i64>` grammar: optional sign, no whitespace, and overflow gives Absent. #15472 needs its last generator run (r22) applied before review. |

**Two representations, on purpose.** `v2.std.text` `string_split` and `string_replace` are the **structural** operations over `String = FreeMonoid<Char>`. The proposed `std.host_text` `split` and `replace` are the **host** operations over kernel `String`. These are two declarations over two distinct types, not nicknames. Per `text_crossings_go_through_the_unfold`, no row may bind one to the other, and a host-text site whose value is structural text takes the unfold, or refuses. The non-empty delimiter matches `string_split`'s `FreeSemigroup<Char>` delimiter, so the two agree on the one point where the seed differs.

**`to_string`** (12 sites in 7 files) stays with zesty-badger-599's Int/Nat `to_string` wave, which already has a ruled home. The seed's `format!` Display over any value is one name with several meanings, varying by receiver type. The wave dispatches each site by its receiver type: `Int` → `std.integer` `int_to_decimal_string`, and so on. A site whose receiver type cannot be read refuses. **No generic `to_string` declaration.**

## C. No ruled home and the seed's meaning is unsettled: ONE ruling ask (4 heads)

Each of these four has a seed behaviour that must not carry over unchanged:
- `substring` and `char_at` each have two seed forms that disagree with each other.
- `code_point` fabricates a default.
- `atom_identity_hash` has no settled layer.

| head | sites | defect in the seed | recommendation (A) | alternative (B) |
|---|---|---|---|---|
| `substring` | 14 in 7 files (v2.compiler.source_authority 4, v2.std.data_initializer_identity 2, v2.compiler.tokenize 2, std.unicode.scalar 2, extdeps.languages.dag.syntax 2, v2.extdeps.languages.dag 1, std.primitive_projection 1) | The free call **clamps** negative indices to 0. The method call **wraps** them (`as usize`). `end <= start` gives `""`. | `std.host_text` `substring(text: String, start: Nat, end: Nat) -> String?`, scalar-indexed with the end excluded. Absent when `start > end` or `end > length`. No clamping, so a negative index is unwritable. | Keep the seed's clamp, `-> String` (total). This is rejected below. |
| `char_at` | 5 in 3 files (v2.extdeps.languages.dag 2, extdeps.numeric.base16 2, std.decimal 1) | The free call clamps and returns `""` when out of range. The method call returns Null. | `std.host_text` `char_at(text: String, index: Nat) -> String?`, one scalar as text and Absent out of range. It keeps `String` because every consumer compares the result with a string literal. | `-> Char?`, plus a site rewrite to `Char` literals. More honest, but more churn. |
| `code_point` | 10 in 2 files (extdeps.numeric.base16 9, v2.std.compilers.target_model 1) | Answers **0 for empty text**, a fabricated default (§5). | **No new declaration.** Rewrite the sites to the existing `std.unicode.char_class` `unicode_char_code_point(c: Char) -> Int` over a `Char` obtained through the declared route (`std.host_text` `chars`, then the element). Empty text then has no `Char` to pass, so the 0 is unwritable. | `std.host_text` `code_point(text: String) -> Int?`. That would be a second code-point authority beside `unicode_char_code_point`, which is a §3 nickname. Rejected. |
| `atom_identity_hash` | 1 (std.content_hash `content_hash_atom`) | Not wrong, but it has no layer. It is FNV-1a 64 over the UTF-8 bytes as 16 lowercase hex digits. `content_hash_atom`'s comment calls it "a target-kernel realization" pending gunbc#11637. | `std.content_hash` `atom_identity_hash(text: String) -> Fnv1a64StructuralDigestHex`, a `HostRealizedSeam` beside its one consumer. `std.content_hash` already owns `HashFamily` and the Fnv1a64 carriers. `extdeps.crypto.hash` explicitly declines to merge with this fingerprint. | An `extdeps` FNV-1a module (FNV is an external, versioned spec). A structural FNV-1a over `utf8_encode_bytes` in `.dag` could later replace the seam, retiring the host realization (the gunbc#11637 trigger). |

Why A for `substring` and `char_at`: the two seed forms already disagree, so a total version would have to pick one of them and silently change what the other form's sites mean. That is a meaning fork the first time either form's sites are swept. `Nat` indices plus Absent make both seed defects unwritable. Each site then states its own handling of absence, as the cardinality rulings require: `dag_absence_is_one_meaning` and `explicit_present_is_the_authoring_policy`.

## D. The ruling ask (one question)

> Rule the homes for `substring`, `char_at`, `code_point` and `atom_identity_hash`, plus `string_length`, which has no RULED home:
> - **A (recommended):** see the table in C.
>   - `substring` → `std.host_text` `substring(text, start: Nat, end: Nat) -> String?`
>   - `char_at` → `std.host_text` `char_at(text, index: Nat) -> String?`
>   - `code_point` → no declaration; rewrite the sites to `std.unicode.char_class` `unicode_char_code_point` via `chars`
>   - `atom_identity_hash` → `std.content_hash` `HostRealizedSeam`
>   - `string_length` → `std.host_text` `string_length(text) -> Int`, counting scalars
> - **B:** the seed's total, clamping forms in `std.host_text`, a `std.host_text` `code_point(text) -> Int?`, and an `extdeps` FNV-1a home for `atom_identity_hash`.

## E. Sequencing (each wave is its own PR, after the gate PR)

1. **Group A rows.** `first`, `get` (receiver-classified), `reverse`, `flat_map`. These are sweep rows only, plus the `flat_map` order RED.
2. **Group B host text.** `starts_with`, `split` and `replace` as one `std.host_text` declaration PR, plus their sweep waves. Its precondition is the empty-separator and empty-needle census.
3. **Ruled C/D heads.** One declaration PR per home, then the waves.
4. **`parse_int`** finishes on #15472. **`to_string`** finishes in zesty-badger-599's wave.

Per wave:
- A RED: an unimported use of the moved name refuses natively with `resolve_reason_unbound_symbol`.
- An imported positive control.
- The binding-diff admission check: only ambient → the declared home may change.
- Deletion of the head's `std.primitives` contract row and its registry entry when the residual reaches zero.
