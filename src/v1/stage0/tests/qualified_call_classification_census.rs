// ONE QUALIFIED SPELLING, EVERY CALL PATH: THE FORK CENSUS FOR "QUALIFIED FUNCTION VS METHOD ON RECEIVER".
//
// The decision "is `a.b.f(...)` a call into module `a.b`, a method on a receiver, or a service
// operation" was made at several sites that each re-read the spelling (`contains(name, ".")`) or a
// symbol-index hit, and was patched one site at a time (#13925, #14085, #14089 -- the last reverted
// by #14147 because it turned effect operations into missing functions). This census pushes one
// arm per distinct denotation through the real compile seam and reports EVERY arm that refuses, so
// a fix to one site cannot hide a sibling that still forks. Hermetic: in-memory sources only.
//
// The test is the RED before the classifier exists and the regression control after it: it asserts
// every arm is accepted and names each refused arm with the compiler's own message.

use std::rc::Rc;

use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{compile_sources, SourceFile};
use v1_compiler::v1_std_core::diagnostic_to_message;

fn diagnostics_of(sources: &[(&str, &str)]) -> Vec<String> {
    let files: Vec<Rc<SourceFile>> = sources
        .iter()
        .map(|(path, content)| {
            Rc::new(SourceFile {
                path: (*path).to_string(),
                content: (*content).to_string(),
            })
        })
        .collect();
    compile_sources(Rc::new(files.into()), RenderTarget::Rust)
        .diagnostics
        .iter()
        .map(|d| diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}

const PROVIDER_B: (&str, &str) = (
    "fx/a/b.dag",
    "module a.b\n\nfn f(xs: List<Int>) -> List<Int> {\n  xs\n}\n",
);
const PROVIDER_C: (&str, &str) = (
    "fx/a/c.dag",
    "module a.c\n\nfn f(xs: List<Int>) -> Int {\n  xs |> count\n}\n",
);
const SERVICE: (&str, &str) = (
    "fx/svc/probe.dag",
    "module svc.probe\nservice Probe {\n  operation Version {\n    input {}\n    output {\n      out: String from \"stdout\"\n    }\n    transport shell { argv: [\"git\", \"--version\"] }\n  }\n}\n",
);

struct Arm {
    name: &'static str,
    consumer: &'static str,
}

const ARMS: &[Arm] = &[
    Arm {
        name: "module_path_call_in_call_position",
        consumer: "module consumer\n\nimport a.b { f }\n\nfn run() -> Int {\n  a.b.f(xs: [1, 0, 2]) |> count\n}\n",
    },
    Arm {
        name: "module_path_call_in_pipe_position",
        consumer: "module consumer\n\nimport a.b { f }\n\nfn run() -> Int {\n  [1, 0, 2] |> a.b.f() |> count\n}\n",
    },
    Arm {
        name: "module_path_call_without_importing_the_leaf",
        consumer: "module consumer\n\nimport a.b\n\nfn run() -> Int {\n  a.b.f(xs: [1]) |> count\n}\n",
    },
    Arm {
        name: "same_leaf_in_two_modules_each_reaches_its_own",
        consumer: "module consumer\n\nimport a.b\nimport a.c\n\nfn run() -> Int {\n  let l = a.b.f(xs: [1])\n  a.c.f(xs: l)\n}\n",
    },
    Arm {
        name: "same_named_local_fn_does_not_capture_the_qualified_call",
        consumer: "module consumer\n\nimport a.b\n\nfn f(xs: List<Int>) -> Int {\n  0\n}\n\nfn run() -> Int {\n  let l = a.b.f(xs: [1])\n  f(xs: l)\n}\n",
    },
    Arm {
        name: "kernel_method_on_a_local_stays_a_method",
        consumer: "module consumer\n\nimport a.b\n\nfn run(xs: List<Int>) -> Int {\n  xs |> count\n}\n",
    },
    Arm {
        name: "local_root_shadowing_a_module_root_is_a_receiver",
        consumer: "module consumer\n\nimport a.b\n\ntype Inner {\n  items: List<Int>\n}\n\ntype Holder {\n  b: Inner\n}\n\nfn run(a: Holder) -> Int {\n  a.b.items |> count\n}\n",
    },
    Arm {
        name: "service_operation_through_an_imported_service_name",
        consumer: "module consumer\n\nimport svc.probe { Probe }\n\nfn run() -> String {\n  let r = Probe.Version()\n  \"\"\n}\n",
    },
];

#[test]
fn every_qualified_call_path_agrees_on_one_classification() {
    let mut refused = Vec::new();
    for arm in ARMS {
        let consumer = ("fx/consumer.dag", arm.consumer);
        let messages = diagnostics_of(&[PROVIDER_B, PROVIDER_C, SERVICE, consumer]);
        if !messages.is_empty() {
            refused.push(format!("{}: {:?}", arm.name, messages));
        }
    }
    assert!(
        refused.is_empty(),
        "qualified-call arms refused ({} of {}):\n{}",
        refused.len(),
        ARMS.len(),
        refused.join("\n")
    );
}

/// A service is registered under its AUTHORED name (v1.compiler.infer scope fold: `svc_registry` is
/// keyed by `authored_name_at(service item)`, its namespace root by `namespace_root_from_properties`),
/// so a module path is not part of a service's identity and `svc.probe.Probe.Version()` names no
/// service. It must REFUSE; the classifier may not turn it into a module call or a receiver method.
#[test]
fn module_qualified_service_spelling_is_refused_not_reclassified() {
    let consumer = (
        "fx/consumer.dag",
        "module consumer\n\nimport svc.probe { Probe }\n\nfn run() -> String {\n  let r = svc.probe.Probe.Version()\n  \"\"\n}\n",
    );
    let messages = diagnostics_of(&[PROVIDER_B, PROVIDER_C, SERVICE, consumer]);
    assert!(
        !messages.is_empty(),
        "a module-qualified service spelling must refuse, but compiled clean"
    );
}
