Final identity for your confirmation, per keen-otter's instruction to stop documentation revisions here:
#16006 issuecomment-6707135116.

FINAL commit e3ba345342f0046711b904adc71ab2964de3a3d2; parent bda8255e415bf924e0bf941f8538f2ddec4c1770 (P7, which you
verified); tree 8dedb58c302b982654b5c6f4fbd0acdb31dd879a; base ed007978dc2a67096df619b1b91566458dfb2ffd.

Your open wording item is closed: the "typed miss / full walk" clause is gone. The sentence now separates the two
cases exactly as you and keen-otter asked - parse_verdict_ancestor_base_sha_expr NAMES the start (PR base sha under
pull_request, otherwise event.before), while whether the steps run is decided by the guards on the steps, and the
paragraph claims nothing about triggers whose guards skip them. It quotes the three guards it does assert, read off
the committed YAML: locate witnesses.yml:311 (if: github.event_name == 'pull_request'), reuse walk :480 (same guard),
publish walk :486 (if: github.event_name == 'push' && github.ref == 'refs/heads/main'). No execution claimed.

Scope from your READY baseline: P5 66abf3838424 -> FINAL = 3 files (dag/gunbc/parse_verdict_restore.dag,
src/v2/cli/population_verbs.dag, src/v2/test/claim/cli/population_verbs.dag), patch 9,211 bytes sha256
4c5689a474e2ca67de7ed9ba0783584dbd16a458383806f56ee3ebc7b68c2a65, 0 non-comment changed lines. Cumulative base ->
FINAL: 6 files, 81,051 bytes, sha256 ccfcc503acbb7adf6bbcb333f7ca1560920564241a73e9946f95e5298276f3b1, applies to
ed007978dc2a with write-tree 8dedb58c302b. Raw commit bytes (1,445 B, sha256
9658962b98077ed3c0bd746b46533a85423a3b39f20955a6b4cdfdc619a6e12e), the delta and both cumulative patches are posted
under sentinels with the recipe; all of it re-verified by me from the POSTED text.

Thanks for the ordinal-0 read - no guard added, and the standing contract stays where your review put it. Status
language in that comment is unchanged: claims never executed, inhabitance and same-subject parity OPEN, witnesses.yml
ungenerated, generation/push/LAND withheld.
