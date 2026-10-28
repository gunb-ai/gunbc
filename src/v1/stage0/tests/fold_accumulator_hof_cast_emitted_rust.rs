// THE LAMBDA PARAMETER OF A GENERIC HOF FED BY AN EMPTY-INIT FOLD CARRIES THE ELEMENT TYPE, AND
// ITS CAST LOWERS — IT DOES NOT PANIC AT RUNTIME.
//
// One shape, two ways to arrive. Both specimens below call the same generic higher-order function
// (`std.list.distinct_by_key<T>(xs: List<T>, key: fn(T) -> String)`) with a lambda whose body casts
// its own parameter. They must emit the SAME key closure.
//
//   * `declared_shape`: the list is a parameter of the calling function, annotated `List<NonEmptyStr>`.
//     The element type is stated, so the HOF's T instantiates and the cast lowers. Green before and
//     after the fix — it is the reference.
//   * `fold_shape`: the list is built by `fold(names, init: [], f: (acc, n) => acc |> list_push(n as
//     NonEmptyStr))`. Nothing states the element type; it has to be RECOVERED from the accumulator's
//     update. Before the fix this one emitted `panic!("unsupported cast from T to String")` for the
//     key lambda — the runtime symptom of a cast checked against a node that carried the CALLEE'S OWN
//     type-parameter NAME in place of a type. The name came from a record-literal field whose expected
//     type was fabricated by substituting `type_variable_node(id: <param name>)` for the owner's
//     uninstantiated parameter (v1.compiler.infer, the unbound-owner-param arm): manufactured
//     evidence, and the unifier's first-wins binding then credited T := T because the candidate matched
//     its own fabricated name.
//
// ASSERTING THE TYPE, NOT JUST THE ABSENCE OF A PANIC. A missing panic string is weak evidence — an
// unrelated change can silence it (measured: at a parallel cast-wall head the emitted store shows no
// panic while the inference defect is fully intact, merely masked). So the assertion is differential
// and structural: the fold-shape key closure must be the declared-shape key closure, which is the
// emission a compiler can only produce when it has established `NonEmptyStr` as the element type on
// both paths. The second pair does the same for `list_push(xs: [], item: <typed>)` typed into a
// `List<NonEmptyStr>` consumer, and pins the wrapper control (`fn push_one<U>(xs: List<U>, item: U)`)
// so the rule cannot be over-broad: a CALLER's own type parameter is evidence, the CALLEE's own
// unsubstituted formal is not, discriminated by binder identity rather than by spelling.
//
// REAL NATIVE EMIT PATH: both specimens go through compile_sources(..., RenderTarget::Rust) — the
// same entry the shipped CLI drives — and the emitted bytes are what is judged. The Rust integration
// targets under src/v1/stage0/tests are the gate that runs this file; it is enrolled as a retained
// harness in v2.self_host.seed_retention_frontier's retained_test_harness list, which is what the
// v2-bootstrap CI job executes (required CI does not otherwise run Rust integration tests).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use v1_compiler::v1_compiler_compile::{compile_sources, RenderTarget, SourceFile};

/// The fold-shape specimen: the element type must be recovered from the accumulator's update.
const FOLD_SHAPE: &str = r#"module probe_fold_shape

import std.types { String, Int, Bool, List, NonEmptyStr }
import std.list { distinct_by_key }
import std.algebra { list_push }

fn probe_case(names: List<String>) -> List<NonEmptyStr> {
  let keys = fold(names, init: [], f: (acc, n) => acc |> list_push(item: n as NonEmptyStr))
  distinct_by_key(xs: keys, key: fn(k) { k as String })
}
"#;

/// The declared-shape reference: the same call, with the element type stated. Green at HEAD; its
/// emission is what the fold shape has to match.
const DECLARED_SHAPE: &str = r#"module probe_declared_shape

import std.types { String, Int, Bool, List, NonEmptyStr }
import std.list { distinct_by_key }
import std.algebra { list_push }

fn probe_case(names: List<String>, keys: List<NonEmptyStr>) -> List<NonEmptyStr> {
  let _seed = fold(names, init: [], f: (acc, n) => acc |> list_push(item: n as NonEmptyStr))
  distinct_by_key(xs: keys, key: fn(k) { k as String })
}
"#;

/// An empty-init `list_push` whose item type is the only evidence: T must instantiate from the item,
/// and the result must be accepted where `List<NonEmptyStr>` is required. Refusal or a fabricated
/// element type both fail this.
const EMPTY_INIT_PUSH: &str = r#"module probe_empty_init_push

import std.types { String, Int, Bool, List, NonEmptyStr }
import std.algebra { list_push }

fn probe_case(n: String) -> List<NonEmptyStr> {
  list_push(xs: [], item: n as NonEmptyStr)
}
"#;

/// The over-breadth control: a generic wrapper around `list_push`. Here the argument's element IS a
/// type variable — the CALLER's own `U` — and binding the callee's `T := U` is legitimate evidence.
/// A refusal rule keyed on "any variable-headed candidate" breaks this specimen.
const GENERIC_WRAPPER: &str = r#"module probe_generic_wrapper

import std.types { String, Int, Bool, List, NonEmptyStr }
import std.algebra { list_push }

fn push_one<U>(xs: List<U>, item: U) -> List<U> {
  list_push(xs: xs, item: item)
}

fn probe_case(n: String) -> List<NonEmptyStr> {
  push_one(xs: [], item: n as NonEmptyStr)
}
"#;

fn module_path(root: &Path, module: &str) -> PathBuf {
    let mut p = root.join(module.replace('.', "/"));
    p.set_extension("dag");
    p
}

/// The import closure of the std modules the specimens use, so the fixture sees the production
/// definitions of `distinct_by_key`/`list_push` rather than a stub.
fn std_closure(root: &Path, entry: &str) -> Vec<Rc<SourceFile>> {
    let mut seen = BTreeSet::new();
    let mut todo = vec!["std.list".to_string(), "std.algebra".to_string()];
    let mut out = Vec::new();
    while let Some(m) = todo.pop() {
        if !seen.insert(m.clone()) {
            continue;
        }
        let path = module_path(root, &m);
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        for line in content.lines() {
            if let Some(rest) = line.strip_prefix("import ") {
                todo.push(
                    rest.chars()
                        .take_while(|c| !c.is_whitespace() && *c != '{')
                        .collect(),
                );
            }
        }
        out.push(Rc::new(SourceFile {
            path: path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            content,
        }));
    }
    let _ = entry;
    out
}

struct Emitted {
    module: String,
    diagnostics: Vec<String>,
}

fn emit(root: &Path, source: &str, module: &str) -> Emitted {
    let mut sources = std_closure(root, module);
    sources.push(Rc::new(SourceFile {
        path: format!("{}.dag", module),
        content: source.into(),
    }));
    let result = compile_sources(Rc::new(sources.into_iter().collect()), RenderTarget::Rust);
    let wanted = format!("src/{}.rs", module);
    let module = result
        .files
        .iter()
        .find(|f| f.path == wanted)
        .map(|f| f.content.clone())
        .unwrap_or_else(|| {
            panic!(
                "emission did not produce {wanted}: {:?}",
                result
                    .files
                    .iter()
                    .map(|f| f.path.clone())
                    .collect::<Vec<_>>()
            )
        });
    Emitted {
        module,
        diagnostics: result
            .diagnostics
            .iter()
            .filter(|d| v1_compiler::v1_std_core::is_error_diagnostic(d.diagnostic.clone()))
            .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
            .collect(),
    }
}

/// The key closure passed to `distinct_by_key`, as emitted: the text after the call's opening paren,
/// which is where a cast that never got a source type renders as a panic.
fn key_closure(emitted: &str) -> String {
    let at = emitted
        .find("distinct_by_key(")
        .expect("no distinct_by_key call in emitted module");
    let rest = &emitted[at..];
    let end = rest.find("\n}").unwrap_or(rest.len());
    rest[..end].to_string()
}

#[test]
fn a_fold_fed_hof_lambda_cast_lowers_instead_of_panicking() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");

    let reference = emit(&repo, DECLARED_SHAPE, "probe_declared_shape");
    let fold = emit(&repo, FOLD_SHAPE, "probe_fold_shape");

    // The runtime symptom, named explicitly so a regression that only re-masks it still fails here.
    assert!(
        !fold.module.contains("unsupported cast from"),
        "the fold-fed key lambda emitted a fabricated cast panic; the lambda parameter's type was \
         never instantiated from the call's element type. Emitted:\n{}",
        key_closure(&fold.module)
    );

    // The type assertion: same call, same element type, therefore the same key closure. This can
    // only hold if the fold path established NonEmptyStr, not a name borrowed from the callee.
    assert_eq!(
        key_closure(&reference.module).replace("probe_case", ""),
        key_closure(&fold.module).replace("probe_case", ""),
        "the fold-fed accumulator typed differently from the declared one:\n  declared: {}\n  fold:   {}",
        key_closure(&reference.module),
        key_closure(&fold.module)
    );

    // No new refusal appeared on the path that used to compile clean.
    assert!(
        fold.diagnostics.is_empty(),
        "the fold shape gained diagnostics: {:?}",
        fold.diagnostics
    );
}

#[test]
fn an_empty_init_list_push_takes_its_element_from_the_item_and_a_wrapper_still_types() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");

    let push = emit(&repo, EMPTY_INIT_PUSH, "probe_empty_init_push");
    assert!(
        !push.module.contains("unsupported cast from"),
        "list_push([], typed item) left an uninstantiated element type: {}",
        push.module
    );
    assert!(
        push.diagnostics.is_empty(),
        "list_push([], item: n as NonEmptyStr) was refused where List<NonEmptyStr> is required: {:?}",
        push.diagnostics
    );

    // Over-breadth control: binding the callee's T to the CALLER's own U is legitimate evidence.
    let wrapper = emit(&repo, GENERIC_WRAPPER, "probe_generic_wrapper");
    assert!(
        !wrapper.module.contains("unsupported cast from"),
        "the generic wrapper regressed (a variable-headed candidate was refused as vacuous when it \
         was the caller's own binder): {}",
        wrapper.module
    );
    assert!(
        wrapper.diagnostics.is_empty(),
        "the generic wrapper was refused: {:?}",
        wrapper.diagnostics
    );
}
