# Canonical content order — draft declaration for fit check (node adhoc-77383faf-d07)

Decision (deep-ferret-305): declare in std now; #12895 portable_value_cmp becomes its interpreter realization.

## 1. Inhabit std.algebra, no parallel vocabulary
- `type TotalOrder<T> { compare: fn(T, T) -> Ordering }` — result is the existing `std.algebra Ordering`.
- `AlgebraProfile` gains `TotalOrderProfile`; `total_order_templates()` = [compare].
- `OrderedRing<T>` / `Field<T>` COMPOSE it: `ordered_ring_templates()` / `approximate_field_templates()`
  take their `compare` row from `total_order_templates()` instead of restating it (one compare concept).

## 2. The canonical order is a DERIVED inhabitance, not a function over a dynamic value
No `.dag` carrier for "any value" exists (PortableValue is seed Rust), and typed `.dag` map keys are
one type K. So the authority is a structural derivation rule — the same shape as
`algebra_profile_equality_extensional` — saying how every admissible key type inhabits TotalOrder:
- Bool: false < true. Int: numeric. String: Unicode-scalar lexicographic (= UTF-8 byte order).
- Float: IEEE 754-2019 §5.10 totalOrder, cited through the binary64 authority of #12547
  (lively-newt-480). Realized by `f64::total_cmp`. NOT raw u64 bit compare (that inverts negatives).
- List: lexicographic by element, then length. Map: lexicographic over entries in canonical key order,
  comparing (key, value). Set: lexicographic over members in canonical order.
- Record: fields compared in canonical field order. Variant: by arm key, then fields as a record.
- Cross-kind rank (Null < Unit < Bool < Int < Float < Str < List < Map < Set < Record < Variant) is
  only reachable in the interpreter's untyped value space; it is the interpreter realization's
  extension and is never observable on a well-typed key.

## 3. Record/variant key: spelling now, declared identity is the climb
Declared identity (owner module + declaration, arm ordinal) is the right key, but it is not carried by
`Value::Variant` / `PortableValue::Variant` today — `gunbc.guarantee_stall.variant_owner_identity_stall`
(NS-0B). Spelling is the only key BOTH realizations carry, and it is total on what they carry.
The residual (two same-spelled owners compare Equal) is exactly that stall's population; the order
switches to declared identity when NS-0B's trigger fires. Stated, not silent.

## 4. Realizations (the one authority, two realizations)
- Interpreter: portable_value_cmp (#12895) realizes §2; to_string/interpolation of Value::Map render
  entries in that order.
- Emitted Rust: map rendering sorts entries by the emitted compare for K (generated from the same
  derivation rows), never by im::HashMap iteration.
- Control: a fixture spanning every kind rank (incl. -0.0/+0.0/negative/NaN floats, same-length lists,
  nested maps) — interpreter and emitted compare agree; a multi-entry map renders byte-identically
  across 2 processes, interpreted and emitted.

## 5. Gate
v2.lens.determinism classifies every remaining host-order path (iteration, Debug formatting) as
HostUnspecifiedOrder; red on any unclassified one.

## Fit check (neat-boar-16) — conditions accepted
1. Interpreter and emitted comparisons are both DERIVED from the rule rows (kind rank, float rule, field/arm
   key), not two hand comparators; control = same multi-entry map, identical bytes, 2 processes, both.
2. OrderedRing/Field DELETE their restated compare in the same change.
3. Ordering reused; no new carrier.
4. Spelling key is a named stand-in on the carrier, trigger = variant_owner_identity_stall (NS-0B);
   spelling is an ordering key only, never identity. OPEN — see below.
5. Float controls: -0/+0, NaN payloads, negatives (totalOrder via #12547).
6. #12895 consumes this authority; sign-off from jolly-boar-500 / sleek-ibex-207 before either lands.

### Condition 4 is not realizable in the interpreter before NS-0B
Value::Variant / PortableValue::Variant carry no owner identity, so the interpreter cannot detect "two
distinct declarations, same spelling": it can neither refuse nor tiebreak by declaring module. In the
emitted Rust the case cannot arise for a typed key (distinct declarations are distinct Rust types; an
enum key distinguishes them by arm). So the residual is interpreter-only and is exactly the stall's
population.

### Ruling on condition 4 (neat-boar-16): option A
- Variant/record order = spelling, THEN full payload by the same canonical order. Keys tie only when they
  render to identical bytes, so tie order is unobservable in output.
- Control: two same-spelled variants with DIFFERENT payloads render in a fixed order across 2 processes.
- §4b drop row, interpreter path only. Population: variant_owner_identity_stall's two members, plus
  same-spelled variant keys from distinct declarations. The interpreter conflates those for ordering AND,
  pre-existing (not introduced here), for equality. Trigger: owner identity carried on Value::Variant and
  PortableValue::Variant (NS-0B), the capability, not a PR. Emitted Rust: no row (distinct types).

### jolly-boar-500 sign-off (conditions)
(a) Kind rank derived from the declaration order of the portable value model's variants. PortableValue's
    order (Null, Unit, Bool, Int, Float, Str, List, Map, Set, Record, Variant) matches the list exactly; no change.
    Correction: Set is OrdSet<String>, so "canonical member order" equals byte-lexicographic member strings.
    #12895's Set rule was already conformant; only the float rule diverges.
(b) Differential control: interpreter vs emitted compare agree on a generated corpus covering every kind
    pair, NaN, -0.0/+0.0, empty vs non-empty collections, nested maps and sets.
