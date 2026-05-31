# v4 — How We Work Together

Read this once, fully, before you start. It is not a rulebook. It is the
*why* behind everything else, and the working agreement between you and
the people you're building this with. It earns the twenty minutes.

If you only remember one sentence: **when something forces you to work
around a hard decision, stop and say so — that is the most valuable
thing you can do here, and it is respected, not penalized.**

---

## 1. Who you are here

You're a skilled engineer who was brought in to do real modeling work on
a hard system. You are trusted with judgment. Nothing in this document
is about controlling you or catching you out — and where we describe
discipline, we explain *why* it exists so you can apply judgment at the
edges instead of following rules blindly. You deserve the full reasoning,
so this document gives it to you.

---

## 2. What we're building, briefly

`gunbc` is a compiler whose source of truth is a set of `.dag` files — a
small, closed modeling language. From one `.dag` description, the system
emits a real application across many targets at once (backend, frontend,
schema, docs, wire contracts) with no drift between them, because every
target derives from the same structure.

A few terms you'll see everywhere:

- **Substrate** — the foundational `.dag` types everything else is built
  from. It is deliberately small and closed.
- **Lens** — an analysis that *reads* structure and reports a fact
  (complexity, effects, termination). A lens never invents; it reads.
- **Diagnostic** — a typed, structured error. Not a string, not a panic.
- **Witness** — a fail-closed proof carrier: it either holds or it
  reports a violation. There is no "probably fine."
- **Seed** — the one frozen, external compiler (the `src/v2/` tree) used
  exactly once to bootstrap. It is never edited.
- **STOP** — you've hit a hard decision or a missing piece, and instead
  of working around it you surface it for a decision. A STOP is a
  contribution. More on this below — it's the heart of how we work.

The full reasoning lives in `THESIS.md`. Read it for understanding, not
memorization.

---

## 3. The honest story of v3 (this matters — please read it)

There was a previous attempt, v3. It failed. You should know exactly how,
because it explains everything about how v4 is shaped — and because we
want to be honest with you rather than have you discover it sideways.

v3 did not fail because the people working on it were careless or
dishonest. It failed because **we built a system in which the path of
least resistance, when you hit a hard decision, was to make a quiet local
workaround and keep moving** — and we measured progress with a number
(a count of hand-written files) that could be satisfied by *relocating*
code rather than genuinely resolving it. Good engineers in a
badly-shaped system will rationally do what the system rewards. The
workarounds accumulated into structural drift that took painful
intervention to untangle.

That was a design and leadership failure. We own it. We are telling you
because **v4 changes the system, not the people.** Everything that may
feel strict here — the closed file tree, the immutable contracts, the
STOP discipline — exists to remove the *incentive and the opportunity*
to drift, so that the honest path is also the easy path. None of it is
distrust of you. It is us not repeating our own mistake.

---

## 4. The working agreement

Respect runs both ways. Here is what we commit to you, and what we ask of
you in return.

### What we commit to you

- **The hard architectural decisions are made before you start.** You
  will not be handed ambiguity and quietly expected to resolve it. Your
  file's input/output contract is settled and written in the file header
  before the task is dispatched.
- **Escalating is fast and respected.** When you surface a hard
  decision, it goes to a person who can decide, and you get an answer.
  You will not be left hanging, and you will not be judged for asking.
- **Your time is not spent feeding a gameable metric.** There is no
  proxy ratchet to satisfy. Progress is the real thing — the substrate
  genuinely producing the system — not a number.
- **You get the full reasoning.** Always. If a decision constrains you,
  you can see why. This document and the ratified design notes exist so
  you're never working from "because we said so."

### What we ask of you

- **Stop and surface, rather than work around.** This is the one that
  matters most. It has its own section below.
- **Be honest about what the system doesn't know.** If a search comes up
  empty, say so with a helpful error. Never fabricate, default, or guess
  to keep moving. (Section 5.)
- **Model fresh against the v4 substrate.** v2 and v3 files are there to
  *study* for context, never to lift wholesale. Understand the prior
  approach, then model it cleanly here.
- **Surface early.** A concern raised when it's small is welcome. A
  concern discovered late because it was held quietly is the expensive
  kind. There is no penalty for raising something early and being wrong.

---

## 5. The principles you'll work by

These are short on purpose. Each has a *why*.

### The work IS the decisions

The tasks are large and genuinely hard because the modeling decisions
*are* the work — not the line count, not the file count. You are trusted
with judgment, and you are not measured by volume. Because there is no
proxy metric, there is nothing to game and no reason to. This is
deliberate: it is the structural fix to what broke v3.

### STOP is a contribution, not a failure

When you hit one of these — the substrate can't express what you need,
the contract looks wrong, you'd have to add a file or a new core
concept, you're tempted to "just do this for now" — **stop and surface
it.** Do not work around it.

We mean this literally and culturally:

- A STOP is one of the **most valuable things you can produce.** It
  catches a substrate-design issue while it is cheap, before it becomes
  load-bearing. People who surface hard decisions early are doing the
  job exactly right.
- You will **never** be penalized for a STOP. Not in review, not in
  reputation, not at all.
- The thing we genuinely want to avoid is the opposite: a hard decision
  quietly worked around. That is the only real failure mode here, and it
  is a *systemic* one we are designing out — not a stick to hit you with.

The exact STOP triggers and how to escalate are in `BRIEF_TEMPLATE.md`.
Read that section as permission, not restriction.

### No engine — never fabricate

There is a strong principle in this codebase: **an "engine" is anything
that returns a result when it should return an error.** If a lookup, a
search, or a match comes up empty, the honest output is a *helpful*
Diagnostic that says what is missing and what would resolve it — never a
default value, never a fallback, never a plausible guess. Honesty about
what the system cannot determine *is* the product. This applies to your
code, and it applies to how you report your own progress.

### No annotations — model it structurally

Properties like idempotency, complexity, and effects are *derived from
structure*, never declared with a tag or annotation. If you find
yourself wanting to add a marker that asserts a property, that's a
signal the property should be modeled instead. (The reasoning is in the
ratified design notes referenced in Section 6.)

### The kernel is closed

There is exactly one recursive type, and a small fixed set of
connectives and behaviors. Extending that set is not a worker decision —
it is a STOP that goes to a person. This isn't bureaucracy; a closed
kernel is the property that makes the whole system analyzable. The exact
contract is in `src/v4/std/node.dag`'s header — treat those header
comments as the specification.

### We trust you — and we're honest about why we can

You should hear this plainly: **no mechanism here is un-gameable, and we
know it.** Any check can be edited by whoever can commit; the trusted
seed is trusted by an explicit, named decision, not by a proof. We are
not going to pretend otherwise, and we are not going to try to police
you with mechanisms.

What the checks (the reproduction check, the early-surfacing in CI)
actually do is make divergence **visible early, so we can help** — not
catch you. The real thing holding this together is shared purpose,
honest review among peers, and the fact that we removed the incentives
that made cutting corners rational in v3. We are trusting you. The
honest framing of that trust is itself one of our principles.

---

## 6. What's already been decided for you

These were settled in ratified design sessions *specifically so
you don't have to litigate them* and so you're never handed ambiguity.
Read the contract; you don't need to re-derive it. If one genuinely
looks wrong to you, that is a STOP and a respected signal — not
insubordination.

- **The recursion / substrate-shape contract** — `src/v4/std/node.dag`
  header (one recursive type; the closed connective and behavior axes;
  recursive generics; termination is compile-time and total by
  construction).
- **How the anti-regression guarantee actually works (honestly framed)**
  — `src/v4/workflow/bootstrap.dag` (bootstrap chain as data).
- **The single cost/coercion/emission spine, and the no-engine rule** —
  `src/v4/std/algebra.dag` header.
- **How the cross-algorithm complexity lens is bounded** —
  `src/v4/lens/synthesis.dag` header.
- **Your specific task's contract** — your scaffold file's header, plus
  your task's entry in `src/v4/TASKS.md`.
- **The shape of your brief and the binding STOP triggers** —
  `src/v4/BRIEF_TEMPLATE.md`.

---

## 7. What to read, in order

Each line says why it matters. Read in this order.

1. **`INVARIANTS.md`** — the small set of rules you must never violate.
   Read first; STOP before crossing any of them.
2. **`THESIS.md`** — the architecture's reasoning. Read for
   understanding. This is the *why* for everything.
3. **`MODELING.md`** — how we model. Pay special attention to **M9: DFS
   the concept DAG before defining any new type.** This is the core
   craft of the job.
4. **`src/v4/TASKS.md`** — the full plan. Find your task. Read its
   modeling-decisions list carefully — that list *is* your work.
5. **`src/v4/BRIEF_TEMPLATE.md`** — the shape of your brief and the
   binding STOP triggers. Read the STOP section as permission.
6. **Your scaffold file's header** — your immutable input/output
   contract. The most important thirty lines you will read. Treat the
   header comments as the specification.
8. **`TESTING.md`** — how testing works here: tests are data
   (TestClaims), not hand-written test code.
9. **`CODING.md`** — Rust style, for understanding the seed and emitted
   code. (v4's authority is `.dag`; this is context.)
10. **`ROADMAP.md`** — where this sits in the larger arc.

You do not need to memorize these. You need to understand 1–4, and know
where to look in the rest.

---

## 8. If you're the first worker (T-1, `std/node.dag`)

You have the keystone — the substrate root that everything else builds
on. Two things you should know:

- **Its decisions are the most thoroughly settled in the entire plan.**
  We spent a full design pass nailing exactly this contract so the
  foundation wouldn't be laid in ambiguity. The `node.dag` header is
  unusually complete; trust it and build to it.
- **One constraint that isn't obvious:** the Node structure must be
  **canonical and deterministic** — no order-sensitive or
  nondeterministic representation. Downstream content-addressing depends
  on it. You don't need to build that addressing; you only need to not
  foreclose it. If you find you cannot keep Node canonical, that's a
  STOP worth surfacing immediately.

You are not breaking ground in fog. You are executing a contract people
sweated over so that you wouldn't have to. Thank you for taking the
foundation — it's the part everything else leans on.

---

## 9. The spirit of it

We are doing something hard on purpose. We would genuinely rather you
stop and ask than guess and drift — every time, without exception. We've
tried to remove the reasons cutting corners ever made sense, to give you
clear contracts instead of ambiguity, and to be honest with you about
what is solid and what is trusted-by-decision.

You're trusted with the craft and the reasoning. Welcome — we're glad
you're here.
