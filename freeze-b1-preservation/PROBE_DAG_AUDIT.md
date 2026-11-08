# wc_verdict_probe.dag — exhaustive line-by-line syntax audit vs the PINNED parser at f08a6ae4
(Owner sleek-seal-226, #15695 lane. Required by tidy-ant-251 msg_949cd2a7 BEFORE freezing anything.
File audited: the FIXED probe file (6 match arms `->` -> `=>`), 127 lines, blob bb6ded7e..., seed commit bce90e1396d767093aee240c2e4a4623fba64113 (tree c911d73665d0549bfcc86398c53c83d2ba78b4bb, parent f08a6ae4, branch session/sleek-seal-226-wc-probe, pushed; one-file name-diff vs f08a6ae4 = the probe path).
Method: every construct is matched to (a) the pinned parser production in src/v1/02_parse.dag / token table src/v1/01_tokenize.dag, or (b) an in-tree .dag AT THE SAME PIN using the same construct in the same position. No local gunbc run was made.)

## The v4 defect, located exactly
Pinned file line 83 was `Absent -> "a=-"`. Parser production `parse_match_arm` (src/v1/02_parse.dag:6263+): after the pattern, `let r2 = expect(tokens: tokens, expected: ExpectFatArrow)` (~6279) — match arms REQUIRE `=>` (FatArrow); `->` is a different token (Arrow), and 01_tokenize.dag maps shapes explicitly (`"^": ShCaret` etc.). 83:12 is the column of that `->`. Same class of miss existed at 6 arms: lines 83, 91, 102, 103, 104, 112 — all now `=>`. In-tree match style at the pin, same shape as ours: wildcard_arm_resolve.dag:86-100 (`match outcome { NativeCensusModuleResolved => WcResolved ... }`).

## Line-by-line coverage

- **1** `module v2.test.claim.namespace_xl0.wc_verdict_probe` — same position/form as the frozen module line 1 (`module v2.test.claim.namespace_xl0.wildcard_arm_resolve`).
- **3-21** `//` comments — used pervasively at the pin (e.g. wildcard_arm_resolve.dag:29+ `// A WILDCARD MATCH ARM...`).
- **23-33** multiline `import <mod> { A, B, ... }` — identical style to frozen module lines 3-13 (`import v2.compiler.source_root_eval { ... }`). All nine imported names EXIST and are exported there: `type WcVerdict = WcResolved | WcRefused { reason: Symbol, at_atom: Optional<Symbol>, sole_chain: Bool } | WcFileRefused | WcAbsent` (lines 75-79), `type WcOutcomes = WcContextRefused | WcOutcomesDecided { arm: WcVerdict, field: WcVerdict, binder: WcVerdict, body: WcVerdict }` (81-83), and `fn wc_outcomes() -> WcOutcomes` (124).
- **34** `import std.optional { Absent, Optional, Present }` — declared dag/std/optional.dag:3-5 (`type Optional<T> = Absent | Present { value: T }`); frozen module line 21 imports the same module with the same names.
- **35** `import std.process { ExitFailure, ProcessExit }` — declared dag/std/process.dag:10,12 (`type ProcessExit ... | ExitFailure { code: Int, reason: String }`); in-tree: binder_atom_keying.dag:7 `import std.process { ExitFailure, ExitSuccess, ProcessExit }`.
- **36** `import v2.std.integer { Int }` — declared src/v2/std/integer.dag:19 (`type Int = GroupCompletion<Nat>`); in-tree: binder_atom_keying.dag:19.
- **37** `import std.integer { int_to_decimal_string }` — declared dag/std/integer.dag:316 (`fn int_to_decimal_string(value: Int) -> String`); in-tree importers exist (std.integer module used across src/v2/test/claim).
- **38** `import std.algebra { Cons, Empty, fold_list }` — frozen module line 23 is the identical line.
- **39** `import v2.std.logic { Bool }` — Bool declared in src/v2/std/logic.dag (BoolWidthFact etc. region; `Bool` used as a type across src/v2; the frozen module uses `Bool` in WcRefused at line 77 via its own logic import chain).
- **40** `import v2.std.node { Symbol, symbol_lexeme }` — `fn symbol_lexeme(sym: Symbol) -> String` at src/v2/std/node.dag:1580; called in-tree `symbol_lexeme(sym: emitted)` (if_arm_reader_differential.dag:271).
- **41** `import v2.std.text { String }` — src/v2/std/text.dag:7 `type String = FreeMonoid<Char>`; frozen module line 22 identical.
- **44-46** `fn char_count(s: String) -> Int { fold_list(xs: s, empty: 0, cons: fn(acc, _) { acc + 1 }) }` — fold_list signature dag/std/algebra.dag:1283 `fn fold_list<T, A>(xs: FreeMonoid<T>, empty: A, cons: fn(A, T) -> A) -> A`; `xs: s` with a String is EXACTLY sanctioned: algebra.dag:1262 comment "`v2.std.text` `String` = `FreeMonoid<Char>`" and in-tree `length(xs: s)` over a String (native_emission_controls.dag:726; infer_algebra_length_homonym_witness.dag:38 fixture `fn probe(s: String) -> Int { length(xs: s) }`). Untyped closure literal `fn(acc, _) { ... }`: in-tree `cons: fn(acc, r) { snoc(...) }` (a_field_read...dag:15) and `cons: fn(seen, s) { seen || s == name }` (wet_host_premise_readback_witness.dag:43). `acc + 1` int addition: activation_dependency.dag:114 `a + 1`.
- **50-76** `fn reason_token(r: Symbol) -> String { if r == ^sym { "r=0" } else if ... else { "r=X" } }` — symbol literal `^name`: token table 01_tokenize.dag:46 `"^": ShCaret`; symbol equality `==` in-tree (frozen module uses `== ^...` comparisons via the resolve reason symbols). The `if ... {} else if ... {} ... else {}` chain in return-expression position: in-tree at this pin in the FROZEN MODULE itself (its reason-token helpers, e.g. lines 55-73 of my audited file mirror wildcard_arm_resolve.dag's resolve-token chain) and machine_intake_disposition_witness.dag style. ALL 25 `^symbols` in the file (24 resolve reasons + `^wc_undeclared_body` + `^probe_pass`) verified present in-tree at f08a6ae4 by exhaustive git-grep census (25/25, none missing).
- **80-85** `fn atom_token(a: Optional<Symbol>) -> String { match a { Present { value: s } => if s == ^wc_undeclared_body { "a=U" } else { "a=?" }  Absent => "a=-" } }` — record pattern with binding `Present { value: s }`: production parse_pattern handles brace patterns; in-tree identical arm at if_arm_reader_differential.dag:274 `Present { value: name } => symbol_lexeme(sym: name)` and approval_decision_store.dag:279 `FieldString { value: s } => ...`; UNBRACED if/else as arm body: approval_decision_store.dag:279 (exact same shape) and activation_dependency.dag:255 `ComputationNode { behavior: Branch } => if want_match { optional_absent() } else { optional_present(value: n) }`. Parser: unbraced body goes to parse_expr (02_parse.dag:6311) which parses if-expressions via parse_prefix.
- **88-93** `match a { Present { value: s } => char_count(s: symbol_lexeme(sym: s))  Absent => 0 }` — same precedents as above; labeled call args `s:` / `sym:` match declared parameter names (char_count above; symbol_lexeme at node.dag:1580).
- **95-97** `fn bool_token(b: Bool) -> String { if b { "T" } else { "F" } }` — if/else with Bool scrutinee in return position: redfish_memory_inventory.dag:391 `if s == "" { none } else { Present { value: s } }` (same position class).
- **100-108** `fn verdict_text(v: WcVerdict) -> String { match v { WcResolved => "R"  WcAbsent => "A"  WcFileRefused => "F"  WcRefused { reason: r, at_atom: a, sole_chain: s } => "X{" + reason_token(r: r) + ... } } }` — simple-name arms and record arms with multiple bindings: frozen module 86-100 has both classes; `Cons { head: _, tail: _ }` wildcard fields at frozen 97-99 (so `_` fields parse). String concatenation `+`: binder_atom_keying.dag:148 `kind + " binder atom is not keyed DeclarationRole: " + row` and :48 `"projected#" + int_to_decimal_string(value: i.value) + "<-" + ...` (identical mixing of literals, calls, vars).
- **110-116** `fn outcomes_text(o: WcOutcomes) -> String { match o { WcContextRefused => "..."  WcOutcomesDecided { arm: arm, field: field, binder: binder, body: body } => "WCVERIFY ctx=OK arm=" + verdict_text(v: arm) + ... } }` — same productions as above; the field names `arm/field/binder/body` match the frozen declaration (wildcard_arm_resolve.dag:83) exactly; constructor-name shadowing as field name (`arm: arm`) is the same form the frozen module uses in `wc_outcomes()` body lines 128-132 (`arm: wc_verdict_in(...)` shows the label:name form; label==variable name appears e.g. `key: name, value: name` reference_site_collector.dag:255).
- **118** `data reason_bound: Int = 193` — in-tree at the pin: reference_conservation_census.dag:648-650 `data sample_rule_byte_cap: Int = 8000` (same top-level `data <lower>: Int = <literal>` form; frozen module line 26 `data live_tree_disposition: LiveTreeDisposition = SubstrateInputsOnly` gives the same file's local form).
- **120-127** `test fn wc_verdict_probe_reports() -> ProcessExit { let reason = outcomes_text(o: wc_outcomes())  if char_count(s: reason) > reason_bound { ExitFailure { code: 4, reason: ... } } else { ExitFailure { code: 3, reason: reason } } }` — `test fn ... -> ProcessExit`: binder_atom_keying.dag:152+ (`test fn ... -> ProcessExit`); `let`: everywhere incl. the parser file itself; `>` on Int: decimal.dag:244 `if i >= string_length(s: s)`; record-literal arm bodies `{ code: Int, reason: String }` (dag/std/process.dag:12) — in-tree `ExitFailure { code: 1, reason: kind + ... }` binder_atom_keying.dag:148. The if/else with record-literal branches in tail position: same-shape `if ... { Present { value: ... } } else { ... }` effect_grant.dag:253.

## Residual risks (PARSE-NEUTRAL, stated for completeness)
All constructs above parse by production or in-tree identity. What the parser does NOT check (claim_batch's entry parse at validate-functions time is parse-only; semantic refusal would surface at evaluation): (1) symbol DECLARATIONS for the 25 `^symbols` — verified present-in-tree by census (necessary, not proven sufficient for route-level resolve); (2) route reachability of `^wc_undeclared_body` from this import set (it is defined in the frozen module's fixture text and the frozen body claim already compares against it — the packet gate re-verifies blob aba6d2be); (3) type-level acceptance of String-as-FreeMonoid<T> in fold_list (precedent above; a refusal here prints PANICKED/other lines -> the packet's own gates exit 2 with the raw line preserved — no silent misread). None of these can produce a PARSE refusal like v4's; the classifier + gates still fail closed to NO_OBSERVATION.

## Disposition
- Fixed file = the one audited (blob bb6ded7e6533df101e3343dac604ba8dffebdcf4). Seed commit 64d1286293d639e2c5da75d492a514a5741b0c26 pushed to session/sleek-seal-226-wc-probe (parent f08a6ae4; one-file delta vs A preserved).
- A packet wc-probe-v5 will be built ONLY on your approval of this audit: v4 bytes with the pins re-pointed (PIN_P=bce90e1396d767093aee240c2e4a4623fba64113, TREE_P=c911d73665d0549bfcc86398c53c83d2ba78b4bb, probe blob bb6ded7e...), job name v5, NOTHING else changed. The frozen reading table, EXIT3 contract, classifier, caps, and all fixtures stay byte-identical.

## Addendum — seed 312fd6ddae2c (ctl_ok/ctl_out control fields; bound 218)

Parent msg_02bc6f7c B2 requires the control INSIDE the probe: ctl_ok = reason_token(^resolve_reason_unbound_symbol)
(declared value 0) and ctl_out = reason_token(^resolve_reason_binder_duplicate_in_frame) (a symbol OUTSIDE the
24 compared names; census item 1 of the 14 outside symbols, so its declared value is X). Both ride both row
shapes, computed unconditionally, as DATA — the probe asserts nothing about them (a claim must not hard-code a
comparison outcome or the control stops being a control).

Constructs used by the diff, each with a same-position precedent at the parent pin f08a6ae4:
1. ZERO-ARGUMENT FN `fn ctl_text() -> String {` — precedent: `fn rtp_outcomes() -> RtpOutcomes` at
   src/v2/test/claim/namespace_xl0/return_tail_position.dag:143 and this file's own `wc_verdict_probe_reports()`.
2. STRING CONCATENATION `+` across a call and literals — precedent: this file line 114 ("WCVERIFY ctx=OK arm=" + ...)
   and return_tail_position.dag throughout.
3. CARET SYMBOL AS NAMED CALL ARGUMENT `reason_token(r: ^resolve_reason_binder_duplicate_in_frame)` — precedent:
   the 24 `reason_token(r: ^resolve_reason_*)` call sites at lines 51-78 of this very file; return_tail_position.dag:363
   passes `atom: ^rtp_undeclared_guard_arm`, a symbol that is NEVER declared anywhere (the fixture's point),
   establishing that a caret symbol needs no declaration to parse or evaluate.
4. `data reason_bound: Int = 218` — same shape as the frozen 193 line; the value moved.
5. Comment text only elsewhere.

LENGTH RECOMPUTATION (verified by string construction, not by hand): per-field widest token
"X{r=23,a=?,s=F,al=9223372036854775807}" = 38 (the al glyph set is exactly "0".."9" per
dag/std/nat.dag:219-231 decimal_digit_glyph; a count can never be negative, so integer.dag's "-" branch is
unreachable). Head 15 + " ctl_ok=r=23"(12) + " ctl_out=r=23"(13) + " arm="+tok(43) + " field="+tok(45) +
" binder="+tok(46) + " body="+tok(44) = 218. RF row: 15 + 12 + 13 + " arm=-"(6)+" field=-"(8)+" binder=-"(9)+
" body=-"(7) = 70. BOUND_VIOLATED arm 28+19 = 47. data reason_bound = 218 (== widest OK row, so the guard
fires only on a genuine layout violation).

CHARSET AUDIT (addendum item 2, parent msg_32d414bd): every character of the reason line is produced by a
fixed literal in this file (ctx heads, separators, payload delimiters "{", ",", "}", "r="/"a="/"s="/"al=" tags,
"T"/"F", "R"/"A"/"F", "X", "-"), by reason_token/atom_token fixed literals, or by int_to_decimal_string over a
fold_list count (digits only, never negative). NO lexeme, file text or host string enters the reason;
symbol_lexeme's result is COUNTED and discarded (atom_len), never concatenated. A newline, ')' or any
brace/comma outside the fixed payload positions is therefore UNPRODUCIBLE by this probe. The validator and
reader still fail closed on such bytes (fixtures v7-esc-*, v7-conf-*).
