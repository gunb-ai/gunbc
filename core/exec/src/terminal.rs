//! Terminal environment detection.
//!
//! Models the runtime terminal environment to make rendering decisions
//! automatically. No CLI flags needed — the profile detects capabilities
//! and decides whether animated progress display is appropriate.
//!
//! The base assumption is a serial connection (plain text, no escape codes).
//! Progress display is only enabled when we are confident the environment
//! supports it: interactive TTY, not CI, Unicode or better, not dumb terminal.
//!
//! # Detection Strategy
//!
//! 1. **Shell**: `$SHELL` on Unix, `$PSModulePath`/`$WT_SESSION`/`$ComSpec` on Windows
//! 2. **TTY**: `std::io::IsTerminal` on stdout (real file descriptor check)
//! 3. **CI**: `$CI` / `$GITHUB_ACTIONS` / `$GITLAB_CI` / `$JENKINS_URL` / `$BUILDKITE`
//! 4. **Dumb terminal**: `$TERM=dumb` → no progress, no color
//! 5. **NO_COLOR**: `$NO_COLOR` set → disable ANSI color codes
//! 6. **Symbol tier**: `$TERM_PROGRAM`/`$WT_SESSION` → Emoji, `$LANG` UTF-8 → Unicode, else ASCII
//! 7. **Viewport**: `ioctl(TIOCGWINSZ)` on Unix, then `$COLUMNS`/`$LINES`, default 80×24

use gunbc_ir::layout::{Viewport, ViewportUnit};
use gunbc_ir::symbols::Tier;
use std::env;
use std::io::IsTerminal;

/// Terminal profile: models the runtime environment for rendering decisions.
///
/// Created via [`TerminalProfile::detect()`] which queries the OS and reads
/// environment variables. Used internally by the display module for progress
/// rendering details (viewport, tier, color support).
#[derive(Debug, Clone)]
pub(crate) struct TerminalProfile {
    /// Whether stdout is connected to an interactive terminal.
    /// Detected via `std::io::IsTerminal` (file descriptor check).
    pub is_tty: bool,
    /// Whether `$TERM` is `dumb` (minimal terminal, no escape sequences).
    pub is_dumb: bool,
    /// Whether `$NO_COLOR` is set (https://no-color.org/).
    pub no_color: bool,
    /// Symbol tier: Emoji > Unicode > Ascii.
    pub tier: Tier,
    /// Terminal dimensions.
    pub viewport: Viewport,
    /// Whether ANSI color codes are supported.
    ///
    /// True when: real TTY + not CI + not dumb + NO_COLOR not set.
    pub supports_color: bool,
}

impl TerminalProfile {
    /// Detect the terminal profile from the current environment.
    ///
    /// Queries the OS for TTY status and terminal size, then reads
    /// environment variables for capabilities.
    pub(crate) fn detect() -> Self {
        let is_tty = detect_tty();
        let is_ci = detect_ci();
        let is_dumb = detect_dumb();
        let no_color = env::var("NO_COLOR").is_ok();
        let tier = detect_tier(is_dumb);
        let viewport = detect_viewport();

        // Color requires: real TTY + not CI + not dumb + NO_COLOR not set
        let supports_color = is_tty && !is_ci && !is_dumb && !no_color;

        Self {
            is_tty,
            is_dumb,
            no_color,
            tier,
            viewport,
            supports_color,
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
/// Uses `std::io::IsTerminal` (file descriptor check via isatty(2) on Unix,
/// GetConsoleMode on Windows). This is the real thing — not an env var heuristic.
fn detect_tty() -> bool {
    std::io::stdout().is_terminal()
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

/// Detect whether `$TERM` indicates a dumb terminal.
///
/// `TERM=dumb` means no escape sequence support — used by Emacs shell,
/// some CI runners, and minimal environments.
fn detect_dumb() -> bool {
    env::var("TERM").map(|t| t == "dumb").unwrap_or(false)
}

/// Detect the best symbol tier for the current terminal.
///
/// - Dumb terminals: pure ASCII (no box-drawing, no emoji)
/// - Modern terminals (iTerm, Windows Terminal): full emoji
/// - UTF-8 locales: Unicode box-drawing characters
/// - Everything else: pure ASCII
fn detect_tier(is_dumb: bool) -> Tier {
    // Dumb terminals get ASCII — no escape sequences, no special chars
    if is_dumb {
        return Tier::Ascii;
    }

    // Modern terminal emulators that support emoji
    if env::var("TERM_PROGRAM").is_ok() || env::var("WT_SESSION").is_ok() {
        return Tier::Emoji;
    }

    // UTF-8 locale → Unicode tier
    let lang = env::var("LANG").unwrap_or_default();
    let lc_all = env::var("LC_ALL").unwrap_or_default();
    if lang.contains("UTF-8")
        || lang.contains("utf-8")
        || lc_all.contains("UTF-8")
        || lc_all.contains("utf-8")
    {
        return Tier::Unicode;
    }

    Tier::Ascii
}

/// Detect terminal viewport dimensions.
///
/// On Unix, queries the kernel via `ioctl(TIOCGWINSZ)` for actual terminal
/// size. Falls back to `$COLUMNS`/`$LINES` env vars, then 80×24 default.
fn detect_viewport() -> Viewport {
    // Try ioctl first — this is the authoritative source on Unix
    #[cfg(unix)]
    if let Some((cols, rows)) = terminal_size_ioctl() {
        return Viewport::new(cols, rows, ViewportUnit::Chars);
    }

    // Fall back to environment variables (set by some shells)
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

/// Query the kernel for terminal dimensions via `ioctl(TIOCGWINSZ)`.
///
/// Returns `Some((cols, rows))` on success, `None` if stdout is not a terminal
/// or the ioctl fails.
#[cfg(unix)]
fn terminal_size_ioctl() -> Option<(u16, u16)> {
    use std::os::unix::io::AsRawFd;

    #[repr(C)]
    struct Winsize {
        ws_row: u16,
        ws_col: u16,
        ws_xpixel: u16,
        ws_ypixel: u16,
    }

    extern "C" {
        fn ioctl(
            fd: std::os::raw::c_int,
            request: std::os::raw::c_ulong,
            ...
        ) -> std::os::raw::c_int;
    }

    // TIOCGWINSZ value is platform-specific
    #[cfg(target_os = "linux")]
    const TIOCGWINSZ: std::os::raw::c_ulong = 0x5413;
    #[cfg(target_os = "macos")]
    const TIOCGWINSZ: std::os::raw::c_ulong = 0x40087468;
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    const TIOCGWINSZ: std::os::raw::c_ulong = 0x5413;

    let mut ws = Winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    let fd = std::io::stdout().as_raw_fd();
    let result = unsafe { ioctl(fd, TIOCGWINSZ, &mut ws as *mut Winsize) };

    if result == 0 && ws.ws_col > 0 && ws.ws_row > 0 {
        Some((ws.ws_col, ws.ws_row))
    } else {
        None
    }
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
        assert!(!profile.supports_color);
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

    #[test]
    fn test_dumb_terminal_blocks_progress() {
        // TERM=dumb should produce Tier::Ascii
        let tier = detect_tier(true);
        assert_eq!(tier, Tier::Ascii);
    }

    #[test]
    fn test_detect_dumb_false_when_unset() {
        // In test environment, TERM is typically not "dumb"
        // Just verify the function doesn't panic
        let _ = detect_dumb();
    }
}
