//! Typed package-manager identities (M15).

use serde::{Deserialize, Serialize};

/// Strict package-manager identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageManagerId {
    Apt,
    Apk,
    Brew,
    Cargo,
    Script,
    GithubRelease,
}

impl PackageManagerId {
    /// Strict parse path: unknown IDs fail closed.
    pub fn parse_strict(raw: &str) -> Result<Self, String> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "apt" => Ok(Self::Apt),
            "apk" => Ok(Self::Apk),
            "brew" => Ok(Self::Brew),
            "cargo" | "cargo_install" => Ok(Self::Cargo),
            "script" => Ok(Self::Script),
            "github_release" => Ok(Self::GithubRelease),
            other => Err(format!("unknown package manager id: {other}")),
        }
    }

    /// Compatibility parse path for legacy/lossy boundaries.
    pub fn parse_compat(raw: &str) -> Option<Self> {
        Self::parse_strict(raw).ok()
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Apt => "apt",
            Self::Apk => "apk",
            Self::Brew => "brew",
            Self::Cargo => "cargo",
            Self::Script => "script",
            Self::GithubRelease => "github_release",
        }
    }
}

impl std::fmt::Display for PackageManagerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::PackageManagerId;

    #[test]
    fn strict_parse_supports_known_ids() {
        assert_eq!(
            PackageManagerId::parse_strict("apt").unwrap(),
            PackageManagerId::Apt
        );
        assert_eq!(
            PackageManagerId::parse_strict("apk").unwrap(),
            PackageManagerId::Apk
        );
        assert_eq!(
            PackageManagerId::parse_strict("brew").unwrap(),
            PackageManagerId::Brew
        );
        assert_eq!(
            PackageManagerId::parse_strict("cargo").unwrap(),
            PackageManagerId::Cargo
        );
        assert_eq!(
            PackageManagerId::parse_strict("cargo_install").unwrap(),
            PackageManagerId::Cargo
        );
        assert_eq!(
            PackageManagerId::parse_strict("script").unwrap(),
            PackageManagerId::Script
        );
        assert_eq!(
            PackageManagerId::parse_strict("github_release").unwrap(),
            PackageManagerId::GithubRelease
        );
    }

    #[test]
    fn strict_parse_fails_closed_for_unknown_ids() {
        assert!(PackageManagerId::parse_strict("nix").is_err());
        assert!(PackageManagerId::parse_strict("unknown").is_err());
    }
}
