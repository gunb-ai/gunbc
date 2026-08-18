# E0308 root partition (mechanism grain)

| field | value |
|---|---|
| git_sha | `4e427773b78f04704dc9425a7acebdf719651da0` |
| modules | M=11 (partition §11.14) |
| distinct E0308 sites | **408** |
| paired rustc E0308 error blocks (summed) | 1555 |

## Per-module E0308 (diagnostic blocks vs distinct sites)

| module | E0308 blocks | distinct sites | share of module errors |
|---|---:|---:|---:|
| 05_emit | 101 | 86 | 6.5% of corpus blocks |
| 06_translate | 101 | 86 | 6.5% of corpus blocks |
| 04_infer | 100 | 85 | 6.4% of corpus blocks |
| 03_ingest | 396 | 358 | 25.5% of corpus blocks |
| emit_host | 242 | 216 | 15.6% of corpus blocks |
| 01_tokenize | 74 | 73 | 4.8% of corpus blocks |
| materialization_carriers | 28 | 16 | 1.8% of corpus blocks |
| emit_module | 109 | 94 | 7.0% of corpus blocks |
| 03_normalize | 90 | 75 | 5.8% of corpus blocks |
| program_partition | 103 | 87 | 6.6% of corpus blocks |
| 05_eval | 211 | 186 | 13.6% of corpus blocks |

## Mechanism roots (site grain)

| root | sites | % of E0308 sites | partition §11 owner |
|---|---:|---:|---|
| T7 | 99 | 24.3% | vivid-wren / checkpoint table |
| R1 | 91 | 22.3% | bold-lark-722 |
| RESIDUE | 59 | 14.5% | misc |
| T2 | 38 | 9.3% | unowned |
| T3 | 32 | 7.8% | unowned |
| B3 | 18 | 4.4% | eager-deer-389 |
| B2 | 17 | 4.2% | eager-deer-389 |
| RESIDUE-witness | 15 | 3.7% | closed (July) |
| R5 | 15 | 3.7% | unowned |
| C | 11 | 2.7% | gentle-dove-833 |
| B1-repr | 6 | 1.5% | eager-deer-389 / §18 |
| RESIDUE-diagnostics | 4 | 1.0% | closed (July) |
| T4 | 3 | 0.7% | unowned |

## Top pair signatures

- 61× `expected `Rc<Fnv1a64Structural>`, found `String``
- 38× `expected `String`, found `Rc<Fnv1a64Structural>``
- 19× `expected `Rc<Vector<_>>`, found `String``
- 18× `expected `Rc<Nat>`, found `i64``
- 18× `expected `Nat`, found `Rc<Nat>``
- 15× `expected `Rc<Nat>`, found `Nat``
- 14× `expected `bool`, found `Bool``
- 13× `expected `Coverage<Rc<...>>`, found `CoverageDefectAcceptanceKey``
- 11× `expected `Rc<Correction>`, found `Option<_>``
- 11× `expected `OrdSet<String>`, found `Rc<PointwisePower<_>>``
- 10× `expected `v2_std_node::OccurrenceId`, found `std_occurrence_identity::OccurrenceId``
- 9× `expected `String`, found `Rc<Vector<_>>``
- 7× `expected `SpanIndex`, found `Rc<SpanIndex>``
- 6× `expected `ScopeRoster`, found `Rc<ScopeRoster>``
- 6× `expected `SubjectRoster`, found `Rc<SubjectRoster>``
- 6× `expected `ConsumerRequirement`, found `Rc<ConsumerRequirement>``
- 5× `expected `OccurrenceId`, found `Rc<NodeOccurrenceId>``
- 4× `expected `String`, found `Option<String>``
- 4× `expected `Rc<Vector<i64>>`, found `String``
- 4× `expected `String`, found `Rc<Vector<i64>>``
- 4× `expected `Rc<DecimalDigitsStep>`, found `DecimalDigitsStep``
- 4× `expected `std_occurrence_identity::OccurrenceId`, found `v2_std_node::OccurrenceId``
- 3× `expected `Witness<ExitOk>`, found `Witness<Rc<Outcome<Rc<...>>>>``
- 3× `expected `HashMap<Rc<EnvironmentBindingKey>, ...>`, found `Rc<PartialFunction<_, _>>``
- 3× `expected `&HashMap<String, _>`, found `&Rc<HashMap<Rc<Node>, Rc<...>>>``

## Decision rules

1. Signature = rustc expected/found pair from span label or message.
2. Root assignment follows partition §11.3/§11.4 mechanism names.
3. One site may map to one root; pair diversity within a root is expected.

Repeat measurement: see [`e0308_root_partition_2026-08-18.md`](e0308_root_partition_2026-08-18.md) Method table.
