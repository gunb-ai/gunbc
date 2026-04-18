//! E-5 / Lane 1 Stage 1c PR 4 — shared test harness that reads each
//! target's `CleanEmissionContract.post_emit_verifier` record and
//! invokes the declared verifier against an emitted source file.
//!
//! Before this module, each pilot roundtrip test (`emit_rust_…`,
//! `emit_go_…`, `emit_python_…`) hardcoded the verifier command +
//! args in Rust. That divergence is exactly what the contract is
//! supposed to eliminate — the spec file declares the verifier
//! config; the harness reads it; nothing drifts. Adding a new target
//! (Swift, Kotlin, Verilog) requires only a `CleanEmissionContract`
//! data item, not a new Rust roundtrip helper.
//!
//! The harness is deliberately minimal:
//! - `parse_post_emit_verifier` walks the declared structural fields
//!   of a `python_clean_emission` / `rust_clean_emission` /
//!   `go_clean_emission` declaration and extracts the five-field
//!   `PostEmitVerifier` record into a typed binding.
//! - `run_post_emit_verifier` invokes `Command::new(binding.command)
//!   .args(&binding.args).arg(source_path)`, collects stdout/stderr,
//!   and applies the binding's `expected_exit_code` +
//!   `output_policy` as the verdict. The source file path is
//!   appended as the final positional argument — that's the
//!   pattern every declared verifier expects (`rustc <file>`,
//!   `gofmt -l <file>`, `python3 -m py_compile <file>`).

use std::path::Path;
use std::process::Command;

use crate::dag::{DeclarationId, FieldValue, LiteralBits};
use crate::Dag;

/// Typed read of `data <target>_clean_emission.post_emit_verifier`.
/// Every field is a structural declaration on the contract — no
/// defaults, no fallbacks; a missing or malformed field fires
/// `VerifierParseError::MalformedSpec` at parse time so the spec-file
/// drift surfaces loudly rather than silently hand the runner a
/// partially-filled binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostEmitVerifierBinding {
    pub command: String,
    pub args: Vec<String>,
    pub syntax_only: bool,
    pub expected_exit_code: i64,
    pub output_policy: VerifierOutputPolicyBinding,
}

/// Typed mirror of `std.clean_emission.VerifierOutputPolicy`. Only
/// closed variants — adding a new policy is a spec extension, not a
/// runner override.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifierOutputPolicyBinding {
    IgnoreVerifierOutput,
    RequireEmptyStdout,
    RequireEmptyStderr,
    RequireEmptyStdoutAndStderr,
}

/// Contract-parse failures. Callers that want to surface the
/// underlying declaration in a diagnostic propagate this error
/// directly — the `DeclarationId` points at the
/// `*_clean_emission` data item so spec-authors can find the
/// offending record.
#[derive(Debug, Clone)]
pub enum VerifierParseError {
    MissingDeclaration,
    MalformedSpec {
        declaration: DeclarationId,
        detail: &'static str,
    },
}

/// Runner failures. `InvocationFailed` fires when the OS cannot
/// spawn the verifier process (binary missing from PATH, etc.);
/// `WrongExitCode` and `PolicyViolation` fire after a successful
/// spawn when the verdict does not match the contract.
#[derive(Debug)]
pub enum VerifierRunError {
    InvocationFailed {
        command: String,
        io_error: String,
    },
    WrongExitCode {
        expected: i64,
        actual: Option<i32>,
        stdout: String,
        stderr: String,
    },
    PolicyViolation {
        policy: VerifierOutputPolicyBinding,
        stdout: String,
        stderr: String,
    },
}

/// Parse `<spec_declaration>.post_emit_verifier` into a typed
/// binding. `spec_declaration` is the id of a
/// `*_clean_emission: CleanEmissionContract` data item (obtained via
/// `dag.rust_clean_emission_spec()` / `dag.go_clean_emission_spec()`
/// / `dag.python_clean_emission_spec()`).
pub fn parse_post_emit_verifier(
    dag: &Dag,
    spec_declaration: DeclarationId,
) -> Result<PostEmitVerifierBinding, VerifierParseError> {
    let decl = dag.declaration(spec_declaration);
    let Some(crate::dag::ValueBody::Structural { fields }) = &decl.value_body else {
        return Err(VerifierParseError::MalformedSpec {
            declaration: spec_declaration,
            detail: "clean_emission spec must be a structural data item",
        });
    };
    let verifier_value = fields
        .iter()
        .find(|(label, _)| label == "post_emit_verifier")
        .map(|(_, value)| value)
        .ok_or(VerifierParseError::MalformedSpec {
            declaration: spec_declaration,
            detail: "clean_emission is missing required `post_emit_verifier` field",
        })?;
    let FieldValue::Record(verifier_fields) = verifier_value else {
        return Err(VerifierParseError::MalformedSpec {
            declaration: spec_declaration,
            detail: "clean_emission.post_emit_verifier must be a structural record",
        });
    };

    let command = require_string(verifier_fields, "command", spec_declaration)?;
    let args = require_string_list(verifier_fields, "args", spec_declaration)?;
    let syntax_only = require_bool(verifier_fields, "syntax_only", spec_declaration)?;
    let expected_exit_code = require_int(verifier_fields, "expected_exit_code", spec_declaration)?;
    let output_policy =
        parse_output_policy(dag, verifier_fields, "output_policy", spec_declaration)?;

    Ok(PostEmitVerifierBinding {
        command,
        args,
        syntax_only,
        expected_exit_code,
        output_policy,
    })
}

/// Invoke the verifier against `source_path` and apply the
/// contract's verdict semantics. The source file path is appended
/// as the final positional argument — every declared verifier
/// today expects the file as its last argument
/// (`rustc <file>` / `gofmt -l <file>` / `python3 -m py_compile
/// <file>`).
pub fn run_post_emit_verifier(
    binding: &PostEmitVerifierBinding,
    source_path: &Path,
) -> Result<(), VerifierRunError> {
    let mut command = Command::new(&binding.command);
    command.args(&binding.args).arg(source_path);
    // Rustc (and py_compile to a lesser extent) produce artifacts
    // next to / in cwd. Pinning cwd to the source's parent directory
    // contains those artifacts inside the caller's tmp dir instead
    // of dropping them into the workspace root.
    if let Some(parent) = source_path.parent() {
        command.current_dir(parent);
    }
    let output = command
        .output()
        .map_err(|err| VerifierRunError::InvocationFailed {
            command: binding.command.clone(),
            io_error: err.to_string(),
        })?;

    let exit_code = output.status.code();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    if exit_code.map(i64::from) != Some(binding.expected_exit_code) {
        return Err(VerifierRunError::WrongExitCode {
            expected: binding.expected_exit_code,
            actual: exit_code,
            stdout,
            stderr,
        });
    }

    let policy = binding.output_policy;
    let stdout_empty = stdout.trim().is_empty();
    let stderr_empty = stderr.trim().is_empty();
    let ok = match policy {
        VerifierOutputPolicyBinding::IgnoreVerifierOutput => true,
        VerifierOutputPolicyBinding::RequireEmptyStdout => stdout_empty,
        VerifierOutputPolicyBinding::RequireEmptyStderr => stderr_empty,
        VerifierOutputPolicyBinding::RequireEmptyStdoutAndStderr => stdout_empty && stderr_empty,
    };
    if !ok {
        return Err(VerifierRunError::PolicyViolation {
            policy,
            stdout,
            stderr,
        });
    }
    Ok(())
}

fn require_string(
    fields: &[(String, FieldValue)],
    name: &'static str,
    declaration: DeclarationId,
) -> Result<String, VerifierParseError> {
    fields
        .iter()
        .find(|(label, _)| label == name)
        .and_then(|(_, value)| match value {
            FieldValue::Literal(LiteralBits::String(s)) => Some(s.clone()),
            _ => None,
        })
        .ok_or(VerifierParseError::MalformedSpec {
            declaration,
            detail: "required string field missing or malformed",
        })
}

fn require_bool(
    fields: &[(String, FieldValue)],
    name: &'static str,
    declaration: DeclarationId,
) -> Result<bool, VerifierParseError> {
    fields
        .iter()
        .find(|(label, _)| label == name)
        .and_then(|(_, value)| match value {
            FieldValue::Literal(LiteralBits::Bool(b)) => Some(*b),
            _ => None,
        })
        .ok_or(VerifierParseError::MalformedSpec {
            declaration,
            detail: "required bool field missing or malformed",
        })
}

fn require_int(
    fields: &[(String, FieldValue)],
    name: &'static str,
    declaration: DeclarationId,
) -> Result<i64, VerifierParseError> {
    fields
        .iter()
        .find(|(label, _)| label == name)
        .and_then(|(_, value)| match value {
            FieldValue::Literal(LiteralBits::Int(n)) => Some(*n),
            _ => None,
        })
        .ok_or(VerifierParseError::MalformedSpec {
            declaration,
            detail: "required int field missing or malformed",
        })
}

fn require_string_list(
    fields: &[(String, FieldValue)],
    name: &'static str,
    declaration: DeclarationId,
) -> Result<Vec<String>, VerifierParseError> {
    let value = fields
        .iter()
        .find(|(label, _)| label == name)
        .map(|(_, value)| value)
        .ok_or(VerifierParseError::MalformedSpec {
            declaration,
            detail: "required list field missing",
        })?;
    let FieldValue::List(entries) = value else {
        return Err(VerifierParseError::MalformedSpec {
            declaration,
            detail: "required field must be a list",
        });
    };
    let mut out = Vec::with_capacity(entries.len());
    for entry in entries {
        let FieldValue::Literal(LiteralBits::String(s)) = entry else {
            return Err(VerifierParseError::MalformedSpec {
                declaration,
                detail: "list entries must be string literals",
            });
        };
        out.push(s.clone());
    }
    Ok(out)
}

fn parse_output_policy(
    dag: &Dag,
    fields: &[(String, FieldValue)],
    name: &'static str,
    declaration: DeclarationId,
) -> Result<VerifierOutputPolicyBinding, VerifierParseError> {
    let value = fields
        .iter()
        .find(|(label, _)| label == name)
        .map(|(_, value)| value)
        .ok_or(VerifierParseError::MalformedSpec {
            declaration,
            detail: "required variant field missing",
        })?;
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return Err(VerifierParseError::MalformedSpec {
            declaration,
            detail: "output_policy must be a VerifierOutputPolicy variant",
        });
    };
    if !payload.is_empty() {
        return Err(VerifierParseError::MalformedSpec {
            declaration,
            detail: "VerifierOutputPolicy variants must not carry payload fields",
        });
    }
    // Variant resolution walks the VerifierOutputPolicy declaration's
    // children once. Unlike `PatternBindingRule`, there is no cached
    // `VerifierOutputPolicyVariants` sidecar yet — when a second
    // consumer lands (Lane 1e generic walker invoking the verifier)
    // this becomes the motivating case for a cache analogous to
    // `PatternBindingRuleVariants`.
    let policy_decl = dag.declaration_by_name("VerifierOutputPolicy").ok_or(
        VerifierParseError::MalformedSpec {
            declaration,
            detail: "VerifierOutputPolicy declaration not found in the DAG",
        },
    )?;
    let crate::dag::TypeConnective::Disj { variants } = &policy_decl.connective else {
        return Err(VerifierParseError::MalformedSpec {
            declaration,
            detail: "VerifierOutputPolicy declaration is not a disjunction",
        });
    };
    for variant in variants {
        if variant.ty != *constructor {
            continue;
        }
        return match variant.label.as_str() {
            "IgnoreVerifierOutput" => Ok(VerifierOutputPolicyBinding::IgnoreVerifierOutput),
            "RequireEmptyStdout" => Ok(VerifierOutputPolicyBinding::RequireEmptyStdout),
            "RequireEmptyStderr" => Ok(VerifierOutputPolicyBinding::RequireEmptyStderr),
            "RequireEmptyStdoutAndStderr" => {
                Ok(VerifierOutputPolicyBinding::RequireEmptyStdoutAndStderr)
            }
            other => Err(VerifierParseError::MalformedSpec {
                declaration,
                detail: {
                    // Static str required by the error shape; fall
                    // back to a named constant naming the offender
                    // would require String support. For now surface
                    // a generic "unknown variant" and include the
                    // label in the variant declaration via a fresh
                    // design pass if a new variant ever ships.
                    let _ = other;
                    "VerifierOutputPolicy variant is not a known policy"
                },
            }),
        };
    }
    Err(VerifierParseError::MalformedSpec {
        declaration,
        detail: "output_policy constructor does not resolve to any VerifierOutputPolicy variant",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parse the Rust contract and assert the structural fields
    /// match the declared values in `src/v3/spec/rust.dag`. Proves
    /// the parser consumes the shared substrate correctly without
    /// hardcoding any of the target-specific command / args values.
    #[test]
    fn parses_rust_post_emit_verifier_contract() {
        let dag = crate::dag::Dag::new();
        let spec = dag
            .rust_clean_emission_spec()
            .expect("rust_clean_emission cached");
        let binding = parse_post_emit_verifier(&dag, spec).expect("parse");
        assert_eq!(binding.command, "rustc");
        assert_eq!(binding.args, vec!["--edition=2021", "-D", "warnings"]);
        assert!(!binding.syntax_only);
        assert_eq!(binding.expected_exit_code, 0);
        assert_eq!(
            binding.output_policy,
            VerifierOutputPolicyBinding::IgnoreVerifierOutput
        );
    }

    #[test]
    fn parses_go_post_emit_verifier_contract() {
        let dag = crate::dag::Dag::new();
        let spec = dag
            .go_clean_emission_spec()
            .expect("go_clean_emission cached");
        let binding = parse_post_emit_verifier(&dag, spec).expect("parse");
        assert_eq!(binding.command, "gofmt");
        assert_eq!(binding.args, vec!["-l"]);
        assert!(binding.syntax_only);
        assert_eq!(binding.expected_exit_code, 0);
        assert_eq!(
            binding.output_policy,
            VerifierOutputPolicyBinding::RequireEmptyStdout
        );
    }

    #[test]
    fn parses_python_post_emit_verifier_contract() {
        let dag = crate::dag::Dag::new();
        let spec = dag
            .python_clean_emission_spec()
            .expect("python_clean_emission cached");
        let binding = parse_post_emit_verifier(&dag, spec).expect("parse");
        assert_eq!(binding.command, "python3");
        assert_eq!(binding.args, vec!["-m", "py_compile"]);
        assert!(binding.syntax_only);
        assert_eq!(binding.expected_exit_code, 0);
        assert_eq!(
            binding.output_policy,
            VerifierOutputPolicyBinding::IgnoreVerifierOutput
        );
    }
}
