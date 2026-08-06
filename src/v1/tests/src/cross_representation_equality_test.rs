use std::sync::Arc as Rc;

use v1_compiler::cli_run;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_interpreter::{self, ExecutionMode, InterpError, Value};
use v1_compiler::v1_std_core::CompilerDiagnostic;

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

// Every dag source in this file imports `std.occurrence_identity` even though no test
// references it directly: the closures all reach `v2.std.node`, whose occurrence facade
// references `std.occurrence_identity.*` by bare namespace path (no import), and this
// helper's pool is import-driven — the bare references bind only if something drags the
// module into the pool (the #6985 Class-B pool-membership shape, used here deliberately).
// Without it every ctx in this module fails resolution with "unresolved type
// 'std.occurrence_identity.…'" — which is how the whole module sat red, unnoticed
// because the rust suite is local-only (CI removed 2026-07-11).
const RECEIPTS_SOURCE: &str = r#"
module test.xrepr

import std.occurrence_identity
import v2.std.logic { Bool }
import v2.std.nat { Nat, Succ, Zero, nat_add }
import v2.std.algebra { Cons, Empty }

fn reconciles_nat_add_eq_int() -> Bool { nat_add(a: 85, b: 32) == 117 }
fn reconciles_nat_add_1_1_eq_2() -> Bool { nat_add(a: 1, b: 1) == 2 }
fn reconciles_nat_add_noncanon() -> Bool { nat_add(a: 85, b: 32) == nat_add(a: 100, b: 17) }
fn reconciles_list_element() -> Bool { [nat_add(a: 1, b: 1)] == [2] }
fn reconciles_succ_zero_eq_one() -> Bool { (Succ { prev: Zero }) == 1 }
fn reconciles_zero_eq_int() -> Bool { Zero == 0 }
fn reconciles_ne_path_false() -> Bool { nat_add(a: 1, b: 1) != 2 }

fn ok_native_int_eq() -> Bool { (85 + 32) == 117 }
fn ok_nat_add_reflexive() -> Bool { nat_add(a: 85, b: 32) == nat_add(a: 85, b: 32) }
fn ok_list_freemonoid_reconciled() -> Bool {
  [1, 2, 3] == Cons { head: 1, tail: Cons { head: 2, tail: Cons { head: 3, tail: Empty } } }
}

fn diff_int() -> Bool { 1 == 2 }
fn diff_succ_zero_vs_zero() -> Bool { (Succ { prev: Zero }) == Zero }
fn diff_nat_add() -> Bool { nat_add(a: 1, b: 1) == 3 }
fn diff_list_elements() -> Bool { [1, 2] == [1, 3] }
"#;

fn non_advisory_diagnostic_msgs(resolved: &ResolvedPipelineResult) -> Vec<String> {
    resolved
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        // `where-refinement unenforced:` is filtered for the same reason
        // `assert_content_hash_resolved` filters it: std.content_hash's refined hex
        // carriers entered every closure via v2.std.node, and those advisory rows are
        // not what these witnesses discriminate.
        .filter(|m| {
            !m.starts_with("complexity: ")
                && !m.starts_with("unlisted import use ")
                && !m.starts_with("where-refinement unenforced:")
        })
        .collect()
}

fn assert_resolved(resolved: &ResolvedPipelineResult) {
    let msgs = non_advisory_diagnostic_msgs(resolved);
    assert!(
        msgs.is_empty() && resolved.graph.is_some(),
        "receipts source should resolve cleanly, got {:?} (graph present: {})",
        msgs,
        resolved.graph.is_some(),
    );
}

fn with_receipts_ctx<R>(body: impl FnOnce(&v1_interpreter::InterpContext) -> R) -> R {
    let ws = workspace_root();
    let roots = [ws.join("src/v2"), ws.join("dag")];
    let sources =
        resolve_imports_transitively_with_source_roots("test.dag", RECEIPTS_SOURCE, &roots);
    let resolved = compile_to_resolved(Rc::new(sources.into()));
    assert_resolved(&resolved);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx =
        cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);
    body(&ctx)
}

#[test]
fn grounded_numeric_tower_reconciles() {
    with_receipts_ctx(|ctx| {
        for f in [
            "reconciles_nat_add_eq_int",
            "reconciles_nat_add_1_1_eq_2",
            "reconciles_nat_add_noncanon",
            "reconciles_list_element",
            "reconciles_succ_zero_eq_one",
            "reconciles_zero_eq_int",
            "ok_native_int_eq",
            "ok_nat_add_reflexive",
            "ok_list_freemonoid_reconciled",
        ] {
            match v1_interpreter::run_in_context(ctx, f, false) {
                Ok(Value::Bool(true)) => {}
                other => panic!(
                    "{f}: expected Bool(true) — grounding makes the native form equal the \
                     modeled form, so this reconciles; a straddle error or wrong bool here is \
                     the regression this witness guards; got {other:?}"
                ),
            }
        }
        match v1_interpreter::run_in_context(ctx, "reconciles_ne_path_false", false) {
            Ok(Value::Bool(false)) => {}
            other => panic!("reconciles_ne_path_false: expected Bool(false), got {other:?}"),
        }
    });
}

#[test]
fn genuine_inequalities_stay_false_not_errors() {
    with_receipts_ctx(|ctx| {
        for f in [
            "diff_int",
            "diff_succ_zero_vs_zero",
            "diff_nat_add",
            "diff_list_elements",
        ] {
            match v1_interpreter::run_in_context(ctx, f, false) {
                Ok(Value::Bool(false)) => {}
                other => panic!(
                    "{f}: expected Bool(false) — grounding reconciles representations without \
                     turning genuine differences into errors; got {other:?}"
                ),
            }
        }
    });
}

// Discriminating witness for the application-site contract wall.
//
// Before the wall, `call_function_inner` inserted caller argument labels into the call env
// without checking them against the callee's declared parameters, and dropped surplus
// positional args. A mismatched call therefore failed only if the body happened to read a
// parameter the caller had not supplied (surfacing later as `undefined variable: X`), and
// computed silently with wrong bindings when the names overlapped.
//
// The wall climbed at the direct-call seam (#7519, `direct_call_shape_diags`): unknown
// labels and surplus positionals now refuse at compile time on every module, including
// `v2.*`. `module_skips_direct_call_arg_check` still exempts `v2.*` from the argument
// TYPE judgment only — labels have no representation gap, so shape checking is not behind
// that exemption (`direct_call_shape_wall_note` in `v1.compiler.infer`).
//
// SCAFFOLD (DESIGN §7 HAND-RUST GATE — explicit deferral): the helpers below are
// seed-retained local-only witnesses in an existing `v1-compiler-tests` module (the rust
// suite is local-only per `gunbc.commit_workflow`, CI removed 2026-07-11). Host-Rust
// because post-#7519 mismatched call fixtures refuse at compile time and cannot inhabit an
// Accepted `.dag` claim row; they must be exercised via `compile_to_resolved` in this
// harness (same class as `compiler_tests::call_shape_wall_witness` and the pre-existing
// interpreter receipts in this file). Lane: compiler guarantee recovery — ROADMAP closing
// check `ct_call_shape_wall_witness_test` ("migrates to the probe corpus as this class's
// durable pair"; sibling of landed direct-call shape wall gunbc#7519).
// Dissolution: migrate these two witnesses into `gunbc.guarantee_probe_corpus` (or fold
// into `call_shape_wall_witness` once the `v2.*` module axis is covered there), then
// delete `compile_contract_source` / `call_shape_diagnostic_msgs` / this section in the
// same change.
const CALL_CONTRACT_BAD_SOURCE: &str = r#"
module v2.test.callcontract

import std.occurrence_identity
import v2.std.logic { Bool }

fn takes_tag(tag: Bool) -> Bool { tag }
fn takes_two(a: Bool, b: Bool) -> Bool { b }

fn bad_label() -> Bool { takes_tag(nope: true) }
fn surplus_positional() -> Bool { takes_tag(true, true) }
fn deficit_positional() -> Bool { takes_two(true) }
"#;

const CALL_CONTRACT_GOOD_SOURCE: &str = r#"
module v2.test.callcontract

import std.occurrence_identity
import v2.std.logic { Bool }

fn takes_tag(tag: Bool) -> Bool { tag }
fn takes_unused(_ignored: Bool, keep: Bool) -> Bool { keep }

fn ok_label() -> Bool { takes_tag(tag: true) }
fn ok_underscore_idiom() -> Bool { takes_unused(ignored: false, keep: true) }
"#;

fn compile_contract_source(source: &str) -> Rc<ResolvedPipelineResult> {
    let ws = workspace_root();
    let roots = [ws.join("src/v2"), ws.join("dag")];
    let sources = resolve_imports_transitively_with_source_roots("test.dag", source, &roots);
    compile_to_resolved(Rc::new(sources.into()))
}

fn call_shape_diagnostic_msgs(resolved: &ResolvedPipelineResult) -> Vec<String> {
    resolved
        .diagnostics
        .iter()
        .filter(|d| {
            matches!(
                *d.diagnostic,
                CompilerDiagnostic::CallArgumentNameUnknown { .. }
                    | CompilerDiagnostic::CallPositionalSurplus { .. }
                    | CompilerDiagnostic::CallPositionalDeficit { .. }
                    | CompilerDiagnostic::CallNamedArgOnFunctionValue { .. }
            )
        })
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}

fn assert_call_shape_diags_block(resolved: &ResolvedPipelineResult) {
    for d in resolved.diagnostics.iter().filter(|d| {
        matches!(
            *d.diagnostic,
            CompilerDiagnostic::CallArgumentNameUnknown { .. }
                | CompilerDiagnostic::CallPositionalSurplus { .. }
                | CompilerDiagnostic::CallPositionalDeficit { .. }
                | CompilerDiagnostic::CallNamedArgOnFunctionValue { .. }
        )
    }) {
        assert!(
            v1_compiler::v1_std_core::is_error_diagnostic(d.diagnostic.clone())
                && v1_compiler::v1_std_core::is_interpreter_blocking_diagnostic(
                    d.diagnostic.clone()
                ),
            "call-shape mismatch must BLOCK — a counted advisory would still emit the \
             silently-reordered realization, got {:?}",
            d.diagnostic
        );
    }
}

/// RED arm: a mismatched application site refuses at the compile seam, typed and located.
///
/// This is the arm that goes red if `direct_call_shape_diags` is removed or narrowed — without
/// it `bad_label` binds `nope` into the env at runtime, leaves `tag` unbound, and fails much
/// later (or not at all), and the emitter silently reorders mislabeled args positionally.
#[test]
fn application_site_contract_mismatch_refuses() {
    let resolved = compile_contract_source(CALL_CONTRACT_BAD_SOURCE);
    let shape_msgs = call_shape_diagnostic_msgs(&resolved);
    assert!(
        shape_msgs
            .iter()
            .any(|m| m.contains("takes_tag") && m.contains("nope")),
        "bad_label must refuse at compile time with an unknown-label diagnostic — the \
         interpreter already refuses this call at runtime, and the emitter silently reorders \
         mislabeled args positionally; got {shape_msgs:?}"
    );
    assert!(
        shape_msgs
            .iter()
            .any(|m| m.contains("takes_tag") && m.contains("too many positional")),
        "surplus_positional must refuse at compile time — the interpreter refuses the same \
         call (too many positional arguments); got {shape_msgs:?}"
    );
    assert!(
        shape_msgs
            .iter()
            .any(|m| m.contains("takes_two") && m.contains("missing required argument")),
        "deficit_positional must refuse at compile time — the interpreter refuses the same \
         call (missing required argument); got {shape_msgs:?}"
    );
    assert_call_shape_diags_block(&resolved);
}

/// GREEN control: valid call shapes compile with no call-shape diagnostic, INCLUDING the
/// corpus idiom where a deliberately-unused parameter is declared `_ignored` and labelled
/// `ignored` at the call site. Ignoring that idiom produced ~65 false positives; this arm
/// pins it. Runtime execution is retained as a second positive control on the accepted graph.
#[test]
fn valid_application_sites_are_unaffected() {
    let resolved = compile_contract_source(CALL_CONTRACT_GOOD_SOURCE);
    let shape_msgs = call_shape_diagnostic_msgs(&resolved);
    assert!(
        shape_msgs.is_empty(),
        "well-formed calls must not trip the call-shape wall — the underscore idiom \
         (`_ignored` declared, `ignored` supplied) is NOT a mismatch; got {shape_msgs:?}"
    );
    let msgs = non_advisory_diagnostic_msgs(&resolved);
    assert!(
        msgs.is_empty() && resolved.graph.is_some(),
        "good contract source should resolve cleanly, got {msgs:?} (graph present: {})",
        resolved.graph.is_some()
    );
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx =
        cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);
    for f in ["ok_label", "ok_underscore_idiom"] {
        match v1_interpreter::run_in_context(&ctx, f, false) {
            Ok(Value::Bool(true)) => {}
            other => panic!(
                "{f}: accepted call shapes must still evaluate to Bool(true) at runtime; \
                 got {other:?}"
            ),
        }
    }
}

const CONTENT_HASH_CROSS_FAMILY_SOURCE: &str = r#"
module test.contenthashxfamily

import std.occurrence_identity
import std.content_hash {
  ContentHash,
  Sha256Digest,
  Sha256DigestHex,
  content_hash_of_value,
  content_hash_atom,
  as_content_hash_cryptographic,
  content_hash_eq_structural,
}
import std.types { Bool }
import extdeps.transports.rest { RestAuthSensitiveIdentity, RestAuthenticated }

data structural: ContentHash = content_hash_of_value(value: "fp-a")

fn authed_identity() -> RestAuthSensitiveIdentity {
  RestAuthenticated {
    scheme: "BearerToken"
    digest: content_hash_atom(value: "BearerToken\0test-replay-token")
  }
}

fn other_token_identity() -> RestAuthSensitiveIdentity {
  RestAuthenticated {
    scheme: "BearerToken"
    digest: content_hash_atom(value: "BearerToken\0other-token")
  }
}
data sha256_row: ContentHash = as_content_hash_cryptographic(
  digest: Sha256Digest {
    hex: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855" as Sha256DigestHex,
  }
)

fn cross_family_eq() -> Bool { structural == sha256_row }
fn cross_family_ne() -> Bool { structural != sha256_row }

fn same_family_eq() -> Bool {
  match structural {
    Fnv1a64(a) =>
      match structural {
        Fnv1a64(b) => content_hash_eq_structural(left: a, right: b)
        Sha256Hash(_) => false
        Sha1Hash(_) => false
      }
    Sha256Hash(_) => false
    Sha1Hash(_) => false
  }
}
"#;

fn assert_content_hash_resolved(resolved: &ResolvedPipelineResult) {
    let msgs: Vec<String> = resolved
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| {
            !m.starts_with("complexity: ")
                && !m.starts_with("unlisted import use ")
                && !m.starts_with("where-refinement unenforced:")
        })
        .collect();
    assert!(
        msgs.is_empty() && resolved.graph.is_some(),
        "content-hash cross-family source should resolve cleanly, got {:?} (graph present: {})",
        msgs,
        resolved.graph.is_some(),
    );
}

fn with_content_hash_ctx<R>(body: impl FnOnce(&v1_interpreter::InterpContext) -> R) -> R {
    let ws = workspace_root();
    let roots = [ws.join("src/v2"), ws.join("dag")];
    let sources = resolve_imports_transitively_with_source_roots(
        "test.dag",
        CONTENT_HASH_CROSS_FAMILY_SOURCE,
        &roots,
    );
    let resolved = compile_to_resolved(Rc::new(sources.into()));
    assert_content_hash_resolved(&resolved);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx =
        cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);
    body(&ctx)
}

#[test]
fn cross_family_content_hash_bare_eq_refuses() {
    with_content_hash_ctx(|ctx| {
        for f in ["cross_family_eq", "cross_family_ne"] {
            match v1_interpreter::run_in_context(ctx, f, false) {
                Err(InterpError::CrossRepresentationEquality { .. }) => {}
                other => panic!(
                    "{f}: cross-family ContentHash bare `==`/`!=` must refuse with \
                     CrossRepresentationEquality, not fabricate false/true; got {other:?}"
                ),
            }
        }
        match v1_interpreter::run_in_context(ctx, "same_family_eq", false) {
            Ok(Value::Bool(true)) => {}
            other => panic!("same_family_eq: expected Bool(true), got {other:?}"),
        }
    });
}

/// Discriminating witness for the authenticated half of the REST replay identity seam
/// (gunbc#7648, second finding — review 47017 item 1 on #7650). The model types
/// `RestAuthenticated.digest` as `Fnv1a64Structural`; before this pin the seed minted
/// `(sym("digest"), Value::Str(<DefaultHasher hex>))` — a bare string OUTSIDE the fnv1a64
/// family, so no dag-authored authenticated fixture could ever match a runtime invocation
/// (the same silent-false fork as `input_digest`, one field over, invisible to the 9/9
/// unauthenticated replay witnesses).
///
/// RED under either half of the old mint: a `Value::Str` digest fails the shape equality;
/// a DefaultHasher digest fails the value equality against the modeled
/// `content_hash_atom("<scheme>\0<secret>")` chain. The inequality arm guards that the
/// digest actually discriminates secrets (a constant digest would pass the first assert
/// on the wrong grounds).
#[test]
fn rest_authenticated_identity_matches_dag_constructed_value() {
    with_content_hash_ctx(|ctx| {
        let dag_side = v1_interpreter::run_in_context(ctx, "authed_identity", false)
            .expect("authed_identity must evaluate");
        let seed_side =
            v1_interpreter::rest_authenticated_identity_for_witness("test-replay-token", ctx);
        assert_eq!(
            dag_side, seed_side,
            "seed authenticated-identity mint diverged from the dag-authored \
             RestAuthenticated (content_hash_atom digest); every authenticated replay \
             fixture would silently fail to match its runtime invocation"
        );
        let other_token = v1_interpreter::run_in_context(ctx, "other_token_identity", false)
            .expect("other_token_identity must evaluate");
        assert_ne!(
            other_token, seed_side,
            "identities for different secrets must differ — a constant digest would make \
             the equality arm above pass without discriminating credentials"
        );
    });
}

#[test]
fn cross_family_content_hash_homonym_variant_names_do_not_trigger_guard() {
    with_content_hash_ctx(|ctx| {
        let homonym_a = Value::Variant {
            type_name: ctx.sym("HomonymHash"),
            variant_name: ctx.sym("Fnv1a64"),
            fields: Rc::new(vec![]),
        };
        let homonym_b = Value::Variant {
            type_name: ctx.sym("HomonymHash"),
            variant_name: ctx.sym("Sha256Hash"),
            fields: Rc::new(vec![]),
        };
        assert!(
            v1_interpreter::cross_family_content_hash_straddle_for_witness(&homonym_a, &homonym_b)
                .is_none(),
            "unrelated coproduct with ContentHash family variant names must not trigger the guard"
        );
    });
}
