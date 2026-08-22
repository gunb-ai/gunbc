# `type_reference_decl_file` answers two questions through one return type

Measured census, 2026-08-22. Subject: `main` at `4585d47ba7`. Producer: a local
instrumented build of the stage0 seed (below), run over one complete self-emit
(`claim_executor --required-regen --source-root dag --source-root src/v2`,
`planned=132 executed=132`, `compile.emit done in 5 minutes`).

## The claim

`v1.compiler.coercion` `type_reference_decl_file` returns a `String`. Its body has two
arms, and each arm can produce an empty or a non-empty file, so the one `String` carries
four structurally distinct provenances:

| cell | `n.inferred` | ident_span | returned | what the answer MEANS |
|---|---|---|---|---|
| 1 `resolved_span` | `Resolved { node: rt }` | `rt.ident_span` present | rt's file | the resolved DECLARATION's module — the question the consumers ask |
| 2 `resolved_no_span` | `Resolved { node: rt }` | absent | `""` | resolution SUCCEEDED, identity was lost |
| 3 `self_span` | not resolved | `n.ident_span` present | n's own file | the module the REFERENCE is written in — a different question |
| 4 `self_no_span` | not resolved | absent | `""` | identity genuinely unknown |

Cells 2 and 4 are indistinguishable downstream (both `""`). Cells 1 and 3 are
indistinguishable downstream (both a non-empty path) while answering different questions.

Cell 3's meaning is fixed at parse time, not inferred here: every named node is built with
`ident_span: default_ident_span(name:, span: name_span)` (`v1.compiler.core`), so an
unresolved type reference's own `ident_span.file` is the file the reference is authored in.
`type_reference_identity_note` describes this arm as "the reference IS the declaration"; that
is one case of it, not its extension.

## Measured classification (60,656 calls, one self-emit)

| cell | calls | share | executed call sites |
|---|---|---|---|
| 1 `resolved_span` | 3,300 | 5.4% | 14 |
| 2 `resolved_no_span` | **0** | 0% | 0 |
| 3 `self_span` | 57,356 | 94.6% | 20 |
| 4 `self_no_span` | **0** | 0% | 0 |

The zeros are exact for this product, not sampled: the instrument flushes on every first
occurrence of a (site, cell, answer) key, so a single call in cell 2 or 4 during this run
would appear. Per-cell totals are exact to the last 500-call flush.

**The grain of the zero claim.** Cells 2 and 4 are uninhabited *in one complete measured
self-emit*. That is not a structural claim that they cannot be produced. Making it structural
needs either a constructor proof that the empty value cannot be minted, or the same
measurement over every other admitted producer. Either way the four consumers written for
that state are dead on this product.

Full per-site rows: `type_reference_decl_file_four_cell_census_2026-08-22.tsv`
(`cell TAB count TAB caller-site TAB answer-file`).

### Two consequences

**The empty string never occurs.** Four downstream guards are written for it —
`decl_file_realizes_natively`, `decl_file_declares_structurally`, `lookup_checkpoint`'s
`decl_file == ""` bypass, and `type_realization_decision`'s
`Refused { cause: "declaration identity unknown: empty decl_file" }` — and across a complete
132-module self-emit none of them is reachable *from this producer*. `""` reaches them only
from the literal-`""` call sites that still exist elsewhere (e.g. `coercion_assertions`).
The prose that governs this function is written almost entirely about a state it does not
produce.

**The dominant real state is the one nobody named.** 94.6% of calls answer "which module is
this reference written in", and that answer is then keyed against rosters of DECLARING
modules. Splitting cell 3 by what it answered:

| cell 3 answer | calls |
|---|---|
| `<kernel:…>` synthetic span | 19,972 |
| ordinary module file | 37,140 |
| `dag/std/nat.dag` or `dag/std/integer.dag` | 244 |

The kernel population is intended — `numeric_realization_declaring_modules` carries the
`<kernel:` prefix deliberately. The 244 are the accidental-correctness population: an
unresolved reference realizes natively because it happens to be *written inside* the
declaring module, not because its declaration was identified. The mirror population — a
reference to `Nat`/`Int` written anywhere else, reaching cell 3 — silently does NOT realize
natively and renders structurally, which is the exact regression
`type_reference_identity_note` says this function exists to prevent. It is not prevented; it
is merely not *observed*, because the failing state is spelled the same way as success.

### Per-call-site (authority grain)

Cell 1 / cell 3 split by enclosing `.dag` function, aggregated from the per-site rows:

| authority fn | cell 1 | cell 3 |
|---|---|---|
| `v1.compiler.emit_rust` `field_access_field_is_boxed` | 34 | 15,615 |
| `v1.compiler.emit_rust` `rust_carrier_realizes_as_machine_scalar` | 2,803 | 12,129 |
| `v1.compiler.emit_rust` `needs_box_wrapping` | 50 | 7,264 |
| `v1.compiler.emit_rust` `render_rust_fn_sig_type` | 0 | 6,330 |
| `v1.compiler.emit` `render_node_type` | 0 | 8,548 |
| `v1.compiler.emit` `render_named_type_base` | 10 | 6,113 |
| `v1.compiler.emit_rust` `is_rust_value_type` | 1 | 659 |
| `v1.compiler.emit_rust` `rust_render_checkpoint_scalar_bare` | 132 | 395 |
| `v1.compiler.emit_rust` `render_rust_alias_rhs_type` | 153 | 228 |
| `v1.compiler.emit_rust` `render_rust_decl_type` | 67 | 49 |
| `v1.compiler.emit_rust` `render_rust_applied_type` | 50 | 23 |
| `v1.compiler.emit_rust` `emit_data_def` | 0 | 3 |

Only `render_rust_applied_type`, `render_rust_decl_type` and
`rust_render_checkpoint_scalar_bare` are majority-resolved. Every high-volume consumer is
answering on the reference's own file.

### A fifth fact: six authority call sites never executed

`type_reference_decl_file` has 29 syntactic call sites across 18 functions in
`v1.compiler.emit` and `v1.compiler.emit_rust`. Twelve functions executed. Six did not, on
this subject: `render_variant_payload_type`, `render_rust_fn_sig_type_applied_binding`,
`emit_typed_method_call`, `emit_struct_field_from_child`, `emit_rust_default_value`,
`emit_cli_param_type_node`. Their cell is unmeasured, not zero — a wider corpus may reach
them. Reported so that "every emission-boundary call" is not read as "every call site
observed".

## The invariant these numbers violate

*(Added after adjudication, 2026-08-22. The first revision of this probe said only "no
emitted-output defect is claimed". That restraint was right about emitted bytes and wrong
about what the numbers already establish, so the stronger reading is stated here and the
narrower one kept below it.)*

**Invariant.** Moving a reference between use sites must not change its type realization
while its resolved declaration identity is unchanged.

The 244 std-numeric cell-3 calls violate it, on main, today. They realize natively *only*
because the reference is authored inside `dag/std/nat.dag` or `dag/std/integer.dag`. Move the
same reference to another module and its realization changes while the declaration it
references does not. Realization here depends on **where the reference is written**, not on
**what declaration it resolves to** — which is the fact the roster is written to key on. This
is present behavior at 94.6% reach, not a latent risk.

So cell 3 is worse than "two questions through one return type". It collapses two *semantic
states* that require different decisions — *this node IS the declaration* versus *this node is
a reference whose declaration identity was not recovered* — into one value, "the file
containing the node", which is then compared against rosters of declaring modules. **No
consumer receiving only that value can be correct for both states.**

That is an instance of a general form, and this census is the specimen of it that carries an
executed reach number: *if a producer projection P maps two semantic states to one value while
the correct decision differs between them, no downstream guard receiving only P's output can
be correct for both.* Two other lanes hit the same class this cycle — a bare-name-keyed emit
summary map collapsing two declarations that share a name, so the ambiguity guard never sees
the homonym; and a dropped use-line candidate collapsing into silence.

## What this still does NOT establish

No emitted-output defect is exhibited: the census measures which question each call answered,
and does not trace a program to wrong emitted Rust. The 244 are correct answers reached by the
wrong route. The mirror population — a roster type referenced from outside the roster module,
therefore rendering structurally — is a prediction of the invariant above that this instrument
does not itself confirm; confirming it needs such a reference traced to emitted bytes. The
acceptance pairs below are exactly that confirmation, and the first of them fails today.

## Next rung

The class is *mitigatable*: the invalid conflation is fully writable and nothing detects it.
The adjudicated repair is not a patch to the `String` but a replacement carrier, naming the
four states with the vocabulary the corpus already uses for declarations and references:

```
TypeReferenceDeclarationStanding
  = ResolvedReference          { declaration_file }
  | DeclarationSelf            { declaration_file }
  | ReferenceIdentityUnavailable { use_file, cause }
  | SourceLocationUnavailable  { cause }
```

Five load-bearing properties, each a thing to check rather than a thing to write:
`ResolvedReference` never falls back to `use_file`; `DeclarationSelf` is constructed only for
an actual declaration node; unresolved identity stays a refusal-capable state; absence is
never represented by the empty string; and every consumer states which variants it can
legitimately accept. `decl_file_realizes_natively`, `decl_file_declares_structurally`,
`lookup_checkpoint` and `type_realization_decision` then consume the typed standing rather
than a plausible filename.

**Two discriminating acceptance pairs**, which are what make the repair falsifiable rather
than merely better-typed:

1. The **same declaration reached through two use sites** — one authored inside its declaring
   module, one authored elsewhere. Both must carry the same resolved declaration identity and
   produce the same realization. This is the executable form of the invariant above, and it
   **fails today**.
2. An **actual declaration node versus an unresolved reference in the same file**. They
   currently collapse to the same answer; the carrier must separate them.

This is a change to the frozen v1 seed plus a full regen cycle, so it goes to
`v1_seed_standing`'s purpose test **as its own repair**, on its own merits: it sits directly
on the v2 self-host path and already produces location-dependent realization. It is named here
and not taken, because this census is the *grounds* for that repair and must not be its
*vehicle*.

## Reproducing

Patch `src/v1/stage0/src/v1_compiler_coercion.rs` (the runnable mirror; NOT committed — it
drifts regen by construction), making `type_reference_decl_file` `#[track_caller]`, deriving
the cell from the same two matches, and recording `(cell, Location::caller(), answer)` into a
map dumped to `$GUNBC_TRDF_DUMP` on every first-seen key and every 500 calls. Then:

```
CTRL_BUILD_MODE=local /opt/cargo/bin/cargo build --release -p v1-compiler --bin claim_executor
GUNBC_TRDF_DUMP=/tmp/trdf.tsv ./target/release/claim_executor --required-regen \
  --source-root dag --source-root src/v2
```

`required-regen` reports `FAIL generated surface drift: v1_compiler_coercion.rs` — that is
the instrument itself, and it is expected. `#[track_caller]` attributes each call to its
syntactic caller in the mirror, which maps back to the authority function above.
