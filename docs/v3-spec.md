// v3 Design Proposal
//
// STATUS: proposal, pending validation via bounded experiments in v2.
// This is a design document, not executable .dag. Types used here
// (NodeId, TypeShape, etc.) are design notation, not imports.
// See docs/v2-retrospective.md for the diagnosis that motivated this.
//
// Two ideas:
//
//   1. PHYSICS — programs desugar to a small set of behaviors that
//      compose. The DAG is the closed world. All structure lives here.
//
//   2. LENSES — analyses (cost, ownership, effects, etc.) are
//      lightweight views over the physics. They observe structure
//      that already exists. If a lens needs information that isn't
//      in the DAG, the physics is incomplete — fix the physics,
//      not the lens.
//
// Design principle: keep behaviors intuitive enough that you can
// explain each one in a sentence. Composition handles complexity.
// Lenses are cheap because the physics is rich.

// =========================================================================
// L1: FUNDAMENTAL BEHAVIORS
// =========================================================================
//
// These cannot be decomposed further. Everything in the language
// desugars to compositions of these five.
//
// The test: can you explain each one to someone who doesn't know
// compilers? If you need jargon, the name is wrong.

// ---------------------------------------------------------------------------
// Value — a known thing
// ---------------------------------------------------------------------------
// A literal, a constant, data that just exists.
// No inputs. One output. Cost = 0.
//
//   3
//   "hello"
//   true

type Value {
  id: NodeId
  data: LiteralValue
  value_type: TypeShape            // what type this value is
}

// ---------------------------------------------------------------------------
// Transform — do something to inputs, get an output
// ---------------------------------------------------------------------------
// The workhorse. Takes inputs, produces output via a known rule.
// Field access, function call, binary op, record construction,
// list construction, string interpolation, type cast — all transforms.
// They differ in WHAT rule applies, not in shape.
//
//   x + y               rule: Add
//   f(x)                rule: Call(f)
//   person.name         rule: FieldAccess("name")
//   Record { a: 1 }     rule: Construct("Record")
//   items[i]            rule: Index
//   x => x + 1          rule: Define (a transform you can pass around)
//
// The rule is DATA, not structure. Adding a new kind of transform
// means adding a rule to the table, not a new behavior.
//
// GENERICS: a generic type (List<T>, Tree<T>, Config<T>) is a
// type-level Define — give it a type parameter, get a concrete type.
// Instantiation (List<Int>) is a type-level Transform — substitute
// the parameter, get the result. Recursive generics (Tree<T> where
// children: List<Tree<T>>) follow the same bounded law as recursive
// functions: self-reference is detected and not expanded. Same
// mechanism at both levels. No special generics system.

type Transform {
  id: NodeId
  rule: TransformRule             // what this transform does
  inputs: List<Port>              // what goes in
  output: Port                    // what comes out
}

type TransformRule =
    // access
    FieldAccess { field: FieldRef }
  | IndexAccess
  | SliceAccess

    // apply
  | Call { target: NodeRef }
  | Method { method: MethodRef }
  | Builtin { builtin: BuiltinKind }

    // arithmetic / logic
  | BinaryOp { op: BinOp, algebra: AlgebraRef? }
  | UnaryOp { op: UnaryOpKind }
  | Cast { target_type: TypeShape }

    // construct
  | Construct { type_ref: TypeRef }       // record, list, string
  | ListBuild
  | StringBuild

    // define (a deferred transform — lambda / function body / callback)
    // A lambda is sugar: x => x + y desugars to a Define where y is
    // an explicit input edge, same as any other dependency in the DAG.
    // No special capture mechanism — the dependency is just visible.
    //
    // CALLBACK RULE: a Define is just code. WHO CALLS IT determines
    // the execution context. When a Define has an edge into a Loop
    // (e.g., passed as the body of map/fold), the lenses read the
    // Loop's bound for the Define's captures:
    //   - Ownership: captured values have fan-out = Loop's bound (N),
    //     not 1. They're used N times, so borrow or clone, not move.
    //   - Termination: self-calls inside the Define are bounded by
    //     the Loop, not the enclosing function's recursion.
    // This isn't a new behavior. It's the lens following the edge
    // from the Define to its consumer and applying the consumer's
    // context. No scheduler, no interrupt model, no async.
  | Define { params: List<ParamPort>, body: NodeRef }

// ---------------------------------------------------------------------------
// Branch — look at something, take a path
// ---------------------------------------------------------------------------
// Inspect a value, choose one of several paths. Both paths are real
// structure. At runtime, one fires.
//
//   if x > 0 then a else b
//   match status { Pending => ..., Done => ... }
//
// `if` is just Branch on Bool with two paths.
// `match` is Branch on any type with N paths.
// Same behavior.
//
// Cost = cost(input) + max(cost per path).
// Ownership: paths are exclusive — value is consumed once, not N times.

type Branch {
  id: NodeId
  input: Port                     // the value we're inspecting
  paths: List<Path>               // possible outcomes
  output: Port                    // result (from whichever path fires)
}

type Path {
  pattern: Pattern                // what activates this path
  bindings: List<Bind>            // values extracted by the pattern
  body: NodeRef                   // what happens on this path
}

// ---------------------------------------------------------------------------
// Loop — repeat something, bounded
// ---------------------------------------------------------------------------
// Apply a body repeatedly, with a known bound on repetitions.
// fold, descend, repeat, for-each — all Loops.
//
//   items |> fold(init: 0, step)     bound = |items|
//   tree |> descend(base, combine)   bound = |tree|
//   repeat(100, init, step)          bound = 100
//   "forever" server                 bound = system limit
//
// Cost = bound × cost(body).
// A "forever" loop has bound = system limit (largest representable value).
// Structurally identical to bound = 5. Just larger.
//
// IMPORTANT: Loop is not necessarily the lowering target. A developer
// writes for/loop, and the compiler recognizes when the body is
// elementwise-independent — then it can RAISE to map/filter (L2),
// which enables parallelism. Lowering goes UP (imperative → parallel),
// not DOWN (functional → sequential). "Parallelism is the default"
// means: if elements don't depend on each other, the compiler sees
// that from the DAG structure and can parallelize.
//
// TERMINATION: Once something IS a Loop with a Bound, termination
// is trivial. The hard part is getting there: classifying recursion
// as Loop, recognizing elementwise independence for Map, proving
// branch exclusivity for ownership. That's real work — but it's
// concentrated in ONE place (the lowering boundary from surface
// syntax to DAG) instead of spread across 33 downstream heuristics.
// v2's 420 CX violations exist because the classification happens
// after the fact. In v3, it happens during DAG construction.

type Loop {
  id: NodeId
  source: Port                    // what provides elements
  init: Port                      // starting accumulator
  body: NodeRef                   // one iteration (a subgraph)
  bound: Bound                    // how many times
  output: Port                    // final result
}

// Bound: how many times does the Loop repeat?
// Just a port carrying an Int. The compiler determines the bound
// during DAG construction (from collection length, tree depth,
// explicit N, or system limit). The Loop doesn't care WHERE the
// bound came from — it just needs the number.
type Bound {
  count: Port                     // carries an Int — the iteration count
}

// ---------------------------------------------------------------------------
// Bind — give something a name
// ---------------------------------------------------------------------------
// Name a value so other nodes can reference it. Not computation —
// just wiring. Cost = 0.
//
//   let x = f(y)        name "x" points to the output of Transform(f, [y])
//   fn add(a, b) { ... } name "add" points to a Define transform
//
// Lambda and named function are the same: both are a Define transform
// pointed to by a Bind. The only difference is whether the Bind has
// a user-visible name.

type Bind {
  id: NodeId
  name: BindingId                 // how other nodes reference this
  value: Port                     // what this name points to
  scope: NodeRef?                 // where this name is visible
}

// =========================================================================
// PORTS — typed connections between behaviors
// =========================================================================
//
// Every input and output is a Port. A Port carries a typed value
// forward in time. No back-edges. No cycles.

type Port {
  id: PortId
  value_type: TypeShape            // what type flows through this port
  produced_by: NodeId?             // which node produced this value (None = parameter/literal)
}

// "Where did this value come from?" is just produced_by on the Port.
// Follow the edge backward to the source node. The relationship
// (sub-value, computed, branched, accumulated) is readable from
// the source node's behavior type. No separate tracking needed.

// =========================================================================
// CONTRACTS = DERIVED FROM PORTS (not a separate structure)
// =========================================================================
//
// Ports ARE the contracts. A behavior's input ports are its imports.
// Its output port is its export. The compiler validates that every
// input port connects to a prior behavior's output port — each
// dependency must be satisfied before the dependent can execute.
// No separate contract type — that would duplicate what ports say.

// =========================================================================
// LENSES — lightweight views over the physics
// =========================================================================
//
// A lens is an observation: you point it at the DAG and you see
// something. Cost, ownership, effects, purity, termination — these
// are all lenses. They don't add information. They read information
// that's already in the structure.
//
// The rule: if a lens needs information that isn't in the DAG, the
// physics is incomplete. Fix the physics, not the lens. This is
// exactly what went wrong in v2 — the complexity lens needed
// "where values come from," but TypeBinding had thrown it away, so v2 built a
// 5,000-line reconstruction engine instead of fixing the physics.
//
// A lens has three parts:
//   - what you're looking at (the DAG, or a subgraph of it)
//   - what you're measuring (cost, fan-out, effect, etc.)
//   - how measurements compose (add for cost, max for branches, etc.)
//
// The compiler ships with built-in lenses. Users can define their own
// OBSERVATIONAL lenses (e.g., security classification that validates
// and rejects). Lenses that influence optimization or emission
// strategy are a harder class of extensibility — not claimed yet.

// ---------------------------------------------------------------------------
// Built-in lenses
// ---------------------------------------------------------------------------

// COST lens — "how expensive is this?"
//   Looks at: each behavior node
//   Measures: operation count / time bound
//   Composes: add for sequence, multiply for loops, max for branches
//
//   Value:     0
//   Transform: rule_cost(rule)
//   Branch:    cost(input) + max(path costs)
//   Loop:      bound × cost(body)
//   Bind:      0

// OWNERSHIP lens — "who uses this value, and how?"
//   Looks at: edges (ports) between nodes
//   Measures: fan-out (how many consumers), consumption kind
//   Composes: count edges from each port
//
//   Fan-out = number of edges leaving a port
//   Last use = edge with latest topological position → can move
//   Exclusive branches = fork consumers don't double-count

// EFFECT lens — "does this touch the outside world?"
//   Looks at: each behavior node
//   Measures: Pure / ServiceCall / Mutation
//   Composes: pick the strongest (effects only escalate, never decrease)
//
//   Value:     Pure
//   Transform: rule_effect(rule)
//   Branch:    join(path effects)
//   Loop:      body effect
//   Bind:      Pure

// PROVENANCE lens — "where did this value come from?"
//   Looks at: each port
//   Measures: Origin / SubValue / Computed / Selected / Accumulated
//   Composes: follows edges backward to source
//
//   Already on every Port (it's part of the physics, not a separate
//   observation). This is the one v2 got wrong — origin tracking
//   was computed during inference then discarded. In v3, every port
//   has produced_by. The lens just reads it.

// TERMINATION lens — "does this finish?"
//   Looks at: loops and recursive calls
//   Measures: does every Loop have a Bound? Are all bounds finite?
//   Composes: structural — the DAG is acyclic, Loops have bounds
//
//   Once the DAG is built, this lens is straightforward: every
//   Loop has a Bound, the DAG has no cycles, done. But the hard
//   part is BUILDING the DAG: classifying recursion as Loop,
//   determining bounds, recognizing independence. That work lives
//   at the lowering boundary (surface syntax → DAG), not in this
//   lens. If the lens itself needs complex analysis (like v2's
//   5,000-line complexity.dag), the physics is wrong.

// ALGEBRA lens — "can this be simplified?"
//   Looks at: adjacent transforms
//   Measures: algebraic properties (involution, inverse, fusion)
//   Composes: checks if adjacent operations simplify
//
//   reverse |> reverse = identity (involution)
//   serialize |> deserialize = identity (inverse pair)
//   map(f) |> map(g) = map(f.g) (functor fusion)
//
//   NOTE: simplification is normalization during DAG CONSTRUCTION,
//   not a post-hoc rewrite pass. When the compiler builds
//   Transform(reverse) → Transform(reverse), it reads the algebra
//   table, sees involution, and produces identity — the redundant
//   nodes never enter the DAG. Same as how the compiler raises
//   for-loops to Map during construction. The DAG is built
//   normalized; no separate optimizer rewrites it afterward.

// SPACE lens — "how much memory does this need?"
//   Looks at: each behavior node
//   Measures: allocation size
//   Composes: add for sequence, max for branches, bound × body for loops
//
//   Analogous to cost lens but tracks allocation instead of time.

// ---------------------------------------------------------------------------
// User-defined lenses (v3.1)
// ---------------------------------------------------------------------------
//
// A user defines a lens the same way:
//   - what to measure (e.g., SecurityLevel = Public | Internal | Secret)
//   - how it composes (e.g., pick the strongest — output ≥ max input)
//   - what to check (e.g., Secret can't flow to a PublicAPI service call)
//
// The compiler points the lens at the DAG and enforces whatever it reveals.
// Same mechanism as cost, ownership, effects for observation/rejection.
// Lenses that influence optimization or lowering need more design.

// =========================================================================
// L2: COMPOSED BEHAVIORS
// =========================================================================
//
// L2 behaviors are compositions of L1. They do not add new primitive
// shapes. They're common patterns that get names for readability.
//
// The test: every L2 can be written as L1 composition. If it can't,
// it belongs in L1.
//
// IMPORTANT: L2 is not just a naming convenience — it's where the
// compiler RAISES imperative code to parallelizable patterns. The
// developer writes a for-loop. The compiler sees that iterations
// are independent (no cross-element dependency in the DAG) and
// raises it to Map — which CAN be parallelized. The lowering goes
// UP (imperative → parallel), not down (functional → sequential).

// Map — elementwise-independent transform. Parallelizable.
//   The developer writes: for item in items { f(item) }
//   The compiler sees: each f(item) is independent (no shared edges)
//   The compiler raises to: Map(items, f) — can parallelize

// Filter — elementwise-independent predicate. Parallelizable.
//   The developer writes: for item in items { if p(item) { keep } }
//   The compiler sees: each p(item) is independent
//   The compiler raises to: Filter(items, p) — can parallelize

// Fold — accumulator-dependent iteration. Sequential.
//   items |> fold(init: z, step: f)
//   Each step depends on the previous accumulator. Cannot parallelize
//   UNLESS the algebra lens sees that f is associative + commutative,
//   in which case → MapReduce (partition, parallel fold, combine).

// Pipeline — sequential composition
//   fetch(id) |> parse |> validate

// Retry — bounded retry with backoff
//   retry(3, action)

// Recursive function — self-call with smaller input
//   fn f(tree) { ... f(child) ... }
//   The child is structurally smaller than tree (the DAG shows this).
//   Desugars to a bounded traversal.

// =========================================================================
// PROPERTIES = LENSES POINTED AT THE PHYSICS
// =========================================================================
//
// Every thesis property is a lens. Not a pass. Not an analysis.
// Just a view over structure that already exists.
//
// Complexity    = cost lens pointed at the DAG
// Ownership     = ownership lens pointed at edges
// Purity        = effect lens → check if result is Pure
// Idempotence   = algebra lens → check if all operations are "do once = do many times"
// Termination   = termination lens → check all Loops have bounds
// Parallelism   = independence from DAG structure (no shared edges)
// Space bounds  = space lens pointed at the DAG
// Security (v3.1) = user-defined lens pointed at the DAG
//
// If any of these needs a 5,000-line reconstruction engine, the physics
// is missing information. Fix the physics.

// =========================================================================
// EMISSION
// =========================================================================
//
// Emission translates behaviors to target syntax using LanguageSpec data.
//
// | Behavior | Rust | Python | Go |
// |----------|------|--------|-----|
// | Value | `const`/`static` | module-level | `var` |
// | Transform | `f(args)` | `f(args)` | `f(args)` |
// | Branch | `match` | `match` | `switch` |
// | Loop | `.iter().fold()` | `functools.reduce` | `for range` |
// | Bind | `let x = ...` | `x = ...` | `x := ...` |
//
// Differences between targets (Rc, &T, async, GC) are LanguageSpec
// data that parameterize the translation. Not separate emitter code.

// =========================================================================
// NON-CONSENSUAL OPTIONALITY
// =========================================================================
//
// The developer doesn't opt into optionality. The system enforces it.
// If an operation can fail (map lookup, optional field, service call),
// the return type IS Optional<T>. The developer MUST Branch before
// accessing the inner value. There is no unwrap. There is no null.
// There is no sentinel value. There is no forgetting to check.
//
// This is the same philosophy as termination: you don't opt into
// termination proofs — you can't write a non-terminating program.
// You don't opt into optionality handling — you can't access an
// optional value without branching.
//
// Why this matters at scale: in large codebases, the #1 source of
// bugs is different developers representing absence differently.
// One uses null, one uses empty string, one uses -1, one forgets
// to check. In v3, absence is structural. Optional<T> is the ONLY
// way to represent "might not be there." The compiler enforces
// handling. The developer doesn't choose how — the system does.
//
// For clones/elision: Branch doesn't clone to unwrap. It destructs
// — the Optional wrapper goes away, the inner value transfers.
// Zero-cost unwrap. The ownership lens sees: the Optional port has
// fan-out = 1 (into the Branch). The inner value in the Some arm
// has fan-out = however many times it's used. Move or reference
// based on that fan-out. No defensive cloning.
//
// For test generation: the compiler knows every Optional boundary.
// It generates tests for both arms:
//   - For every map_get: test with key present AND key absent
//   - For every Optional<T> field: test with Some and None
//   - For every Branch: verify both arms produce correct results
// These tests fall out of the port types. No hand-writing.

// =========================================================================
// E2E SCENARIOS
// =========================================================================
//
// Each scenario walks: what the developer writes → what the DAG
// looks like → what each lens sees → what gets emitted → what
// tests are generated. If the story isn't clear at any stage,
// the design has a gap.

// ---------------------------------------------------------------------------
// Scenario 1: Nested optional map lookup
// ---------------------------------------------------------------------------
//
// FRONTEND (what the developer writes):
//
//   type Address { street: String, city: String, zip: String? }
//   type Person { name: String, address: Address? }
//   let people: Map<String, Person> = { ... }
//   let zip = people |> map_get("alice")?.address?.zip
//
// DAG (what the compiler builds):
//
//   Transform(MapGet, [people, "alice"]) → Port<Optional<Person>>
//   Branch(person_port, [
//     Some { p: Person } →
//       Transform(FieldAccess("address"), [p]) → Port<Optional<Address>>
//       Branch(address_port, [
//         Some { addr: Address } →
//           Transform(FieldAccess("zip"), [addr]) → Port<Optional<String>>
//         None → Value(None) → Port<Optional<String>>
//       ])
//     None → Value(None) → Port<Optional<String>>
//   ])
//
// LENSES:
//   Cost: cost(MapGet) + cost(Branch) + cost(FieldAccess) + cost(Branch) + cost(FieldAccess)
//   Ownership: person_port fan-out=1 (into Branch), p fan-out=1 (into FieldAccess) → all move, zero clones
//   Effect: all Pure
//   Cardinality: read each port's type. Optional<Person> → optional. Address → required. Trivial.
//
// EMISSION (Rust):
//   people.get("alice").and_then(|p| p.address.as_ref()).and_then(|a| a.zip.clone())
//
// EMISSION (Go):
//   if p, ok := people["alice"]; ok { if p.Address != nil { return p.Address.Zip } }
//
// GENERATED TESTS:
//   - map_get with existing key → Some path fires, zip extracted
//   - map_get with missing key → None path fires, result is None
//   - existing key, address is None → second Branch takes None path
//   - existing key, address exists, zip is None → innermost None
//   - existing key, address exists, zip exists → full extraction

// ---------------------------------------------------------------------------
// Scenario 2: Mutual recursion → Loop lowering → TCO
// ---------------------------------------------------------------------------
//
// FRONTEND:
//
//   fn is_even(n: Int) -> Bool {
//     if n == 0 then true
//     else is_odd(n - 1)
//   }
//   fn is_odd(n: Int) -> Bool {
//     if n == 0 then false
//     else is_even(n - 1)
//   }
//
// DAG CONSTRUCTION:
//   The compiler sees mutual recursion: is_even → is_odd → is_even.
//   Both calls use n-1 (arithmetic descent). Both are tail calls.
//
//   Lowering: the mutual recursion forms a cycle in the call graph.
//   The compiler detects the SCC {is_even, is_odd} and lowers to:
//
//   Loop {
//     bound: Explicit(n)           // n decreases by 1 each cycle
//     body: Branch(phase, [        // alternate between even/odd
//       Even → Branch(n == 0, [True → return true, False → continue with (n-1, Odd)])
//       Odd  → Branch(n == 0, [True → return false, False → continue with (n-1, Even)])
//     ])
//   }
//
//   The mutual recursion became a single Loop with a phase tag.
//   No stack frames. No call overhead. TCO is automatic because
//   Loop IS the TCO'd form — there's no separate TCO pass.
//
// LENSES:
//   Cost: O(n) — Loop with bound n, body is O(1) Branch
//   Termination: trivially bounded — Loop has Explicit(n) bound
//   Effect: Pure
//   Ownership: n is a Value (Int), copied. No Rc, no clone.
//
// EMISSION (Rust):
//   let mut n = n;
//   let mut phase = Phase::Even;
//   loop {
//     match phase {
//       Phase::Even => { if n == 0 { return true; } n -= 1; phase = Phase::Odd; }
//       Phase::Odd  => { if n == 0 { return false; } n -= 1; phase = Phase::Even; }
//     }
//   }
//
// EMISSION (Go):
//   Similar loop with phase enum.
//
// GENERATED TESTS:
//   - is_even(0) → true, is_even(1) → false, is_even(4) → true
//   - is_odd(0) → false, is_odd(1) → true, is_odd(3) → true
//   - Boundary: is_even(max_int) terminates (Loop bound guarantees it)

// ---------------------------------------------------------------------------
// Scenario 3: Imperative loop → raised to Map → parallelism
// ---------------------------------------------------------------------------
//
// FRONTEND:
//
//   let results = items |> map(item => process(item))
//
// DAG CONSTRUCTION:
//   Developer writes map (or equivalently a for-loop that builds a list).
//   The compiler builds:
//
//   Loop {
//     source: items
//     bound: CollectionSize(items)
//     body: Transform(Call(process), [element])
//     output: collected results
//   }
//
//   The compiler inspects the DAG: does the body have edges to the
//   accumulator? No — each Transform(process, [element]) only connects
//   to its element, not to any prior result. Elements are independent.
//
//   The compiler RAISES to Map (L2): elementwise-independent, parallelizable.
//
// LENSES:
//   Cost: |items| × cost(process)
//   Effect: effect(process) — if Pure, the whole Map is Pure
//   Ownership: each element consumed once by its Transform. No sharing.
//   Algebra: if process is associative+commutative (from algebra lens)
//     → can partition and MapReduce
//
// EMISSION (Rust, process is Pure):
//   items.par_iter().map(|item| process(item)).collect()    // Rayon parallel
//
// EMISSION (Rust, process has ServiceCall effect):
//   items.iter().map(|item| process(item)).collect()        // sequential (effects may not commute)
//
// EMISSION (Go, process is Pure):
//   // goroutine per element, WaitGroup, collect results
//
// GENERATED TESTS:
//   - Empty items → empty results
//   - Single item → process called once
//   - Multiple items → all processed, order preserved (or unordered if commutative)
//   - Parallel and sequential execution produce same results (wave test)

// ---------------------------------------------------------------------------
// Scenario 4: Clone/elision through a function pipeline
// ---------------------------------------------------------------------------
//
// FRONTEND:
//
//   fn transform_person(p: Person) -> Summary {
//     let name = p.name
//     let upper = name |> to_upper
//     let age_str = p.age |> to_string
//     Summary { display_name: upper, age: age_str }
//   }
//
// DAG:
//   Bind(p, param_port<Person>)
//   Transform(FieldAccess("name"), [p]) → name_port<String>
//   Transform(Call(to_upper), [name_port]) → upper_port<String>
//   Transform(FieldAccess("age"), [p]) → age_port<Int>
//   Transform(Call(to_string), [age_port]) → age_str_port<String>
//   Transform(Construct("Summary"), [upper_port, age_str_port]) → output<Summary>
//
// OWNERSHIP LENS reads fan-out:
//   p: fan-out = 2 (FieldAccess("name") + FieldAccess("age"))
//     → both are read-only (FieldAccess doesn't mutate)
//     → BORROW, not clone
//   name_port: fan-out = 1 (to_upper) → MOVE
//   upper_port: fan-out = 1 (Summary construct) → MOVE
//   age_port: fan-out = 1 (to_string) → MOVE
//   age_str_port: fan-out = 1 (Summary construct) → MOVE
//
//   Result: p is borrowed twice. Everything else moves. ZERO clones.
//
// EMISSION (Rust):
//   fn transform_person(p: &Person) -> Summary {   // borrow, not owned
//     let upper = p.name.to_uppercase();            // borrow field, call moves
//     let age_str = p.age.to_string();              // borrow field, call moves
//     Summary { display_name: upper, age: age_str } // moves into struct
//   }
//
// EMISSION (Python):
//   def transform_person(p):  # reference semantics, no clone issue
//     return Summary(display_name=p.name.upper(), age=str(p.age))
//
// EMISSION (Go):
//   func TransformPerson(p *Person) Summary {  // pointer for large struct
//     return Summary{DisplayName: strings.ToUpper(p.Name), Age: strconv.Itoa(p.Age)}
//   }
//
// The ownership lens produces: p should be borrowed, everything else
// moves. The emitter reads this + LanguageSpec (Rust: &T for borrow,
// Go: *T for pointer, Python: no-op) and renders. No defensive cloning.
// No "emit Rc<T> everywhere then try to optimize."

// ---------------------------------------------------------------------------
// Scenario 5: Recursive generics — Tree<T> and Tree<Tree<Int>>
// ---------------------------------------------------------------------------
//
// Generics are anonymous compositions, like lambdas. A generic type
// is a Define at the type level: give it a type parameter, get a
// concrete type. Recursive generics follow the same bounded-recursion
// law as recursive functions.
//
// FRONTEND:
//
//   type Tree<T> {
//     value: T
//     children: List<Tree<T>>
//   }
//
//   fn sum_tree(tree: Tree<Int>) -> Int {
//     tree.value + tree.children |> fold(init: 0, fn: (acc, child) =>
//       acc + sum_tree(child)
//     )
//   }
//
//   // And the nested case:
//   let nested: Tree<Tree<Int>> = ...
//   let flat: Tree<Int> = flatten(nested)
//
// TYPE-LEVEL DAG (generic instantiation):
//
//   Tree is a type-level Define: (T) → { value: T, children: List<Tree<T>> }
//
//   Tree<Int>:
//     Substitute T → Int:
//     { value: Int, children: List<Tree<Int>> }
//     The Tree<Int> in children is a SELF-REFERENCE — not expanded further.
//     Substitution terminates in one step. Done.
//
//   Tree<Tree<Int>>:
//     Substitute T → Tree<Int>:
//     { value: Tree<Int>, children: List<Tree<Tree<Int>>> }
//     The Tree<Tree<Int>> in children is a self-reference — not expanded.
//     One substitution step. Done.
//
//   SAME LAW AS VALUE-LEVEL RECURSION:
//     Value: fn f(tree) { f(child) } → child is smaller → bounded
//     Type:  Tree<T> { children: List<Tree<T>> } → self-ref → not expanded
//     Both detect self-reference. Both refuse to expand infinitely.
//     Both are bounded by structure.
//
// VALUE-LEVEL DAG (sum_tree):
//
//   sum_tree is recursive: calls itself on child (sub-value of tree).
//   Lowers to Loop:
//
//   Loop {
//     source: tree.children              // iterate children
//     bound: TreeSize(tree)              // bounded by tree structure
//     init: tree.value                   // start with root's value
//     body: Transform(BinaryOp(Add), [acc, recursive_result])
//   }
//
//   Where recursive_result is itself a Loop over child.children.
//   The recursion unwinds into nested Loops, each bounded by
//   the subtree size. Total work = O(|tree|).
//
// LENSES:
//   Cost: O(|tree|) — one visit per node, O(1) work each
//   Termination: trivially bounded — Loop has TreeSize bound,
//     tree gets structurally smaller at each level
//   Ownership:
//     tree: fan-out = 2 (value access + children access) → borrow
//     child: fan-out = 1 (into recursive call) → move
//     acc: fan-out = 1 (into Add) → move (Int, copied)
//     Zero Rc clones. Tree nodes are borrowed during traversal.
//   Effect: Pure
//
// EMISSION (Rust):
//
//   // Tree<Int> becomes:
//   struct Tree<T> {
//     value: T,
//     children: Vec<Tree<T>>,    // Rc<Vec<...>> based on ownership lens
//   }
//
//   fn sum_tree(tree: &Tree<i64>) -> i64 {     // borrow — fan-out > 1
//     tree.value + tree.children.iter().fold(0, |acc, child| {
//       acc + sum_tree(child)                    // borrow child too
//     })
//   }
//
//   // Tree<Tree<Int>> becomes:
//   // Tree<Tree<i64>> — same struct, nested. No special handling.
//
// EMISSION (Go):
//   type Tree[T any] struct {
//     Value    T
//     Children []Tree[T]
//   }
//   func SumTree(tree *Tree[int64]) int64 { ... }
//
// GENERATED TESTS:
//   - Empty tree (no children): sum = value
//   - Single child: sum = parent.value + child.value
//   - Deep tree: sum = all values (verifies recursion terminates)
//   - Tree<Tree<Int>>: nested instantiation produces valid struct
//   - Type substitution: Tree<String> produces { value: String, children: List<Tree<String>> }

// ---------------------------------------------------------------------------
// Scenario 6: Generics + optionality + ownership combined
// ---------------------------------------------------------------------------
//
// The hardest case: generic container with optional fields, accessed
// through a map lookup, with ownership tracking at every level.
//
// FRONTEND:
//
//   type Config<T> {
//     default_value: T
//     overrides: Map<String, T>
//   }
//
//   fn resolve<T>(config: Config<T>, key: String) -> T {
//     match config.overrides |> map_get(key) {
//       Some { value: v } => v
//       None => config.default_value
//     }
//   }
//
//   let config: Config<Int> = Config { default_value: 0, overrides: { "timeout": 30 } }
//   let timeout = resolve(config, "timeout")
//
// TYPE-LEVEL:
//   Config<Int>: substitute T → Int
//   { default_value: Int, overrides: Map<String, Int> }
//   resolve<Int>: substitute T → Int in signature
//   (Config<Int>, String) → Int
//   One step. Done.
//
// VALUE-LEVEL DAG:
//
//   Bind(config, param_port<Config<Int>>)
//   Transform(FieldAccess("overrides"), [config]) → overrides_port<Map<String,Int>>
//   Transform(Method(MapGet), [overrides_port, key]) → lookup_port<Optional<Int>>
//   Branch(lookup_port, [
//     Some { v: Int } => v                      // use the override
//     None =>
//       Transform(FieldAccess("default_value"), [config]) → default_port<Int>
//   ])
//   output: Port<Int>                           // result is non-optional!
//
// LENSES:
//   Cost: O(log n) for map_get + O(1) for field access
//   Ownership:
//     config: fan-out = 2 (overrides access + default_value access)
//       BUT the accesses are in EXCLUSIVE branches of the Branch.
//       Some arm uses overrides. None arm uses default_value.
//       The compiler sees: config is consumed in exclusive paths.
//       → borrow config, not clone. Even with fan-out = 2.
//     v: fan-out = 1 → move
//     default_value: fan-out = 1 → move
//     ZERO clones.
//   Effect: Pure
//   Cardinality: output is Int (Required), not Optional<Int>.
//     The Branch consumed the optionality — both arms return Int.
//
// EMISSION (Rust):
//   fn resolve(config: &Config<i64>, key: &str) -> i64 {
//     match config.overrides.get(key) {
//       Some(v) => *v,
//       None => config.default_value,
//     }
//   }
//
// GENERATED TESTS:
//   - Key exists: returns override value
//   - Key missing: returns default_value
//   - Generic instantiation: Config<String>, Config<List<Int>> all work

// =========================================================================
// DIAGNOSTICS = CORRECTIONS, NOT ERROR MESSAGES
// =========================================================================
//
// In a closed system with complete structural knowledge, the compiler
// knows the fix — not just the error. A diagnostic is not "error on
// line 42." It is: what's wrong, why it's wrong, and the corrected code.
//
// This is emission. The compiler emits corrected .dag code to the
// terminal the same way it emits Rust code to a file. Same mechanism,
// different LanguageSpec. A diagnostic IS an artifact.
//
// Levels of correction (progressive, not all-or-nothing):
//
// Level 0: SHOW the error.
//   "Type mismatch: expected Int, got String"
//   This is what most compilers do. Necessary but insufficient.
//
// Level 1: SHOW the fix.
//   "Type mismatch: expected Int, got String.
//    Fix: change return type to String, or wrap in to_int()"
//   The compiler enumerates all structurally valid repairs.
//   In a closed system, this set is finite.
//
// Level 2: EMIT the fix.
//   The compiler outputs the corrected .dag source code.
//   The developer reviews and applies it.
//   This is how NonExhaustiveMatch already works in v2 —
//   the compiler shows the missing arms with placeholder bodies.
//
// Level 3: APPLY the fix (non-consensual correction).
//   ONLY when exactly one repair is structurally valid AND
//   semantic equivalence is provable. Otherwise this becomes
//   fabrication disguised as helpfulness. Start with Level 1-2;
//   Level 3 earns trust incrementally.
//
// This applies to ALL lenses, not just types:
//
// | Lens | What it catches | What it shows/fixes |
// |------|----------------|---------------------|
// | Types | Type mismatch | The valid type options or coercions |
// | Cardinality | Missing None handling | The Branch with both arms |
// | Termination | Missing bound | "Rewrite as repeat(N, ...)" |
// | Cost | O(n²) where O(n) exists | The O(n) version of the code |
// | Ownership | Unnecessary clone | Which clones to remove |
// | Exhaustiveness | Missing Branch arms | The missing patterns |
// | Effect | Non-idempotent in retry | Which operation breaks it |
//
// The cost lens is particularly interesting: if the compiler sees
// Loop(items, Loop(items, ...)) = O(n²) and there exists a single
// Loop that produces the same result = O(n), it can:
// - Level 1: "This is O(n²). An O(n) equivalent exists."
// - Level 2: Show the O(n) code.
// - Level 3: Just use the O(n) code. The developer wrote intent;
//   the compiler finds the optimal realization. Same as raising
//   a for-loop to Map for parallelism — the developer doesn't
//   need to know.
//
// The principle: the developer declares WHAT. The compiler decides
// HOW — including fixing the developer's non-optimal HOW when a
// better structural equivalent exists.

// =========================================================================
// HARD RULES
// =========================================================================
//
// 1. Every expression desugars to L1 behaviors. No exceptions.
// 2. L2 behaviors compose L1. They never add new shapes.
// 3. Properties are lens readings. Never separate passes.
//    If a lens needs info that isn't in the DAG, fix the physics.
// 4. New transforms add a rule to the table. Not a new behavior.
// 5. Lambda = Bind + Define transform. Not a special behavior.
// 6. if = Branch on Bool. match = Branch on any. Same behavior.
// 7. Every input port must connect to a prior output port.
// 8. Emitter reads behaviors + LanguageSpec. Never decides semantics.

// =========================================================================
// ACCEPTANCE TESTS
// =========================================================================
//
// T1: New transform test
//     Adding a new builtin requires one TransformRule value.
//     Zero changes to cost, ownership, emission, or any consumer.
//
// T2: L2 decomposition test
//     Every L2 behavior can be rewritten as explicit L1 composition
//     with identical semantics.
//
// T3: Lens sufficiency test
//     Every property the thesis claims (cost, ownership, effects,
//     termination, purity, idempotence) is readable by pointing a
//     lens at the DAG. No reconstruction. No heuristics. If a lens
//     can't read it, the physics is missing structure.
//
// T4: Lambda = function test
//     Replace any lambda with an equivalent named function + Bind.
//     All tests pass. The compiler doesn't distinguish them.
//
// T5: Wave parallelism test
//     Nodes in the same wave produce the same result whether
//     executed sequentially or concurrently.
//
// T6: Name scramble test
//     Rename all user identifiers. Semantic and emission decisions
//     don't change. Names are labels, not authority.
