//! Platform detection.

use gunbc_ir::resource::{AccessMode, Resource, ResourceId, ResourceKind};
use gunbc_ir::Value;
use gunbc_ir::{ExecutionEnv, Os, RuntimePlatform, TargetTriple};
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
        let runtime = RuntimePlatform::detect_current();
        Self::from(runtime.host.os)
    }

    /// Parse a platform from a string.
    pub fn parse(s: &str) -> Self {
        Self::from(Os::parse(s))
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

impl From<Platform> for Os {
    fn from(value: Platform) -> Self {
        match value {
            Platform::Linux => Os::Linux,
            Platform::Macos => Os::Macos,
            Platform::Windows => Os::Windows,
            Platform::Unknown => Os::Other("unknown".to_string()),
        }
    }
}

impl From<Os> for Platform {
    fn from(value: Os) -> Self {
        match value {
            Os::Linux => Platform::Linux,
            Os::Macos => Platform::Macos,
            Os::Windows => Platform::Windows,
            Os::Other(_) | Os::Freebsd | Os::Android | Os::Ios | Os::Wasi => Platform::Unknown,
        }
    }
}

impl From<Platform> for RuntimePlatform {
    fn from(value: Platform) -> Self {
        let mut host = TargetTriple::detect_host();
        host.os = value.into();
        RuntimePlatform::new(host, ExecutionEnv::Native)
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

    #[test]
    fn test_platform_os_compat_adapters() {
        assert_eq!(Platform::from(Os::Linux), Platform::Linux);
        assert_eq!(Platform::from(Os::Macos), Platform::Macos);
        assert_eq!(Platform::from(Os::Windows), Platform::Windows);
        assert_eq!(Platform::from(Os::Freebsd), Platform::Unknown);

        assert_eq!(Os::from(Platform::Linux), Os::Linux);
        assert_eq!(Os::from(Platform::Macos), Os::Macos);
        assert_eq!(Os::from(Platform::Windows), Os::Windows);
        assert_eq!(
            Os::from(Platform::Unknown),
            Os::Other("unknown".to_string())
        );
    }

    #[test]
    fn test_platform_to_runtime_platform_adapter() {
        let runtime = RuntimePlatform::from(Platform::Linux);
        assert_eq!(runtime.host.os, Os::Linux);
        assert_eq!(runtime.env, ExecutionEnv::Native);
    }
}
