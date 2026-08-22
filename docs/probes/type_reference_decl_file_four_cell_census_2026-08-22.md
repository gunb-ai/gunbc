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

The zeros are exact, not sampled: the instrument flushes on every first occurrence of a
(site, cell, answer) key, so a single call in cell 2 or 4 would appear. Per-cell totals are
exact to the last 500-call flush.

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

## What this does NOT establish

No emitted-output defect is demonstrated here. The census measures which question each call
answered; it does not exhibit a program that emits the wrong Rust because of it. The 244
std-numeric cell-3 calls are correct answers reached by the wrong route, and the mirror
population is a hypothesis this instrument cannot confirm — confirming it needs a reference
to a roster type authored outside the roster module, traced to emitted bytes.

## Next rung

The class is *mitigatable*: the invalid conflation is fully writable and nothing detects it.
The construction move that would make it unwritable is to stop projecting four provenances
onto one `String` — a coproduct returned by `type_reference_decl_file`
(`ResolvedDeclaration { file } | ResolvedWithoutIdentity | ReferenceOwnModule { file } |
Unidentified`), so a consumer that means "the declaring module" cannot silently accept the
referencing module, and the two `""` cells stop sharing a spelling. That is a change to the
frozen v1 seed and a full regen cycle; it is named here as the trigger rather than taken,
because this work item asked for the classification and the purpose test for touching the
seed (`v1_seed_standing`) should be answered on the merits of that change, not smuggled in
behind a measurement.

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
