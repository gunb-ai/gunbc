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
/// Every field here is a structural declaration on the contract that
/// the runner actively consults — no defaults, no fallbacks; a
/// missing or malformed field fires `VerifierParseError::MalformedSpec`
/// at parse time so the spec-file drift surfaces loudly rather than
/// silently hand the runner a partially-filled binding.
///
/// The authored `PostEmitVerifier` record has a fifth field,
/// `syntax_only: Bool`, which the current runner does not consume —
/// it is metadata about the verifier's depth (shallow syntax check
/// vs full compile) that downstream schedulers (per-PR CI vs
/// nightly) will dispatch on. Per E-6, a spec field lands only with
/// a same-PR consumer; this binding therefore does NOT parse
/// `syntax_only` yet. The field remains in `std/clean_emission.dag`
/// because the three existing target spec files already declare it
/// (landed in PRs 1/2/3 alongside the rest of the contract shape);
/// a follow-up PR that adds the first downstream consumer will
/// add the parse + storage here atomically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostEmitVerifierBinding {
    pub command: String,
    pub args: Vec<String>,
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
    let expected_exit_code = require_int(verifier_fields, "expected_exit_code", spec_declaration)?;
    let output_policy =
        parse_output_policy(dag, verifier_fields, "output_policy", spec_declaration)?;

    Ok(PostEmitVerifierBinding {
        command,
        args,
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
    //
    // `Path::parent()` returns `Some("")` for single-component
    // relative paths (e.g. `main.rs`); `current_dir("")` fails the
    // spawn on most platforms. Filter empties so relative paths in
    // the current directory fall through to the inherited cwd
    // instead of rejecting valid inputs at spawn time.
    if let Some(parent) = source_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
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
    // Variant resolution dispatches on the cached
    // `VerifierOutputPolicyVariants` typed handles — the Dag
    // resolves the Disj's variant ids once at bootstrap end and
    // every downstream consumer compares `DeclarationId`s here. No
    // variant.label string lookups at parse time; INVARIANTS.md
    // §Layer opacity / §Semantic authority forbids post-lowering
    // consumers from reaching for names. Mirrors the PR 2.5
    // `PatternBindingRuleVariants` structure.
    let variants = dag.verifier_output_policy_variants();
    let ignore_output = variants
        .ignore_output
        .ok_or(VerifierParseError::MalformedSpec {
            declaration,
            detail: "VerifierOutputPolicy.IgnoreVerifierOutput declaration was not found",
        })?;
    let require_empty_stdout =
        variants
            .require_empty_stdout
            .ok_or(VerifierParseError::MalformedSpec {
                declaration,
                detail: "VerifierOutputPolicy.RequireEmptyStdout declaration was not found",
            })?;
    let require_empty_stderr =
        variants
            .require_empty_stderr
            .ok_or(VerifierParseError::MalformedSpec {
                declaration,
                detail: "VerifierOutputPolicy.RequireEmptyStderr declaration was not found",
            })?;
    let require_empty_stdout_and_stderr =
        variants
            .require_empty_stdout_and_stderr
            .ok_or(VerifierParseError::MalformedSpec {
                declaration,
                detail:
                    "VerifierOutputPolicy.RequireEmptyStdoutAndStderr declaration was not found",
            })?;
    if *constructor == ignore_output {
        Ok(VerifierOutputPolicyBinding::IgnoreVerifierOutput)
    } else if *constructor == require_empty_stdout {
        Ok(VerifierOutputPolicyBinding::RequireEmptyStdout)
    } else if *constructor == require_empty_stderr {
        Ok(VerifierOutputPolicyBinding::RequireEmptyStderr)
    } else if *constructor == require_empty_stdout_and_stderr {
        Ok(VerifierOutputPolicyBinding::RequireEmptyStdoutAndStderr)
    } else {
        Err(VerifierParseError::MalformedSpec {
            declaration,
            detail: "output_policy constructor is not a known VerifierOutputPolicy variant",
        })
    }
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
        assert_eq!(binding.expected_exit_code, 0);
        assert_eq!(
            binding.output_policy,
            VerifierOutputPolicyBinding::IgnoreVerifierOutput
        );
    }

    /// Regression: `Path::parent()` returns `Some("")` for
    /// single-component relative paths like `main.rs`. Setting
    /// `current_dir("")` fails the spawn on most platforms, so the
    /// runner used to reject valid inputs whenever callers passed
    /// a bare relative filename. The fix filters empty parents so
    /// the spawn inherits the ambient cwd instead. This test uses
    /// an environment-probe binary that exists on every
    /// POSIX-shaped test runner and expects exit 0 regardless of
    /// the path passed; the specific file does not need to exist
    /// because the probe doesn't read it.
    #[cfg(unix)]
    #[test]
    fn run_accepts_single_component_relative_source_path() {
        use std::path::PathBuf;
        let binding = PostEmitVerifierBinding {
            command: "true".to_string(),
            args: Vec::new(),
            expected_exit_code: 0,
            output_policy: VerifierOutputPolicyBinding::IgnoreVerifierOutput,
        };
        let relative = PathBuf::from("main.rs");
        run_post_emit_verifier(&binding, &relative)
            .expect("single-component relative source path must not fail the spawn");
    }
}
