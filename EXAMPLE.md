Same idea, but let's use **money / payment processing**, which has the exact same structure as the integer case: an exact internal amount, and external systems that each demand a fixed number of "minor units" (USD = 2 decimals/cents, JPY = 0, Bahraini dinar = 3). Sending money *out* to a processor can lose precision or be rejected; reading money *in* is always exact. Same total-one-way / fallible-the-other asymmetry as overflow — just "rounding" instead of "overflow."

---

## The task

Your ledger holds an **exact** amount. You hand it to systems that each have a fixed scale:
- internal ledger — arbitrary precision (a fraction of a cent is fine)
- Stripe/USD — integer **cents** (2 decimals), with a per-charge max
- a JPY processor — **whole yen** (0 decimals)

Ledger → processor is **fallible** (sub-cent amount must round or be rejected; can exceed the cap). Processor → ledger is **total** (always exact — the ledger has no scale limit, the way Python `int` has no width).

---

## A. The traditional way — objects + helpers

```python
class Money:
    def __init__(self, cents, currency): self.cents, self.currency = cents, currency

class Ledger:                                  # exact, arbitrary precision
    def __init__(self, amount): self.amount = amount        # e.g. Decimal/Fraction

CURRENCY_SCALE = {"USD": 2, "JPY": 0, "BHD": 3}            # minor-unit exponents
PROCESSOR_MAX  = {"stripe": 99_999_99, "jpy_gw": 9_999_999}

class Converter:
    def to_processor(self, ledger, currency, processor):    # exact → fixed scale
        scale = CURRENCY_SCALE[currency]
        scaled = ledger.amount * (10 ** scale)
        if scaled != scaled.to_integral_value():            # sub-unit residue → must decide
            scaled = self._round(scaled, currency)          # rounding policy lives HERE
        units = int(scaled)
        if units > PROCESSOR_MAX[processor]:                # cap check lives HERE
            raise AmountTooLarge(units, processor)
        return Money(units, currency)

    def to_ledger(self, money):                             # fixed scale → exact: always safe
        scale = CURRENCY_SCALE[money.currency]
        return Ledger(money.cents / (10 ** scale))          # widening, no guard

    def _round(self, scaled, currency):
        # banker's? half-up? truncate? differs per market/regulation → more branches
        ...
```

~45 LOC for *money, two processors*. The rule "exact→fixed can lose/reject, fixed→exact can't" is **hand-coded inside `to_processor`/`to_ledger`**; rounding policy is re-decided in every narrowing branch.

**Cost of change:**
- Add **BHD** (3 decimals, different rounding): touch `CURRENCY_SCALE`, `to_processor`, `_round`.
- Add **PayPal** (different cap, different rounding, maybe forbids JPY): a new column in every method → the conversion logic trends **O(currencies × processors)**.
- "The ledger is exact / can't lose precision" is asserted nowhere — it's *implied* by the absence of a guard in `to_ledger`. Nobody can point to the one place that says it.

---

## B. The daglang way — declare the scale, derive the conversion

Same discipline as the real `integer.dag` in the repo (`Int = GroupCompletion<Nat>`, `Int64 = Compose<Int, MachineWidth<Word64>>`, `OverflowDisposition`). Applied to money it reads:

```
# one abstract amount — UNBOUNDED scale, like Int is unbounded width
type Amount = GroupCompletion<Rational>

type Compose<carrier, dimension>                      # refine a carrier by an axis
type Money<Currency> = Compose<Amount, MinorUnitScale<Currency>>   # amount + a fixed scale

type RoundingDisposition = Banker | HalfUp | Truncate | Reject     # ↔ OverflowDisposition
```

```
# each currency / processor is pure DATA — no code
ledger     :  inhabits OrderedField, scale = unbounded            # ← the "exact" home, ONE place
usd_cents  :  Compose<Amount, MinorUnitScale<exponent: 2>>, rounding: Banker, max: 99_999_99
jpy_whole  :  Compose<Amount, MinorUnitScale<exponent: 0>>, rounding: HalfUp
bhd_fils   :  Compose<Amount, MinorUnitScale<exponent: 3>>, rounding: Banker
```

The conversion is then the **generic** resolution walk that exists once for all types:

```
resolve(Amount, ledger):     no scale refinement  → exact            [TOTAL]
resolve(Amount, usd_cents):  adds MinorUnitScale<2> → its predicate
                             "value divides into 10⁻²  AND  ≤ max"    [PARTIAL]
```

The asymmetry **is not a rule anyone wrote.** It's one structural fact: `usd_cents` *adds* a `MinorUnitScale` refinement; the ledger *doesn't*.

- Processor → ledger = **drop a refinement** → always inhabited → total, no guard.
- Ledger → processor = **add `MinorUnitScale<2>`** → its predicate can fail → `Rejected { Diagnostic{ amount_not_representable } }` (sub-cent), or `Rejected { amount_exceeds_max }`.

And the punchline transfers exactly: **adding two amounts is total** on the exact ledger (`amount_add : (Amount,Amount)->Amount`, no rounding). "We lost a penny on this total" is **not a property of `+`** — it's a property of *emitting the sum to a 2-decimal processor*. Localized to one boundary, named, typed — not smeared across the arithmetic.

**Cost of change:**
- Add **BHD / PayPal**: add a data row declaring its scale + rounding + cap. **Zero** edits to conversion code — the generic resolver already turns "carrier + scale refinement → representability predicate." Adding the Nth system is **O(1)**; there's no currency×processor matrix because nothing is written per pair.
- "The ledger is exact" has exactly one home: `ledger` carries no `MinorUnitScale`. Point at it.

---

## The one-liner for your friend

> **Traditional:** the rounding/cap rule is *procedure* — hand-written branches inside a converter, re-decided for every currency-and-processor pair, growing ~O(N²).
>
> **daglang:** the rule is a *consequence* — the ledger simply lacks the `MinorUnitScale` refinement that "cents" has. One generic resolver reads that structural difference and *derives* "exact one way, fallible the other." Adding a currency or processor is adding a data row, not code.

The deeper move in both the money and the integer version: **you never write "convert A → B."** You write what each thing *is* (which algebra it inhabits, which refinements it adds), and the translation is the model doing arithmetic on those refinements.

*(Honest footnote: the **integer/overflow** version is real code in the repo — `src/v2/std/integer.dag`. This **money** version is the same gunbc vocabulary — `Compose`, refinement predicates, `Outcome<T>`, a disposition enum — applied to business logic to illustrate; there isn't a `Money` model checked in. The structure is a faithful 1:1 with the integer one.)*

---

*(Background, unchanged: §1.8 #4701 CI watch still running on `40b28f5875`; I'll surface when `v2_lens_gate` resolves.)*
