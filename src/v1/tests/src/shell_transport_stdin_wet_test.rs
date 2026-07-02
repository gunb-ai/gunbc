use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_interpreter::{self, ExecutionMode, Value};

use crate::helpers::resolve_imports_transitively_with_source_roots;

const SOURCE: &str = r#"module shell_stdin_wet_test

service shell.Pipe {
  operation Send {
    input { data: String }
    output {
      success: Bool from "exit_success"
      stdout: String from "stdout"
    }
    transport shell {
      argv: ["bash", "-s"]
      stdin: data
    }
    exit {
      0 => Unit
      nonzero => String "bash stdin script failed"
    }
  }
}

fn witness_bash_stdin_executes_body() -> Bool {
  let result = shell.Pipe.Send(data: "echo shell-stdin-wet-marker")
  result.success
    && string_contains(s: result.stdout, pattern: "shell-stdin-wet-marker")
}
"#;

fn run_wet(entry: &str) -> Value {
    let roots = crate::helpers::v2_layer_roots();
    let sources: Vec<Rc<SourceFile>> =
        resolve_imports_transitively_with_source_roots("shell_stdin_wet_test.dag", SOURCE, &roots);
    let resolved = compile_to_resolved(Rc::new(sources));
    let graph = resolved
        .graph
        .as_ref()
        .expect("resolved graph for shell stdin wet test");
    v1_compiler::v1_interpreter::run_with_options(
        graph,
        resolved.source_indices.clone(),
        entry,
        ExecutionMode::Wet,
        true,
    )
    .unwrap_or_else(|e| panic!("wet run {entry}: {e:?}"))
}

#[test]
fn shell_transport_stdin_is_wired_in_interpreter_wet_mode() {
    match run_wet("witness_bash_stdin_executes_body") {
        Value::Bool(true) => {}
        other => panic!("expected wet bash -s stdin witness true, got {other:?}"),
    }
}
