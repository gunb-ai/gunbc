# What .dag catches: concrete examples

Part of: [THESIS.md](../THESIS.md) §[What .dag catches](../THESIS.md#what-dag-catches-that-normal-compilers-dont)
Related: [ROADMAP.md](../ROADMAP.md) §[Status at a glance](../ROADMAP.md#status-at-a-glance) |
[std/effects.dag](../dsl/std/effects.dag) |
[std/algebra.dag](../dsl/std/algebra.dag) |
[std/termination.dag](../dsl/std/termination.dag)

Every example below compiles in Rust, Go, and Python. Every example
is rejected by .dag at compile time. No lint rules, no annotations,
no opt-in — the errors emerge from the algebraic structure that
`.dag` already has.

**These serve as TDD targets:** each example is a test case. The
`.dag` code is the test input; the error message is the acceptance
criterion. When the feature lands, the test should pass.

Each example shows:
1. The `.dag` code
2. The compiler error
3. Why it catches it — which algebra, with links to the source
4. How it emerges from modeling — not a special-case check
5. Why a traditional compiler can't catch it

---

## 1. Non-terminating recursion through type resolution

**Severity:** production crashes. This bug class has hit TypeScript
(recursive type aliases), Rust (trait resolution cycles), and
Haskell (type family expansion). It's not a beginner mistake — it's
a fundamental design issue that manifests in mature, well-tested
compilers.

```dag
fn check_type(t: Node) -> Bool {
  match t.connective {
    Leaf => true
    Generic => {
      let resolved = lookup_type_def(name: t.name)
      check_type(t: resolved)
    }
    Conj => t.children |> all(c => check_type(t: c))
  }
}
```

**Compiler error:**
```
error[CX]: cannot prove termination of check_type

  check_type(t: resolved)
               ^^^^^^^^^
  `resolved` came from lookup_type_def() — a lookup, not structural
  descent on `t`. SubValueRelation: Unknown.

  The Conj branch is fine:
    check_type(t: c) where c is IteratedSubValue of t.children ✓

  The Generic branch is the problem:
    check_type(t: resolved) — no descent relationship to t.
    If the looked-up type contains Generic references, this
    recurses without bound.

  Fix: separate resolution from walking. Resolve all type
  references in a prior pass, then walk the resolved tree
  where descent on Node.children is structurally bounded.
```

**Algebra:** [std/termination.dag](../dsl/std/termination.dag) —
well-founded descent. [std/induction.dag](../dsl/std/induction.dag)
— SubValueRelation tracks whether each argument is structurally
smaller. See [THESIS.md §Correctness dimensions](../THESIS.md#correctness-dimensions).

**Why a traditional compiler can't catch it:** Rust/Go/Python
have no concept of "structural descent." They check types, not
termination. A function that recurses on a lookup result is
syntactically identical to one that recurses on a child — the
type system can't distinguish them. Only a system with bounded
iteration primitives and mandatory descent proofs catches this.

**TDD target:** [ROADMAP.md §Status at a glance](../ROADMAP.md#status-at-a-glance)
— CX gate. Example 1 is a generalization of our own
`render_node_type` recursion through `n.inferred`.

---

## 2. Cross-service data corruption through non-idempotent retry

**Severity:** silent data corruption in production. The retry
succeeds, the system appears healthy, but audit logs have
duplicates, billing gets double-charged, or notifications fire
twice. The bug is invisible until someone audits the data.

```dag
func process_payment(order: Order) {
  uses payment.Gateway, billing.Ledger, notify.Email

  let charge = payment.Gateway.Charge(
    customer_id: order.customer_id,
    amount: order.amount,
    idempotency_key: order.id
  )

  billing.Ledger.AppendEntry(entry: LedgerEntry {
    order_id: order.id,
    amount: order.amount,
    charge_id: charge.id
  })

  notify.Email.Send(
    to: order.customer_email,
    template: "payment_confirmed",
    data: charge
  )
}

func process_with_retry(order: Order) {
  uses retry
  retry.WithRetries(max: 3, action: () => process_payment(order: order))
}
```

**Compiler error:**
```
error[EFFECT]: non-idempotent operations in retry context

  retry.WithRetries(action: () => process_payment(order: order))
                                  ^^^^^^^^^^^^^^^
  process_payment() is NOT idempotent:

    payment.Gateway.Charge(idempotency_key: order.id)
      POST /charges with key → UpsertEffect ✓ (key guards retry)

    billing.Ledger.AppendEntry(entry: ...)
      POST /entries (no key) → AppendEffect ✗
      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
      Duplicate ledger entries on retry. Double-charges the order.

    notify.Email.Send(to: ..., template: ...)
      POST /send (no key) → AppendEffect ✗
      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
      Duplicate emails on retry.

  Effect composition: UpsertEffect ∘ AppendEffect ∘ AppendEffect
  = non-idempotent. Retry context requires idempotent composition.

  Fix:
    billing.Ledger.UpsertEntry(key: order.id, entry: ...)
    notify.Email.SendOnce(key: order.id, ...)
  Or: move ledger + email outside the retry block.
```

**Algebra:** [std/effects.dag](../dsl/std/effects.dag) — EffectShape
composition. [docs/thesis/what-else-falls-out.md §Algebraic simplification](./thesis/what-else-falls-out.md#algebraic-simplification-idempotency-cancellation-redundancy).

**Why a traditional compiler can't catch it:** The types are all
correct. The function signatures match. The control flow is valid.
No compiler in any mainstream language tracks effect shapes through
function composition to detect that retrying a workflow with an
`Append` operation produces duplicates. This requires algebraic
reasoning about state effects — lattice meets compose safely,
monoid appends don't.

**TDD target:** Effect algebra consumption. The types exist in
`std/effects.dag`; compiler consumption is not wired.

---

## 3. Redundant computation across service boundaries

**Severity:** wasted latency, money, and API quota. Not a crash,
but a systemic cost that multiplies with traffic.

```dag
func enrich_user_profile(user_id: String) -> EnrichedProfile {
  uses crm.Service, analytics.Service

  let crm_data = crm.Service.GetUser(id: user_id)
  let analytics_data = analytics.Service.GetUserMetrics(id: user_id)

  let enriched = merge_profile(crm: crm_data, analytics: analytics_data)

  // Bug: fetches CRM data again to get the same name field
  let display_name = crm.Service.GetUser(id: user_id).name

  EnrichedProfile {
    profile: enriched,
    display_name: display_name
  }
}
```

**Compiler error:**
```
error[EFFECT]: redundant service call — result already available

  let display_name = crm.Service.GetUser(id: user_id).name
                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  crm.Service.GetUser(id: user_id) was already called at line 4.

  GetUser is ReadEffect (GET /users/{id}) — deterministic for the
  same input within this scope. The result is already bound to
  `crm_data`.

  Equivalent: crm_data.name
  Cost saved: 1 network round-trip + API quota.
```

**Algebra:** ReadEffect is idempotent and deterministic. Two calls
with the same arguments in the same scope produce the same result.
The second call is algebraically redundant — it's `f(x)` when
`f(x)` is already bound to `crm_data`.

**Why a traditional compiler can't catch it:** The two calls cross
a service boundary. A traditional compiler sees two function calls
with the same arguments and has no idea they're deterministic (the
function might have side effects, read mutable state, etc.). .dag
knows the effect shape (ReadEffect = deterministic) from the
transport declaration (GET), so it can prove the redundancy.

---

## 4. Accidentally quadratic with a non-obvious cause

**Severity:** O(n²) masquerading as O(n). Works fine in testing
with small datasets. Falls over in production with real data.

```dag
fn deduplicate(items: List<Record>) -> List<Record> {
  items |> fold(init: [], f: (seen, item) =>
    if seen |> any(s => s.id == item.id) {
      seen
    } else {
      seen |> append(item)
    }
  )
}
```

**Compiler error:**
```
error[CX]: O(n^2) complexity — cheaper equivalent exists

  items |> fold(init: [], f: (seen, item) =>
    if seen |> any(s => s.id == item.id) {
                  ^^^
  Inner fold: `any` over `seen` (grows to length n).
  Outer fold: over `items` (length n).

  Cost: O(n) * O(n) = O(n^2)

  `seen |> any(s => s.id == item.id)` is a membership test.
  Membership on List is O(n). Membership on Set is O(1).

  Cheaper equivalent using Set:
    let seen_ids = items |> fold(init: empty_set(), f: (ids, item) =>
      ids |> set_insert(item.id))
    items |> filter(item => seen_ids |> set_contains(item.id))

  Cost: O(n)
```

**Algebra:** [std/algebra.dag](../dsl/std/algebra.dag) — FreeMonoid
(List) vs BooleanAlgebra (Set). Membership cost is declared per
algebraic structure. CX composes costs through fold bodies.
See [ROADMAP.md §Post-A/B Lane Plan](../ROADMAP.md#post-ab-lane-plan).

**Why a traditional compiler can't catch it:** The code is
type-correct. The algorithm is correct. It produces the right
output. It's just slow. No mainstream compiler reasons about
asymptotic complexity — they optimize constant factors (inlining,
vectorization) but can't detect algorithmic inefficiency. .dag has
cost algebra on every operation, so it can compose costs and
compare against known cheaper patterns.

**TDD target:** [ROADMAP.md §Post-A/B Lane Plan](../ROADMAP.md#post-ab-lane-plan)
— optimization catalog in `std/optimization.dag`.

---

## 5. Infrastructure drift through partial failure

**Severity:** infrastructure in an inconsistent state. The workflow
ran, some operations succeeded, some failed. Re-running either
duplicates or skips — depending on which operations are idempotent.
The operator can't tell which state they're in.

```dag
func deploy_service(config: DeployConfig) {
  uses cloud.Registry, cloud.Compute, cloud.LoadBalancer

  // Step 1: push image (idempotent — same tag = same image)
  cloud.Registry.Push(tag: config.image_tag, image: config.image)

  // Step 2: create instance (NOT idempotent — no key!)
  cloud.Compute.Create(spec: config.machine_spec)

  // Step 3: register with load balancer (idempotent — keyed by name)
  cloud.LoadBalancer.Register(
    name: config.service_name,
    target: config.instance_id
  )
}
```

**Compiler error:**
```
error[EFFECT]: non-idempotent operation in deployment workflow

  cloud.Compute.Create(spec: config.machine_spec)
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  POST /instances (no key) → CreateEffect (not idempotent).

  If this workflow fails after step 2 and is retried:
    Step 1 (Registry.Push): safe — UpsertEffect, same result ✓
    Step 2 (Compute.Create): UNSAFE — creates duplicate instance
    Step 3 (LoadBalancer.Register): safe — UpsertEffect ✓

  The workflow is not safe to retry. Partial failure leaves
  infrastructure in an inconsistent state.

  Fix: use Compute.CreateOrUpdate with an explicit instance key:
    cloud.Compute.CreateOrUpdate(
      id: config.service_name,
      spec: config.machine_spec
    )
```

**Algebra:** [std/effects.dag](../dsl/std/effects.dag) — CreateEffect
vs UpsertEffect. The compiler traces the partial-failure scenario
structurally: which operations have already committed, which haven't,
and whether re-running is safe.

**Why a traditional compiler can't catch it:** Infrastructure-as-code
tools (Terraform, Pulumi) handle this at runtime with state files.
.dag catches it at compile time because the effect algebra
distinguishes Create (non-idempotent, unsafe to retry) from
CreateOrUpdate (idempotent, safe to retry) based on whether a
key exists in the transport declaration.

---

## 6. Semantic cancellation across function boundaries

**Severity:** wasted computation that produces no net effect. The
program is correct but does unnecessary work that costs time and
resources. Unlike dead code, both operations ARE used — they just
cancel each other out.

```dag
fn process_message(msg: Message) -> ProcessedMessage {
  let compressed = compress(data: msg.body)
  let encrypted = encrypt(data: compressed, key: msg.recipient_key)
  let decrypted = decrypt(data: encrypted, key: msg.recipient_key)
  let validated = validate_schema(data: decrypted)
  ProcessedMessage { original: msg, validated: validated }
}
```

**Compiler error:**
```
error[OPT]: operation cancellation — encrypt/decrypt is identity

  let encrypted = encrypt(data: compressed, key: msg.recipient_key)
  let decrypted = decrypt(data: encrypted, key: msg.recipient_key)
      ^^^^^^^^^
  encrypt and decrypt are declared inverses (with matching keys).

    encrypt(key: k) ∘ decrypt(key: k) = identity

  `decrypted` is equivalent to `compressed`. The encrypt/decrypt
  pair has no net effect.

  Equivalent:
    let validated = validate_schema(data: compressed)

  Cost saved: encrypt + decrypt operations.
```

**Algebra:** `encrypt` and `decrypt` are declared as an inverse
pair in their algebra (group inverse with key parameter). The
compiler composes operations symbolically and detects when an
operation and its inverse are adjacent. This works across `let`
bindings — it's not pattern matching on syntax, it's algebraic
simplification. See [THESIS.md §Concept unification](../THESIS.md#concept-unification).

**Why a traditional compiler can't catch it:** A traditional
compiler doesn't know that `encrypt` and `decrypt` are inverses.
They're just function calls. Even with inlining, the compiler
would need to prove that the bit-level operations cancel — which
is intractable in general. .dag doesn't prove bit-level
cancellation. It reads the declared algebraic relationship
(inverse pair) and simplifies symbolically. The proof is: "you
declared these are inverses, and you composed them. The
composition is identity."

---

## 7. Exponential blowup from unguarded recursive branching

**Severity:** code that works on small inputs and takes hours or
OOMs on slightly larger inputs. Classic in tree-processing code.

```dag
fn count_paths(tree: Node) -> Int {
  match tree.connective {
    Leaf => 1
    Conj => {
      let left = tree.children |> first
      let right = tree.children |> last
      match left {
        Some { value: l } => match right {
          Some { value: r } =>
            count_paths(tree: l) + count_paths(tree: r)
          None => count_paths(tree: l)
        }
        None => 0
      }
    }
  }
}
```

**Compiler error:**
```
error[CX]: O(2^n) — exponential branching

  count_paths(tree: l) + count_paths(tree: r)
  ^^^^^^^^^^^^^^^^^^^    ^^^^^^^^^^^^^^^^^^^
  Two recursive calls on the same path, both with StrictSubValue
  descent. Each call produces two more calls.

  This is structurally identical to naive fibonacci:
    fib(n-1) + fib(n-2) → O(2^n)

  For a balanced binary tree of depth d: 2^d calls.

  Fix: if paths overlap, memoize:
    fn count_paths(tree: Node, memo: Map<Int, Int>) -> ...
  Or restructure to single-pass fold:
    tree.children |> fold(init: 0, f: (acc, c) =>
      acc + count_paths(tree: c))
```

**Algebra:** [std/termination.dag](../dsl/std/termination.dag) —
CX branching guard. Multiple recursive calls with descent on the
same path produce exponential cost. The function terminates (each
call descends), but its complexity is O(2^n), which CX rejects
when a polynomial equivalent exists.
See [ROADMAP.md §Post-A/B Lane Plan](../ROADMAP.md#post-ab-lane-plan).

**Why a traditional compiler can't catch it:** The code is correct.
It terminates. The types are fine. Traditional compilers have no
complexity analysis — they'll happily compile O(2^n) code. .dag's
CX proves cost bounds on every function and rejects exponential
complexity when the branching structure indicates it.

---

## The pattern

Every example is the same mechanism:

1. **The programmer models facts honestly** — types, operations,
   transports, algebraic inhabitants. No special annotations.

2. **The compiler reads the algebra** — lattice laws, group
   inverses, involutions, cost functions. Declared in
   [std/](../dsl/std/). See [THESIS.md §Correctness dimensions](../THESIS.md#correctness-dimensions).

3. **The compiler composes and simplifies** — symbolic composition
   of operations under their algebraic laws.

4. **Contradictions surface automatically** — a non-terminating
   recursion is a missing descent proof. A non-idempotent retry
   is a lattice violation. A dead effect is lattice absorption.
   Redundant work is algebraic simplification. Exponential
   blowup is branching guard rejection.

No lint rules. No special-case checks. No opt-in. The algebra
does the work. Adding a new algebraic law to `std/` makes every
program that uses that algebra gain the corresponding check — for
free, retroactively, without touching any user code.

See [THESIS.md §What else falls out](../THESIS.md#what-else-falls-out)
for how these properties emerge from the closed system.
