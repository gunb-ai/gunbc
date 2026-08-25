# Raw rustc streams: namespace-cut emitted v1 seed

Transient measurement artifacts for bright-wren-428's attribution pass. Delete
once attributed; this branch is not for merge.

| file | tree | result |
|---|---|---|
| `cand.err.gz` | integration/namespace-cut @9bbd88a0bf, candidate overlaid (dir replaced) | 1924 errors |
| `cutcand.err.gz` | same, candidate overlaid preserving `src/bin` | 1924 errors |
| `ctrl.err.gz` | 1238d68d014, its OWN committed cut-vintage seed | 1816 errors |

Control, measured with the identical harness: main @05c627fd60 with its own
regen candidate overlaid builds CLEAN. So the failure is introduced by the
branch, not inherited from main.

## The wrong-bind question, answered from these streams

Of 1539 qualified `expected .. / found ..` note pairs in `cutcand.err`,
**0 have differing module paths**; 1539 have the same or no module path.
rustc never names two module paths. So the misbind thesis explains none of
these errors and the emitter's Rc/sharing analysis owns them.
