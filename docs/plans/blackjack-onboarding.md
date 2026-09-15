# Blackjack — a first .dag project, read through Python

**Status:** hand-off brief for a new contributor. Nothing here is implemented; the five module files under `dag/examples/blackjack/` exist as markers with one anchor declaration each, and everything else is yours to write. **Consumer of this document:** the person building the example. **Authority for every judgement about a change:** `DESIGN.md` — this page tells you which of its sections you will meet first (§10 below), so you do not have to read all of it on day one.

This project is framed as a guided tour of the recurring anatomy of a `.dag` program rather than as a card game. Blackjack is the vehicle because its rules are small, its state machine is real, and the interesting question — *is strategy A better than strategy B?* — needs simulation, which forces the one design decision this language is built around: **keep the engine pure and push randomness to the edge.**

Every feature follows the same four-view loop. Get used to asking the four questions in this order, because the repository's tooling is arranged around them:

1. **Model the facts.** Which concepts exist, and which module owns each one? (types first, functions second)
2. **Run a pure example.** Feed a hand-built input to a function and look at the value that comes back.
3. **Inspect the test evidence.** A test is a `test fn ... -> Bool` whose input, expected value and actual value you can read — not a green checkmark.
4. **Inspect the emitted Rust.** The `.dag` source is the program; Rust is one projection of it. Read the projection, never repair it by hand.

Randomness and I/O stay explicit boundaries throughout: most of what you write never touches either, and the one place that does is a single narrow module.

## 1. The one design decision

> **The blackjack engine is deterministic. Randomness enters at the edge.**

The round engine receives an explicitly ordered `Shoe` (a list of cards, top card first). It never shuffles, never asks for entropy, never reads a clock. Randomness is decomposed into three separate facts, each owned by a different layer:

1. **Entropy acquisition** — an external effect. The repository already models it: `extdeps.entropy` declares `Urandom.ReadBytes(count: Int)`, returning `{ octets_b64: String }`. You reuse that; you do not invent a blackjack randomness source.
2. **Seed selection** — a value. A `ShuffleSeed` is an `Int` that can be recorded in a receipt and replayed.
3. **Shuffling** — a pure function `shuffle(deck: List<Card>, seed: ShuffleSeed) -> Shoe`. Same deck, same seed, same shoe, every time, on every machine.

What that buys you, concretely:

- Domain logic needs no I/O, so it runs in the interpreter, in a test, and in emitted Rust without any glue.
- Tests supply a known shoe or a known seed. "Mocking randomness" is not a framework; it is constructing the input instead of computing it.
- A failed simulation round is reproduced by replaying its seed.
- The same engine serves unit tests, interactive play, and Monte Carlo analysis. There is one blackjack, not three.

This is the repository's standing distinction between **interface, realization and policy** (DESIGN §3): what a dependency *means* (`extdeps.entropy` owns the shape), how it is *reached* (a shell transport today — not your concern), and how the application *uses* it (your `SimulationPlan` decides the seed) are three facts in three places.

```
External entropy            extdeps.entropy (exists)
      |
      v
Recorded seed               ShuffleSeed { value: Int }
      |
      v
Deterministic shuffle       shuffle(deck, seed) -> Shoe          examples.blackjack.shuffle
      |
      v
Ordered Shoe                Shoe { cards: List<Card> }
      |
      v
Pure round engine           deal / apply_player_action /       examples.blackjack.round
      |                     play_dealer_turn / settle
      v
SimulatedRound            one record per round
      |
      v
Aggregation                 summarize(observations)             examples.blackjack.simulation
      |
      v
SimulationSummary           counts that must reconcile
```

## 2. Rosetta stone — `.dag` read through Python

The compiler emits Python (`--target python`) as well as Rust, and the Python it emits is the most direct statement of what each `.dag` construct *is*: a product type becomes a `@dataclass`, a sum type becomes a `Union` of dataclasses, a function becomes a `def`. Use the table as the bridge from what you already know. Where Java, JavaScript, C or SQL has a closer analogue it is named in the third column.

| `.dag` | Python | also like |
| --- | --- | --- |
| `module examples.blackjack.cards` | one file = one module; the header names its dotted path | Java `package`; JS ES module |
| `import examples.blackjack.cards { Card, Rank }` | `from examples.blackjack.cards import Card, Rank` — every name you use from elsewhere is imported explicitly | Java `import`; JS named import |
| `type Card { rank: Rank, suit: Suit }` | `@dataclass class Card: rank: Rank; suit: Suit` — a record; immutable | Java record; C struct; SQL row |
| `type Suit = Clubs \| Diamonds \| Hearts \| Spades` | `Enum` — a closed set of named alternatives | Java enum; SQL CHECK constraint |
| `type RoundState = PlayerTurn { … } \| DealerTurn { … } \| RoundComplete { … }` | `Union[PlayerTurn, DealerTurn, RoundComplete]` of dataclasses — alternatives that carry data (a *sum type*) | Java sealed interface; TypeScript discriminated union; C tagged union |
| `fn hand_value(hand: Hand) -> HandValue { … }` | `def hand_value(hand: Hand) -> HandValue: return …` — the body is one expression; its value is the result | Java static method; C function |
| `hand_value(hand: h)` | `hand_value(hand=h)` — arguments to functions you write are named at the call; builtins such as `filter` and `fold` take their list positionally | Python keyword args |
| `let total = hard_total(ranks: rs)` | `total = hard_total(ranks=rs)` — bound once, never reassigned | Java `final`; JS `const` |
| `if total > 21 { a } else { b }` | `a if total > 21 else b` — an expression that has a value, so both branches are required | C ternary |
| `match state { PlayerTurn { hand } => … DealerTurn { … } => … RoundComplete { … } => … }` | `match state: case PlayerTurn(hand=hand): …` — and **every alternative must have an arm**; a missing one is refused by the compiler (the check runs on CI's witness floor), not a runtime surprise | Java switch over sealed type; SQL CASE |
| `cards \|> filter(c => is_ace(c: c)) \|> map(c => c.rank)` | `[c.rank for c in cards if is_ace(c)]` — a pipeline; `x => …` is a lambda | JS `.filter().map()`; SQL WHERE / SELECT |
| `fold(cards, init: 0, f: (acc, c) => acc + pips(c: c))` | `total = 0; for c in cards: total += pips(c)` — **there is no `for`**; the accumulator loop is spelled as a fold | `functools.reduce`; JS `.reduce()`; SQL SUM |
| `fn soften(total: Int, aces: Int) -> Int { if … { soften(total: total - 10, aces: aces - 1) } else { total } }` | a recursive `def` — **there is no `while`**; a loop whose trip count you cannot write as a fold is a recursion, and the compiler emits it as a `loop` (you will see `__tco_0` in the Rust) | tail recursion |
| `length(cards)` · `first(cards)` · `all(xs, x => …)` · `any(…)` · `contains(…)` · `take(…)` · `skip(…)` · `reverse(…)` · `enumerate(…)` · `concat(a, b)` | `len` · `xs[0]` · `all` · `any` · `in` · slicing · `reversed` · `enumerate` · `+` | builtins — the same names as Python's where Python has one |
| `Card?` and `match x { Present { value } => … Absent => … }` | `Optional[Card]`, and the caller **must** handle `None` — there is no implicit unwrap | Java `Optional`; TS `T \| undefined` |
| `data standard_deck: List<Card> = …` | a module-level constant | Java `static final`; SQL a reference table |
| `type RoundStep = RoundStepped { state: RoundState } \| RoundRefused { cause: RoundRefusal }` | a return type that carries the failure as data. **There are no exceptions**: a function that can refuse says so in its type, and the caller matches on it | Rust `Result`; Go `(value, err)` |
| `test fn ace_ace_nine_is_twenty_one() -> Bool { … }` | `def test_ace_ace_nine_is_twenty_one(): assert …` — a test is a function returning `Bool`; it is discovered by its `test` keyword and its location under `dag/test/claim/` | pytest / JUnit |
| `service Urandom { operation ReadBytes { input { count: Int } output { octets_b64: String } … } }` and `Urandom.ReadBytes(count: 16).octets_b64` | an interface to the outside world, declared once with its input and output shape; calling it looks like a function call. The transport (shell, HTTP, SDK) is bound to the shape, not part of it | an abstract class + one adapter; a DB driver behind a repository |
| `// prose above a module-scope declaration` | `#` comments — but **only** as a standalone block directly above a `type`, `fn` or `data` at module scope. A comment inside a function body, or trailing a line, is a parse error (DESIGN §4c) | Javadoc placement |

Three habits to unlearn on day one. **No mutation:** every value is built once; "update the hand" means "construct a new `Hand` with one more card". **No loops:** `fold` for accumulate-over-a-list, recursion for everything else — both compile to loops. **No `null`, no exceptions, no default cases:** an absent value is `Absent`, a failure is a variant of the return type, and a `match` must name every alternative. The compiler refuses the program otherwise, and that refusal is the feature — it is what makes "the engine cannot hit after the round is complete" a fact about the *types* rather than a fact about your test coverage.

## 3. Scope

Deliberately small so that a coherent vertical slice — model, engine, boundary, analysis, emission — finishes inside the timebox. A broad but incomplete blackjack teaches less than a narrow complete one.

### In

- one player, one dealer; one standard 52-card deck, freshly shuffled for every round
- player actions: hit, stand
- ace handling (soft / hard totals), busts, natural blackjack
- dealer play: stands on soft 17 by default; `BlackjackRules { dealer_hits_soft_17: Bool }` is the one modeled variant
- outcomes: player wins, dealer wins, push
- two fixed strategies, compared by simulation

### Out

- splitting, doubling down, surrender, insurance, betting systems, multiple players, a user interface, performance work
- repairing the Python target — see §8; it is a reading aid here, not a deliverable

## 4. The model, module by module

Five modules under `dag/examples/blackjack/`, one concept family each. Each file already exists with **one anchor declaration** so that you start from something that parses; the rest of each module is sketched below as a skeleton. The skeletons are shapes to inhabit, not text to paste — where a body is left as `…` it is yours, and where a name is given it is because the later milestones and the tests refer to it.

**Naming, and why it is not cosmetic here.** Every module in `dag/` shares one name census, and a bare type or variant name that collides with one declared elsewhere is resolved by precedence rather than refused (a known deficit, `gunbc.recurring_failure_mode`). Before you name a variant, grep the corpus for it. `Hit` (`std.cache_interface`), `Push` (`std.stack`), `Complete` (`std.shell_stream_capture`) and `RoundObservation` (`gunbc.harness.harness_throughput`) are all taken, which is why the shapes below say `PlayerHits`, `Pushed`, `RoundComplete` and `SimulatedRound`. `Suit`, `Rank`, `Card`, `Hand`, `Shoe` were free at the time of writing.

### 4.1 `examples.blackjack.cards` — the facts

Anchor in the file: `Suit`. You add `Rank`, `Card`, the pip value of a rank, and the standard deck as a `data` constant (a list comprehension over ranks × suits is a `flat_map` over ranks of a `map` over suits).

```
module examples.blackjack.cards

type Suit = Clubs | Diamonds | Hearts | Spades

type Rank = Ace | Two | Three | Four | Five | Six | Seven | Eight | Nine | Ten | Jack | Queen | King

type Card { rank: Rank, suit: Suit }

// Ace counts 11 here; hand evaluation decides when it must count 1.
fn rank_pips(r: Rank) -> Int { … }

fn is_ace(c: Card) -> Bool { … }

// 52 cards, in a fixed canonical order.
data standard_deck: List<Card> = …
```

Python reading: two enums, one dataclass, two functions, one module constant.

### 4.2 `examples.blackjack.hand` — evaluation

Anchor in the file: `HandValue`. This is the first real function you write and the place the four-view loop is first run end to end. A raw total is one fact; *soft*, *bust* and *blackjack* are derived from it and from the hand's shape.

```
module examples.blackjack.hand

import examples.blackjack.cards { Card, Rank, Ace, rank_pips, is_ace }

type Hand { cards: List<Card> }

type HandValue { total: Int, soft: Bool }

// Sum of pips with every ace at 11 — a fold, the for-loop of this language.
fn hard_total(hand: Hand) -> Int {
  fold(hand.cards, init: 0, f: (acc, c) => acc + rank_pips(r: c.rank))
}

// Demote aces from 11 to 1 while the total busts and an ace is still soft.
// Recursion, not a while loop; the compiler emits a loop.
fn hand_value(hand: Hand) -> HandValue { … }

fn is_bust(v: HandValue) -> Bool { … }

// Exactly two cards totalling 21.
fn is_blackjack(hand: Hand) -> Bool { … }

// No mutation: a new Hand with one more card.
fn add_card(hand: Hand, card: Card) -> Hand { … }
```

Representative cases the tests must cover — write these before the body, they are the specification:

```
[Ace, Nine]        -> total 20, soft
[Ace, Ace, Nine]   -> total 21, soft
[Ace, King]        -> total 21, blackjack
[King, Queen]      -> total 20, hard
[King, Nine, Five] -> bust
[Ace, Ace, Ten]    -> total 12, soft=false (both aces demoted)
```

### 4.3 `examples.blackjack.round` — the state machine

Anchor in the file: `PlayerAction`. A round is explicit state; an action is a transformation from one state to the next; anything that is not a lawful transition is a **typed refusal**, never a default. This is where you stop seeing a program as a mutable loop with implicit conditions and start seeing it as a graph of lawful transitions.

```
module examples.blackjack.round

import examples.blackjack.cards { Card }
import examples.blackjack.hand { Hand, HandValue, hand_value, is_bust, is_blackjack, add_card }

type PlayerAction = PlayerHits | PlayerStands

type BlackjackRules { dealer_hits_soft_17: Bool }

// Ordered; the head of cards is the next card dealt.
type Shoe { cards: List<Card> }

type RoundState
  = PlayerTurn    { player: Hand, dealer: Hand, shoe: Shoe }
  | DealerTurn    { player: Hand, dealer: Hand, shoe: Shoe }
  | RoundComplete { player: Hand, dealer: Hand, outcome: RoundOutcome }

type RoundOutcome = PlayerWon | DealerWon | Pushed

type RoundRefusal
  = ShoeExhausted
  | ActionAfterRoundComplete
  | DealerActedOutOfTurn
  | PlayerActedOutOfTurn

type RoundStep = RoundStepped { state: RoundState } | RoundRefused { cause: RoundRefusal }

// Two cards each. A natural blackjack on either side ends the round here.
fn deal(shoe: Shoe) -> RoundStep { … }

// Only lawful from PlayerTurn. A bust ends the round; a stand hands over to the dealer.
fn apply_player_action(state: RoundState, action: PlayerAction) -> RoundStep { … }

// Only lawful from DealerTurn. Draws until the rules say stand, then settles.
fn play_dealer_turn(state: RoundState, rules: BlackjackRules) -> RoundStep { … }

// Pure comparison of two final hands.
fn settle(player: HandValue, dealer: HandValue) -> RoundOutcome { … }
```

The refusals you must be able to produce, each with a test that goes red if it stops firing: hitting after `RoundComplete`; drawing from an empty shoe; asking the dealer to act during `PlayerTurn`; applying a player action during `DealerTurn`. Note that the Python-shaped reflex — `raise InvalidAction` — is unavailable, and that is the point: `RoundRefused { cause: ActionAfterRoundComplete }` is a value the caller must match on, so the invalid transition is unrepresentable as a *successful* result.

### 4.4 `examples.blackjack.shuffle` — the boundary

Anchor in the file: `ShuffleSeed`. Two pure functions and nothing else: a seed-to-permutation and a bytes-to-seed. The **only** effectful call in the whole project lives in one test (§7), not here.

```
module examples.blackjack.shuffle

import examples.blackjack.cards { Card }
import examples.blackjack.round { Shoe }

type ShuffleSeed { value: Int }

// Deterministic: same deck + same seed = same shoe. A linear congruential step
// threaded through a fold over positions is enough; this is a teaching fixture,
// not a claim about randomness quality.
fn shuffle(deck: List<Card>, seed: ShuffleSeed) -> Shoe { … }

// The step function of the generator, exposed so a test can pin its sequence.
fn next_seed(seed: ShuffleSeed) -> ShuffleSeed { … }

// The bridge from the entropy boundary: fold some octets into an Int.
fn seed_from_octets(octets: List<Int>) -> ShuffleSeed { … }
```

### 4.5 `examples.blackjack.simulation` — analysis

Anchor in the file: `Strategy`. Reuses the engine; reimplements nothing. Each round emits one observation; a summary is a fold over observations; and the reconciliation invariant is a **test**, not an assumption of the report.

```
module examples.blackjack.simulation

import examples.blackjack.cards { Card, standard_deck }
import examples.blackjack.hand { Hand }
import examples.blackjack.round { BlackjackRules, RoundOutcome, RoundState, PlayerAction, deal, apply_player_action, play_dealer_turn }
import examples.blackjack.shuffle { ShuffleSeed, shuffle, next_seed }

// Two fixtures, not blackjack advice: hit below the threshold, otherwise stand.
type Strategy = HitBelow { threshold: Int }

type SimulationPlan { rounds: Int, seed: ShuffleSeed, rules: BlackjackRules, strategy: Strategy }

type SimulatedRound {
  round_index: Int,
  seed: ShuffleSeed,
  player_actions: List<PlayerAction>,
  player_total: Int,
  dealer_total: Int,
  outcome: RoundOutcome,
}

type SimulationSummary {
  rounds_requested: Int,
  rounds_completed: Int,
  player_wins: Int,
  dealer_wins: Int,
  pushes: Int,
  player_blackjacks: Int,
  player_busts: Int,
}

fn choose(strategy: Strategy, player: Hand) -> PlayerAction { … }

// One full round from a seed: shuffle, deal, drive the player by strategy, dealer plays.
fn play_round(index: Int, seed: ShuffleSeed, rules: BlackjackRules, strategy: Strategy) -> SimulatedRound { … }

// rounds, each with next_seed(previous) — so round k of plan P is replayable alone.
fn simulate(plan: SimulationPlan) -> List<SimulatedRound> { … }

fn summarize(plan: SimulationPlan, observations: List<SimulatedRound>) -> SimulationSummary { … }

// player_wins + dealer_wins + pushes == rounds_completed — a first-class test.
fn summary_reconciles(s: SimulationSummary) -> Bool { … }
```

`win_rate` is deliberately absent from the summary: a ratio is derived from two counts, so storing it beside them is a second copy of one fact (DESIGN §2). Compute it where it is displayed.

## 5. Tests, evidence, and what "mocking" means here

Tests live under `dag/test/claim/` — that directory is what CI's witness floor discovers. Put yours at `dag/test/claim/examples/blackjack_hand_witness_test.dag`, `…_round_…`, `…_shuffle_…`, `…_simulation_…`, one file per module under test. A test file is an ordinary module whose functions are marked `test fn` and return `Bool`, plus one declaration every claim file carries:

```
module test.claim.examples.blackjack_hand_witness

import v2.std.live_tree { LiveTreeDisposition, SubstrateInputsOnly }
import examples.blackjack.cards { Card, Ace, Nine, Clubs, Spades }
import examples.blackjack.hand { Hand, hand_value }

// Declares that this file reads nothing from the host: no filesystem, no corpus scan.
data live_tree_disposition: LiveTreeDisposition = SubstrateInputsOnly

fn ace_nine() -> Hand {
  Hand { cards: [Card { rank: Ace, suit: Clubs }, Card { rank: Nine, suit: Spades }] }
}

test fn ace_nine_is_soft_twenty() -> Bool {
  let v = hand_value(hand: ace_nine())
  v.total == 20 && v.soft
}
```

Run one file's tests, or one function, with the claim runner (built in §8):

```
./target/release/claim_batch --source-root dag --source-root src/v2 \
  --entry dag/test/claim/examples/blackjack_hand_witness_test.dag \
  --functions ace_nine_is_soft_twenty
```

Four levels, and you will use all four. The question to ask before writing any test is not *how do I mock this?* but **what value crosses the boundary I am testing?**

1. **Pure unit tests** supply cards, hands and values directly. Most of your tests.
2. **State-transition tests** supply a complete `RoundState` and an action, and assert the exact next state or the exact refusal. One test per refusal arm.
3. **Boundary tests** supply the value an effectful producer *would* return — a `List<Int>` of octets handed to `seed_from_octets`, a `ShuffleSeed` handed to `shuffle` — without calling the producer.
4. **One integration test** calls the real producer, `Urandom.ReadBytes`, and establishes only that its output inhabits the shape the boundary tests assumed (the right number of octets, decodable). It does not assert that a random shoe has any particular order.

Level 3 without level 4 is the trap DESIGN §3 names: a suite that is fast, green, and proves no program, because every boundary was supplied and none was ever executed. Level 4 is what turns your supplied inputs from hypotheses into readings. Keep it to one test, keep it narrow, and know that it is *wet* — it shells out — so it runs with `--wet` locally and is the one that can fail for reasons that are not yours.

**Make the evidence visible.** A `Bool` that comes back `false` tells you nothing about *why*. The convention here: name the test after its case (`ace_ace_nine_is_twenty_one`), build the input in a small named function so the test body is the assertion alone, and for a red test read the value by running a wrapper through `gunbc run` (the interpreter refuses to print a bare value; wrap it: `fn show() -> ProcessExit { … ExitSuccess / ExitFailure … }` from `std.process`). The point is to keep the input, the expected value and the actual value in front of you, so that the tooling never collapses the learning into an unexplained green.

Every test you write must be one you can make go **red** by breaking the code it covers. Before you call a milestone done, break each function on purpose once — flip a comparison, drop the ace demotion — and confirm the right test fails. A test that stays green through the mutation is a decoration.

## 6. The randomness boundary

Only after deterministic rounds run from hand-ordered shoes do you connect entropy — and even then, the connection is one function call in one test. `dag/test/claim/random_bytes_csprng_witness_test.dag` is the in-tree exemplar: it calls `Urandom.ReadBytes(count: 16).octets_b64`, base64-decodes it to `List<UInt8>?`, and asserts the count. Your integration test does the same, then feeds the octets through `seed_from_octets` and `shuffle`, and asserts that the result is a 52-card shoe containing each card exactly once. That last assertion is a property of `shuffle`, not of the entropy; it is here because it is the one place the whole route executes.

The seed is the replay handle. Every `SimulatedRound` carries the seed its shoe was built from, so a surprising row in a 10,000-round simulation is reproduced by one call to `play_round` with that seed — no reruns, no logging, no luck.

## 7. Analysis

`simulate` is a recursion over the round index threading the seed forward; `summarize` is a fold over observations; `summary_reconciles` is a test. The comparison you are after is two plans differing only in `strategy`, run from the same starting seed, so the two strategies see the *same* shoes — a paired comparison, which is the honest one. `HitBelow { threshold: 17 }` against `HitBelow { threshold: 19 }` is enough to see a difference; neither is a claim about optimal play.

What the summary may and may not say (DESIGN §4d): the counts are deduced; any sentence of the form *strategy A is better* is an inference from a sample and belongs beside its sample size and seed, never in a bare field of the summary type.

## 8. Emission — reading the Rust

`dag/examples/weather/weather.dag` is the reference: domain types, a sum type, `match`, pure functions, list pipelines, and it compiles to a Rust crate that passes `cargo check` with no handwritten glue. Read it first; your example follows the same route. These commands were run against this repository on 2026-09-15 and are the ones to start from — the compiler moves quickly, so record the exact commands and commit you used in your PR rather than trusting this page forever:

```
# once: the compiler and the claim runner, built into the repository's own target/ (docs/onboarding.md step 3 has this host's caveats)
cargo build --release -p v1-compiler --bin gunbc --bin claim_batch

# emit the example's closure (only the modules it imports) as a Rust crate
OUT=/tmp/blackjack-rs
./target/release/gunbc compile --source-root dag \
  --entry dag/examples/blackjack/simulation.dag \
  --output-dir "$OUT" --target rust

# the projection must type-check without edits
cargo check --manifest-path "$OUT/Cargo.toml"
```

What to look for in `$OUT/src/examples_blackjack_*.rs`: a `type` became a `pub struct` or `pub enum`; a `fn` became a `pub fn` over `Rc<…>` values; your `fold` became a `for` loop building a result; your recursive `hand_value` became a `loop` with `__tco_` temporaries — the compiler proved the recursion was a loop and emitted one. That is DESIGN §4 in one file: recursion is sugar over `Loop`.

**Never repair the emitted Rust.** If `cargo check` fails, the defect is in the `.dag` or in the compiler, and either way the fix lands upstream of the projection. Ask when you hit one; a compiler bug found by a new example is a real contribution.

`--target python` also runs, and its type projection — `@dataclass` per record, `Union` per sum type — is the best illustration of the Rosetta table in §2. Its function bodies are **not** currently valid Python (a `let` chain is emitted as `return name = …`), so read the Python for the shapes and check the Rust for the behavior. Do not repair the Python target in this project; it is out of scope and named here so you do not lose an afternoon to it.

## 9. Milestones

Five, each ending in a small PR so review can focus on one conceptual layer. Each milestone's *done* is stated as something you can run.

1. **Tour and skeleton.** Build the two binaries; compile the weather example to Rust and `cargo check` it; read `weather.dag` against the Rosetta table. Done when you can name, for the weather example, which file is domain and which files are generated — and you have noticed that it has no `.dag` test file of its own (its checks live in the Rust seed), so yours will be the first example carrying claims.
2. **Cards and hand evaluation.** `cards.dag`, `hand.dag`, and `blackjack_hand_witness_test.dag` covering the six cases in §4.2. Done when every case is green, and each goes red under one deliberate mutation.
3. **Deterministic round engine.** `round.dag` and its tests: a full round replayed from a hand-ordered `Shoe`, one test per refusal arm, the soft-17 rule variant. Done when a complete round is a pure function of its shoe and the four refusals each have a red-capable test.
4. **Randomness boundary.** `shuffle.dag`, its pure tests (seed → shoe reproducible; a shuffle is a permutation), and the one wet integration test through `Urandom.ReadBytes`. Done when the same seed yields the same shoe on your machine and in CI, and the wet test passes with `--wet`.
5. **Analysis and emission.** `simulation.dag`; the reconciliation test; a paired comparison of the two strategies over the same seeds; the example's closure emitted to Rust and `cargo check`ed; a `README.md` beside the example that explains the project through the four-view loop rather than through blackjack rules. Done when the acceptance bar below holds.

## Acceptance bar

- the model represents the scoped rules with no stringly substitutes — no `String` where a variant belongs, no `Int` standing in for a suit
- hand evaluation handles every ace and blackjack case in §4.2
- a round replays exactly from the same ordered shoe
- each of the four invalid transitions refuses with its named cause, and a test covers each
- shuffle output is reproducible from a recorded seed
- no unit test depends on live entropy; exactly one test exercises the real entropy route
- two strategies compare through the same engine over the same seeds
- `player_wins + dealer_wins + pushes == rounds_completed` is a test, and it is green
- every test names its case and can be made red by a mutation of the code it covers
- the example's closure emits Rust and the crate passes `cargo check` unedited
- the README explains the project through the lenses, not the rules

## 10. What to read in DESIGN.md, and when

`DESIGN.md` is the authority for every judgement about a change and is not a first-day document. This is the subset this project walks you into, in the order you will meet it:

| Before… | read |
| --- | --- |
| your first `type` | §2 *minimize redundancy* — model a concept once (why `win_rate` is not a field), and decompose to grounded atoms (why a card is a `Rank` and a `Suit`, not a string) |
| your first `//` comment | §4c — where a comment may sit, and that a comment is never evidence |
| your first refusal variant | §5 *fail-closed* — a wrong answer is a loud error, never a default; the three reviewer questions |
| your first test | §3 *a witness discriminates at one interface* — supply inputs at the boundary, and owe one real-route test per supplied boundary |
| your first PR | §3b conformance and §3c consumption — every declaration you add has a named consumer, and that consumer is in the same PR |
| the strategy comparison | §4d *epistemic humility* — assert the counts, type the inference |

Everything else in that document can wait until a review points you at it.

## Dissolution trigger (DESIGN §6)

Delete this brief when the five modules under dag/examples/blackjack/ and their claims under dag/test/claim/examples/ are landed and green on the required floor, and the example carries its own README written through the four-view loop -- at which point the example is the onboarding artifact and this document, whose only consumer was the person building it, has no subject left.
