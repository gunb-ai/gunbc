# First fresh-process shadow experiment

The first observation is `v1.std.core::build_newline_index` as consumed on ordinary `gunbc run`. The result contains the logical file name, newline offsets (in Unicode scalar positions) **and all source character codes**. An ASCII edit `1` → `2` leaves every newline offset fixed and still changes this result. This is a discriminating test of the proposed result identity.

## Predictions, before observations

Use one private workspace, the same compiler executable, two reached source files (entry and helper), and a fresh OS process for each invocation. Each edit starts from the named baseline. Compare the same logical file and compiler question across processes. Start with an empty private shadow directory. No production read of a saved result is permitted in this experiment.

| Condition | Entry newline index | Helper newline index | Command behavior |
| --- | --- | --- | --- |
| Empty directory | Miss, fresh | Miss, fresh | Executes the selected entry |
| Identical inputs | Hit, equal to fresh | Hit, equal to fresh | Same return and output |
| Edit only a file outside the entry's demands | Hit, equal | Hit, equal | Same return and output. No index observation for the unrelated file |
| Change the helper's literal `1` to `2` | Hit, equal | Miss, fresh output differs from baseline | Caller observes the new value, including with shadow disabled |
| Revert the helper | Hit, equal | Hit, equal to original | Original outcome |
| Start with a second empty store on the **same** host | Miss, fresh output equal to baseline | Miss, fresh output equal to baseline | Original outcome. This is the missing-record control, not a second-worker result |
| Damage the previously stored helper record | Hit, equal | Refuse that stored record | The ordinary compiler still supplies the fresh answer in shadow mode |

A hit with a different fresh result fails the experiment, regardless of the predicted outcome. A miss is a lookup fact. Calling its recomputation equal requires a comparable, independently produced baseline result. A compiler stage the request never reached has no hit/miss outcome. This experiment predicts no Rust emission on the interpreted `run` path.

The integration test owns the executable assertions. The final criterion for a shared store is a real second worker with an empty local store, admitted shared bytes, fresh execution of the checkout check, and a deliberately mismatched checkout refusal. Copying a directory on one host cannot establish that criterion. For a later serving change, compare ordinary outcomes to forced-cold outcomes and measure full wall time including lookup, verification and hydration. Shadow mode recomputes every product; producer calls avoided in this experiment are zero.
