use std::rc::Rc;

use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{compile_sources, SourceFile};
use v1_compiler::v1_std_core::CompilerDiagnostic;

// `sole_constructor` restricts CONSTRUCTION to the declaring module. The judgment is keyed on the
// declaration's module (the symbol index's `<module>.<Name>`), never on the file a span names, and it
// covers a coproduct's arms as well as a record.
fn violating_modules(files: &[(&str, &str)]) -> Vec<String> {
    let sources: im::Vector<Rc<SourceFile>> = files
        .iter()
        .map(|(path, content)| {
            Rc::new(SourceFile {
                path: path.to_string(),
                content: content.to_string(),
            })
        })
        .collect();
    compile_sources(Rc::new(sources), RenderTarget::Rust)
        .diagnostics
        .iter()
        .filter(|d| {
            matches!(
                *d.diagnostic,
                CompilerDiagnostic::SoleConstructorViolation { .. }
            )
        })
        .map(|d| d.module_name.to_string())
        .collect()
}

const SEALED_CHOICE: (&str, &str) = (
    "d.dag",
    "module d\ntype Choice sole_constructor = ArmA { n: Int } | ArmB { s: String } | ArmU\nfn mk() -> Choice { ArmA { n: 1 } }\n",
);

#[test]
fn arm_literal_outside_the_defining_module_is_refused() {
    let e = (
        "e.dag",
        "module e\nimport d { Choice, ArmA }\nfn forged() -> Choice { ArmA { n: 9 } }\n",
    );
    assert_eq!(
        violating_modules(&[SEALED_CHOICE, e]),
        vec!["e".to_string()]
    );
}

#[test]
fn arm_literal_inside_the_defining_module_and_a_match_outside_are_clean() {
    let e = (
        "e.dag",
        "module e\nimport d { Choice, ArmA, ArmB, mk }\nfn read() -> Int { match mk() { ArmA { n: n } => n  ArmB { s: _ } => 0  ArmU => 0 } }\n",
    );
    assert_eq!(violating_modules(&[SEALED_CHOICE, e]), Vec::<String>::new());
}

#[test]
fn record_construction_in_another_file_of_the_defining_module_is_clean() {
    let d = ("d1.dag", "module d\ntype S sole_constructor { x: Int }\n");
    let d2 = ("d2.dag", "module d\nfn mk() -> S { S { x: 1 } }\n");
    assert_eq!(violating_modules(&[d, d2]), Vec::<String>::new());
}

#[test]
fn record_construction_in_the_defining_module_is_clean() {
    let d = (
        "d.dag",
        "module d\ntype S sole_constructor { x: Int }\nfn mk() -> S { S { x: 1 } }\n",
    );
    assert_eq!(violating_modules(&[d]), Vec::<String>::new());
}

#[test]
fn arm_literal_in_a_module_declaring_a_same_named_type_is_refused() {
    let e = (
        "e.dag",
        "module e\nimport d { ArmA }\ntype Choice = Other { k: Int }\nfn forged() -> Int { ArmA { n: 9 }\n 0 }\n",
    );
    assert_eq!(
        violating_modules(&[SEALED_CHOICE, e]),
        vec!["e".to_string()]
    );
}
