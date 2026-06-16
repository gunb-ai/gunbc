# gunbc demo: write once, run anywhere

This walkthrough shows a programmer who has never seen gunbc what it can do today.
One program, three outputs: execution, TypeScript, and English prose.
Every command below was run in this repository and shows actual output.

---

## 1. The program

```
module v2.test.mvp1.add

fn add(x: Int, y: Int) -> Int {
  x + y
}
```

Source: `fixtures/v2-mvp1/add/add.dag`

The source contains no target-language syntax.
There is no `function`, no `def`, no `func`, no `return`, no `:`.
The types (`Int`) and the operation (`+`) are semantic concepts in the compiler model.
What you emit from this source is chosen at emit time, not written into the source.

---

## 2. Run it

The interpreter evaluates `add(2, 3) = 5` by execution — no compilation step, no host language:

```
gunbc run \
  --source-root src/v2 \
  --entry src/v2/test/claim/manual/comprep_eval_by_execution.dag \
  --function comprep_eval_source_driven_add_executes_holds \
  --claim-run
```

Output:

```
resolved 53 sources
running comprep_eval_source_driven_add_executes_holds()...
true
```

The function is a Bool witness: it resolves the `add` source, bridges it through the body
producer, calls `add(2, 3)` through the interpreter, and asserts the result equals 5 by
byte count. `true` means the execution matched.

---

## 3. Emit TypeScript

The same source emits TypeScript through the dissolved 06\_translate fold:

```
gunbc run \
  --source-root src/v2 \
  --entry src/v2/test/claim/manual/mvp1_typescript_add_translate.dag \
  --function mvp1_ts_emit_add_fn_accepts_holds \
  --claim-run
```

Output:

```
resolved 64 sources
running mvp1_ts_emit_add_fn_accepts_holds()...
true
```

The witness asserts the emitted text equals the authority defined at
`src/v2/extdeps/languages/typescript.dag:302`:

```typescript
function add(x: number, y: number): number { return x + y; }
```

The `Int` type mapped to `number`. The `x + y` body mapped to `return x + y;`.
Neither mapping was hand-coded for this function; they follow from the target model rows
for TypeScript's number type and arithmetic operators.

---

## 4. Emit English

> **Branch note:** English emit lives on PR #4790 (`session/sunny-hawk-310`), which is not
> yet merged to main. The commands below run from the `sunny-hawk-310` worktree.
> Flip the `--source-root` to `src/v2` on main after the PR merges.

```
gunbc run \
  --source-root src/v2 \
  --entry src/v2/test/claim/manual/english_emit_add.dag \
  --function english_emit_add_prose_holds \
  --claim-run
```

Output (run from `sunny-hawk-310`):

```
resolved 51 sources
running english_emit_add_prose_holds()...
true
```

The witness asserts the emitted text equals the authority defined at
`src/v2/extdeps/languages/english.dag:391`:

```
the add is a function. the add takes the x. the add takes the y. the add returns the sum. the sum adds the x. the sum adds the y.
```

Same source. Same fold. Different target model rows — this time producing controlled-English
noun phrases instead of TypeScript tokens.

---

## 5. What this covers today

The TypeScript emit witness (`mvp1_ts_emit_add_fn_accepts_holds`) is emit-only: it
verifies the emitted source text equals the authority string. There is no executed
tokenize+parse consumer for the emitted `function add(...)` text on main today;
`src/v2/test/claim/manual/comprep_add_body_emit_typescript.dag` explicitly scopes
itself as an emit-only receipt (E-10). The landed ingest round-trip is for the `.dag`
target (`mvp1_dag_add_emit_ingest_round_trip_holds`), not for TypeScript text.
Arbitrary TypeScript programs are also not yet ingestible; no TS text-ingest witness
exists until a future grammar claim is authored and executed.

Rust, Python, and Go emit for the same add slice are also verified by the same mechanism
(`mvp1_rust_emit_add_fn_accepts_holds`, `mvp1_python_emit_add_fn_accepts_holds`,
`mvp1_go_emit_add_fn_accepts_holds`). The interpreter, the TypeScript emit witness, and the
English emit witness shown here are the three distinct evaluation modes active today.
