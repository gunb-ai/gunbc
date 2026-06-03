# v4 Stable Rustc Bands RCA Worksheet — 2026-06-03

> **Status:** DRAFT RCA WORKSHEET — routes stable bands without creating a broad SG-3 lane.
> **Authority:** PR #4140; stable-band counts matched exactly on regenerated Jun1 probe.

---

## Band Table

| Code | Count | Primary readout | Route |
| --- | ---:| --- | --- |
| `E0277` | 330 | trait bound / Ord eligibility on realized operands | Target trait eligibility + collection realization (SG-5) |
| `E0573` | 159 | expected type, found variant | SG-8 path/type-vs-variant projection |
| `E0560` | 126 | struct/variant missing field | record/variant field admission + target constructor projection |
| `E0369` | 110 | binary op on `Rc<T>` | SG-RC-LAYERING / operator operand unwrapping |
| `E0121` | 44 | `_` placeholder in item signatures | signature projection must fail closed before Rust text |
| `E0391` | 29 | recursive aliases / drop-check cycles | hollow/self-alias realization |
| `E0599` | 28 | missing method, often generic trait bound fallout | remeasure after SG-2 + trait eligibility |
| `E0061` | 12 | argument-count mismatch | function/call signature projection |

---

## Worksheet Drafts

### Stable-A: Trait eligibility at realized boundary

```text
Representative:
  Set element type DiffId is not Ord-eligible for BTreeSet
  binary/operator sites where realized operand is Rc<T> but trait is on T

Single-authority fact:
  TargetTraitEligibility or an approved field on TargetCollectionRealization naming required target traits for selected storage forms.

Existing worksheet:
  SG-5 covers Set/Map collection realization and BoundedLattice completeness (#4121)

Dispatch:
  SG-5 implementation may absorb the Set/BTreeSet eligibility subset.
  Non-collection trait bounds should wait for a post-SG-RC remeasure.
```

### Stable-B: Path/type-vs-variant projection

```text
Representative:
  E0573 expected type, found variant.

Single-authority fact:
  Same defining-module/path-role authority as SG-8. Type imports and variant imports must be separated by resolved item kind.

Dispatch:
  Attach to SG-8 implementation (#4127, §8 ratified #4143). Do not create a separate SG-3 worker.
```

### Stable-C: Record/variant constructor field admission

```text
Representative:
  E0560 missing struct field / variant field mismatch.

Single-authority fact:
  Constructor field set must come from the resolved record/variant declaration, including target-specific field omission/admission if any.

Required worksheet before implementation:
  A small constructor-field-admission worksheet if E0560 remains after SG-8 and SG-2. Current +4 delta vs #4122 suggests it is not the critical path.
```

### Stable-D: Placeholder and hollow alias bans

```text
Representative:
  E0121 `_` in item signature.
  E0391 recursive aliases such as type CppMachineWidth8 = CppMachineWidth8.

Single-authority fact:
  Type projection must be total or diagnostic before emission; hollow/self aliases are invalid per modeling discipline.

Dispatch:
  Fail-closed gates, not Rust syntax patches. If a type expression cannot be projected, emit a typed diagnostic/probe failure rather than `_` or a self-alias.
```

---

## Non-Goals

- Opening a broad SG-3 implementation lane.
- Treating `E0277` as "add derives/trait impls everywhere."
- Replacing `_` with guessed generic parameters.
- Adding recursive indirection to self-aliases without modeling the real carrier.

---

## Manager Decision

Stable bands are real residuals, but they are not the growth driver. Route `E0573`
to SG-8, collection/Ord `E0277` to SG-5, `E0369` to SG-RC, and hold
`E0560`/`E0121`/`E0391` as small fail-closed worksheets after the P0 fanout
lands or a worker proves an independent single-authority fact.
