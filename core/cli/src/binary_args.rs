//! Shared CLI argument parser for gunbc binary entry points.
//!
//! Replaces the hand-rolled while-loop parsers in each `gunbc-dag/src/bin/*.rs`.
//! All binaries share `-n`/`--dry-run` and `-h`/`--help`; `--mode` and
//! additional string params are opt-in via builder methods.

use std::collections::HashMap;

use gunbc_ir::{resource::ExecMode, Value};

use crate::{parse, CliParam, ParamType, ParseError};

const MODE_PARAM_NAME: &str = "mode";

/// Supported subcommands for generated step-mode CLIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepModeSubcommand {
    Run,
    Step,
    ListSteps,
    Help,
}

/// Parsed step-mode CLI command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedStepModeArgs {
    /// The resolved subcommand.
    pub subcommand: StepModeSubcommand,
    /// Remaining argv items to pass to the selected handler.
    pub args: Vec<String>,
}

/// Parse errors for step-mode command dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepModeParseError {
    UnknownSubcommand { subcommand: String },
}

impl std::fmt::Display for StepModeParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepModeParseError::UnknownSubcommand { subcommand } => {
                write!(f, "unknown subcommand '{}'", subcommand)
            }
        }
    }
}

impl std::error::Error for StepModeParseError {}

/// Parse shared step-mode subcommand dispatch.
///
/// Behavior:
/// - `run ...` => `Run` with trailing args
/// - `step ...` => `Step` with trailing args
/// - `list-steps` => `ListSteps`
/// - `help` / `-h` / `--help` => `Help`
/// - first arg begins with `-` => `Run` (backwards-compatible flag passthrough)
/// - no subcommand => `Run`
pub fn parse_step_mode(argv: &[String]) -> Result<ParsedStepModeArgs, StepModeParseError> {
    let subcommand = argv.get(1).map(String::as_str);
    match subcommand {
        Some("run") => Ok(ParsedStepModeArgs {
            subcommand: StepModeSubcommand::Run,
            args: argv.iter().skip(2).cloned().collect(),
        }),
        Some("step") => Ok(ParsedStepModeArgs {
            subcommand: StepModeSubcommand::Step,
            args: argv.iter().skip(2).cloned().collect(),
        }),
        Some("list-steps") => Ok(ParsedStepModeArgs {
            subcommand: StepModeSubcommand::ListSteps,
            args: Vec::new(),
        }),
        Some("help") | Some("-h") | Some("--help") => Ok(ParsedStepModeArgs {
            subcommand: StepModeSubcommand::Help,
            args: Vec::new(),
        }),
        Some(first) if first.starts_with('-') => Ok(ParsedStepModeArgs {
            subcommand: StepModeSubcommand::Run,
            args: argv.iter().skip(1).cloned().collect(),
        }),
        Some(other) => Err(StepModeParseError::UnknownSubcommand {
            subcommand: other.to_string(),
        }),
        None => Ok(ParsedStepModeArgs {
            subcommand: StepModeSubcommand::Run,
            args: Vec::new(),
        }),
    }
}

/// Definition for a string-valued CLI parameter.
struct StringParamDef {
    name: String,
    short: Option<char>,
    default: Option<String>,
}

/// Builder for binary CLI argument parsing.
///
/// All binaries get `-n`/`--dry-run` and `-h`/`--help` for free.
/// Optional features are enabled via builder methods:
/// - `with_mode()` — `--mode=VALUE` / `--mode VALUE`
/// - `with_string_param()` — canonical `--<kebab(name)> VALUE` / optional short flag
pub struct BinaryArgs {
    enable_mode: bool,
    string_params: Vec<StringParamDef>,
}

/// Result of parsing binary CLI arguments.
pub struct ParsedBinaryArgs {
    /// Whether `--dry-run` / `-n` was present.
    pub dry_run: bool,
    /// Whether `--help` / `-h` was present.
    pub help: bool,
    /// Parsed resource mode from `--mode`.
    pub resource_mode: Option<ExecMode>,
    /// String parameter values keyed by name.
    string_values: HashMap<String, String>,
}

impl Default for BinaryArgs {
    fn default() -> Self {
        Self::new()
    }
}

impl BinaryArgs {
    /// Create a new builder with `-n`/`--dry-run` and `-h`/`--help` always enabled.
    pub fn new() -> Self {
        Self {
            enable_mode: false,
            string_params: Vec::new(),
        }
    }

    /// Enable `--mode=VALUE` / `--mode VALUE` parsing.
    pub fn with_mode(mut self) -> Self {
        self.enable_mode = true;
        self
    }

    /// Add a string-valued parameter.
    ///
    /// The long flag is canonicalized from `name` using kebab-case.
    /// Example: `output_dir` -> `--output-dir`.
    pub fn with_string_param(
        mut self,
        name: &str,
        short: Option<char>,
        default: Option<&str>,
    ) -> Self {
        self.string_params.push(StringParamDef {
            name: name.to_string(),
            short,
            default: default.map(|s| s.to_string()),
        });
        self
    }

    /// Parse the given argv slice (index 0 = program name, skipped).
    pub fn parse(self, argv: &[String]) -> Result<ParsedBinaryArgs, ParseError> {
        let mut schema: Vec<CliParam> = Vec::new();

        if self.enable_mode {
            schema.push(CliParam::new(MODE_PARAM_NAME, ParamType::Str));
        }

        for param in &self.string_params {
            let mut cli = CliParam::new(&param.name, ParamType::Str);
            if let Some(c) = param.short {
                cli = cli.short(c);
            }
            if let Some(ref default) = param.default {
                cli = cli.default(default);
            }
            schema.push(cli);
        }

        let parsed = parse(argv, &schema)?;

        let resource_mode = match parsed.values.get(MODE_PARAM_NAME) {
            Some(Value::Str(mode)) => {
                Some(
                    ExecMode::parse_strict(mode).map_err(|_| ParseError::InvalidValue {
                        flag: "--mode".to_string(),
                        value: mode.clone(),
                    })?,
                )
            }
            _ => None,
        };

        let mut string_values: HashMap<String, String> = HashMap::new();
        for param in &self.string_params {
            if let Some(Value::Str(value)) = parsed.values.get(&param.name) {
                string_values.insert(param.name.clone(), value.clone());
            }
        }

        Ok(ParsedBinaryArgs {
            dry_run: parsed.dry_run,
            help: parsed.help,
            resource_mode,
            string_values,
        })
    }

    /// Parse from `std::env::args()`, printing errors and exiting on failure.
    pub fn parse_env(self) -> ParsedBinaryArgs {
        let argv: Vec<String> = std::env::args().collect();
        match self.parse(&argv) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

impl ParsedBinaryArgs {
    /// Get a string parameter value by name.
    pub fn get_string(&self, name: &str) -> Option<&str> {
        self.string_values.get(name).map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(args: &[&str]) -> Vec<String> {
        args.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_simple_dry_run() {
        let parsed = BinaryArgs::new().parse(&argv(&["prog", "-n"])).unwrap();
        assert!(parsed.dry_run);
        assert!(!parsed.help);
    }

    #[test]
    fn test_simple_dry_run_long() {
        let parsed = BinaryArgs::new()
            .parse(&argv(&["prog", "--dry-run"]))
            .unwrap();
        assert!(parsed.dry_run);
    }

    #[test]
    fn test_simple_help() {
        let parsed = BinaryArgs::new().parse(&argv(&["prog", "--help"])).unwrap();
        assert!(parsed.help);
        assert!(!parsed.dry_run);
    }

    #[test]
    fn test_simple_help_short() {
        let parsed = BinaryArgs::new().parse(&argv(&["prog", "-h"])).unwrap();
        assert!(parsed.help);
    }

    #[test]
    fn test_unknown_flag_errors() {
        let result = BinaryArgs::new().parse(&argv(&["prog", "--unknown"]));
        assert!(matches!(result, Err(ParseError::UnknownFlag { .. })));
    }

    #[test]
    fn test_mode_equals() {
        let parsed = BinaryArgs::new()
            .with_mode()
            .parse(&argv(&["prog", "--mode=verify"]))
            .unwrap();
        assert_eq!(parsed.resource_mode, Some(ExecMode::Verify));
    }

    #[test]
    fn test_mode_space() {
        let parsed = BinaryArgs::new()
            .with_mode()
            .parse(&argv(&["prog", "--mode", "ensure"]))
            .unwrap();
        assert_eq!(parsed.resource_mode, Some(ExecMode::Ensure));
    }

    #[test]
    fn test_mode_missing_value() {
        let result = BinaryArgs::new()
            .with_mode()
            .parse(&argv(&["prog", "--mode"]));
        assert!(matches!(result, Err(ParseError::MissingValue { .. })));
    }

    #[test]
    fn test_mode_invalid_value() {
        let result = BinaryArgs::new()
            .with_mode()
            .parse(&argv(&["prog", "--mode=bogus"]));
        assert!(matches!(result, Err(ParseError::InvalidValue { .. })));
    }

    #[test]
    fn test_deprecated_check_flags_are_unknown() {
        let result_short = BinaryArgs::new().with_mode().parse(&argv(&["prog", "-c"]));
        let result_long = BinaryArgs::new()
            .with_mode()
            .parse(&argv(&["prog", "--check"]));
        assert!(matches!(result_short, Err(ParseError::UnknownFlag { .. })));
        assert!(matches!(result_long, Err(ParseError::UnknownFlag { .. })));
    }

    #[test]
    fn test_string_param_short() {
        let parsed = BinaryArgs::new()
            .with_string_param("path", Some('o'), Some("Makefile"))
            .parse(&argv(&["prog", "-o", "output.mk"]))
            .unwrap();
        assert_eq!(parsed.get_string("path"), Some("output.mk"));
    }

    #[test]
    fn test_string_param_default() {
        let parsed = BinaryArgs::new()
            .with_string_param("path", Some('o'), Some("Makefile"))
            .parse(&argv(&["prog"]))
            .unwrap();
        assert_eq!(parsed.get_string("path"), Some("Makefile"));
    }

    #[test]
    fn test_string_param_long() {
        let parsed = BinaryArgs::new()
            .with_string_param("path", Some('o'), None)
            .parse(&argv(&["prog", "--path", "foo"]))
            .unwrap();
        assert_eq!(parsed.get_string("path"), Some("foo"));
    }

    #[test]
    fn test_string_param_missing_value() {
        let result = BinaryArgs::new()
            .with_string_param("path", Some('o'), None)
            .parse(&argv(&["prog", "--path"]));
        assert!(matches!(result, Err(ParseError::MissingValue { .. })));
    }

    #[test]
    fn test_combined_flags() {
        let parsed = BinaryArgs::new()
            .with_mode()
            .with_string_param("path", Some('o'), Some("Makefile"))
            .parse(&argv(&["prog", "-n", "--mode=verify", "-o", "out.mk"]))
            .unwrap();
        assert!(parsed.dry_run);
        assert_eq!(parsed.resource_mode, Some(ExecMode::Verify));
        assert_eq!(parsed.get_string("path"), Some("out.mk"));
    }

    #[test]
    fn test_no_args() {
        let parsed = BinaryArgs::new().parse(&argv(&["prog"])).unwrap();
        assert!(!parsed.dry_run);
        assert!(!parsed.help);
        assert_eq!(parsed.resource_mode, None);
    }

    #[test]
    fn test_mode_without_enable_is_unknown() {
        let result = BinaryArgs::new().parse(&argv(&["prog", "--mode=verify"]));
        assert!(matches!(result, Err(ParseError::UnknownFlag { .. })));
    }

    #[test]
    fn test_string_param_no_default_absent() {
        let parsed = BinaryArgs::new()
            .with_string_param("path", Some('o'), None)
            .parse(&argv(&["prog"]))
            .unwrap();
        assert_eq!(parsed.get_string("path"), None);
    }

    #[test]
    fn test_string_param_uses_canonical_long_name() {
        let parsed = BinaryArgs::new()
            .with_string_param("output_dir", None, None)
            .parse(&argv(&["prog", "--output-dir", "generated"]))
            .unwrap();
        assert_eq!(parsed.get_string("output_dir"), Some("generated"));
    }

    #[test]
    fn test_step_mode_defaults_to_run() {
        let parsed = parse_step_mode(&argv(&["prog"])).unwrap();
        assert_eq!(parsed.subcommand, StepModeSubcommand::Run);
        assert!(parsed.args.is_empty());
    }

    #[test]
    fn test_step_mode_run_explicit() {
        let parsed = parse_step_mode(&argv(&["prog", "run", "-n"])).unwrap();
        assert_eq!(parsed.subcommand, StepModeSubcommand::Run);
        assert_eq!(parsed.args, vec!["-n".to_string()]);
    }

    #[test]
    fn test_step_mode_step_subcommand() {
        let parsed = parse_step_mode(&argv(&["prog", "step", "node_a", "-n"])).unwrap();
        assert_eq!(parsed.subcommand, StepModeSubcommand::Step);
        assert_eq!(parsed.args, vec!["node_a".to_string(), "-n".to_string()]);
    }

    #[test]
    fn test_step_mode_list_steps_subcommand() {
        let parsed = parse_step_mode(&argv(&["prog", "list-steps"])).unwrap();
        assert_eq!(parsed.subcommand, StepModeSubcommand::ListSteps);
        assert!(parsed.args.is_empty());
    }

    #[test]
    fn test_step_mode_flags_passthrough_to_run() {
        let parsed = parse_step_mode(&argv(&["prog", "--dry-run", "--mode=verify"])).unwrap();
        assert_eq!(parsed.subcommand, StepModeSubcommand::Run);
        assert_eq!(
            parsed.args,
            vec!["--dry-run".to_string(), "--mode=verify".to_string()]
        );
    }

    #[test]
    fn test_step_mode_unknown_subcommand_errors() {
        let err = parse_step_mode(&argv(&["prog", "deploy"])).unwrap_err();
        assert!(matches!(
            err,
            StepModeParseError::UnknownSubcommand { .. }
        ));
    }
}
