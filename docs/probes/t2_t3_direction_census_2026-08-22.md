# T2/T3 are not one cluster: the direction census

**OBSERVED ON:** `docs/probes/e0308_partition_2026-08-21/sites_classified.tsv`, board sha
`2a2bd0ad59cdc4d37f0ef35a72232bac57c9bbe7`, entry `src/v2/compiler/03_ingest.dag`, `M=1`.
**CLAIM ABOUT:** that artifact only. A refresh against current main is in flight and this
document is not re-stated against it; if the refresh moves the population, the shape below is
re-measured before it is quoted, not carried forward.

## The question

The work item premised T2 (34 sites, text carrier vs host `String`) and T3 (25 sites, collection
carrier fork) as "the ONE plausibly-shared realization cluster". Sharing a cluster means sharing
an arbiter. An arbiter that disagrees with itself is *position-dependent*; a mapping that is
simply absent is *one-directional*. Those are different defects with different repairs, and the
direction census separates them without needing the emitted Rust.

## The discriminator

For each root, count emitted positions (`file:line:col`) at which the same carrier pair is
reported in **both** directions.

| root | sites | positions with both directions | modules |
|---|---:|---:|---:|
| T2 | 34 | **4** | 3 (32/34 in `v2_compiler_tokenize.rs`) |
| T3 | 25 | **0** | 10 |

The four T2 positions are `v2_compiler_tokenize.rs` lines 228, 251, 272, 340 — each at column 13,
each carrying `Rc<Vector<_>> <- String` *and* `String <- Rc<Vector<_>>` at that one column, with a
third `Rc<Vector<_>> <- String` at column 25 of the same line.

T3's directions are `expected host, found modeled` at 24 of 25 sites — `OrdSet<String> <-
Rc<PointwisePower<_>>` (8), `HashMap<..> <- Rc<PartialFunction<_,_>>` (10), the rest scalar-vs-
`Vector` arity in `std_state_durability.rs`. The single reverse site is
`v2_std_compilers_target_model.rs:6056:5`, `Rc<Rc<PartialFunction<..>>> <- Rc<HashMap<_,_>>` — a
different file, a different position, not a within-construct disagreement.

## What this establishes

**T2 and T3 do not share an arbiter.** T2 exhibits within-position disagreement: one dag text
carrier rendered two ways at a single emitted column. T3 exhibits a uniform modeled→host
direction: the collection surface (`PointwisePower`, `PartialFunction`) is not lowered to its host
realization, everywhere, the same way. A repair to a position-dependent arbiter does not address
an absent lowering, and vice versa.

The brief's premise is therefore refuted by measurement, which was the brief's own first
deliverable.

## What this does NOT establish

The **mechanism** behind the T2 reversal. The emitted Rust for those lines is a probe artifact
and is not in the tree, so what construct sits at `tokenize.rs:228:13` is unread. Both directions
at one column is consistent with more than one construct (a comparison whose operands were
rendered from one dag type, a scrutinee/arm pair, a call whose parameter and return were arbitrated
separately). The shape of the divergence is measured; its cause is not. Naming a cause here would
be the fabricated-plausible-output failure this lane has already corrected itself for twice.

Nor does it establish anything about the *other* 174 E0308 sites, or about T2/T3 under the
refreshed board.
