# Impossible Bug Classes — Worked Examples

> **Parent:** `THESIS.md` §"Enumerable impossible-bug classes" + §"Correctness is structural, not behavioral (meta-claim)"

## Focus

The compiler industry has already moved past Python-class bugs. Rust catches
null dereferences, data races, non-exhaustive matches, and use-after-free
— that's table stakes for any modern production compiler. The interesting
question, and the one gunbc is accountable to, is:

**Which bugs do Rust and C++ — the most rigorous production compilers in
common use — still let through? Where do engineers building systems in
best-in-class structured languages still bleed?**

This doc audits gunbc against *those* classes. Bugs Rust already handles
are covered briefly at the end as table stakes.

## Governing rule

The list below is a **completeness specification**, not a demo suite. If a
bug class is not handled, the response is not "schedule it" — it's a **design
audit**: where did the structure miss this? Can the structure be enhanced so
the compiler takes on more of the work?

R1 ships when every class here has a satisfactory structural story: either
it's already IBC/CE in gunbc, or there is a named structural path to make
it so.

## Classification rubric

For each bug class, we record **two independent verdicts**:

- **Rust / C++ status** — what do the best-in-class structured compilers do? One of: `IBC` (impossible by construction), `CE` (compile error), `idiom` (library / discipline only), `runtime` (panics / UB at runtime), `no-help` (compiler offers nothing).
- **gunbc status** — same rubric: `IBC` / `CE` / `PARTIAL` / `GAP`.

A class is interesting for gunbc when Rust/C++ is `idiom` / `runtime` /
`no-help` AND gunbc is `IBC` / `CE` (or has a named structural path).

## How this doc is used

- THESIS commits to the list of bug classes (the *what*).
- This doc records *how* Rust/C++ handle each + *how* gunbc handles it +
  *what's missing* if anything.
- Gaps trace to ROADMAP follow-up lanes.
- When a new bug class surfaces externally (user reports, security
  audits, design-review findings), add it here first; fix structure
  second.

---

## Group A — Cross-system correctness (Rust/C++ scope limits)

Rust and C++ are single-binary languages. They don't model
cross-service correctness or cross-target consistency because those are
out of scope for a traditional compiler. gunbc's omni-emission puts
these in scope structurally.

### 1. Schema drift between client and server

**Traditional example (Rust client + Rust server, different crates).**
```rust
// Server crate
#[derive(Serialize)] struct User { id: u64, email: String }

// Client crate — one refactor later, diverges
#[derive(Deserialize)] struct User { id: u64, email_address: String }
```
Runtime `serde` parse fails; UI breaks; crash in production.

**Rust / C++ status: `no-help`.** Two crates, two type systems, no
shared authority. Protobuf / OpenAPI codegen helps but is out-of-band
tooling.

**gunbc status: CE** (trending IBC).
Client, server, and any other target derive from the same `.dag`
declaration. A field's type is declared once; all targets emit from it.
There is no second authority to drift against.

**Evidence:** `dsl/std/coercion.dag` (single-authority type checkpoint);
THESIS §"Omni-emission" and §"Target realization efficiency".

---

### 2. Transport / type drift (REST path mismatched with typed route)

**Traditional example.** Server declares `GET /orders/:id` where `id` is
expected as UUID; Rust client builds URL: `format!("/orders/{}",
order_number_as_u64)`. No compile-time check that `u64` matches UUID.

**Rust / C++ status: `idiom`.** Rust web frameworks (axum, actix) have
typed extractors for server-side; client-side URL construction is still
`format!` or string concat with type information lost at the boundary.

**gunbc status: CE.**
Path templates parse into typed tokens (`std.http_path::PathTemplate`).
Parameter types declared at `service` level, consumed at call sites;
type mismatch is a compile error across the call boundary.

**Evidence:** `dsl/std/http_path.dag:18-79` (PathTemplate + typed tokens);
`dsl/std/effects.dag:73-91` (KeySource typed discriminator).

---

### 3. Cross-language behavior divergence (same program, different targets)

**Traditional example.** Service ported from Rust → Python for a new
team. Slight float-rounding behavior differs. Bug in prod under load.

**Rust / C++ status: `no-help`.** Not in scope for any single-language
compiler. Cross-language equivalence requires differential testing.

**gunbc status: CE** (gated on v2-oracle cementing test in Lane E;
structural by emission-from-one-source).
Same `.dag` emits to Rust, Python, Go. Cementing tests prove behavioral
equivalence per structural form. Differential drift becomes a compile-
time assertion in Lane E's acceptance claim.

**Evidence:** `src/v3/spec/rust.dag` / `python.dag` / `go.dag` (per-
target specs derive from shared substrate); Lane E cementing test plan.

---

### 4. API version mismatch between services

**Traditional example.** Service A upgrades from `Order v1` to
`Order v2` (new `currency` field). Service B still speaks v1. Silent
behavior changes at the boundary.

**Rust / C++ status: `no-help`.** Semantic versioning is convention.
Runtime error-or-surprise.

**gunbc status: PARTIAL.**
Schema drift (class 1) is covered; versioned protocol evolution isn't
yet explicit. Current thinking: model `OrderV1` / `OrderV2` as distinct
types with explicit conversion functions. Callers that try to send V2 to
a V1-expecting endpoint get a compile error.

**Structural path.** Protocol-versioning model in `std.services`. Size:
M. Folds naturally into T-LLM-Services pattern.

**R1 status.** PARTIAL → close for R1. Lane extension.

---

## Group B — Effects and properties (beyond any mainstream compiler)

Rust's `impl Trait` and effect-adjacent bounds (`Send` / `Sync`) stop at
thread safety. Effect enumeration, idempotency, and semantic invariants
aren't there in any mainstream language.

### 5. Unenumerated effects (declared ≠ actual)

**Traditional example.**
```rust
/// reads from S3
async fn process(data: Item) -> Summary {
    let raw = s3_get(data.key).await?;      // documented
    slack_post(&format!("{raw}")).await?;   // NOT documented
    summarize(raw)
}
```
Docstring lies. Runtime discovers the lie when auditing network
egress.

**Rust / C++ status: `no-help`.** `async` tracks "this function may
await" but nothing finer. Effect systems require type-system extensions
(Koka, Eff, OCaml 5 has them partially).

**gunbc status: GAP.**
`std.effects` has effect-shape carriers (`ReadEffect`, `UpsertEffect`);
composition logic exists. Missing: declared-effect syntax on function
signatures + an enumeration walker that aggregates actual effects from
body up the call graph + a comparator that fails on mismatch.

**Structural path.** Three pieces (size: L):
1. Surface syntax: `fn foo(...) @effects [Http, Db] ...`
2. Effect-enumeration walker (lens)
3. Comparator (declared vs enumerated → compile error)

**R1 status.** GAP → close for R1. Dedicated lane (T-Effects, L).

---

### 6. Idempotency violation (retry-replay)

**Traditional example.**
```rust
#[retry(max_attempts = 3)]
async fn run_workflow() {
    fetch_issues().await?;
    summarize().await?;
    slack_post(summary).await?;   // not idempotent; duplicates on retry
}
```
`retry` over a tail with non-idempotent steps produces duplicate Slack
posts. Classic at-least-once delivery pain.

**Rust / C++ status: `no-help`.** Idempotency is a property of the body,
invisible to any mainstream type system.

**gunbc status: IBC** (v3 BEHAVIORALLY COMPLETE).
Idempotency is a property of the effect shape's algebraic structure.
`is_idempotent_effect(shape)` pattern-matches on `EffectShape`;
`compose_effects` checks the composition. `retry_on_failure` applied to
a workflow with non-idempotent tail is a compile error.

**Evidence:** `dsl/std/effects.dag:108-150`;
`docs/v3-lens-capability-register.md:43` (v3 COMPLETE);
`src/v3/lenses/idempotency.dag`.

---

### 7. Suboptimal complexity (contract violation)

**Traditional example.** Engineer writes `fn dedupe(xs)` with documented
`O(n log n)`. Three refactors later it's `O(n²)` (hash table replaced
with nested-loop scan). Nothing catches it until production latency.

**Rust / C++ status: `no-help`.** Complexity isn't part of any type
system. Review catches it sometimes; profilers catch it in production.

**gunbc status: CE in v2, PROXY in v3.**
v2's complexity lens derives `CostExpr(work, span, asymptotic_class,
certainty)` structurally; functions exceeding declared bounds are
compile errors. v3 currently produces a single integer depth per port
(structurally terminal, behaviorally weaker). Lane E is the port
program to restore v2 parity.

**Evidence:** `docs/v3-lens-capability-register.md:40-41` (PROXY
downgrade); `src/v3/lenses/complexity.dag` (v3 proxy);
`src/v2/complexity.dag` (v2 CE).

**R1 status.** v3 must reach v2 parity (T-LaneE XL, already in R1).

---

## Group C — Units / structural constraints (Rust partial via libraries)

Rust has `uom` (library) and newtypes; C++ has template meta-programming.
Both require significant discipline. gunbc makes them substrate.

### 8. Unit / dimension mismatch

**Traditional example.**
```rust
fn sleep_ms(duration: Duration) { ... }   // duration actually seconds
sleep_ms(timeout_seconds);                  // 1000× error, silent
```
`Duration` is often one type for both millis and seconds; engineers name
variables to distinguish and occasionally lie.

**Rust / C++ status: `idiom`.** Rust's `uom` crate is opt-in with
verbose phantom-typed values. C++ does this with templates but the
ergonomics are punishing. Neither is the default path.

**gunbc status: PARTIAL.**
DB-3 (user-declared dimensions) core landed per ROADMAP:226 —
`docs/db-history/db-3.md` documents what shipped: the generic dimension
framework + dimensional type parameters. What hasn't landed: **unit-
mismatch enforcement consumer** — the lens / pass that rejects
assigning `Duration<Millisecond>` where `Duration<Second>` is expected.
Infrastructure exists; enforcement wire-up missing. Plus generic `.dag`
lowering of user-authored dimensions + example-authoring per DB-3's
named follow-ups.

**Structural path.** Wire DB-3 carriers into a unit-mismatch lens that
produces a compile error on dimension-incompatible assignments /
function applications. Size: M (consumer-wire-up, not substrate
extension).

**R1 status.** PARTIAL → close for R1. T-Dimensions lane now scoped
against DB-3 core rather than from scratch.

---

### 9. Nested-optional flatten (`Option<Option<T>>`)

**Traditional example (Rust).**
```rust
let config: Option<Option<PortNumber>> = load_config();
let port = config.flatten().unwrap_or(8080);   // user flattens by hand
```
This pattern — `Option<Option<_>>`, `Option<Result<_, _>>`, `Result<Result<_, _>, _>` —
appears constantly in real Rust code. The types are valid; users must
remember to flatten.

**Rust / C++ status: `idiom`.** Rust allows the nesting; `.flatten()` is
manual. C++ has `std::optional<std::optional<T>>` with no flatten in
the standard.

**gunbc status: GAP.**
Users can currently construct `Option<Option<T>>`. Cardinality
refinement (which would collapse the nesting to `AtMost(1)`
automatically) is deferred.

**Structural path.** Extend `Cardinality` composition law:
`Cardinality<Cardinality<T, AtMost(1)>, AtMost(1)> ≡ Cardinality<T, AtMost(1)>`.
Substrate type-system work. Size: M-L.

**R1 status.** GAP → close for R1. Lane (T-Cardinality, M-L; may fold
into T-Sub).

---

### 10. Secret leak to logs / outputs

**Traditional example (Rust).**
```rust
#[derive(Debug)] struct Token(String);   // Debug derive leaks in logs
info!("auth failed: {token:?}");          // token printed to CloudWatch
```
Convention: don't derive `Debug` on secrets. Enforcement: code review.

**Rust / C++ status: `idiom`.** Rust's `secrecy` crate wraps and
restricts `Debug`; still opt-in. No compile-time block against logging.
Anyone deriving `Debug` or adding a `.to_string()` breaks the discipline.

**gunbc status: GAP.**
`dsl/std/types.dag:237` declares `Secret = String` as a **type alias**,
not an opaque nominal type. `dsl/std/coercion.dag:114` documents that
alias casts are identity at emit — a `Secret` value coerces freely to
`String`.

**Structural path.** Make `Secret<T>` a nominal opaque wrapper (not an
alias). Construction only through authenticated sinks; coercion back to
the underlying type disallowed. Logging / string-concat of a `Secret<T>`
is a compile error. Size: S-M.

**R1 status.** GAP → close for R1. Lane (T-Secret).

---

### 11. Resource leak (file handle, DB connection, transaction not closed)

**Traditional example (Rust).**
```rust
let conn = pool.get()?;
do_thing(&conn);
// Drop fires at end of scope — but you forgot to commit a transaction
```
Drop covers close, not transaction semantics. `Rc<Cycle>` leaks silently.
`mem::forget` exists.

**Rust / C++ status: `idiom`.** Rust's `Drop` / RAII handles simple
close-on-scope-end. `Drop` does NOT run on `mem::forget`, on process
termination paths, or on `Rc` cycles. C++ RAII is similar. Linear typing
(Haskell has it) would strictly enforce "use exactly once"; Rust is
affine (use at most once).

**gunbc status: PARTIAL.**
`dsl/std/resources.dag` declares `ResourceHandle` with acquire/release
semantics, opaque by intent (though see class 10 — enforcement is
weaker than documented). No compile-time check that every acquire is
paired with a release.

**Structural path.** Linear typing on `ResourceHandle<T>`: each handle
must be used exactly once (consumed by release or a transforming op).
Forgetting to release is a "value not consumed" compile error. Size: M
(extends ownership lens).

**R1 status.** PARTIAL → close for R1. Lane (T-Resource-Ownership).

---

## Group D — Runtime-safety footguns (Rust/C++ both lose here)

Rust panics on overflow in debug, wraps in release (silently incorrect).
Array bounds panic at runtime. Division by zero panics. C++ is
undefined behavior across the board. Neither has compile-time safety
here by default.

### 12. Array out-of-bounds

**Traditional example.**
```rust
let arr = [0u8; 10];
let x = arr[15];   // panics at runtime in Rust, UB in C++
```
Static-length arrays could have this checked at compile time; neither
language does for arbitrary expressions.

**Rust / C++ status: `runtime`.** Rust panics (safe but crashes prod).
C++ is undefined behavior (unsafe — memory corruption).

**gunbc status: GAP.**
No `FixedArray<T, n>` with bounded index types yet. `List<T>` iteration
is safe; direct indexing with unchecked integers doesn't exist in user
code, but emission targets need it.

**Structural path.** `Cardinality(element, Exact(n))` as type-level
length; indices typed `BoundedInt<0, n>`. In-bounds is a type check.
Size: subsumed by T-Tier2.

**R1 status.** GAP → close for R1. Part of T-Tier2.

---

### 13. Division by zero

**Traditional example.**
```rust
let rate = total / count;   // panics if count == 0 in Rust; UB in C++ for ints
```

**Rust / C++ status: `runtime`.** Rust panics. C++ integer div by zero
is UB.

**gunbc status: GAP.**
`dsl/std/float.dag:13-14` declares `Float` as `Field<Word64>`;
Field admits division without non-zero-divisor proof. Tier 2 runtime
safety is THESIS-committed but not substrate-landed.

**Structural path.** Either (a) `div` returns `Option<T>`, callers
pattern-match (IBC via API shape); or (b) divisor typed `NonZero<T>`
with compile-time proof from surrounding structure (CE via type wall).

**R1 status.** GAP → close for R1. T-Tier2.

---

### 14. Integer overflow

**Traditional example (Rust in release mode).**
```rust
let a: u32 = u32::MAX;
let b = a + 1;   // wraps to 0 silently in release; panics in debug
```
Release builds *silently wrap*, producing wrong answers that pass all
tests not looking for them.

**Rust / C++ status: `runtime`/`UB`.** Rust's split debug-vs-release
behavior is well-known as a production footgun. C++ is UB for signed
overflow.

**gunbc status: GAP.**
`dsl/std/integer.dag` defines `Int64` as `OrderedRing<Word64>`;
`OrderedRing` doesn't prevent overflow on `+` / `*`.

**Structural path.** Default to unbounded integer (`BigInt`) with
explicit fixed-width opt-in; fixed-width arithmetic returns `Option<T>`
or requires overflow proof. Size: part of T-Tier2.

**R1 status.** GAP → close for R1. T-Tier2.

---

### 15. Stack overflow from unbounded recursion

**Traditional example.**
```rust
fn bad(n: u64) -> u64 { bad(n + 1) }   // stack overflow at runtime
```
Compiler doesn't prove termination; loops crash at scale.

**Rust / C++ status: `no-help`.** Termination isn't checked.
Tail-recursion optimization is a runtime implementation detail, not a
correctness proof.

**gunbc status: CE.**
`.dag` code is decidable — termination bounds must be structurally
proven (descent evidence on recursive calls). Infinite recursion is a
compile error. See `feedback_decidability_invariant` (load-bearing
invariant).

**Evidence:** INVARIANTS §P4 Decidability; `dsl/std/induction.dag`
(SubValueRelation carriers).

---

## Group E — Injection family (Rust libraries help; strings still work)

Rust's type-checked query builders (sqlx, diesel) are great; raw string
query construction is still syntactically valid. SQL, shell, log,
template, and path injection are all the same structural problem.

### 16. SQL / shell / log / template injection

**Traditional example (Rust, and every other language).**
```rust
let query = format!("SELECT * FROM users WHERE id = {user_id}");
conn.execute(&query)?;
```
`format!` is syntactically valid; runtime evaluates the user-controlled
string. SQL libraries like sqlx offer typed queries but raw
`execute(&str)` still compiles.

**Rust / C++ status: `idiom`.** Rust's sqlx provides `query!(...)` with
compile-time SQL parsing, but `query(&string)` (non-macro) still exists.
C++ has prepared-statement APIs but string APIs are still present.

**gunbc status: IBC.**
User code cannot concatenate or interpolate strings. No `+` for strings,
no template-literal syntax, no `concat()` in `dsl/std/` exposed to user
programs. Database queries must be structured data (typed operation +
typed parameters). The same property covers shell, log-format, and path
injection — all are the same construction-level guarantee.

**Evidence:** `dsl/std/types.dag` (String primitive with no concat
exposure); `dsl/std/containers.dag:19` (concat for monoid operations,
not for scalar String in user code).

---

### 17. Path traversal

**Traditional example.**
```rust
let requested = format!("/static/{user_input}");
fs::read(&requested)?;   // user_input = "../../etc/passwd"
```
Rust's `Path` / `PathBuf` help carry path semantics but don't prevent
traversal patterns at construction time.

**Rust / C++ status: `idiom`.** Typed path abstractions exist but
sanitization is user discipline.

**gunbc status: PARTIAL** (foundation: no string concat; no user code
can build raw filesystem paths without going through typed APIs).

**Structural path.** Typed `SafePath` with construction rules disallowing
`..` components; filesystem ops take only `SafePath`. Size: S (extends
existing no-string-concat guarantee).

**R1 status.** PARTIAL → can close for R1 cheaply once
`T-Secret` pattern is established (nominal types with construction
rules). Fold into T-Secret lane.

---

## Group F — Table stakes (Rust already handles)

These are well-covered by Rust and any rigorous compiler. gunbc inherits
the guarantee but does not claim novelty here. Listed for completeness
so the full bug-class ledger is visible.

| # | Class | Rust status | gunbc status | Note |
|---|---|---|---|---|
| T1 | Null dereference | IBC (Option) | IBC | Pattern-match only; no null |
| T2 | Use-after-free | IBC (ownership) | IBC | No raw references; pure-functional |
| T3 | Double-free | IBC | IBC | Same |
| T4 | Data race on shared state | IBC (Send/Sync) | IBC | Pure-functional; no mutable shared state |
| T5 | Non-exhaustive match | CE | CE | Enforced in `src/v3/compiler/src/infer.rs` |
| T6 | Force-unwrap panic | idiom (exists but not used) | IBC | No `unwrap()` method in `dsl/std/` |
| T7 | Empty-list head access | CE (Option return) | CE | `first()` returns Option |
| T8 | Iterator invalidation | IBC | IBC | Immutable collections |
| T9 | Dangling reference | IBC | IBC | No raw references |

gunbc's guarantee here is no stronger than Rust's; the pitch is that
these classes *stay* closed while the substrate adds Group A-E coverage
Rust can't touch.

---

## Summary table (interesting classes)

| # | Bug class | Rust/C++ | v2 | v3 | Gap → path |
|---|---|---|---|---|---|
| 1 | Schema drift (client/server) | no-help | CE | CE | — |
| 2 | Transport/type drift (REST path) | idiom | CE | CE | — |
| 3 | Cross-language behavior divergence | no-help | partial | CE (Lane E) | Cementing test |
| 4 | API version mismatch | no-help | PARTIAL | PARTIAL | Protocol-versioning model |
| 5 | Unenumerated effects | no-help | GAP | GAP (carriers partial) | **T-Effects** (L) |
| 6 | Idempotency violation | no-help | CE | COMPLETE | — |
| 7 | Suboptimal complexity | no-help | CE | PROXY | **T-LaneE** (XL, in R1) |
| 8 | Unit / dimension mismatch | idiom | GAP | GAP | **T-Dimensions** (L) |
| 9 | Nested-optional flatten | idiom | GAP | GAP | **T-Cardinality** (M-L) |
| 10 | Secret leak to logs | idiom | GAP | GAP | **T-Secret** (S-M) |
| 11 | Resource leak | idiom | PARTIAL | PARTIAL | **T-Resource-Ownership** (M) |
| 12 | Array out-of-bounds | runtime | GAP | GAP | **T-Tier2** (XL) |
| 13 | Division by zero | runtime | GAP | GAP | **T-Tier2** |
| 14 | Integer overflow | runtime/UB | GAP | GAP | **T-Tier2** |
| 15 | Stack overflow from unbounded recursion | no-help | CE | CE | — |
| 16 | SQL / shell / log injection | idiom | IBC | IBC | — |
| 17 | Path traversal | idiom | PARTIAL | PARTIAL | Fold into T-Secret |

## Scoreboard

- **Handled today** (IBC / CE / BEHAVIORALLY COMPLETE): 6/17 interesting classes + 9/9 table stakes
- **Gap with named structural path:** 8/17 interesting
- **Special (v3 regression being restored):** 1/17 (complexity — Lane E)
- **Requires runtime-only mitigation:** 0

Every interesting class Rust/C++ can't / won't catch is either handled
today or has a named structural path. No class is "we'll catch it at
runtime" by default.

## R1 scope implications

Per the governing rule (every GAP must close for R1), the audit names
these additions to PR #669's R1 program:

| Lane | Classes closed | Relative size |
|---|---|---|
| **T-Tier2** | 12 (bounds), 13 (div-zero), 14 (overflow) | **XL** — new XL lane, likely own manager (M5) |
| **T-Effects** | 5 (unenumerated effects) | L |
| **T-Dimensions** | 8 (unit mismatch) | L |
| **T-Cardinality** | 9 (nested-optional), partial 12 | M-L (may fold into T-Sub) |
| **T-Secret** | 10 (secret leak), 17 (path traversal) | S-M |
| **T-Resource-Ownership** | 11 (resource leak) | M (extends ownership lens) |
| **T-LaneE** | 7 (complexity) | XL (already in R1) |
| **T-LLM-Services** extension | 4 (API versioning) | minor extension |

**Critical-path effect:** T-Tier2 becomes a third XL lane alongside
T-LaneE and T-PB-A. Likely introduces **M5** manager dedicated to
Tier-2 substrate work.

**Rough aggregate:** ~50-80% more work than the PR #669 R1 framing
committed. Not a doubling; not cheap. Every added class is Rust/C++-class
differentiation, not Python-class paper-scores.

## Maintaining this doc

- **New bug class surfaced externally** (user report, security audit,
  design-review finding): add a row with Rust/C++ status + traditional
  example + gunbc status. If the latter is GAP, escalate to ROADMAP debt
  ledger and R1 scope review.
- **Structural addition lands:** update gunbc status (GAP → CE / IBC),
  link closing PR / lane.
- **Rust/C++ field advances** (e.g., a new effect-system extension):
  update Rust/C++ status so we track where the bar moves.

This doc is parallel authority with THESIS §"Enumerable impossible-bug
classes" on the *bug-class list*. When they diverge, THESIS is the list
authority; this doc is the how/status authority. Adding a class means
touching both.
