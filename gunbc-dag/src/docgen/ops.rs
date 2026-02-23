//! Doc generation operations.
//!
//! Produces documentation content by stitching together handwritten text
//! and live code/test excerpts.

use gunbc_exec::{
    require_response, require_str, ExecError, Executable, OutputMap, TransportResponseExt,
};
use gunbc_ir::transport::{FileOp, FileRequest, TransportRequest};
use gunbc_ir::Value;
use std::collections::HashMap;

pub const AB_DOC_PATH: &str = "docs/ab-writing-workflows.md";
pub const CLIPPY_GRAPH_MOCK_PATH: &str = "lib/tools/clippy/src/graph_mock.rs";
pub const CLIPPY_GENERATED_TESTS_PATH: &str = "lib/tools/clippy/src/generated_tests.rs";
pub const CLIPPY_CONFIG_PATH: &str = "lib/tools/clippy/src/config.rs";
pub const CLIPPY_GRAPH_PATH: &str = "lib/tools/clippy/src/graph.rs";
pub const CLIPPY_LIB_PATH: &str = "lib/tools/clippy/src/lib.rs";
pub const CLIPPY_LINT_PATH: &str = "lib/tools/clippy/src/lint.rs";
pub const CLIPPY_OPS_PATH: &str = "lib/tools/clippy/src/ops.rs";
pub const CLIPPY_POLICY_PATH: &str = "lib/tools/clippy/src/policy.rs";
const MARKER_CLIPPY_MOCK_SPEC: &str = "clippy_mock_spec";
const MARKER_CLIPPY_GENERATED_TEST_EXCERPT: &str = "clippy_generated_test_excerpt";
const MARKER_APPENDIX_A_CLIPPY: &str = "appendix_a_clippy";
const MARKER_APPENDIX_B: &str = "appendix_b";
const MARKER_APPENDIX_C: &str = "appendix_c";
const MARKER_APPENDIX_D: &str = "appendix_d";

/// Docgen operations (pure, no transport I/O).
#[derive(Debug, Clone)]
pub enum DocgenOp {
    RenderAbWorkflowsDoc,
    PrepareFileRead { path: String },
    ParseFileContent { path: String, allow_missing: bool },
}

impl Executable for DocgenOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            DocgenOp::RenderAbWorkflowsDoc => execute_render_ab_workflows_doc(inputs),
            DocgenOp::PrepareFileRead { path } => execute_prepare_file_read(path),
            DocgenOp::ParseFileContent {
                path,
                allow_missing,
            } => execute_parse_file_content(path, *allow_missing, inputs),
        }
    }
}

#[derive(Debug, Clone)]
struct DocgenSources {
    template: String,
    clippy_graph_mock: String,
    clippy_generated_tests: String,
    clippy_config: String,
    clippy_graph: String,
    clippy_lib: String,
    clippy_lint: String,
    clippy_ops: String,
    clippy_policy: String,
}

impl DocgenSources {
    fn from_inputs(inputs: HashMap<String, Value>) -> Result<Self, ExecError> {
        let template = require_str(&inputs, "template")?.to_string();
        let clippy_graph_mock = require_str(&inputs, "clippy_graph_mock")?.to_string();
        let clippy_generated_tests = require_str(&inputs, "clippy_generated_tests")?.to_string();
        let clippy_config = require_str(&inputs, "clippy_config")?.to_string();
        let clippy_graph = require_str(&inputs, "clippy_graph")?.to_string();
        let clippy_lib = require_str(&inputs, "clippy_lib")?.to_string();
        let clippy_lint = require_str(&inputs, "clippy_lint")?.to_string();
        let clippy_ops = require_str(&inputs, "clippy_ops")?.to_string();
        let clippy_policy = require_str(&inputs, "clippy_policy")?.to_string();

        Ok(Self {
            template,
            clippy_graph_mock,
            clippy_generated_tests,
            clippy_config,
            clippy_graph,
            clippy_lib,
            clippy_lint,
            clippy_ops,
            clippy_policy,
        })
    }
}

fn execute_prepare_file_read(path: &str) -> Result<HashMap<String, Value>, ExecError> {
    let request = TransportRequest::File(FileRequest::read(path));
    OutputMap::new()
        .request("request", request)
        .bool("skip", false)
        .ok()
}

fn execute_parse_file_content(
    path: &str,
    allow_missing: bool,
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let response = require_response(&inputs, "response")?;
    let file_resp = response.require_file()?;

    if file_resp.operation != FileOp::Read {
        return Err(ExecError::new(format!(
            "docgen: expected file read response for {path}"
        )));
    }

    if !file_resp.success {
        let err = file_resp.error.as_deref().unwrap_or("unknown error");
        if allow_missing {
            return OutputMap::new()
                .str("content", format!("// Missing file: {path} ({err})"))
                .ok();
        }
        return Err(ExecError::new(format!(
            "docgen: failed to read {path}: {err}"
        )));
    }

    let content = match &file_resp.content {
        Some(content) => content.clone(),
        None => {
            if allow_missing {
                format!("// Missing file: {path} (empty response)")
            } else {
                return Err(ExecError::new(format!(
                    "docgen: missing content for {path}"
                )));
            }
        }
    };

    OutputMap::new().str("content", content).ok()
}

fn execute_render_ab_workflows_doc(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let sources = DocgenSources::from_inputs(inputs)?;
    let content = render_ab_workflows_doc(&sources)?;
    OutputMap::new()
        .str("content", content)
        .str("path", AB_DOC_PATH)
        .ok()
}

fn render_ab_workflows_doc(sources: &DocgenSources) -> Result<String, ExecError> {
    let mut output = sources.template.clone();
    output = replace_section(
        &output,
        MARKER_CLIPPY_MOCK_SPEC,
        &render_clippy_mock_spec(&sources.clippy_graph_mock)?,
    )?;
    output = replace_section(
        &output,
        MARKER_CLIPPY_GENERATED_TEST_EXCERPT,
        &render_clippy_generated_test_excerpt(&sources.clippy_generated_tests)?,
    )?;
    output = replace_section(
        &output,
        MARKER_APPENDIX_A_CLIPPY,
        &render_appendix_a_clippy(&sources.clippy_generated_tests)?,
    )?;
    output = replace_section(&output, MARKER_APPENDIX_B, &render_appendix_b(sources)?)?;
    output = replace_section(&output, MARKER_APPENDIX_C, &render_appendix_c())?;
    output = replace_section(&output, MARKER_APPENDIX_D, &render_appendix_d(sources))?;

    Ok(output)
}

fn render_appendix_d(sources: &DocgenSources) -> String {
    let sections = [
        (
            "Clippy MockSpec",
            CLIPPY_GRAPH_MOCK_PATH,
            sources.clippy_graph_mock.as_str(),
        ),
        (
            "Clippy Generated Tests",
            CLIPPY_GENERATED_TESTS_PATH,
            sources.clippy_generated_tests.as_str(),
        ),
    ];

    let mut out = Vec::new();
    out.push("This appendix is generated by gunbc-docgen. Do not edit manually.".to_string());
    out.push(String::new());
    out.push("Regenerate with:".to_string());
    out.push("- `cargo run -p gunbc-dag --bin gunbc-docgen --release`".to_string());
    out.push(String::new());
    out.push("<details>".to_string());
    out.push("<summary><strong>Menu</strong></summary>".to_string());
    out.push(String::new());
    out.push("- [Clippy MockSpec](#appendix-d-clippy-mockspec)".to_string());
    out.push("- [Clippy Generated Tests](#appendix-d-clippy-generated-tests)".to_string());
    out.push(String::new());
    out.push("</details>".to_string());
    out.push(String::new());

    for (idx, (title, path, content)) in sections.iter().enumerate() {
        out.push(String::new());
        let section_num = idx + 1;
        let anchor = match *title {
            "Clippy MockSpec" => "appendix-d-clippy-mockspec",
            "Clippy Generated Tests" => "appendix-d-clippy-generated-tests",
            _ => "appendix-d-unknown",
        };
        out.push(format!("<a id=\"{anchor}\"></a>"));
        out.push(format!("### D.{section_num} {title}"));
        out.push(String::new());
        out.push(format!("Source: `{}`", path));
        out.push(String::new());
        out.push("```rust".to_string());
        out.push(content.trim_end().to_string());
        out.push("```".to_string());
        out.push(String::new());
        out.push("[Back to Appendix D](#appendix-d-generated-artifacts)".to_string());
    }

    out.push(String::new());
    out.join("\n")
}

fn render_clippy_mock_spec(src: &str) -> Result<String, ExecError> {
    let snippet = strip_module_docs(src);
    Ok(wrap_rust(&snippet))
}

fn render_clippy_generated_test_excerpt(src: &str) -> Result<String, ExecError> {
    let mut snippet = extract_fn(src, "test_dryrun_completion").ok_or_else(|| {
        ExecError::new("docgen: missing test_dryrun_completion in clippy generated tests")
    })?;
    snippet = strip_guard_test(&snippet);
    snippet = normalize_colons(&snippet);
    Ok(wrap_rust_with_prefix(
        "// Generated by gunbc-testgen (trimmed: guard_test omitted)",
        &snippet,
    ))
}

fn render_appendix_a_clippy(src: &str) -> Result<String, ExecError> {
    render_clippy_generated_test_excerpt(src)
}

fn render_appendix_b(sources: &DocgenSources) -> Result<String, ExecError> {
    let clippy_generated = collect_tests_from_content(&sources.clippy_generated_tests, false);
    let mut clippy_manual = Vec::new();
    let manual_sources = [
        &sources.clippy_config,
        &sources.clippy_graph,
        &sources.clippy_graph_mock,
        &sources.clippy_lib,
        &sources.clippy_lint,
        &sources.clippy_ops,
        &sources.clippy_policy,
    ];
    for src in manual_sources {
        clippy_manual.extend(collect_tests_from_content(src, true));
    }
    clippy_manual.sort_by(|a, b| a.name.cmp(&b.name));

    let mut out = Vec::new();

    out.push(details_block(
        &format!("B.1 Clippy Generated Tests ({})", clippy_generated.len()),
        &clippy_generated,
    ));
    out.push(String::new());
    out.push(details_block(
        &format!("B.2 Clippy Manual Unit Tests ({})", clippy_manual.len()),
        &clippy_manual,
    ));

    Ok(out.join("\n"))
}

fn render_appendix_c() -> String {
    let mut out = Vec::new();

    out.push("### C.1 Clippy".to_string());
    out.push(String::new());
    out.push("Generated graph (flattened SubDag):".to_string());
    out.push(String::new());
    out.push("```text".to_string());
    out.push("check -> create -> resolve".to_string());
    out.push("```".to_string());
    out.push(String::new());
    out.push("Generated code (Rust, testgen output):".to_string());
    out.push(String::new());
    out.push("`lib/tools/clippy/src/generated_tests.rs`".to_string());
    out.push(String::new());
    out.push("Generated tests:".to_string());
    out.push(String::new());
    out.push("`lib/tools/clippy/src/generated_tests.rs`".to_string());
    out.push(String::new());
    out.push("Generated integration tests:".to_string());
    out.push(String::new());
    out.push(
        "(None yet — add when clippy gets a CLI codegen target or integration harness.)"
            .to_string(),
    );
    out.push(String::new());

    out.push("Appendix (generated artifacts): see **Appendix D**.".to_string());

    out.join("\n")
}

fn details_block(title: &str, tests: &[TestCase]) -> String {
    let mut out = Vec::new();
    out.push("<details>".to_string());
    out.push(format!("<summary><strong>{}</strong></summary>", title));
    out.push(String::new());
    for test in tests {
        out.push(format!("- {} — {}", test.name, test.description));
    }
    out.push(String::new());
    out.push("</details>".to_string());
    out.join("\n")
}

fn replace_section(doc: &str, key: &str, replacement: &str) -> Result<String, ExecError> {
    let begin = format!("<!-- BEGIN GENERATED:{key} -->");
    let end = format!("<!-- END GENERATED:{key} -->");

    let start = doc
        .find(&begin)
        .ok_or_else(|| ExecError::new(format!("docgen: missing begin marker: {begin}")))?;
    let rest = &doc[start + begin.len()..];
    let end_rel = rest
        .find(&end)
        .ok_or_else(|| ExecError::new(format!("docgen: missing end marker: {end}")))?;

    let before = &doc[..start + begin.len()];
    let after = &rest[end_rel..];
    Ok(format!("{before}\n{replacement}\n{after}"))
}

fn wrap_rust(snippet: &str) -> String {
    format!("```rust\n{}\n```", snippet.trim_end())
}

fn wrap_rust_with_prefix(prefix: &str, snippet: &str) -> String {
    format!("```rust\n{}\n{}\n```", prefix, snippet.trim_end())
}

fn strip_module_docs(src: &str) -> String {
    let mut out = Vec::new();
    let mut skipping = true;
    for line in src.lines() {
        if skipping {
            if line.trim_start().starts_with("//!") || line.trim().is_empty() {
                continue;
            }
            skipping = false;
        }
        out.push(line.to_string());
    }
    out.join("\n")
}

fn extract_fn(src: &str, fn_name: &str) -> Option<String> {
    let lines: Vec<&str> = src.lines().collect();
    let target = format!("fn {fn_name}");
    for (idx, line) in lines.iter().enumerate() {
        if !line.contains(&target) {
            continue;
        }

        let mut start = idx;
        while start > 0 {
            let prev = lines[start - 1].trim_start();
            if prev.starts_with("///") || prev.starts_with("#[") {
                start -= 1;
                continue;
            }
            if prev.is_empty() {
                start -= 1;
                continue;
            }
            break;
        }

        let mut buf = Vec::new();
        let mut brace_count = 0i32;
        let mut saw_brace = false;
        for line in &lines[start..] {
            buf.push((*line).to_string());
            for ch in line.chars() {
                if ch == '{' {
                    brace_count += 1;
                    saw_brace = true;
                } else if ch == '}' {
                    brace_count -= 1;
                }
            }
            if saw_brace && brace_count == 0 {
                break;
            }
        }

        if !buf.is_empty() {
            return Some(buf.join("\n"));
        }
    }

    None
}

fn strip_guard_test(src: &str) -> String {
    let mut out = Vec::new();
    let mut skip_guard = false;
    for line in src.lines() {
        let trimmed = line.trim();
        if line.contains("guard_test(") {
            skip_guard = true;
            continue;
        }
        if skip_guard {
            if trimmed == "return ();"
                || trimmed == "return();"
                || trimmed == "return ()"
                || trimmed == "return;"
                || trimmed == "return ;"
            {
                continue;
            }
            if trimmed == "};" {
                skip_guard = false;
                continue;
            }
        }
        out.push(line.to_string());
    }
    out.join("\n")
}

fn normalize_colons(src: &str) -> String {
    let mut s = src.replace(" :: ", "::");
    s = s.replace(" ::", "::");
    s = s.replace(":: ", "::");
    s = s.replace("::\n", "::");
    s
}

#[derive(Debug, Clone)]
struct TestCase {
    name: String,
    description: String,
}

fn collect_tests_from_content(content: &str, allow_non_test_attr: bool) -> Vec<TestCase> {
    let mut tests = Vec::new();
    let mut doc_lines: Vec<String> = Vec::new();
    let mut saw_test_attr = false;

    for line in content.lines() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("///") {
            doc_lines.push(trimmed.trim_start_matches("///").trim().to_string());
            continue;
        }

        if trimmed.starts_with("#[test]") {
            saw_test_attr = true;
            continue;
        }

        if trimmed.starts_with("#[") && allow_non_test_attr {
            continue;
        }

        if let Some(name) = parse_fn_name(trimmed) {
            if saw_test_attr || name.starts_with("test_") {
                let desc = if !doc_lines.is_empty() {
                    first_sentence(&doc_lines.join(" "))
                } else {
                    name_to_desc(&name)
                };
                tests.push(TestCase {
                    name,
                    description: desc,
                });
            }
            doc_lines.clear();
            saw_test_attr = false;
            continue;
        }

        if !trimmed.is_empty() {
            doc_lines.clear();
        }
    }

    tests
}

fn parse_fn_name(line: &str) -> Option<String> {
    let idx = line.find("fn ")?;
    let rest = &line[idx + 3..];
    let name: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn first_sentence(text: &str) -> String {
    if let Some(pos) = text.find('.') {
        text[..=pos].trim().to_string()
    } else {
        text.trim().to_string()
    }
}

fn name_to_desc(name: &str) -> String {
    let raw = name.strip_prefix("test_").unwrap_or(name);
    let spaced = raw.replace('_', " ");
    let mut chars = spaced.chars();
    match chars.next() {
        None => spaced,
        Some(first) => {
            let mut result = String::new();
            result.extend(first.to_uppercase());
            result.push_str(chars.as_str());
            result
        }
    }
}
