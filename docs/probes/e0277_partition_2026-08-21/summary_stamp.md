# E0277 root partition (trait × self-type grain)

| field | value |
|---|---|
| git_sha | `bb21f8563849b01cce9c978e4a1d9b170058c418` |
| modules | M=6, one dispatch, one checkout |
| distinct E0277 sites | **82** |
| paired rustc E0277 error blocks (summed) | 365 |
| inflation within E0277 | 4.45x |
| unclassified | **0** (classifier fail-closed; RESIDUE arm known-positive-controlled) |

## Per-module

| module | E0277 blocks | distinct sites |
|---|---:|---:|
| 03_ingest | 100 | 63 |
| emit_host | 100 | 53 |
| 05_eval | 80 | 44 |
| 05_emit | 57 | 32 |
| 01_tokenize | 15 | 11 |
| materialization_carriers | 13 | 10 |

## Roots

| root | sites | % |
|---|---:|---:|
| T5b — serde/Debug demanded over closure-bearing values | 35 | 42.7% |
| A — generic parameter bound not emitted | 30 | 36.6% |
| R3 — `Rc<dyn Fn..>` where an `Fn` bound is expected | 9 | 11.0% |
| T7 — map-key derives (Hash/Eq) missing on `Fnv1a64Structural` | 7 | 8.5% |
| T5a — map-key derive (Eq) missing on `OccurrenceId` | 1 | 1.2% |

## By trait

25 Clone (A) · 19 serde::Deserialize (T5b) · 12 Debug (T5b) · 9 Fn (R3) · 5 Ord (A) ·
4 serde::Serialize (T5b) · 4 std::hash::Hash (T7) · 3 Eq (T7) · 1 Eq (T5a)

## Top self types

A: `T` 15 · `P` 5 · `U` 3 · `A` 3 · `B` 2 · `S` 1 · `C` 1
T5b: `PartialFunction<String, ...>` 8 · `CompiledLexRule` 4 · `EffectIoEvalBundle` 3 ·
`ValueInterpreter` 3 · `TransformInterpreter`/`BranchInterpreter`/`LoopInterpreter`/
`BindInterpreter`/`MatchInterpreter` 2 each · bare `dyn Fn(..) -> ..` 5
T7: `Fnv1a64Structural` 7 · T5a: `OccurrenceId` 1

## Decision rules

1. Unit of count: one distinct `(file, line, col)` in the emitted crate.
2. Key: `(unsatisfied trait, self type)` parsed from the rustc message.
3. Self type that is a bare generic parameter -> A; concrete closure-bearing carrier under a
   serde/Debug demand -> T5b; `expected a Fn(..) closure, found Rc<dyn Fn..>` -> R3;
   Hash/Eq on a carrier used as a map key -> T7/T5a.
4. An unmatched message shape or an unmatched `(trait, self)` raises RESIDUE; nothing is absorbed
   into a neighbouring root.

Repeat measurement: see [`e0277_root_partition_2026-08-21.md`](../e0277_root_partition_2026-08-21.md) Method table.
