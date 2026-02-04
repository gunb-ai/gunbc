//! Terminal environment detection.
//!
//! Models the runtime terminal environment to make rendering decisions
//! automatically. No CLI flags needed — the profile detects capabilities
//! and decides whether animated progress display is appropriate.
//!
//! # Detection Strategy
//!
//! 1. **Shell**: `$SHELL` on Unix, `$PSModulePath`/`$WT_SESSION`/`$ComSpec` on Windows
//! 2. **TTY**: `$TERM` set (Unix convention for interactive terminals)
//! 3. **CI**: `$CI` set → never show progress (plain structured output)
//! 4. **Symbol tier**: `$TERM_PROGRAM`/`$WT_SESSION` → Emoji, `$LANG` UTF-8 → Unicode, else ASCII
//! 5. **Viewport**: `$COLUMNS`/`$LINES` with 80×24 fallback

use gunbc_ir::layout::{Viewport, ViewportUnit};
use gunbc_ir::symbols::Tier;
use std::env;

/// Shell environment type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Cmd,
    Unknown(String),
}

/// Terminal profile: models the runtime environment for rendering decisions.
///
/// Created via [`TerminalProfile::detect()`] which reads environment variables.
/// The [`supports_progress`](Self::supports_progress) field is the single
/// decision point for whether to show animated progress display.
#[derive(Debug, Clone)]
pub struct TerminalProfile {
    /// Detected shell.
    pub shell: Shell,
    /// Whether stdout is connected to an interactive terminal.
    pub is_tty: bool,
    /// Whether running inside a CI system (GitHub Actions, GitLab CI, etc.).
    pub is_ci: bool,
    /// Symbol tier: Emoji > Unicode > Ascii.
    pub tier: Tier,
    /// Terminal dimensions.
    pub viewport: Viewport,
    /// Whether animated progress display is supported.
    ///
    /// True when: interactive TTY + not CI + Unicode or better.
    /// False otherwise (pipes, CI, dumb terminals).
    pub supports_progress: bool,
}

impl TerminalProfile {
    /// Detect the terminal profile from the current environment.
    ///
    /// Reads standard environment variables to determine shell type,
    /// TTY status, CI context, symbol capabilities, and viewport size.
    pub fn detect() -> Self {
        let shell = detect_shell();
        let is_tty = detect_tty();
        let is_ci = detect_ci();
        let tier = detect_tier();
        let viewport = detect_viewport();

        // Progress requires: interactive terminal + not CI + at least Unicode
        let supports_progress = is_tty && !is_ci && !matches!(tier, Tier::Ascii);

        Self {
            shell,
            is_tty,
            is_ci,
            tier,
            viewport,
            supports_progress,
        }
    }

    /// Create a non-interactive profile for testing or explicit plain output.
    pub fn plain() -> Self {
        Self {
            shell: Shell::Unknown("none".into()),
            is_tty: false,
            is_ci: false,
            tier: Tier::Ascii,
            viewport: Viewport::new(80, 24, ViewportUnit::Chars),
            supports_progress: false,
        }
    }
}

/// Detect the current shell from environment variables.
fn detect_shell() -> Shell {
    // Unix: $SHELL is the login shell path
    if let Ok(shell) = env::var("SHELL") {
        return match shell.rsplit('/').next().unwrap_or("") {
            "bash" => Shell::Bash,
            "zsh" => Shell::Zsh,
            "fish" => Shell::Fish,
            other => Shell::Unknown(other.to_string()),
        };
    }

    // Windows: PowerShell sets PSModulePath
    if env::var("PSModulePath").is_ok() {
        return Shell::PowerShell;
    }

    // Windows: cmd.exe sets ComSpec but not PSModulePath
    if env::var("ComSpec").is_ok() {
        return Shell::Cmd;
    }

    Shell::Unknown(String::new())
}

/// Detect whether stdout is connected to an interactive terminal.
///
/// Uses the `$TERM` variable as a proxy: if set, we're in a terminal.
/// Pipes and redirections don't set `$TERM`.
fn detect_tty() -> bool {
    env::var("TERM").is_ok()
}

/// Detect whether we're running inside a CI environment.
///
/// Most CI systems set `$CI=true`. We also check provider-specific
/// variables for robustness.
fn detect_ci() -> bool {
    env::var("CI").is_ok()
        || env::var("GITHUB_ACTIONS").is_ok()
        || env::var("GITLAB_CI").is_ok()
        || env::var("JENKINS_URL").is_ok()
        || env::var("BUILDKITE").is_ok()
}

/// Detect the best symbol tier for the current terminal.
///
/// - Modern terminals (iTerm, Windows Terminal): full emoji
/// - UTF-8 locales: Unicode box-drawing characters
/// - Everything else: pure ASCII
fn detect_tier() -> Tier {
    // Modern terminal emulators that support emoji
    if env::var("TERM_PROGRAM").is_ok() || env::var("WT_SESSION").is_ok() {
        return Tier::Emoji;
    }

    // UTF-8 locale → Unicode tier
    let lang = env::var("LANG").unwrap_or_default();
    let lc_all = env::var("LC_ALL").unwrap_or_default();
    if lang.contains("UTF-8") || lang.contains("utf-8")
        || lc_all.contains("UTF-8") || lc_all.contains("utf-8")
    {
        return Tier::Unicode;
    }

    Tier::Ascii
}

/// Detect terminal viewport dimensions.
///
/// Reads `$COLUMNS` and `$LINES`, falling back to 80×24.
fn detect_viewport() -> Viewport {
    let cols = env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(80);
    let rows = env::var("LINES")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(24);

    Viewport::new(cols, rows, ViewportUnit::Chars)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_profile() {
        let profile = TerminalProfile::plain();
        assert!(!profile.is_tty);
        assert!(!profile.is_ci);
        assert!(!profile.supports_progress);
        assert_eq!(profile.tier, Tier::Ascii);
        assert_eq!(profile.shell, Shell::Unknown("none".into()));
    }

    #[test]
    fn test_shell_display() {
        // Ensure Shell variants are distinguishable
        assert_ne!(Shell::Bash, Shell::Zsh);
        assert_ne!(Shell::PowerShell, Shell::Cmd);
        assert_eq!(Shell::Unknown("sh".into()), Shell::Unknown("sh".into()));
    }

    #[test]
    fn test_detect_produces_valid_viewport() {
        let profile = TerminalProfile::detect();
        assert!(profile.viewport.width > 0);
        assert!(profile.viewport.height > 0);
    }
}
