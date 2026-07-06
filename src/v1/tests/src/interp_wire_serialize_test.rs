use std::rc::Rc;

use v1_compiler::cli_run::value_to_wire_json;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_interpreter::{self, Value};

use crate::helpers::{resolve_imports_transitively, workspace_root};

fn assert_resolved_no_hard_errors(result: &ResolvedPipelineResult) {
    let msgs: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && result.graph.is_some(),
        "expected resolved graph, got diagnostics {:?} (graph present: {})",
        msgs,
        result.graph.is_some()
    );
}

#[test]
fn interpreter_value_to_json_rejects_naive_variant_branch() {
    let source = read_v2_file("src/v1/stage0/src/v1_interpreter.rs");
    assert!(
        source.contains(
            "value_to_json must not serialize coproduct variants; use value_to_wire_json"
        ),
        "interpreter must fail-closed on naive variant JSON dump"
    );
}

#[test]
fn anthropic_chat_message_user_role_matches_modeled_wire_contract() {
    let src = r#"module test.anthropic_wire

import extdeps.llm.anthropic { AnthropicChatMessage, AnthropicUserContentBlock }

fn witness() -> List<AnthropicChatMessage> {
  [
    UserMessage {
      content: [UserTextBlock { text: "hello" }]
    }
  ]
}
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources.into()));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx = v1_interpreter::InterpContext::new(
        graph,
        resolved.source_indices.clone(),
        v1_interpreter::ExecutionMode::Wet,
    );

    let val = v1_interpreter::run_in_context(&ctx, "witness", false).expect("witness runs");
    let Value::List(items) = val else {
        panic!("expected list, got {val:?}");
    };
    let first = items.iter().next().expect("one message");
    let wire = value_to_wire_json(first, &ctx).expect("wire serialize");
    assert_eq!(
        wire["role"], "user",
        "UserMessage must serialize role=user per anthropic_chat_message_wire_contract; got {wire}"
    );
    assert_eq!(wire["content"][0]["type"], "text");
    assert_eq!(wire["content"][0]["text"], "hello");
}

fn read_v2_file(relative_path: &str) -> String {
    let path = workspace_root().join(relative_path);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e))
}
