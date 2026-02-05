//! Platform detection.

use gunbc_ir::resource::{AccessMode, Resource, ResourceId, ResourceKind};
use gunbc_ir::Value;
use serde::{Deserialize, Serialize};

/// Target platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Linux,
    Macos,
    Windows,
    Unknown,
}

impl Platform {
    /// Detect the current host platform.
    pub fn detect() -> Self {
        #[cfg(target_os = "linux")]
        return Platform::Linux;

        #[cfg(target_os = "macos")]
        return Platform::Macos;

        #[cfg(target_os = "windows")]
        return Platform::Windows;

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        return Platform::Unknown;
    }

    /// Parse a platform from a string.
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "linux" => Platform::Linux,
            "macos" | "darwin" | "osx" => Platform::Macos,
            "windows" | "win32" | "win" => Platform::Windows,
            _ => Platform::Unknown,
        }
    }

    /// Get the platform name as a string.
    pub fn name(&self) -> &'static str {
        match self {
            Platform::Linux => "linux",
            Platform::Macos => "macos",
            Platform::Windows => "windows",
            Platform::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Resource for Platform {
    fn resource_id(&self) -> ResourceId {
        ResourceId::new("platform")
    }

    fn access_mode(&self) -> AccessMode {
        AccessMode::Read
    }

    fn kind(&self) -> ResourceKind {
        ResourceKind::Observation
    }
}

impl From<Platform> for Value {
    fn from(val: Platform) -> Self {
        Value::Str(val.name().to_string())
    }
}

impl TryFrom<&Value> for Platform {
    type Error = String;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let s = value
            .as_str()
            .ok_or_else(|| "expected string for Platform".to_string())?;
        Ok(Platform::parse(s))
    }
}

impl TryFrom<Value> for Platform {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Platform::try_from(&value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_platform() {
        assert_eq!(Platform::parse("linux"), Platform::Linux);
        assert_eq!(Platform::parse("LINUX"), Platform::Linux);
        assert_eq!(Platform::parse("macos"), Platform::Macos);
        assert_eq!(Platform::parse("darwin"), Platform::Macos);
        assert_eq!(Platform::parse("windows"), Platform::Windows);
        assert_eq!(Platform::parse("win32"), Platform::Windows);
        assert_eq!(Platform::parse("unknown"), Platform::Unknown);
    }

    #[test]
    fn test_detect_returns_known_platform() {
        let platform = Platform::detect();
        // Should be one of the known platforms on any dev machine
        assert!(matches!(
            platform,
            Platform::Linux | Platform::Macos | Platform::Windows
        ));
    }
}
