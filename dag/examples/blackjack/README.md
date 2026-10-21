# Blackjack: a small Dag model

This example is a guided tour of Dag’s four-view loop. It models a small
Blackjack domain; it is not gambling advice.

## 1. Model the facts

The model is split by responsibility:

- `cards.dag` defines `Card`, `Rank`, `Suit`, and `standard_deck`.
- `hand.dag` derives a hand total, softness, bust status, and Blackjack status.
- `round.dag` is the pure round state machine. Invalid transitions are explicit
  `RoundRefused` values rather than exceptions or default results.
- `shuffle.dag` turns a recorded `ShuffleSeed` into a deterministic `Shoe`.
- `simulation.dag` runs seeded rounds, aggregates completed observations, and
  preserves any engine refusal with its replay seed and round index.

## 2. Run a pure example

A round has no hidden randomness. The same ordered shoe and rules always
produce the same result.

Randomness is kept at the boundary:

```text
Urandom.ReadBytes -> Base64 decode -> ShuffleSeed -> shuffle -> Shoe
```

Once a seed is recorded, a shuffle and every simulated round can be replayed.
Each simulated round uses a fresh shuffled 52-card shoe.

## 3. Inspect test evidence

Witness tests live under `dag/test/claim/examples/`.

They cover hand evaluation, round transitions, deterministic shuffle behavior,
simulation reconciliation, and paired strategies running over the same seeded
shoes. The simulation witness also supplies a deliberately short second shoe
and verifies that the run stops with `SimulationRefused` at the correct index,
seed, and `ShoeExhausted` cause.

The only entropy-dependent test is the wet shuffle-boundary test. It checks
that real decoded entropy produces a 52-card permutation, without asserting a
particular card order.

## 4. Inspect emitted Rust

Dag is the source program. Rust is a generated projection and must not be
edited by hand.

```bash
OUT=$(mktemp -d)

./target/release/gunbc compile \
  --source-root dag \
  --entry dag/examples/blackjack/simulation.dag \
  --output-dir "$OUT" \
  --target rust

cargo check --manifest-path "$OUT/Cargo.toml"
```

The model’s key summary invariant is:

```text
player_wins + dealer_wins + pushes == rounds
```

The summary reports observed counts. It does not store a `win_rate` or claim
that one strategy is universally better than another from a finite sample.