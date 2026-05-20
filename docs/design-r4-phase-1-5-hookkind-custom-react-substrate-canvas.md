# R4 Phase‑1.5 — `HookKind` **Custom** arm (React substrate) — Practice‑4 promotion canvas

**Authority:** `docs/design-r4-full-stack-omni-emission-canvas.md` §10 (R4‑Phase‑1.5), §11 anti‑pattern #8, and the `HookKind` sketch in §4 Q2.

**Substrate home:** `src/v4/extdeps/frameworks/react.dag` — `ReactHookSite` coproduct arm `CustomHook { implementation_ref: … }`.

## 1. Problem

`Custom` is the user‑input boundary on the closed built‑in roster. Lenses and emitters must not invent ad‑hoc parallel carriers for “custom hook shape” without a ratified promotion plan (INVARIANTS **P2** single authority; **P1** faithfulness to the React surface).

## 2. Default posture (Phase‑2 until proven otherwise)

Treat **every** custom hook call site as **opaque at this layer**: only `implementation_ref` (declaration edge to the hook function body) is modeled. Any classification (“this custom hook is really an effect bundle”, “memoizing”, etc.) is **derived in consumers** from lowered facts inside that declaration — **not** a second discriminated type alongside `ReactHookSite`.

### 2.1 §12 YELLOW vs this canvas

`ReactHookSite` (including `CustomHook`) remains **🟡 YELLOW** Practice‑4 per `docs/design-r4-full-stack-omni-emission-canvas.md` §12 — this Phase‑1.5 canvas **does not** “close” that carrier to 🟢. It only ratifies **how** `CustomHook` may gain fields **without** parallel lifecycle sums. **Dissolution** of the built‑in roster + Custom boundary still requires the **semver / hooks‑index** edit path on the substrate PR.

**Single authority with Practice 9:** the dissolution record for the pinned Hooks-index call-shape substrate lives as inline `// 🟡` marks on the relevant `ReactHookSite` declarations in `src/v4/extdeps/frameworks/react.dag` (the whole sum **🟡** per §12); the paired discipline / Practice‑9 rules live in `docs/modeling-discipline.md` and the T-4.7 bullets of `src/v4/TASKS.md`. No standalone `DECISIONS.md` ledger path is reintroduced.

## 3. Promotion trigger (Practice‑4)

If a **concrete** emit or lens pipeline needs to branch on structural classes of custom hooks **before** lowering to `Node`, open a follow‑up that **extends** the `CustomHook` arm with additional fields (or a nested coproduct owned in this same file) in the **same PR** as the consumer proof — never a parallel `CustomHookEffect` / lifecycle sum. Effect **lifecycle** facts remain on the three effect built‑in arms only (`UseEffect` / `UseLayoutEffect` / `UseInsertionEffect` payload shape per design‑r4 §4).

## 4. Receipt

This canvas is the **named Phase‑1.5 artifact** required before treating `CustomHook` as anything other than opaque in downstream generated code. Landed alongside the `ReactHookSite` substrate in PRs that introduce or materially widen the Custom arm.
