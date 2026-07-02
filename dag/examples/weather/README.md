# Weather forecast (hero demo)

A self-contained daglang program: domain types, pattern matching on a sum
type, pure functions, and a list pipeline. It compiles to Rust with no
hand-written glue.

## What it shows

| Feature | Where |
|---------|--------|
| Product types | `Temperature`, `Forecast` |
| Sum types | `Condition` (four variants, two with payloads) |
| Pattern matching | `describe_condition` |
| Collection pipeline | `freezing_locations` (`filter` / `map`) |
| String formatting | `daily_summary` |

## Compile gate

From the repo root (build the compiler once):

```bash
cargo build --release -p v1-compiler --bin gunbc
```

Then compile and type-check the emitted crate:

```bash
OUT=/tmp/weather-out
./target/release/gunbc compile \
  --source-root dag/examples/weather \
  --source-root dag/std \
  --output-dir "$OUT" \
  --target rust
cargo check --manifest-path "$OUT/Cargo.toml"
```

Expected: `compiled: … 0 diagnostics` and `cargo check` finishes clean.

## Source

All logic lives in [`weather.dag`](weather.dag). The compiler pulls in
`dag/std/` transitively for `Float`, `List`, `concat`, and friends.

## Tests in this repo

`src/v1/tests` includes structural checks over the emitted Rust (L4
bootstrap). That path is heavier than the gate above; the gate is the
fast public smoke test.
