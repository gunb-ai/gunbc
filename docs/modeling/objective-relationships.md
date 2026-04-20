### Objective relationships

The compositional stacking between types must itself be factual:

```
logic.dag:       Classical = True | False           ← bivalent logic (math)
    ↓
bit.dag:         Bit = Classical where width(1)     ← definitional
                 Byte = List<Bit> where length(8)   ← IEC 80000-13
    ↓
integer.dag:     Int64 = Word64 where signed        ← two's complement
    ↓
float.dag:       Float64 = Word64 where ieee754     ← IEEE 754
    ↓
string_type.dag: String = { bytes, encoding }       ← definitional
                 Char = Int where range(0, 1114111)  ← Unicode scalar range
    ↓
unicode.dag:     block ranges from Unicode Standard  ← Unicode 15.0
```

Each relationship is a fact, not a design choice. "A byte IS 8 bits"
is IEC 80000-13. "IEEE 754 binary64 IS a 64-bit word" is the spec.
The relationship itself is non-controversial.

Cross-domain relationships follow the same rule. GitHub has a branching
concept that IS Git's branching model — that's documented in GitHub's
own docs. So `github.dag` should reference types from `git.dag` where
the relationship is real. The test: can you cite the documentation that
establishes the relationship?

