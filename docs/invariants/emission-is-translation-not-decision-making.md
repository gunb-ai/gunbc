### Emission is translation, not decision-making

The emitter translates an annotated graph to target-language text. It
does not make structural, semantic, or rendering decisions. Every fact
the emitter needs — sharing strategy, type representation, clone
behavior, import requirements — must be in the graph or in LanguageSpec
data before emission begins. If the emitter branches on a type name,
checks a hardcoded list, or guesses a rendering choice, a fact was
lost at an upstream boundary.

**The principle:** emission is a pure function from (annotated graph +
LanguageSpec) to text. No heuristics, no fallbacks, no per-language
decision logic. Language-specific facts live in LanguageSpec data
declarations. The shared emitter reads them.

**The test:** if adding a new target language requires writing emission
*logic* (not just data declarations), the shared emitter is making
decisions that should be data-driven. Target-language-specific code
paths in the emitter are dual representations of facts that should be
in LanguageSpec.

**Fail-closed:** if the emitter encounters a type or construct for
which it lacks a rendering annotation, it must produce a diagnostic
error — not silently emit placeholder or structurally wrong code. A
`compile_error!("...")` in generated Rust is a fabrication fallback;
the compiler should have caught the gap before reaching emission.

**Known violations (2026-03-29):**

| Decision | Current state | LanguageSpec target |
|----------|--------------|---------------------|
| Sharing/wrapping | `rc_types` map (Rust only). Go emits bare value-type structs. | `sharing_wrap_template`, `sharing_construct_template` per language |
| Clone semantics | Hardcoded `.clone()` in Rust emitter; ownership analysis elides for fan-out=1 function params | Language-level clone/copy strategy in LanguageSpec |
| Option/absence | Emitter heuristic | Absence variant spec in LanguageSpec |
| Async/await | Hardcoded `"async fn"` in Rust emitter | Async syntax template |
| Import generation | Per-emitter logic | Module system spec |
| Container iteration | Hardcoded `.iter().cloned()` | Iterator pattern template |
| Record literal Rc wrap | `Rc::new(...)` hardcoded at construction | Driven by sharing strategy |
| Empty list in record field | Emits bare `vec![]` instead of `Rc::new(vec![])` | Should derive from sharing + type |

The sharing model is the canonical instance. `.dag` has value semantics.
Each target language has its own way of expressing shared ownership:
Rust uses `Rc<T>`, Go uses `*T`, Python has reference semantics by
default. This is ONE cross-language fact with per-language syntax —
not three independent implementations in three emitters.

