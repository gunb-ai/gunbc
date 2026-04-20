### Why Int and String are too wide

| "Primitive" | What it actually hides |
|---|---|
| `Int` | Bit width? Signed? Two's complement? Arbitrary precision? |
| `String` | Encoding? Null-terminated? Length-prefixed? Byte-level or code-point-level? |
| `Float` | IEEE 754 binary32? binary64? Decimal? Platform-dependent? |
| `Bool` | Unambiguous — this IS the primitive. |

Every compiler answers these questions differently. C's `int` is
platform-dependent. Rust's `i32` is exactly 32-bit two's complement.
Python's `int` is arbitrary precision. JavaScript's `number` is IEEE
754 double. The "primitive" is doing hidden work — implicit decisions
masquerading as axioms.

The fix: make the decisions explicit. Define Int as a composition with
a precise specification. The composition IS the definition. The
representation is the backend's job.

