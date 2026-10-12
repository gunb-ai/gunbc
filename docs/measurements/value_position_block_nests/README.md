# Value-position block nests

Synthetic fixtures for the cost shape fixed in `body_lower_value_lowered` (recurring failure mode
`a_value_position_block_re_lowered_through_the_walker`). Each is one module whose function nests `d`
match arms whose bodies are `{ }` blocks: `single_dN` has a sole nested match per block,
`multi_dN` has a `let` statement before it.

Re-derive a timing with the emitted compiler built from the revision under test (the binary that
`gunbc test //gunbc/instruments:v2-native-census` emits):

    time <emitted-compiler> census-infer docs/measurements/value_position_block_nests/single_d14/dag

Run it at main and at the head. Before the fix wall time is exponential in `d` (3^d); after, flat.
These are a measurement, not an enrolled control: no harness can bound a native run yet (see the
row's next-rung trigger).
