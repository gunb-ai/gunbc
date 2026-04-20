# P0-A — Fix `dsl/std/render.dag:repeat_string` (silent wrong execution) `(S)`

## Context

Exploratory analysis found a real bug with silent wrong execution in `dsl/std/render.dag:177-180`:

```
fn repeat_string(s: String, n: Int) -> String {
  if n <= 0 { "" }
  else { [0] |> fold(init: "", f: fn(acc, _) { concat(acc, s) }) }
}
```

The fold source is the singleton `[0]`. For every `n > 0`, the result is exactly **one** copy of `s`, never `n` copies. The function's declared contract — repeat `s` exactly `n` times — is violated at runtime.

Propagates to `dsl/std/render.dag:200-202`:

```
let pad = repeat_string(s: unit, n: level)
"\{pad\}\{text\}"
```

So `indent_text` is broken for all levels `> 1`. Any consumer reading indented output gets incorrect whitespace.

This is not structural debt. It's an actual "doesn't execute as declared" bug (THESIS.md §"What gunbc is").

## Read first

- `dsl/std/render.dag` — full file, focus on `repeat_string` and `indent_text`
- `dsl/std/algebra.dag` — look for an existing `repeat` / `replicate` combinator (FreeMonoid-related)
- `dsl/std/integer.dag` or `dsl/std/types.dag` — look for a way to produce a length-`n` list (range, replicate, etc.)

## Work

1. **Fix `repeat_string`**: fold over a list of length `n`, not a singleton.
   - Preferred: if `replicate(n: Int, elem: T) -> List<T>` or a similar combinator exists in std, use it: `replicate(n, 0) |> fold(init: "", ...)`.
   - If no such combinator exists, add one in `dsl/std/algebra.dag` (it's a natural FreeMonoid operation) and use it here. Do not inline a recursive helper.
2. **Verify `indent_text`** produces `s × level` correctly for `level ∈ {0, 1, 2, 3}`.
3. **Add a regression test** under the appropriate `src/v3/compiler/tests/` location (or v2 equivalent if std/render tests live there). Names per TESTING.md convention: `repeat_string_returns_n_copies`, `indent_text_produces_level_times_unit`.
4. **Grep for other consumers** of `repeat_string` (`git grep "repeat_string"`) — verify each was relying on the broken behavior (in which case they're also broken) or the correct behavior (in which case they'll fix naturally).

## Acceptance

- `repeat_string("x", 3)` returns `"xxx"` (not `"x"`).
- `repeat_string("", n)` returns `""` for all n.
- `repeat_string(s, 0)` returns `""`.
- `repeat_string(s, -1)` returns `""` (preserve existing guard).
- Regression test locked in.
- No new string-level fabrication. No placeholder shim. Real structural fix.

## STOP-AND-ESCALATE

- If no length-producing combinator exists and adding one to `algebra.dag` touches >50 lines or affects unrelated code paths — surface, name the scope.
- If `repeat_string` has consumers that were relying on the broken behavior (a weird fixture, maybe), flag those — they need independent attention.
- If the fix reveals that `[0] |> fold(...)` is a broader anti-pattern in std (other functions constructing a singleton fold instead of a length-`n` fold), STOP, list them, and schedule a separate sweep.

## Non-goals

- No algebra.dag rewrite beyond adding `replicate` if needed.
- No `render.dag` reorganization.
- No global string-handling refactor.

## Size: S (small — single function fix + possibly a helper + regression test).
