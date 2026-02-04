//! Filesystem-aware filename sanitization.
//!
//! Models real filesystem naming constraints as data, following the same
//! `const` static pattern as `ToolDef`/`PlatformDef` in `core/ir`.
//!
//! # Design
//!
//! Each filesystem has hard rules (characters it cannot store, reserved names,
//! trailing character restrictions). Rather than hardcoding sanitization logic,
//! we model each filesystem's actual constraints as data. Sanitization then
//! *emerges* from composing these constraints — cross-platform safety is the
//! union of all target filesystems' forbidden sets.
//!
//! # Filesystems Modeled
//!
//! | Filesystem | Platforms | Key Constraints |
//! |------------|-----------|-----------------|
//! | ext4       | Linux (default) | Only `/` forbidden, case-sensitive |
//! | XFS        | RHEL/CentOS | Same as ext4 |
//! | Btrfs      | Linux (COW) | Same as ext4 |
//! | ZFS        | Linux/FreeBSD | Same as ext4 |
//! | APFS       | macOS 10.13+ | `/` and `:` forbidden, case-insensitive |
//! | HFS+       | macOS legacy | Same as APFS |
//! | NTFS       | Windows | 9 forbidden chars, 22 reserved names, case-insensitive |
//! | FAT32      | USB/SD cards | Same restrictions as NTFS |
//! | exFAT      | Large USB/SD | Same restrictions as NTFS |
//!
//! # Example
//!
//! ```
//! use gunbc_primitives::filename::{sanitize, CROSS_PLATFORM, EXT4, NTFS, APFS};
//!
//! // Single filesystem — only `/` replaced on ext4
//! let safe = sanitize("claude/branch:name", &[&EXT4], '-');
//! assert_eq!(safe, "claude-branch:name");
//!
//! // Cross-platform — union of ext4 + NTFS + APFS constraints
//! let safe = sanitize("claude/branch:name", CROSS_PLATFORM, '-');
//! assert_eq!(safe, "claude-branch-name");
//! ```

// ============================================================================
// Filesystem Model
// ============================================================================

/// A filesystem's filename constraints modeled as data.
///
/// Captures the naming rules for a single filesystem. All fields are static
/// data — no runtime allocation needed for the well-known constants.
///
/// This is analogous to `ToolDef` and `PlatformDef` in `core/ir` — constraints
/// as data rather than code.
#[derive(Debug, Clone, PartialEq)]
pub struct Filesystem {
    /// Unique identifier (e.g., "ext4", "ntfs", "apfs").
    pub id: &'static str,

    /// Characters the filesystem cannot store in filename components.
    /// Null byte (0x00) is implicitly forbidden — Rust strings can't contain it.
    pub forbidden_chars: &'static [char],

    /// Whether ASCII control characters (0x01–0x1F) are forbidden.
    /// True for NTFS/FAT32/exFAT; false for POSIX filesystems (where control
    /// chars are technically valid but never desirable).
    pub forbid_control_chars: bool,

    /// Names reserved by the OS/filesystem layer. On case-insensitive
    /// filesystems, these are checked case-insensitively.
    /// Example: CON, PRN, NUL, COM1–COM9, LPT1–LPT9 on NTFS.
    pub reserved_names: &'static [&'static str],

    /// Maximum filename component length in bytes.
    pub max_component_bytes: usize,

    /// Whether the filesystem distinguishes upper/lower case.
    pub case_sensitive: bool,

    /// Characters forbidden at the trailing position of a filename component.
    /// Example: `.` and ` ` on NTFS/FAT.
    pub forbidden_trailing: &'static [char],
}

impl Filesystem {
    /// Check if a character is forbidden on this filesystem.
    pub fn is_forbidden(&self, c: char) -> bool {
        if self.forbidden_chars.contains(&c) {
            return true;
        }
        if self.forbid_control_chars && (c as u32) >= 1 && (c as u32) <= 0x1F {
            return true;
        }
        false
    }

    /// Check if a name (or its stem before the first `.`) is reserved.
    pub fn is_reserved_name(&self, name: &str) -> bool {
        let stem = name.split('.').next().unwrap_or(name);
        if self.case_sensitive {
            self.reserved_names.contains(&stem)
        } else {
            self.reserved_names
                .iter()
                .any(|r| r.eq_ignore_ascii_case(stem))
        }
    }

    /// Check if a character is forbidden at the trailing position.
    pub fn is_forbidden_trailing(&self, c: char) -> bool {
        self.forbidden_trailing.contains(&c)
    }
}

// ============================================================================
// Linux Filesystems
// ============================================================================

/// ext4 — default Linux filesystem (Debian, Ubuntu, Arch, etc.).
///
/// The most permissive common filesystem. Only `/` is forbidden as a filename
/// character (it's the path separator). Case-sensitive. No reserved names.
/// 255-byte component limit.
pub static EXT4: Filesystem = Filesystem {
    id: "ext4",
    forbidden_chars: &['/'],
    forbid_control_chars: false,
    reserved_names: &[],
    max_component_bytes: 255,
    case_sensitive: true,
    forbidden_trailing: &[],
};

/// XFS — default RHEL/CentOS filesystem.
///
/// Same practical filename constraints as ext4.
pub static XFS: Filesystem = Filesystem {
    id: "xfs",
    forbidden_chars: &['/'],
    forbid_control_chars: false,
    reserved_names: &[],
    max_component_bytes: 255,
    case_sensitive: true,
    forbidden_trailing: &[],
};

/// Btrfs — Linux copy-on-write filesystem.
///
/// Same practical filename constraints as ext4.
pub static BTRFS: Filesystem = Filesystem {
    id: "btrfs",
    forbidden_chars: &['/'],
    forbid_control_chars: false,
    reserved_names: &[],
    max_component_bytes: 255,
    case_sensitive: true,
    forbidden_trailing: &[],
};

/// ZFS — Linux/FreeBSD advanced filesystem.
///
/// Same practical filename constraints as ext4.
pub static ZFS: Filesystem = Filesystem {
    id: "zfs",
    forbidden_chars: &['/'],
    forbid_control_chars: false,
    reserved_names: &[],
    max_component_bytes: 255,
    case_sensitive: true,
    forbidden_trailing: &[],
};

// ============================================================================
// macOS Filesystems
// ============================================================================

/// APFS — Apple File System (macOS 10.13+, iOS 10.3+).
///
/// On disk, APFS forbids `:` (and null). The macOS POSIX layer swaps `/`
/// and `:` for display — Finder shows `/` where the filesystem stores `:`.
/// Both characters are effectively forbidden for safe cross-layer use.
///
/// Case-insensitive by default. macOS supports case-sensitive APFS volumes,
/// but the default install is case-insensitive (case-preserving).
pub static APFS: Filesystem = Filesystem {
    id: "apfs",
    forbidden_chars: &['/', ':'],
    forbid_control_chars: false,
    reserved_names: &[],
    max_component_bytes: 255,
    case_sensitive: false,
    forbidden_trailing: &[],
};

/// HFS+ — legacy macOS filesystem (pre-10.13).
///
/// Same practical constraints as APFS. macOS transparently converts
/// HFS+ volumes to APFS on upgrade.
pub static HFS_PLUS: Filesystem = Filesystem {
    id: "hfs+",
    forbidden_chars: &['/', ':'],
    forbid_control_chars: false,
    reserved_names: &[],
    max_component_bytes: 255,
    case_sensitive: false,
    forbidden_trailing: &[],
};

// ============================================================================
// Windows Filesystems
// ============================================================================

/// Windows reserved device names.
///
/// These names (with or without extensions) cannot be used as filenames
/// on NTFS, FAT32, or exFAT when accessed through the Windows API layer.
/// Matching is case-insensitive: `con`, `CON`, `Con` are all reserved.
const WINDOWS_RESERVED_NAMES: &[&str] = &[
    "CON", "PRN", "AUX", "NUL",
    "COM0", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
    "LPT0", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Characters forbidden on Windows filesystems.
///
/// These 9 printable characters cannot appear in NTFS/FAT32/exFAT filenames.
/// Additionally, control characters (0x01–0x1F) are forbidden (modeled via
/// `forbid_control_chars`).
const WINDOWS_FORBIDDEN_CHARS: &[char] = &['/', '\\', ':', '*', '?', '"', '<', '>', '|'];

/// Characters forbidden at the end of a Windows filename.
const WINDOWS_FORBIDDEN_TRAILING: &[char] = &['.', ' '];

/// NTFS — Windows NT File System.
///
/// The most restrictive common filesystem:
/// - 9 forbidden printable characters plus control chars 0x01–0x1F
/// - 22 reserved device names (CON, PRN, AUX, NUL, COM0–COM9, LPT0–LPT9)
/// - Cannot end with `.` or space
/// - Case-insensitive (case-preserving)
/// - 255-byte component limit
pub static NTFS: Filesystem = Filesystem {
    id: "ntfs",
    forbidden_chars: WINDOWS_FORBIDDEN_CHARS,
    forbid_control_chars: true,
    reserved_names: WINDOWS_RESERVED_NAMES,
    max_component_bytes: 255,
    case_sensitive: false,
    forbidden_trailing: WINDOWS_FORBIDDEN_TRAILING,
};

/// FAT32 — legacy Windows / USB / SD card filesystem.
///
/// Same filename constraints as NTFS (enforced by the Windows API layer).
/// 4 GB file size limit but same naming rules.
pub static FAT32: Filesystem = Filesystem {
    id: "fat32",
    forbidden_chars: WINDOWS_FORBIDDEN_CHARS,
    forbid_control_chars: true,
    reserved_names: WINDOWS_RESERVED_NAMES,
    max_component_bytes: 255,
    case_sensitive: false,
    forbidden_trailing: WINDOWS_FORBIDDEN_TRAILING,
};

/// exFAT — extended FAT for large files/SD cards.
///
/// Same filename constraints as NTFS/FAT32.
pub static EXFAT: Filesystem = Filesystem {
    id: "exfat",
    forbidden_chars: WINDOWS_FORBIDDEN_CHARS,
    forbid_control_chars: true,
    reserved_names: WINDOWS_RESERVED_NAMES,
    max_component_bytes: 255,
    case_sensitive: false,
    forbidden_trailing: WINDOWS_FORBIDDEN_TRAILING,
};

// ============================================================================
// Platform-to-Filesystem Mapping
// ============================================================================

/// Convenience set: all major platform default filesystems.
///
/// ext4 (Linux) + NTFS (Windows) + APFS (macOS).
/// Sanitizing against this set produces filenames safe on all three platforms.
pub static CROSS_PLATFORM: &[&Filesystem] = &[&EXT4, &NTFS, &APFS];

// ============================================================================
// Constraint-Driven Sanitization
// ============================================================================

/// Sanitize a string for use as a filename component on all given filesystems.
///
/// Composes constraints from all target filesystems:
/// 1. **Forbidden chars**: union across all filesystems → replaced with `replacement`
/// 2. **Control chars**: if any filesystem forbids them → replaced
/// 3. **Trailing chars**: union of forbidden trailing → stripped
/// 4. **Consecutive replacements**: collapsed into one
/// 5. **Reserved names**: union across all filesystems → prefixed with `_`
/// 6. **Length**: truncated to minimum `max_component_bytes` at UTF-8 boundary
/// 7. **Empty result**: falls back to `"untitled"`
///
/// The `replacement` character itself must not be forbidden on any target
/// filesystem (typically `-` or `_`).
pub fn sanitize(input: &str, filesystems: &[&Filesystem], replacement: char) -> String {
    if filesystems.is_empty() {
        return input.to_string();
    }

    // Compose: should we forbid control chars?
    let forbid_control = filesystems.iter().any(|fs| fs.forbid_control_chars);

    // Step 1: Replace forbidden chars
    let replaced: String = input
        .chars()
        .map(|c| {
            // Check control chars first (avoids scanning forbidden_chars arrays)
            if forbid_control && (c as u32) >= 1 && (c as u32) <= 0x1F {
                return replacement;
            }
            // Check each filesystem's forbidden chars
            for fs in filesystems {
                if fs.forbidden_chars.contains(&c) {
                    return replacement;
                }
            }
            c
        })
        .collect();

    // Step 2: Collapse consecutive replacement chars
    let mut result = String::with_capacity(replaced.len());
    let mut prev_was_replacement = false;
    for c in replaced.chars() {
        if c == replacement {
            if !prev_was_replacement {
                result.push(c);
            }
            prev_was_replacement = true;
        } else {
            result.push(c);
            prev_was_replacement = false;
        }
    }

    // Step 3: Strip forbidden trailing chars (union of all filesystems + replacement)
    let is_trimmable = |c: char| -> bool {
        if c == replacement {
            return true;
        }
        filesystems
            .iter()
            .any(|fs| fs.forbidden_trailing.contains(&c))
    };
    let trimmed = result.trim_matches(is_trimmable);
    result = trimmed.to_string();

    // Step 4: Check reserved names (union across all filesystems)
    let is_reserved = filesystems.iter().any(|fs| fs.is_reserved_name(&result));
    if is_reserved {
        result = format!("_{}", result);
    }

    // Step 5: Truncate to minimum max_component_bytes
    let max_bytes = filesystems
        .iter()
        .map(|fs| fs.max_component_bytes)
        .min()
        .unwrap_or(255);

    if result.len() > max_bytes {
        // Truncate at a valid UTF-8 boundary
        let mut end = max_bytes;
        while end > 0 && !result.is_char_boundary(end) {
            end -= 1;
        }
        result.truncate(end);
        // Re-strip trailing forbidden chars after truncation
        let trimmed = result.trim_end_matches(is_trimmable);
        result = trimmed.to_string();
    }

    // Step 6: Fallback for empty result
    if result.is_empty() {
        "untitled".to_string()
    } else {
        result
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ====================================================================
    // Filesystem model tests
    // ====================================================================

    #[test]
    fn test_ext4_allows_most_chars() {
        assert!(!EXT4.is_forbidden(':'));
        assert!(!EXT4.is_forbidden('\\'));
        assert!(!EXT4.is_forbidden('*'));
        assert!(!EXT4.is_forbidden(' '));
        // Only `/` is forbidden
        assert!(EXT4.is_forbidden('/'));
    }

    #[test]
    fn test_apfs_forbids_colon_and_slash() {
        assert!(APFS.is_forbidden('/'));
        assert!(APFS.is_forbidden(':'));
        assert!(!APFS.is_forbidden('\\'));
        assert!(!APFS.is_forbidden('*'));
    }

    #[test]
    fn test_ntfs_forbids_nine_printable_chars() {
        let forbidden = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
        for c in &forbidden {
            assert!(NTFS.is_forbidden(*c), "NTFS should forbid '{}'", c);
        }
        // But allows other special chars
        assert!(!NTFS.is_forbidden('#'));
        assert!(!NTFS.is_forbidden('@'));
        assert!(!NTFS.is_forbidden('!'));
    }

    #[test]
    fn test_ntfs_forbids_control_chars() {
        assert!(NTFS.is_forbidden('\x01'));
        assert!(NTFS.is_forbidden('\x1F'));
        // But not printable chars that aren't in the forbidden list
        assert!(!NTFS.is_forbidden('a'));
    }

    #[test]
    fn test_ext4_allows_control_chars() {
        // Technically valid on ext4, even if bad practice
        assert!(!EXT4.is_forbidden('\x01'));
        assert!(!EXT4.is_forbidden('\x1F'));
    }

    #[test]
    fn test_ntfs_reserved_names() {
        assert!(NTFS.is_reserved_name("CON"));
        assert!(NTFS.is_reserved_name("con")); // case-insensitive
        assert!(NTFS.is_reserved_name("Con"));
        assert!(NTFS.is_reserved_name("PRN"));
        assert!(NTFS.is_reserved_name("AUX"));
        assert!(NTFS.is_reserved_name("NUL"));
        assert!(NTFS.is_reserved_name("COM1"));
        assert!(NTFS.is_reserved_name("LPT9"));
        // With extension — stem is still reserved
        assert!(NTFS.is_reserved_name("CON.md"));
        assert!(NTFS.is_reserved_name("nul.txt"));
    }

    #[test]
    fn test_ntfs_non_reserved_names() {
        assert!(!NTFS.is_reserved_name("CONSTANT"));
        assert!(!NTFS.is_reserved_name("console"));
        assert!(!NTFS.is_reserved_name("auxiliary"));
        assert!(!NTFS.is_reserved_name("hello"));
    }

    #[test]
    fn test_ext4_has_no_reserved_names() {
        assert!(!EXT4.is_reserved_name("CON"));
        assert!(!EXT4.is_reserved_name("NUL"));
    }

    #[test]
    fn test_ntfs_forbidden_trailing() {
        assert!(NTFS.is_forbidden_trailing('.'));
        assert!(NTFS.is_forbidden_trailing(' '));
        assert!(!NTFS.is_forbidden_trailing('-'));
    }

    #[test]
    fn test_ext4_no_forbidden_trailing() {
        assert!(!EXT4.is_forbidden_trailing('.'));
        assert!(!EXT4.is_forbidden_trailing(' '));
    }

    #[test]
    fn test_filesystem_ids_are_distinct() {
        let all = [&EXT4, &XFS, &BTRFS, &ZFS, &APFS, &HFS_PLUS, &NTFS, &FAT32, &EXFAT];
        let mut ids: Vec<&str> = all.iter().map(|fs| fs.id).collect();
        let before = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), before, "filesystem IDs must be unique");
    }

    #[test]
    fn test_case_sensitivity_matches_reality() {
        // Linux filesystems: case-sensitive
        assert!(EXT4.case_sensitive);
        assert!(XFS.case_sensitive);
        assert!(BTRFS.case_sensitive);
        assert!(ZFS.case_sensitive);
        // macOS: case-insensitive by default
        assert!(!APFS.case_sensitive);
        assert!(!HFS_PLUS.case_sensitive);
        // Windows: case-insensitive
        assert!(!NTFS.case_sensitive);
        assert!(!FAT32.case_sensitive);
        assert!(!EXFAT.case_sensitive);
    }

    // ====================================================================
    // Sanitize: ext4-only (most permissive)
    // ====================================================================

    #[test]
    fn test_sanitize_ext4_only_replaces_slash() {
        let s = sanitize("claude/branch:name", &[&EXT4], '-');
        assert_eq!(s, "claude-branch:name");
    }

    #[test]
    fn test_sanitize_ext4_preserves_windows_special_chars() {
        let s = sanitize("file<name>with*stars", &[&EXT4], '-');
        assert_eq!(s, "file<name>with*stars");
    }

    // ====================================================================
    // Sanitize: APFS-only
    // ====================================================================

    #[test]
    fn test_sanitize_apfs_replaces_slash_and_colon() {
        let s = sanitize("refs/heads:main", &[&APFS], '-');
        assert_eq!(s, "refs-heads-main");
    }

    #[test]
    fn test_sanitize_apfs_preserves_backslash() {
        // Backslash is valid on APFS (unlike NTFS)
        let s = sanitize("path\\file", &[&APFS], '-');
        assert_eq!(s, "path\\file");
    }

    // ====================================================================
    // Sanitize: NTFS-only (most restrictive)
    // ====================================================================

    #[test]
    fn test_sanitize_ntfs_replaces_all_forbidden() {
        let s = sanitize("a/b\\c:d*e?f\"g<h>i|j", &[&NTFS], '-');
        assert_eq!(s, "a-b-c-d-e-f-g-h-i-j");
    }

    #[test]
    fn test_sanitize_ntfs_strips_trailing_dot_and_space() {
        let s = sanitize("filename. ", &[&NTFS], '-');
        assert_eq!(s, "filename");
    }

    #[test]
    fn test_sanitize_ntfs_prefixes_reserved_names() {
        let s = sanitize("CON", &[&NTFS], '-');
        assert_eq!(s, "_CON");

        let s = sanitize("nul", &[&NTFS], '-');
        assert_eq!(s, "_nul");
    }

    #[test]
    fn test_sanitize_ntfs_reserved_with_extension() {
        // "CON.md" — stem "CON" is reserved
        let s = sanitize("CON.md", &[&NTFS], '-');
        assert_eq!(s, "_CON.md");
    }

    #[test]
    fn test_sanitize_ntfs_non_reserved_prefix() {
        // "CONSTANT" stem is "CONSTANT", not "CON"
        let s = sanitize("CONSTANT", &[&NTFS], '-');
        assert_eq!(s, "CONSTANT");
    }

    #[test]
    fn test_sanitize_ntfs_control_chars() {
        let s = sanitize("hello\x01world", &[&NTFS], '-');
        assert_eq!(s, "hello-world");
    }

    // ====================================================================
    // Sanitize: cross-platform (ext4 + NTFS + APFS)
    // ====================================================================

    #[test]
    fn test_sanitize_cross_platform_basic() {
        let s = sanitize("main", CROSS_PLATFORM, '-');
        assert_eq!(s, "main");
    }

    #[test]
    fn test_sanitize_cross_platform_git_branch() {
        let s = sanitize("claude/branch-name", CROSS_PLATFORM, '-');
        assert_eq!(s, "claude-branch-name");
    }

    #[test]
    fn test_sanitize_cross_platform_colon_from_apfs() {
        // `:` is forbidden on APFS, so cross-platform must catch it
        let s = sanitize("branch:name", CROSS_PLATFORM, '-');
        assert_eq!(s, "branch-name");
    }

    #[test]
    fn test_sanitize_cross_platform_windows_chars() {
        let s = sanitize("a*b?c<d>e|f", CROSS_PLATFORM, '-');
        assert_eq!(s, "a-b-c-d-e-f");
    }

    #[test]
    fn test_sanitize_cross_platform_collapses_consecutive() {
        let s = sanitize("a//b", CROSS_PLATFORM, '-');
        assert_eq!(s, "a-b");

        let s = sanitize("a///b", CROSS_PLATFORM, '-');
        assert_eq!(s, "a-b");

        let s = sanitize("a/:/b", CROSS_PLATFORM, '-');
        assert_eq!(s, "a-b");
    }

    #[test]
    fn test_sanitize_cross_platform_trims_replacement() {
        let s = sanitize("/branch", CROSS_PLATFORM, '-');
        assert_eq!(s, "branch");

        let s = sanitize("branch/", CROSS_PLATFORM, '-');
        assert_eq!(s, "branch");

        let s = sanitize("/branch/", CROSS_PLATFORM, '-');
        assert_eq!(s, "branch");
    }

    #[test]
    fn test_sanitize_cross_platform_trims_trailing_dot_space() {
        // Trailing dot/space forbidden on NTFS
        let s = sanitize("file. ", CROSS_PLATFORM, '-');
        assert_eq!(s, "file");
    }

    #[test]
    fn test_sanitize_cross_platform_reserved_con() {
        let s = sanitize("CON", CROSS_PLATFORM, '-');
        assert_eq!(s, "_CON");
    }

    #[test]
    fn test_sanitize_cross_platform_preserves_dots_underscores() {
        let s = sanitize("v1.0.0_rc1", CROSS_PLATFORM, '-');
        assert_eq!(s, "v1.0.0_rc1");
    }

    // ====================================================================
    // Sanitize: edge cases
    // ====================================================================

    #[test]
    fn test_sanitize_empty_input() {
        let s = sanitize("", CROSS_PLATFORM, '-');
        assert_eq!(s, "untitled");
    }

    #[test]
    fn test_sanitize_all_forbidden() {
        let s = sanitize("///", CROSS_PLATFORM, '-');
        assert_eq!(s, "untitled");
    }

    #[test]
    fn test_sanitize_only_spaces() {
        // Spaces are not forbidden chars on any FS, but are forbidden trailing on NTFS
        // "   " → trimming removes leading/trailing replacement (but spaces aren't replacement)
        // Actually spaces aren't in forbidden_chars, so they pass through.
        // But trailing space is forbidden on NTFS, so trailing ones get stripped.
        // Leading space isn't forbidden trailing, so it stays unless it's the replacement char.
        let s = sanitize("   ", CROSS_PLATFORM, '-');
        // Leading spaces: not the replacement char, not forbidden trailing → kept
        // Trailing spaces: forbidden trailing on NTFS → stripped
        // But trim_matches trims both ends for the replacement char
        // Actually our trim_matches trims both leading and trailing.
        // Spaces: not the replacement char '-'. trim_matches checks is_trimmable:
        //   c == '-' → no. forbidden_trailing.contains → NTFS has ' ' → yes.
        // So leading+trailing spaces are trimmed. Result = "".
        assert_eq!(s, "untitled");
    }

    #[test]
    fn test_sanitize_no_filesystems() {
        let s = sanitize("anything/goes:here", &[], '-');
        assert_eq!(s, "anything/goes:here");
    }

    #[test]
    fn test_sanitize_underscore_replacement() {
        let s = sanitize("claude/branch", CROSS_PLATFORM, '_');
        assert_eq!(s, "claude_branch");
    }

    #[test]
    fn test_sanitize_preserves_unicode() {
        let s = sanitize("feature/日本語", CROSS_PLATFORM, '-');
        assert_eq!(s, "feature-日本語");
    }

    #[test]
    fn test_sanitize_long_filename_truncated() {
        let long = "a".repeat(300);
        let s = sanitize(&long, CROSS_PLATFORM, '-');
        assert_eq!(s.len(), 255);
        assert!(s.chars().all(|c| c == 'a'));
    }

    #[test]
    fn test_sanitize_truncation_respects_utf8_boundary() {
        // 254 ASCII bytes + one 2-byte UTF-8 char = 256 bytes → must truncate
        let mut input = "a".repeat(254);
        input.push('ä'); // 2 bytes in UTF-8
        assert_eq!(input.len(), 256);

        let s = sanitize(&input, CROSS_PLATFORM, '-');
        assert!(s.len() <= 255);
        assert!(s.is_char_boundary(s.len())); // valid UTF-8
        // Should truncate to 254 'a's (dropping the 'ä' which doesn't fit)
        assert_eq!(s.len(), 254);
    }

    // ====================================================================
    // Sanitize: real git branch patterns
    // ====================================================================

    #[test]
    fn test_sanitize_feature_branch() {
        let s = sanitize("feature/add-login", CROSS_PLATFORM, '-');
        assert_eq!(s, "feature-add-login");
    }

    #[test]
    fn test_sanitize_deeply_nested_branch() {
        let s = sanitize("user/team/project/feature", CROSS_PLATFORM, '-');
        assert_eq!(s, "user-team-project-feature");
    }

    #[test]
    fn test_sanitize_branch_with_at_sign() {
        // @ is valid on all filesystems
        let s = sanitize("user@feature", CROSS_PLATFORM, '-');
        assert_eq!(s, "user@feature");
    }

    #[test]
    fn test_sanitize_release_version() {
        let s = sanitize("release/v2.0_rc1", CROSS_PLATFORM, '-');
        assert_eq!(s, "release-v2.0_rc1");
    }

    #[test]
    fn test_sanitize_dependabot_branch() {
        let s = sanitize("dependabot/npm_and_yarn/lodash-4.17.21", CROSS_PLATFORM, '-');
        assert_eq!(s, "dependabot-npm_and_yarn-lodash-4.17.21");
    }

    // ====================================================================
    // Cross-platform composition property tests
    // ====================================================================

    #[test]
    fn test_cross_platform_is_superset_of_individual() {
        // Anything safe for cross-platform must be safe for each individual FS
        let inputs = [
            "hello/world",
            "file:name",
            "a*b?c",
            "CON",
            "test. ",
            "normal",
        ];

        for input in &inputs {
            let cross = sanitize(input, CROSS_PLATFORM, '-');
            let ext4_only = sanitize(&cross, &[&EXT4], '-');
            let ntfs_only = sanitize(&cross, &[&NTFS], '-');
            let apfs_only = sanitize(&cross, &[&APFS], '-');

            // After cross-platform sanitization, re-sanitizing for any
            // individual FS should be a no-op (idempotent)
            assert_eq!(
                cross, ext4_only,
                "cross-platform result '{}' not safe on ext4 for input '{}'",
                cross, input
            );
            assert_eq!(
                cross, ntfs_only,
                "cross-platform result '{}' not safe on NTFS for input '{}'",
                cross, input
            );
            assert_eq!(
                cross, apfs_only,
                "cross-platform result '{}' not safe on APFS for input '{}'",
                cross, input
            );
        }
    }

    #[test]
    fn test_sanitize_is_idempotent() {
        let inputs = [
            "claude/branch-name",
            "feature/foo:bar",
            "CON.md",
            "a///b",
            "test. ",
        ];

        for input in &inputs {
            let once = sanitize(input, CROSS_PLATFORM, '-');
            let twice = sanitize(&once, CROSS_PLATFORM, '-');
            assert_eq!(
                once, twice,
                "sanitize not idempotent for input '{}': '{}' → '{}'",
                input, once, twice
            );
        }
    }
}
