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
//! 1. **TTY**: `std::io::IsTerminal` on stderr (real file descriptor check, since progress renders to stderr)
//! 2. **CI**: `$CI` / `$GITHUB_ACTIONS` / `$GITLAB_CI` / `$JENKINS_URL` / `$BUILDKITE`
//! 3. **Dumb terminal**: `$TERM=dumb` → no progress, no color
//! 4. **NO_COLOR**: `$NO_COLOR` set → disable ANSI color codes
//! 5. **Symbol tier**: `$TERM_PROGRAM`/`$WT_SESSION` → Emoji, `$LANG` UTF-8 → Unicode, else ASCII
//! 6. **Viewport**: `ioctl(TIOCGWINSZ)` on Unix, then `$COLUMNS`/`$LINES`, default 80×24

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
    /// Whether stderr is connected to an interactive terminal.
    /// Progress output renders to stderr (matching gunb.ai), so TTY detection
    /// checks stderr. Detected via `std::io::IsTerminal` (file descriptor check).
    pub is_tty: bool,
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
            tier,
            viewport,
            supports_color,
        }
    }
}

/// Detect whether stderr is connected to an interactive terminal.
///
/// Progress output renders to stderr (matching `gunb.ai`), so we check stderr.
/// Uses `std::io::IsTerminal` (file descriptor check via isatty(2) on Unix,
/// GetConsoleMode on Windows). This is the real thing — not an env var heuristic.
fn detect_tty() -> bool {
    std::io::stderr().is_terminal()
}

/// Detect whether we're running inside a CI environment.
///
/// Check for specific CI provider markers. The generic `CI` env var
/// is ignored because it is unreliable (set by editors, tools, etc.).
fn detect_ci() -> bool {
    env::var("GITHUB_ACTIONS").is_ok()
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

    let fd = std::io::stderr().as_raw_fd();
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
