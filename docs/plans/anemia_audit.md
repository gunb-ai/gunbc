# Anemia Lens — Audit

_Generated 2026-06-20 from a live run of the pipeline (real haiku calls, REST, temperature 0)._

## What we have (the pipeline)

A 3-stage pipeline that finds **anemic nicknames** — a field typed as a bare `String` whose *name*
coincides with a real concept that already exists in the codebase, so the field should point at that
type instead of re-spelling it as text (`base_url: String` while a `Url` type exists; `content_hash:
String` → `ContentHash`).

```
  EXTRACT  ───────▶  DECIDE  ───────▶  CONFIRM  ───────▶  verdict
  concept index      Signal-A name      haiku judge        REAL / CLEARED
  (real, not text)   coincidence        (4-tuple, temp 0)
```

- **EXTRACT** — `v2.lens.anemia` (green by execution, PR #5416). Folds the compiler's real concept
  index (`enumerate_concepts()`), so it reads *type-level concepts*, not source text. It therefore
  cannot mistake a coproduct arm for a type — a class of false-positive that a regex scan produces.
- **DECIDE** — deterministic, in the lens. A field is a *candidate* when its declared type is bare
  (`String` / `NonEmptyStr`) **and** its name coincides (case- and underscore-insensitive, whole-name
  or head-noun) with an existing concept name. Provable signals and §5 escapes are decided here; only
  the genuinely-semantic, name-only residual is sent to the judge.
- **CONFIRM** — `gunbc.tools.anemia_confirm`. Calls `llm.Anthropic.Messages` (claude-haiku-4-5, REST,
  temperature 0, realized as curl by `gunbc run`), one call per candidate, and adjudicates **REAL**
  (anemic) or **CLEARED** (legitimately a string).

## The input (what the judge actually sees)

For each candidate the judge gets exactly four lines — no source code, no file path, no extra context:

```
enclosing_concept: <the type the field lives in>
field:             <field name>
declared_type:     String | NonEmptyStr
coincides_with:    <the existing concept it matches>
```

plus a fixed system prompt that frames the decision generatively: *decompose the value — if it has
named internal parts or a closed/fixed structure → REAL; if it is an extensible registry, a
constraint/grammar string, or an opaque/free-form value → CLEARED.* Output is one token.

The `enclosing_concept` line is load-bearing: it is what lets the judge tell a dependency *version
constraint* (`CargoDepSource.RegistryDep`) from a plain version value.

## The verdicts (12 live cases across the codebase)

| # | located | input `enclosing.field : type → coincides_with` | verdict | assessment |
|---|---|---|---|---|
| 1 | dsl/std/types.dag:480 | `TransportRequest.url : String → Url` | **REAL** | ✅ real nickname |
| 2 | dsl/extdeps/cloud/cloud.dag:74 | `ServiceEndpoint.base_url : String → Url` | **REAL** | ✅ real nickname |
| 3 | src/v2/lens/fact_cardinality.dag:32 | `FactCardinalityDeclFact.content_hash : String → ContentHash` | **REAL** | ✅ real nickname (algo:hash) |
| 4 | dsl/extdeps/github/gists.dag:21 | `GistFile.language : String → Language` | CLEARED | ~ defensible: gist language is an open set |
| 5 | dsl/extdeps/github/pulls.dag:155 | `IssueComment.diff : String → Diff` | REAL | ~ borderline: `Diff` is a real record type |
| 6 | dsl/extdeps/rust/cargo.dag | `CargoDepSource.RegistryDep.version : String → SemVer` | **CLEARED** | ✅ §5 constraint-grammar (got the context) |
| 7 | dsl/extdeps/container/oci | `OciDescriptor.mediaType : String → OciMediaType` | REAL | ❌ §5 extensible-registry — should CLEAR |
| 8 | dsl/extdeps/github/github.dag:63 | `Pagination.cursor : String → Cursor` | **CLEARED** | ✅ §5 opaque token |
| 9 | src/v2/compiler/01_tokenize.dag:142 | `RepeatState.lexeme : String → Lexeme` | REAL | ❌ §5 parse-input — should CLEAR |
| 10 | dsl/extdeps/llm/anthropic_rest.dag:42 | `AnthropicMessagesRequest.system : String → System` | **CLEARED** | ✅ free-form prose |
| 11 | dsl/std/os/types.dag:16 | `OperatingSystemSurface.distro_or_product : NonEmptyStr → Product` | REAL | ~ weak/coincidental match |
| 12 | dsl/gunbc/tools/cron_tag.dag | `CronEntry.schedule : String → Schedule` | REAL | ❌ §5 cron-grammar — should CLEAR |

## Reading the results

**Where the judge is strong:** the clear nicknames (1–3) all land REAL, and it correctly CLEARs three
distinct kinds of "legitimately a string" case — a dependency **version constraint** (6), an **opaque
pagination token** (8), and **free-form prose** (10). Case 8 is worth noting: a pagination `cursor`
was a rubber-stamped false-positive in earlier experiments; with the `Pagination` enclosing context +
the opaque-token guidance, it now CLEARs correctly.

**The judge's blind spot (the 3 misses — all the same shape):** cases 7, 9, 12 are §5 "legitimately a
string" cases where the *correct* call needs domain knowledge the bare type name doesn't carry, so the
judge sees a plausible-looking target and assumes it is closed:
- `mediaType → OciMediaType` — OCI media types are an **open, extensible registry** (a closed enum
  would reject valid custom types).
- `lexeme → Lexeme` — a lexeme is **raw parse input**, not a value that should be re-typed.
- `schedule → Schedule` — a cron `schedule` is a **grammar string**; the matched `Schedule` type is a
  list of runnables — the *wrong* target entirely.

All three are the same failure: **name-only candidates invite over-confirmation.** The judge reasons
well about *clean* inputs but cannot infer "this registry is extensible" or "this matched type is the
wrong one" from a type name alone.

**The takeaway:** the precision bottleneck is upstream of the judge — it is *candidate quality and the
context fed in*, not the judge's reasoning. The two levers that close the misses:
1. carry the **target's actual definition** into the 4-tuple (extensible? a grammar? opaque?), so the
   judge can see "OCI media types are an open registry" rather than guess from the name; and
2. let **DECIDE** clear the provable §5 cases deterministically (parse-input fields, `Other`/`Unknown`
   escape arms) so they never reach the judge.

## How to run it yourself

```
ANTHROPIC_API_KEY=$(grep -oE 'CLAUDE_API_KEY=.*' ~/.env | cut -d= -f2-) \
  gunbc run --source-root dsl --entry dsl/gunbc/tools/anemia_confirm.dag --function anemia_confirm
```

Edit the `sample_findings` list in `dsl/gunbc/tools/anemia_confirm.dag` to audit other candidates.
(The run prints the verdict list, then exits non-zero on a cosmetic return-type contract — the entry
function returns a rich record rather than `ProcessExit`; wrapping that is a one-line follow-up.)

_Note: candidates here are hand-fed (mirroring what the v2 lens emits) because the v2 lens and the dsl
LLM op live in the two std trees that can't co-load yet (the dsl↔v2 de-fork). The verdicts are real;
the lens→judge handoff becomes automatic once that de-fork lands._
