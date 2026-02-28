//! Shared step-mode subcommand parsing for generated CLIs.

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

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(args: &[&str]) -> Vec<String> {
        args.iter().map(|s| s.to_string()).collect()
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
        assert!(matches!(err, StepModeParseError::UnknownSubcommand { .. }));
    }
}
