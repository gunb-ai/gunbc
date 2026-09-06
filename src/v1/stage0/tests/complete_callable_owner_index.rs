// THE 1c DISCRIMINATING RED: the callable owner index is COMPLETE BEFORE ANY BODY, and mutual
// reference is what makes that observable.
//
// The index used to be an accumulator extended inside the body-realization fold, so a module's
// callable parent answer was a function of how much of the population had been realized before it.
// Under the reference-derived recursion that is invisible for a one-way call -- a provider is
// realized before its consumer by construction -- so a one-way fixture would pass under BOTH the
// old and the new arrangement and prove nothing.
//
// MUTUAL REFERENCE IS THE STATE THAT DISCRIMINATES. Reference providers carry no acyclicity law,
// so `probe.a` calling `probe.b` while `probe.b` calls `probe.a` is legal and reachable. The
// recursion's `visiting` bound stops the second descent, so under incremental publication whichever
// module is realized first is realized against an index that does not yet contain the other, and
// exactly one direction of the pair fails to bind. A complete phase-1 index binds both.
//
// STATE OF THIS FILE, STATED BECAUSE A PASSING TEST THAT PROVES NOTHING IS WORSE THAN NO TEST.
//
// THE MUTATION HAS BEEN EXECUTED AND IT DID NOT GO RED. Restoring incremental publication --
// realize_module resolving against an index built from the modules realized SO FAR
// (callable_owner_index_from_envs over dep_state.module_index) instead of the complete phase-1
// index -- left all arms passing. So the mutual arm below does NOT currently discriminate the
// property it names, and it is NOT enrolled as evidence for the 1c deletion.
//
// What IS established, by execution:
//   - the harness reaches callee resolution: `a_call_to_a_nonexistent_callee_is_an_error` fails
//     the fixture that names a callee existing nowhere, so the diagnostic channel is not blind;
//   - the one-way arm binds a genuine cross-module call, so the qualified reference form in these
//     fixtures does resolve across modules.
// Both of those hold. What is NOT established is that a diagnostic is the observable through
// which an incomplete index becomes visible -- the mutation says it is not.
//
// The open question, and the next probe: an unresolved-but-existing callee may be handed to the
// builtin bridge rather than refused, in which case the loss is visible in the INFERRED call
// result rather than in the diagnostic list. Until that is settled this file stands as the
// harness plus two executed controls, not as the 1c red.

use std::path::{Path, PathBuf};
use std::rc::Rc;

fn scratch_root(tag: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("complete_owner_index_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("probe_root")).expect("scratch root");
    dir
}

fn author(dir: &Path, basename: &str, content: &str) {
    std::fs::write(dir.join("probe_root").join(basename), content).expect("write fixture");
}

/// Typecheck exactly the named fixtures as one closure and return every error diagnostic's
/// rendered message.
fn error_messages(dir: &Path, basenames: &[&str]) -> Vec<String> {
    let mut modules: Vec<Rc<v1_compiler::v1_std_core::Node>> = Vec::new();
    let mut indices = im::HashMap::new();
    let mut intern_table = v1_compiler::v1_std_core::empty_intern_table();
    for basename in basenames {
        let path = dir.join("probe_root").join(basename);
        let content = std::fs::read_to_string(&path).expect("fixture source");
        let key = basename.to_string();
        let tokens = v1_compiler::v1_compiler_tokenize::tokenize(content.clone(), key.clone());
        let index = v1_compiler::v1_std_core::build_newline_index(key.clone(), content);
        indices.insert(key, index);
        let parsed = v1_compiler::v1_compiler_parse::parse_with_table(
            tokens,
            Rc::new(indices.clone()),
            intern_table.clone(),
        );
        intern_table = parsed.intern_table.clone();
        modules.push(
            parsed
                .result
                .module
                .as_ref()
                .expect("fixture must parse")
                .clone(),
        );
    }
    let source_indices = Rc::new(indices);
    let graph = v1_compiler::v1_compiler_resolve::resolve_modules(
        Rc::new(modules.into_iter().collect()),
        source_indices.clone(),
    );
    let typed = v1_compiler::v1_compiler_infer::typecheck(graph, source_indices, intern_table);
    typed
        .diagnostics
        .iter()
        .filter(|d| v1_compiler::v1_std_core::is_error_diagnostic(d.diagnostic.clone()))
        .map(|d| format!("{:?}", d.diagnostic))
        .collect()
}

const MODULE_A: &str = "module probe.a\n\nfn af(x: Int) -> Int {\n  probe.b.bf(x: x)\n}\n";
const MODULE_B: &str = "module probe.b\n\nfn bf(x: Int) -> Int {\n  x\n}\n\nfn bg(x: Int) -> Int {\n  probe.a.af(x: x)\n}\n";

// POSITIVE CONTROL. A one-way call binds under any index construction, complete or incremental.
// If this arm ever goes red the fixture is broken rather than the property, and the mutual arm
// below would be reporting the breakage instead of the phase split.
#[test]
fn one_way_cross_module_call_binds() {
    let dir = scratch_root("oneway");
    author(&dir, "a.dag", MODULE_A);
    author(
        &dir,
        "b.dag",
        "module probe.b\n\nfn bf(x: Int) -> Int {\n  x\n}\n",
    );
    let errors = error_messages(&dir, &["a.dag", "b.dag"]);
    assert!(
        errors.is_empty(),
        "a one-way cross-module call must bind: {errors:?}"
    );
}

// BLINDNESS CONTROL, ADDED AFTER THE MUTATION FAILED TO GO RED. If a call to a function that
// does not exist anywhere produces no error diagnostic, then the diagnostic channel cannot see
// callee resolution at all and every assertion in this file over `errors.is_empty()` is vacuous.
// This arm exists to decide that, and it is EXPECTED TO FAIL while the channel is blind.
#[test]
fn a_call_to_a_nonexistent_callee_is_an_error() {
    let dir = scratch_root("absent");
    author(
        &dir,
        "a.dag",
        "module probe.a\n\nfn af(x: Int) -> Int {\n  probe.b.no_such_function(x: x)\n}\n",
    );
    author(
        &dir,
        "b.dag",
        "module probe.b\n\nfn bf(x: Int) -> Int {\n  x\n}\n",
    );
    let errors = error_messages(&dir, &["a.dag", "b.dag"]);
    assert!(
        !errors.is_empty(),
        "BLIND CHANNEL: a call to a callee that exists nowhere produced no error diagnostic, so \
         this file's `errors.is_empty()` assertions measure nothing"
    );
}

// THE DISCRIMINATING ARM.
#[test]
fn mutually_referencing_modules_both_bind() {
    let dir = scratch_root("mutual");
    author(&dir, "a.dag", MODULE_A);
    author(&dir, "b.dag", MODULE_B);
    let errors = error_messages(&dir, &["a.dag", "b.dag"]);
    assert!(
        errors.is_empty(),
        "both directions of a mutually-referencing pair must bind: the callable owner index is \
         built over the whole population before any body is realized, so neither module's answer \
         depends on which of the two was realized first. A failure here means the index is being \
         published incrementally during body realization again: {errors:?}"
    );
}
