# The emitter produces invalid Rust from a service-level `Filesystem` call (2026-08-22)

**Not this lane's to repair.** Recorded here because it was surfaced by the carrier-realization census
and has no existing authority; ownership belongs elsewhere.

## OBSERVED ON / CLAIM ABOUT

- **OBSERVED ON:** the `witnesses` CI job for gunbc#8816 at `993163ae32`, step *Build the witness
  fold*, compiling the committed stage0 mirror of `v1.tests.claim.carrier_realization_census`.
- **CLAIM ABOUT:** the Rust the regen producer emits for a **service-level `Filesystem` operation**
  called from an ordinary (non-async) `fn` in a module inside the v1 seed closure.

## The specimen

Source, in `src/v1/tests/claim/carrier_realization_census.dag`:

```
let read_out = Filesystem.Read(path: path)
```

Emitted, in `src/v1/stage0/src/v1_tests_claim_carrier_realization_census.rs`:

```rust
let read_out = filesystem.read(path.clone()).await?;
```

Diagnostics — **two independent invalidities from one construct**:

```
error[E0425]: cannot find value `filesystem` in this scope
  --> src/v1/stage0/src/v1_tests_claim_carrier_realization_census.rs:292:28
error[E0728]: `await` is only allowed inside `async` functions and blocks   (x4)
```

The emitter bound no receiver named `filesystem`, and placed an `.await` inside a fn it did not make
`async`.

## Why this is below floor rather than a low rung

The source typechecked. The emitter did **not** refuse, did **not** diagnose, and did **not** stop the
line — it produced plausible-looking Rust that fails only at `rustc`. That is fabricated plausible
output at the emission boundary (DESIGN §5), which is outside the ladder, not a rung on it. A typed,
located refusal — *"this effect is not emittable to this target"* — would be strictly better than
today's behaviour.

## Scope, stated narrowly on purpose

What is established is **one construct, in one context, on one target**. It is *not* established that
every `Filesystem` call in every seed-closure module is unbuildable, and two neighbouring authorities
say the surface is partitioned rather than uniform:

- `std.emit_on_demand` `emitted_effect_family_boundary_note` names
  `EmittedEffectFamily = FilesystemReadFamily | ShellExecRunFamily` as *the emitted-native host-effect
  subset*, explicitly excluding `Filesystem.Write` and generic transports — so an emitted-native path
  exists for some read shapes.
- `tools.frontier_ingestion_probe` `frontier_read_refusal_disposition_note` and
  `test.claim.seed_mirror_constant_lens_witness_test` `seed_read_has_no_success_arm_note` both record
  that the **seed primitive** `filesystem_read` (returning a one-field `FilesystemReadResult`) is a
  *different carrier* from the **service-level** `Filesystem.Read` operation's three-field contract.

So the honest statement is that the service-level operation, reached from this position, emits invalid
Rust — and which positions are emittable is exactly what nobody has written down.

## A second, independent defect in the same mirror: the emitted method ignores its argument

Reading the full emitted `Filesystem` impl (recovered from the deleted mirror in git) shows the
service methods do not do what they are named. Every one of them — `read`, `write`, `write_owner_only`,
`delete`, `list` — has the same body shape:

```rust
/// Modifiers: readonly
pub async fn read(&self, path: String) -> Result<(String, bool, String), Box<dyn Error>> {
    if self.dry_run.is_dry_run() { ... panic!("no mock data available ...") }
    else {
        let path = format!("{}/{}", self.base_path, "read");   // <- shadows the argument
        let content = std::fs::read_to_string(&path)?;
        let parsed: serde_json::Value = serde_json::from_str(&content)?;
        Ok(serde_json::from_value(parsed)?)
    }
}
```

The caller's `path` is **shadowed by a path built from the operation's own name**, so `Filesystem.Read`
would read a file literally called `./read` and parse it as JSON, whatever the caller asked for. The
same substitution appears in all five operations. This is independent of the call-site defect above:
even if the emitter bound a receiver and marked the caller `async`, the operation would not perform the
read its contract describes.

Both defects are silent at the `.dag` layer. The first fails at `rustc`; **the second would compile**
and fail at runtime with a wrong or missing file — which is the more dangerous of the two.

## Why it matters beyond the lane that found it

Nothing reports this until `rustc`. It is invisible at the `.dag` layer, so it is a standing trap for
any future lane that reaches for a file from inside the seed closure, and it means the v2 self-host
program currently carries an effect in its own vocabulary that it cannot emit.

**Consequence already forced on this lane:** the carrier-realization census cannot source its own
input in the terminal witness shape, because reading the corpus is not an optional readout. Its input
must arrive from a harness that has already crossed the boundary.

---

## SUBJECT CORRECTION — "two defects" is the wrong frame; it is one binding question with two symptoms

Confirmed in the emitter source by another lane after this document was written. **What is observed
above stands. What it is a defect *in* has moved.**

**Defect 2 is probably not a defect.** `src/v1/05_emit_rust.dag` `file_transport` emission
unconditionally builds the path from `base_path` plus the **operation name** — there is no branch by
which a caller-supplied path could reach it. So the reading above was exact, but the thing being read
is a **file transport**: *fetch the recorded result for operation X from a directory of files named
after operations*, parsing JSON when the return type is a multi-field product and returning text
otherwise. That is correct fixture behaviour, sitting beside shell and REST as one of N realizations
of a service operation. The surrounding machinery agrees — `00_core.dag` carries `file_transport_node`,
`is_file_transport` and `transport_base_path`; `languages.dag` carries `file_ctor`. The dry-run arm's
*"no mock data available"* is the same fact from the other side.

So the real question is **why `Filesystem`'s operations are bound to the file transport at all**, and
the likely answer is that a real filesystem handler is missing and the binding fell through silently.
That is DESIGN §3's interface-versus-realization seam. It also means **teaching the file transport to
honour a path argument would break the one thing it does correctly** while still giving nobody a real
read.

**The independence claim above needs the same correction.** It was accurate as an observation about
two code paths and is probably wrong as a claim about two root causes: defect 1's unbound receiver is
plausibly the same misbinding surfacing at the call site rather than an independent template bug.

**And it is two targets, which neither this document nor the lane that corrected it found first.**
`src/v1/05_emit_python.dag` emits the identical construction, so a Rust-only repair leaves Python
broken and the next target gets a third copy — one operation body forked per target instead of one
shape with N bound handlers, DESIGN §3's fused-transport tell in its literal form. The population is
**per-target emitters**, not one function.

**Unaffected by this correction:** defect 2 being strictly worse than defect 1, for the reason given
above — one stops the line at `rustc`, the other compiles and would have silently read `./read` and
parsed it as JSON. Build-time fabrication versus runtime fabrication.

This is the third time in this lane that a correct observation was filed under a wrong subject (the
pre-reconcile phase break, the variant-blindness absence, and now this). The guard that catches it is
stating **OBSERVED ON** and **CLAIM ABOUT** separately — which this document does at the top, and
which is why the observation survived the correction intact.
