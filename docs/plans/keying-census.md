# Keying census: the classified population

**Status:** measured 2026-08-25 on the working tree. Owner: `neat-fox-901`.
**Subject:** every String-keyed collection in the corpus, classified by what its key
denotes, and the FreeText denominator that sizes the repointing work.

**Layering — this document is the POPULATION, and cites rather than restates:**

| document | role |
|---|---|
| `keying-relation-design.md` | the MODEL — operator-agreed 2026-08-15 |
| this file | the POPULATION — measured and classified |
| `namespace-cut-postmortem-and-identity-program.md` | the PROGRAM — cites both |

The law this census is taken against is stated in that postmortem's §5 and is not
restated here. It was derived independently by two lanes on the same day from different
evidence (this lane from the keying census, `snappy-dove-250` from the emitter's
`shared_types` lookup), which is the only cheap evidence available that the restatement
is the subject's shape rather than one author's preference.

---

## 1. Headline

**The FreeText denominator is 8 of 2272 sites — 0.35%.**

Essentially nothing in the compiler population is legitimately String-keyed. The corpus
is not sloppy about keying and it is not lazily using `String` where a type would do:
it is spelling *declared identities* as text, at scale, because until
`rust_nominal_identity_carrier_type_eligible` returns a real predicate the realization
cannot express anything else.

## 2. Population

Derived mechanically from source over `src/v1` and `dag`; `src/v2` reported separately
because it is not Track D's subject.

| root | String-keyed sites |
|---|---|
| `src/v1` | 2093 |
| `dag` | 179 |
| `src/v2` | 52 |

## 3. Classification

Over `src/v1` + `dag` (2272 sites). Every site lands in exactly one bucket; unmatched
names land in a **named UNDECIDED bucket**, never silently clean (DESIGN §5).

| class | sites | share |
|---|---|---|
| `ResourceLocator` | 838 | 36% |
| `SubjectKey_as_text` | 817 | 35% |
| `UNDECIDED_anonymous_position` | 314 | 13% |
| `UNDECIDED_unclassified` | 222 | 9% |
| `UNDECIDED_inherits_producer` | 73 | 3% |
| **`FreeText_upstream`** | **8** | **0.35%** |

`StateRevision`, `ContentIdentity` and `DisplayLabel` have **zero** sites in this
population. They are real distinctions in the model and this corpus's String keys do not
occupy them — worth recording, because it means the impostor separation is not
speculative breadth: the two arms that need to exist first are `SubjectKey` and
`ResourceLocator`.

### 3a. The FreeText 8, enumerated because the number is load-bearing

All in `extdeps/`, exactly where the model predicts genuine free text:

- `env: Map<String, String>` — POSIX environment variable names
  (`extdeps/transports/shell.dag`, `extdeps/rust/cargo_build.dag`)
- `attribute_mapping: Map<String, String>` — upstream GCP IAM attribute names
  (`extdeps/cloud/gcp/*.dag`)
- `ts_keyword_set` / `ts_keywords` / `ts_keyword_literals` — cited TypeScript keyword
  spellings (`extdeps/languages/typescript/syntax.dag`)

The TypeScript rows are the interesting arm: the key *is* the spelling, and the spelling
*is* the identity under the upstream grammar's relation. That is a correct String key,
not a tolerated one — the upstream vocabulary case.

### 3b. Two key ROLES this census's vocabulary cannot express

Raised by adversarial review of this PR, and it is a real defect in the classification
scheme rather than a measurement error — recorded because a bucket that does not exist
cannot receive a site, so its absence is invisible in the table above.

The six buckets classify what a key DENOTES. They do not express two legitimate key
ROLES:

- **cache determinant** — *is the prior result semantically reusable?* The minimal such
  key includes every result-determining input (source digest, dependency interfaces,
  compiler identity, target realization, result-changing options) and excludes the rest.
  It is deliberately MORE than the subject's identity, so it is not `SubjectKey` and it is
  not a defect.
- **grouping key** — *which equivalence class does this fall into?* Multiplicity is
  expected, so it must never go through a unique roster.

**Measured occupancy in this population: effectively zero.** The `Map<String, _>` sites
whose names suggest memoization — `seen`, `visited`, `depth_map`, `accepted_map`, and the
`*_index` family — are memo or membership structures whose KEY is still a declared
identity (a module, function or declaration name), so they are correctly
`SubjectKey_as_text`. Exactly one genuine cache-keyed String map exists corpus-wide,
`census_cache` in `v2.lens.reference_deps`, and it is outside this denominator. The
repo's real cache determinants are keyed on content hashes and realization plans, not on
`Map<String, _>`.

So the counts in §3 do not move. What moves is the **law's phrasing**, and that matters
beyond this census: *"nothing else is spellable in key position"* is false for a cache
determinant, where including more than subject identity is correct. The relation-
parameterised form survives the counterexample:

> `key_R(x) = key_R(y)` **iff** `same_R(x, y)`

Breaking the forward direction is under-keying (collapse); breaking the reverse is
over-keying (aliasing). Both error terms fall out of one statement, and because the
relation `R` is a parameter, cache determinants, grouping keys and locators are ordinary
instances rather than exceptions carved out of a subject-identity rule. **A keying program
must classify the key's ROLE first**, or it degenerates into wrapping every map key in
`SubjectKey`.

## 4. Track D is ~238 decisions, not 2093 sites

This is the finding that should size the work, and it is not visible from the site count.

**238 distinct binding names cover the 1958 named sites.**

| top N names | sites covered | share |
|---|---|---|
| 1 | 726 | 37% |
| 5 | 1207 | 61% |
| 20 | 1497 | 76% |
| 50 | 1680 | 85% |
| 100 | 1814 | 92% |

132 names occur exactly once.

**One parameter is 37% of the population.** `source_indices` (726) plus its abbreviation
`si` (106) is 832 sites — a single `Map<String, NewlineIndex>` threaded through the
compiler, keyed on a source file path. It is one keying decision replicated 832 times by
parameter threading, not 832 decisions. Repointing it is one change with a large diff;
counting it as 832 units of work overstates Track D by more than a third.

The next four — `shared_types` (179), `registry` (159), `type_summaries` (37),
`variant_to_enum` (35) — are all declaration-name-keyed, and `shared_types` is the
emitter defect the postmortem leads with.

## 5. The classifier's own defect, recorded because it is the census's subject

The classification function keys on the **binding name**. That is itself an over-keyed
lookup, and it produced a wrong answer inside this census: `env` denotes a *type
environment* in `src/v1` (a declared identity) and *POSIX environment variables* in
`dag/extdeps/transports/shell.dag` (genuine free text). One spelling, two subjects, two
roots — and the first pass classified all of them as `SubjectKey_as_text`.

It was caught by reading the declarations rather than by the classifier, corrected with a
path-aware rule, and is recorded rather than quietly fixed because it is a live instance
of the law under census: **a name is not a sufficient key for a subject, including when
the subject is "what does this key denote".** It is also the argument for §7's terminal
instrument — resolve the declaration, never match the spelling.

## 6. What this census does NOT establish

- **It is text-derived, not Node-derived.** The requested shape was a Node-tree census.
  This is a mechanical extraction over source text, so it cannot see a key type reached
  through an alias, a generic instantiation, or a re-export, and it classifies by binding
  name rather than by resolved declaration. Every count above is therefore a **lower
  bound on identity-keying and an upper bound on nothing**.
- **The UNDECIDED 609 (27%) are not FreeText by default.** Sampling the unclassified tail
  found `param_names`, `fn_decl_items`, `deps_map`, `variant_surfaces`, `service_registry`,
  `scc_index`, `local_func_set`, `descent_vars`, `by_name`, `scope_locals` — all declared
  identities. The measured FreeText floor is 0.35%; the arithmetic ceiling if every
  UNDECIDED site were free text is 27%, and the sample says it is nowhere near that.
- **No site was repointed and none should be** until the emitter's nominal-identity path
  is real. `src/v1/05_emit_rust.dag` `rust_nominal_identity_carrier_type_eligible` returns
  constant `false`; `rust_nominal_identity_carrier_def` — which emits
  `pub struct Name(pub String);` — is authored and unreachable, with 5 call sites waiting
  on the predicate. Repointing before that moves the fork rather than closing it, and
  greens a model-side check while the realization keeps lying (DESIGN §4b: a class's rung
  is the MINIMUM across its paths).

## 7. The terminal instrument, and why it does not exist yet

A re-derivable census must consume the Node tree. The builtin for it exists:
`decl_facts` (`coproduct_reflection.rs` `decl_facts_corpus_walk`) genuinely parses —
`parse_dag_file`, with `DeclFactRaw` carrying `node: Rc<Node>` and `source_indices` — and
is registered as a `.dag` primitive, so a lens can fold declaration Node trees directly.
`v2.lens.grounding` already consumes the sibling `concept_decl_facts(pool_roots:)` and is
the shape to copy.

**Two cautions for whoever builds it**, both found while looking for it:

- `fact_cardinality_decl_facts` — the other "live declarations" builtin, and the more
  obvious one to reach for — does **not** parse. It is a line-oriented text scanner
  (`extract_top_level_decls` matches `line.starts_with(kw)` and hashes raw body text).
  A census built on it would be text-derived while appearing tree-derived.
- `decl_facts_corpus_walk` **silently skips unparseable files** (`let Some(parsed) = … else
  { continue }`) and excludes test files. It does return `files_scanned` and `files_parsed`,
  so the skew is observable — but only if the consumer reads both. A lens that reads only
  `facts` reports a narrowed population as a complete one, which is the empty-observation
  narrow.

Until that lens exists this census is a one-time measurement with its method stated, not
an instrument. It is deliberately not committed as a script: an ad-hoc `.sh`/`.py` here is
§6 unmodeled realization, per the 2026-08-24 ruling that bankrupted `docs/probes/`.

## 8. Read on the impostor separation

**Authorable today against real consumers, for two arms only.**

`SubjectKey<K>` and `ResourceLocator<L>` each have a large, already-identified consumer in
this population — 817 and 838 sites — and the two are not interchangeable at a single site
in it, so the distinction does real work immediately rather than being asserted breadth.
`ContentIdentity<H>` has an obvious consumer one layer over in `std.content_hash` but
**zero sites in this population**, and `StateRevision` / `DisplayLabel` have neither. Per
`keying-relation-design.md` §6's own rule — nothing is built until a real consumer needs
it — the honest first landing is two arms, not five.

The blocker is not authorability. It is §6 above: the arms are declarable now, and nothing
may repoint onto them until emission can carry them.
