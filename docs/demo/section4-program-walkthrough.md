# gunbc demo: §4 first runnable program (TypeScript-first)

This walkthrough shows the §4 north-star artifact — `src/v4/program.dag`, the first
runnable v4 program with I/O — through TypeScript emission and execution first, then
through the in-language interpreter and run loop. Every command below was run in this
repository and shows actual output.

---

## 1. The program

Source: `src/v4/program.dag`

The program composes three §4 commitments:

- **Branch body (§2):** a `pick_if`-shaped branch whose then-arm runs an effect-IO
  roundtrip (`ReadResource` → `WriteResource`).
- **Effect handlers:** pure/test handler bundle bound at run start (no per-effect Rust
  glue in the compiler).
- **Run loop (§4c):** `runtime_run` schedules `kw_true` then the IO step, appending
  receipts sequentially.

The main witness `program_runs_holds` asserts all three layers together.

---

## 2. Emit TypeScript for effect IO

TypeScript comes first in this demo: applied `WriteResource` and `ReadResource` effects
emit as concrete call expressions through the descriptor projection path (not hand-listed
per-effect compiler arms):

```
gunbc run \
  --source-root src/v4 \
  --entry src/v4/test/claim/manual/typescript_effect_io_emit.dag \
  --function ts_effect_io_emit_holds \
  --claim-run
```

Output:

```
resolved 47 sources
running ts_effect_io_emit_holds()...
true
```

The witness asserts the emitted text equals:

```typescript
__gunbc_effect_write(ioPath, ioContent)
__gunbc_effect_read(ioPath)
```

The callee names and operand binding sites (`ioPath`, `ioContent`) come from the
TypeScript target model rows and binding spellings — not from compiler-stage string
templates.

---

## 3. Run the emitted TypeScript under node

The emit witness above is structural (text equality). The executable receipt proves the
**compiler-emitted** call expressions perform real file IO when bound to `fs` shims and
run under `node`:

```
cargo test -p v2-compiler-tests typescript_effect_io_receipt -- --nocapture
```

Output (abbreviated):

```
test typescript_effect_io_receipt_emitted_calls_perform_real_file_io ... ok
```

The test assembles a node program from the emitted write/read call text verbatim, binds
`__gunbc_effect_write` / `__gunbc_effect_read` to `fs.writeFileSync` / `fs.readFileSync`,
writes a marker to a temp file, reads it back, and asserts stdout matches. No per-effect
logic is hand-listed in the compiler (#4623 anti-cement).

---

## 4. Run the program in the interpreter

The same §4 spine runs inside gunbc's evaluator with handler-bound effect IO and the
`runtime_run` loop:

```
gunbc run \
  --source-root src/v4 \
  --entry src/v4/program.dag \
  --function program_runs_holds \
  --claim-run
```

Output:

```
resolved 49 sources
running program_runs_holds()...
true
```

`true` means: the Branch+IO eval path holds, the §2 `pick_if` body executes under the
handler-bound context, and `runtime_run` completes with a two-entry receipt log over the
scheduled frontier.

This witness is enrolled in the v4 claim-witness CI corpus as `§4 program runs keystone`.

---

## 5. What this covers today

| Layer | Witness | Mode |
|-------|---------|------|
| TS effect emit | `ts_effect_io_emit_holds` | emit text equality |
| TS real IO | `typescript_effect_io_receipt` | node execution |
| §4 interpreter + run loop | `program_runs_holds` | claim-run eval |

Full multi-target emit of `program.dag` (Rust/Python/Go bodies, not just effect-call
fragments) is §3 emit-breadth work and is intentionally out of scope for the §4-thin lane.
The TypeScript effect-IO slice shown here is the TS-first north-star proof that emitted
target text can perform real host IO without bespoke per-effect compiler glue.
