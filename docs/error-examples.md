# What .dag catches: concrete examples

Every example below compiles in Rust, Go, and Python. Every example
is rejected by .dag at compile time. No lint rules, no annotations,
no opt-in — the errors emerge from the algebraic structure that
`.dag` already has.

Each example shows:
1. The `.dag` code
2. The compiler error
3. Why it catches it (which algebra)
4. How it emerges from modeling — not a special-case check

---

## 1. Non-terminating recursion through lookup

```dag
fn resolve_type(t: Node) -> Node {
  match t.connective {
    Leaf => t
    Generic => {
      let resolved = lookup_type(name: t.name)
      resolve_type(t: resolved)
    }
    Conj => make_conj(children: t.children |> map(c => resolve_type(t: c)))
  }
}
```

**Compiler error:**
```
error[CX]: cannot prove termination of resolve_type

  resolve_type(t: resolved)
               ^^^^^^^^^
  `resolved` came from lookup_type() — a lookup, not structural
  descent on `t`. SubValueRelation: Unknown.

  The Conj branch is fine:
    resolve_type(t: c) where c is IteratedSubValue of t.children ✓

  The Generic branch is the problem:
    resolve_type(t: resolved) where resolved has no descent
    relationship to t. If the looked-up type contains Generic
    references, this recurses forever.

  Fix: separate resolution from walking. Resolve all type
  references in a prior pass, then walk the resolved tree.
```

**Why it catches it:** CX tracks `SubValueRelation` per argument.
`lookup_type(name: t.name)` returns a fresh value — no descent
relationship to `t`. The compiler doesn't need a special "recursive
type alias" check. It falls out of the structural descent proof
that every recursive function requires.

**How it emerges from modeling:** The bounded kernel invariant says
Node is the only recursive type, and recursion must be on
`Node.children`. This isn't a rule someone added for this case —
it's the foundational property that makes all descent proofs work.
The type alias bug is just one consequence.

---

## 2. Redundant work: reverse of reverse

```dag
fn normalize_tokens(tokens: List<Token>) -> List<Token> {
  tokens
    |> reverse
    |> filter(t => t.kind != Whitespace)
    |> reverse
}
```

**Compiler error:**
```
error[OPT]: redundant operation sequence

  tokens |> reverse |> filter(...) |> reverse
          ^^^^^^^^^                ^^^^^^^^^
  `reverse` is an involution (reverse ∘ reverse = identity).
  The outer reverse ∘ inner reverse cancels.

  Equivalent: tokens |> filter(t => t.kind != Whitespace)
  Cost: O(n) saved (two unnecessary traversals).

  If the reversed order matters for the filter, use:
    tokens |> reverse |> filter(...)
  without the second reverse.
```

**Why it catches it:** `reverse` is declared in `std/algebra.dag`
as an involution on `FreeMonoid<T>` — `f(f(x)) = x`. The compiler
composes operations symbolically and applies simplification laws.
When two involutions are adjacent (even with operations between
them that don't affect order), they cancel.

**How it emerges from modeling:** The programmer didn't annotate
anything. `List<T>` inhabits `FreeMonoid<T>`, which declares
`reverse` as an involution. The compiler reads the algebra and
simplifies. Adding a new involution to any type gets the same
check for free.

---

## 3. Non-idempotent workflow in retry context

```dag
func provision_database() {
  uses cloud.Sql, audit.Logger

  let db = cloud.Sql.CreateOrUpdate(
    name: "orders-db",
    spec: db_config
  )

  audit.Logger.Append(entry: LogEntry {
    action: "provisioned",
    resource: db.name,
    timestamp: now()
  })
}

func deploy_with_retry() {
  uses retry

  retry.WithRetries(
    max: 3,
    action: () => provision_database()
  )
}
```

**Compiler error:**
```
error[EFFECT]: non-idempotent operation in retry context

  retry.WithRetries(action: () => provision_database())
                                  ^^^^^^^^^^^^^^^^^^^^
  provision_database() is NOT idempotent because:

    cloud.Sql.CreateOrUpdate(name: "orders-db", ...)
      PUT /instances/{name} → Map upsert → lattice meet ✓

    audit.Logger.Append(entry: ...)
      POST /logs → List append → NOT lattice
      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
      Append creates duplicate entries on retry.

  Effect composition: meet ∘ append = non-idempotent.
  The retry context requires all operations to be idempotent.

  Fix: either
    (a) Move the audit log outside the retry block, or
    (b) Use audit.Logger.Upsert with a deduplication key:
        audit.Logger.Upsert(key: request_id, entry: ...)
```

**Why it catches it:** The effect algebra (`std/effects.dag`)
derives idempotency from effect shape. `CreateOrUpdate` with a
key is a lattice meet (idempotent). `Append` without a key is a
monoid concatenation (not idempotent). The `retry` context
declares that its action must be idempotent. Composition fails.

**How it emerges from modeling:** The programmer modeled their
services honestly — `CreateOrUpdate` uses PUT with a key,
`Append` uses POST without a key. The transport declarations
carry enough information to derive the effect shape. The `retry`
combinator declares its contract. The contradiction between
"retry requires idempotent" and "Append is not idempotent"
is found automatically.

---

## 4. Dead effect: write then overwrite

```dag
func update_config(db: Database, new_config: Config) {
  uses cloud.Sql

  cloud.Sql.UpdateConfig(name: db.name, config: default_config())

  let merged = merge_configs(base: default_config(), override: new_config)

  cloud.Sql.UpdateConfig(name: db.name, config: merged)
}
```

**Compiler error:**
```
error[EFFECT]: dead effect — first write is subsumed

  cloud.Sql.UpdateConfig(name: db.name, config: default_config())
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  This effect is overwritten by:

  cloud.Sql.UpdateConfig(name: db.name, config: merged)
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  at line 8.

  Both operations target the same key (db.name) with UpsertEffect.
  The second upsert subsumes the first:
    upsert(k, v1) ∘ upsert(k, v2) = upsert(k, v2)

  The first UpdateConfig has no observable effect.

  Fix: remove the first UpdateConfig call.
```

**Why it catches it:** Two upserts to the same key compose to
just the second upsert. The effect algebra knows that
`meet(state, {k: v1})` followed by `meet(state, {k: v2})`
equals `meet(state, {k: v2})` — the first is absorbed.

**How it emerges from modeling:** The `UpdateConfig` operation
declares its transport as `PUT /configs/{name}`. The `{name}`
in the path is the key. Two PUTs to the same key = the first
is dead. No special "dead store" analysis — it falls out of
lattice absorption.

---

## 5. Accidentally quadratic

```dag
fn find_duplicates(items: List<Item>) -> List<Item> {
  items |> filter(item =>
    items |> count(other => other.id == item.id) > 1
  )
}
```

**Compiler error:**
```
error[CX]: O(n^2) — cheaper equivalent exists

  items |> filter(item =>
    items |> count(other => other.id == item.id) > 1
    ^^^^^
    Inner fold over `items` (length n) inside outer fold
    over `items` (length n).
  )

  Cost: O(n) * O(n) = O(n^2)

  Cheaper equivalent using Map grouping: O(n)
    let counts = items |> fold(init: {}, f: (acc, item) =>
      acc |> map_upsert(key: item.id, value: 1, merge: add))
    items |> filter(item => counts |> get(key: item.id) > 1)
```

**Why it catches it:** CX computes cost by composing fold costs.
`filter(n, body: count(n, ...))` = `fold(n, fold(n, ...))` = O(n²).
If the compiler's optimization catalog has a cheaper equivalent
(group-by via Map, O(n)), it rejects the quadratic version.

**How it emerges from modeling:** `List<T>` operations have
declared costs in `std/`. `filter` is O(n). `count` inside
filter makes the body O(n), so total is O(n²). The `Map`
group-by pattern is declared as O(n). CX compares and rejects.

---

## 6. Infrastructure bringup: already running is benign

```dag
func bring_up_service(config: ServiceConfig) {
  uses cloud.Compute, cloud.Firewall, cloud.Dns

  cloud.Compute.CreateOrUpdate(
    id: config.service_id,
    spec: config.machine_spec
  )

  cloud.Firewall.CreateOrUpdate(
    name: concat(config.service_id, "-allow-https"),
    rule: config.firewall_rule
  )

  cloud.Dns.CreateOrUpdate(
    name: config.hostname,
    target: config.service_id
  )
}
```

**Compiler output (not an error — a proof):**
```
proof[EFFECT]: bring_up_service is idempotent ✓

  cloud.Compute.CreateOrUpdate: PUT /instances/{id}
    → Map<id, spec> upsert (lattice meet) ✓
  cloud.Firewall.CreateOrUpdate: PUT /firewalls/{name}
    → Map<name, rule> upsert (lattice meet) ✓
  cloud.Dns.CreateOrUpdate: PUT /dns/{name}
    → Map<name, target> upsert (lattice meet) ✓

  Composition: meet ∘ meet ∘ meet = meet ✓
  Running again when services are already up: no state change.

  Generated test:
    test "bring_up_service is idempotent" {
      let s1 = bring_up_service(config: test_config)
      let s2 = bring_up_service(config: test_config)
      assert s1 == s2
    }
```

**Why it works:** Every operation uses PUT with a key. PUT with
a key = Map upsert = lattice meet. Lattice meets compose. The
workflow is idempotent by construction. Running it when services
are already up is a no-op — every upsert converges to the same
state.

**How it emerges from modeling:** The programmer just used
`CreateOrUpdate` with named resources. The compiler reads the
transport declarations (PUT + key), derives the effect shape
(UpsertEffect), checks the algebra (lattice meet), and proves
idempotency. No `@idempotent` annotation. No `@safe_to_retry`.
The structure carries the proof.

---

## The pattern

Every example above is the same mechanism at work:

1. **The programmer models facts honestly** — types, operations,
   transports, algebraic inhabitants. No special annotations.

2. **The compiler reads the algebra** — lattice laws, group
   inverses, involutions, cost functions. All declared in `std/`.

3. **The compiler composes and simplifies** — symbolic composition
   of operations under their algebraic laws.

4. **Contradictions surface automatically** — a non-terminating
   recursion is a missing descent proof. A non-idempotent retry
   is a lattice violation. A dead effect is lattice absorption.
   Redundant work is an algebraic simplification.

No lint rules. No special-case checks. No opt-in. The algebra
does the work. Adding a new algebraic law to `std/` makes every
program that uses that algebra gain the corresponding check — for
free, retroactively, without touching any user code.
