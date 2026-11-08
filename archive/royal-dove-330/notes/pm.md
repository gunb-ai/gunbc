## walk-cost model, per-member decomposition: read-only receipts plus source, nothing executed

Append-only, third record in this lane. References 6707290503 (the model) and 6707319965 (the hierarchy correction); both left unedited. Answers the brief's "what does the parse-population walk do per member?". Nothing was compiled, claimed, tested, regened, run, dispatched, pushed or landed by this session: the inputs are retained CI logs, the committed workflow YAML, and static reading of the source, plus `find`/`wc` on the working tree for corpus size.

### Corpus, measured at my base tree `ed007978dc2a`

10,100 `.dag` files - 8,201 under `dag/` and 1,899 under `src/v2/` - totalling 144,822,130 B. Mean 14,338 B, median 6,572 B, largest 1,542,406 B. This is the validation population (reported as 10,106 bound modules including the entries the walk names). Sizes are from my base tree, not from the subject tree of each run quoted below, so treat them as approximate for those runs.

### What one member does, from source

`population_parse_member` (`src/v2/cli/population_verbs.dag:486`) locates the member in the layout; `population_parse_member_decided` (`:471`) then does, in order:
1. `module_file_key(file:)` - the `v2:<relative path>` key;
2. `content_hash_atom(value: member.content)` - a digest over the member's full text, always, whether or not the verdict will be reused;
3. `population_parse_standing` (`:415`) - the per-member standing decision, keyed on the recomputed digest plus the head/base parse-compiler identity;
4. `population_parse_counted` (`:438`);
5. only when standing is not `Reused`, `acquisition_parse` (`src/v2/compiler/module_acquisition.dag:909`), which is `tokenize_prepared` against the shared prepared lexer `env.lex` and then `parse_module_prepared` against the shared prepared grammar `env.grammar` (`bind_outcome` at `:872-877`);
6. on refusal only, `acquisition_refusal_at` runs `line_starts` over the content to render the `<line>:<column>` locus.

The parse tree is discarded - the streaming verdict carries the cause and never the tree, as the `MemberParse` type comment at `:892-896` states. Both routes execute the same emitted native binary (`${{ runner.temp }}/v2-native-cli-gen-one parse-population`, `witnesses.yml:477` for the PR route with `--reuse-from .`, `:483` for the push route with `--write-identity`/`--receipts`), so there is no interpreted-versus-native route asymmetry to reach for.

### The decomposition, and how it was obtained

LABEL FIRST, because it matters: these per-member figures are **derived by subtraction across different runs, at two ends of an observed range. There is no per-member receipt.** They are aggregate rates divided by member counts, not measurements of a single member, and the two ends come from separate runs with different subjects.

- **Hit end - no per-member parsing.** Four retained hit walks reused 10,101 / 10,102 / 10,102 / 10,087 members in 5 s, 6 s, 6 s and 16 s. Every member is still located, read from disk and content-hashed on that path, because the digest is what standing keys on. So the entire fixed cost of the walk - 10,100 files, 144.8 MB of read plus hash, plus all tally and receipt work - is 5-16 s: about **0.5-1.6 ms per member**, i.e. 9-29 MB/s of read-plus-hash.
- **Miss end - full parsing.** Run 42339827972 took 1,366 s for 10,094 counted members with `reused=0 miss_identity_mismatch=10094`: about **135 ms per member**, about **106 KB/s** of source through tokenize plus parse.
- **Subtracting the two ends**, the marginal cost of actually parsing is roughly 1,359 s of the 1,366 s. So **at least 99% of a full walk is `acquisition_parsed`**, and the whole tally-and-receipt machinery - standing, counted, refusals, the receipt Map, the fold - is under 1% of it.

### The limits of that claim, kept deliberately

- **The ~106 KB/s parse rate is UNEXPLAINED.** The walk step emits no phase rows at all: in the retained log for 42339827972 the interval 05:55:1x to 06:18:1x contains 419 lines and **zero** `[pre-entry]` rows - the only output is the single `parse-verdict reuse:` counter line at 06:18:02.969Z. Attribution for the walk is as thin as it is on the emit side, so any cause offered for that rate - interning, allocation, Map building, an interpreter boundary - would be unmeasured. I am not offering one.
- **The only thing ruled out from source is a per-member lexer or grammar rebuild**: `env.lex` and `env.grammar` are prepared once per walk and passed into each member call.
- The cross-step overlap I can see - the walk re-tokenizing modules the emit steps' compile transaction also tokenized (`pool_source_tokenize`, 14.3 s recorded) - is about 1% of one walk and is not worth a change.
- Therefore there is **no duplicated work inside the walk worth attacking**. The per-member parse is the required semantics, not overhead: the population must be shown to parse. It can be made cheaper, not removed, and the reuse route removes the *need* for it on members whose verdict already stands - which is the entire measured prize: 5-16 s against 22m46s.

Status unchanged and deliberately so: Change 1 FINAL (`e3ba345342f0046711b904adc71ab2964de3a3d2`, tree `8dedb58c302b982654b5c6f4fbd0acdb31dd879a`) is reviewed SOURCE READY and stays local and unpushed; no PR; the claims have never run; real-producer inhabitance and same-subject cold-versus-reused equality are OPEN; `witnesses.yml` is ungenerated; generation, execution, push and LAND are withheld by keen-otter.
