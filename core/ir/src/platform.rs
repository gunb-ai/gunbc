//! Canonical platform/target/execution environment model.
//!
//! This module provides the shared, typed representation for platform-aware
//! behavior across deps/tooling/runtime layers.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// CPU architecture component of a target triple.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Arch {
    X86_64,
    X86,
    Aarch64,
    Arm,
    Armv7,
    Mips,
    Mipsel,
    Mips64,
    Mips64el,
    Riscv64,
    Wasm32,
    Other(String),
}

impl Arch {
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "x86_64" | "amd64" => Self::X86_64,
            "x86" | "i686" | "i586" => Self::X86,
            "aarch64" | "arm64" => Self::Aarch64,
            "arm" => Self::Arm,
            "armv7" | "armv7l" => Self::Armv7,
            "mips" => Self::Mips,
            "mipsel" => Self::Mipsel,
            "mips64" => Self::Mips64,
            "mips64el" => Self::Mips64el,
            "riscv64" => Self::Riscv64,
            "wasm32" => Self::Wasm32,
            other => Self::Other(other.to_string()),
        }
    }

    pub fn as_token(&self) -> &str {
        match self {
            Self::X86_64 => "x86_64",
            Self::X86 => "x86",
            Self::Aarch64 => "aarch64",
            Self::Arm => "arm",
            Self::Armv7 => "armv7",
            Self::Mips => "mips",
            Self::Mipsel => "mipsel",
            Self::Mips64 => "mips64",
            Self::Mips64el => "mips64el",
            Self::Riscv64 => "riscv64",
            Self::Wasm32 => "wasm32",
            Self::Other(v) => v.as_str(),
        }
    }
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_token())
    }
}

impl FromStr for Arch {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse(s))
    }
}

/// Vendor component of a target triple.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Vendor {
    Unknown,
    Pc,
    Apple,
    W64,
    Other(String),
}

impl Vendor {
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "unknown" => Self::Unknown,
            "pc" => Self::Pc,
            "apple" => Self::Apple,
            "w64" => Self::W64,
            other => Self::Other(other.to_string()),
        }
    }

    pub fn as_token(&self) -> &str {
        match self {
            Self::Unknown => "unknown",
            Self::Pc => "pc",
            Self::Apple => "apple",
            Self::W64 => "w64",
            Self::Other(v) => v.as_str(),
        }
    }
}

impl fmt::Display for Vendor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_token())
    }
}

impl FromStr for Vendor {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse(s))
    }
}

/// Operating system component of a target triple.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Os {
    Linux,
    Macos,
    Windows,
    Freebsd,
    Android,
    Ios,
    Wasi,
    Other(String),
}

impl Os {
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "linux" => Self::Linux,
            "darwin" | "macos" | "osx" => Self::Macos,
            "windows" | "win32" | "win" => Self::Windows,
            "freebsd" => Self::Freebsd,
            "android" => Self::Android,
            "ios" => Self::Ios,
            "wasi" => Self::Wasi,
            other => Self::Other(other.to_string()),
        }
    }

    pub fn as_token(&self) -> &str {
        match self {
            Self::Linux => "linux",
            Self::Macos => "macos",
            Self::Windows => "windows",
            Self::Freebsd => "freebsd",
            Self::Android => "android",
            Self::Ios => "ios",
            Self::Wasi => "wasi",
            Self::Other(v) => v.as_str(),
        }
    }

    /// Parse DSL `Platform` variants into canonical OS tokens.
    ///
    /// Supports both the current spelling (`Macos`) and legacy spelling
    /// (`MacOS`) used in older DSL/docs snapshots.
    pub fn parse_dsl_platform(value: &str) -> Self {
        match value.trim() {
            "Linux" => Self::Linux,
            "Macos" | "MacOS" => Self::Macos,
            "Windows" => Self::Windows,
            other => Self::parse(other),
        }
    }

    /// Render this OS as a DSL `Platform` variant token.
    pub fn to_dsl_platform_variant(&self) -> String {
        match self {
            Self::Linux => "Linux".to_string(),
            Self::Macos => "Macos".to_string(),
            Self::Windows => "Windows".to_string(),
            _ => self.as_token().to_string(),
        }
    }
}

impl fmt::Display for Os {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_token())
    }
}

impl FromStr for Os {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse(s))
    }
}

/// ABI / environment component of a target triple.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AbiEnv {
    None,
    Gnu,
    GnuEabi,
    GnuEabihf,
    Musl,
    Msvc,
    Android,
    Eabi,
    Eabihf,
    Other(String),
}

impl AbiEnv {
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "" | "none" => Self::None,
            "gnu" => Self::Gnu,
            "gnueabi" => Self::GnuEabi,
            "gnueabihf" => Self::GnuEabihf,
            "musl" => Self::Musl,
            "msvc" => Self::Msvc,
            "android" => Self::Android,
            "eabi" => Self::Eabi,
            "eabihf" => Self::Eabihf,
            other => Self::Other(other.to_string()),
        }
    }

    pub fn as_token(&self) -> Option<&str> {
        match self {
            Self::None => None,
            Self::Gnu => Some("gnu"),
            Self::GnuEabi => Some("gnueabi"),
            Self::GnuEabihf => Some("gnueabihf"),
            Self::Musl => Some("musl"),
            Self::Msvc => Some("msvc"),
            Self::Android => Some("android"),
            Self::Eabi => Some("eabi"),
            Self::Eabihf => Some("eabihf"),
            Self::Other(v) => Some(v.as_str()),
        }
    }
}

impl fmt::Display for AbiEnv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.as_token() {
            Some(token) => write!(f, "{token}"),
            None => write!(f, "none"),
        }
    }
}

impl FromStr for AbiEnv {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse(s))
    }
}

/// Canonical target triple representation (`arch-vendor-os[-env]`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TargetTriple {
    pub arch: Arch,
    pub vendor: Vendor,
    pub os: Os,
    pub env: AbiEnv,
}

impl TargetTriple {
    pub fn new(arch: Arch, vendor: Vendor, os: Os, env: AbiEnv) -> Self {
        Self {
            arch,
            vendor,
            os,
            env,
        }
    }

    /// Detect a best-effort host triple for the current process.
    pub fn detect_host() -> Self {
        let arch = Arch::parse(std::env::consts::ARCH);
        let os = Os::parse(std::env::consts::OS);
        let vendor = detect_vendor();
        let env = detect_abi_env();
        Self {
            arch,
            vendor,
            os,
            env,
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        let segments: Vec<&str> = value
            .trim()
            .split('-')
            .filter(|segment| !segment.is_empty())
            .collect();
        if segments.len() < 3 {
            return Err(format!(
                "invalid target triple '{value}': expected arch-vendor-os[-env]"
            ));
        }

        let arch = Arch::parse(segments[0]);
        let vendor = Vendor::parse(segments[1]);
        let os = Os::parse(segments[2]);
        let env = if segments.len() > 3 {
            AbiEnv::parse(&segments[3..].join("-"))
        } else {
            AbiEnv::None
        };

        Ok(Self {
            arch,
            vendor,
            os,
            env,
        })
    }

    pub fn to_triple_string(&self) -> String {
        let mut base = format!("{}-{}-{}", self.arch, self.vendor, self.os);
        if let Some(env) = self.env.as_token() {
            base.push('-');
            base.push_str(env);
        }
        base
    }
}

impl fmt::Display for TargetTriple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_triple_string())
    }
}

impl FromStr for TargetTriple {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// Runtime execution environment over a host target.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionEnv {
    Native,
    Wsl,
    Container,
    Ci,
    Emulator,
}

impl ExecutionEnv {
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "native" => Self::Native,
            "wsl" => Self::Wsl,
            "container" | "docker" | "podman" => Self::Container,
            "ci" => Self::Ci,
            "emulator" | "qemu" => Self::Emulator,
            _ => Self::Native,
        }
    }

    pub fn as_token(&self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Wsl => "wsl",
            Self::Container => "container",
            Self::Ci => "ci",
            Self::Emulator => "emulator",
        }
    }
}

impl fmt::Display for ExecutionEnv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_token())
    }
}

impl FromStr for ExecutionEnv {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse(s))
    }
}

/// Full runtime platform descriptor: host triple + execution environment.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuntimePlatform {
    pub host: TargetTriple,
    pub env: ExecutionEnv,
}

impl RuntimePlatform {
    pub fn new(host: TargetTriple, env: ExecutionEnv) -> Self {
        Self { host, env }
    }

    pub fn detect_current() -> Self {
        Self {
            host: TargetTriple::detect_host(),
            env: detect_execution_env(),
        }
    }
}

/// Toolchain command surface for platform-specific build/run workflows.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ToolchainCommands {
    pub assembler: String,
    pub linker: String,
    pub emulator: Option<String>,
}

impl ToolchainCommands {
    /// Canonical MIPS Linux GNU toolchain command names.
    pub fn mips_linux_gnu() -> Self {
        Self {
            assembler: "mips-linux-gnu-as".to_string(),
            linker: "mips-linux-gnu-ld".to_string(),
            emulator: Some("qemu-mips".to_string()),
        }
    }
}

fn detect_execution_env() -> ExecutionEnv {
    if std::env::var("GUNBC_EXEC_ENV").is_ok() {
        return ExecutionEnv::parse(&std::env::var("GUNBC_EXEC_ENV").unwrap_or_default());
    }

    if std::env::var_os("WSL_DISTRO_NAME").is_some() || std::env::var_os("WSL_INTEROP").is_some() {
        return ExecutionEnv::Wsl;
    }
    if std::env::var_os("QEMU_LD_PREFIX").is_some() || std::env::var_os("GUNBC_EMULATOR").is_some()
    {
        return ExecutionEnv::Emulator;
    }
    if std::env::var_os("CI").is_some() {
        return ExecutionEnv::Ci;
    }
    if std::env::var_os("container").is_some() || std::env::var_os("DOCKER_CONTAINER").is_some() {
        return ExecutionEnv::Container;
    }

    ExecutionEnv::Native
}

fn detect_vendor() -> Vendor {
    if cfg!(target_vendor = "apple") {
        Vendor::Apple
    } else if cfg!(target_vendor = "pc") {
        Vendor::Pc
    } else {
        Vendor::Unknown
    }
}

fn detect_abi_env() -> AbiEnv {
    if cfg!(target_env = "gnu") {
        AbiEnv::Gnu
    } else if cfg!(target_env = "musl") {
        AbiEnv::Musl
    } else if cfg!(target_env = "msvc") {
        AbiEnv::Msvc
    } else {
        AbiEnv::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_triple_parse_round_trip() {
        let triple = TargetTriple::parse("x86_64-unknown-linux-gnu").expect("parse triple");
        assert_eq!(triple.arch, Arch::X86_64);
        assert_eq!(triple.vendor, Vendor::Unknown);
        assert_eq!(triple.os, Os::Linux);
        assert_eq!(triple.env, AbiEnv::Gnu);
        assert_eq!(triple.to_string(), "x86_64-unknown-linux-gnu");
    }

    #[test]
    fn target_triple_parse_without_env_uses_none() {
        let triple = TargetTriple::parse("aarch64-apple-darwin").expect("parse triple");
        assert_eq!(triple.arch, Arch::Aarch64);
        assert_eq!(triple.vendor, Vendor::Apple);
        assert_eq!(triple.os, Os::Macos);
        assert_eq!(triple.env, AbiEnv::None);
        assert_eq!(triple.to_string(), "aarch64-apple-macos");
    }

    #[test]
    fn detect_host_and_runtime_platform_are_usable() {
        let host = TargetTriple::detect_host();
        assert!(!host.to_string().is_empty());

        let runtime = RuntimePlatform::detect_current();
        assert!(!runtime.host.to_string().is_empty());
    }

    #[test]
    fn mips_toolchain_commands_are_defined() {
        let toolchain = ToolchainCommands::mips_linux_gnu();
        assert_eq!(toolchain.assembler, "mips-linux-gnu-as");
        assert_eq!(toolchain.linker, "mips-linux-gnu-ld");
        assert_eq!(toolchain.emulator.as_deref(), Some("qemu-mips"));
    }

    #[test]
    fn target_triple_conformance_linux_gnu_vs_musl() {
        let gnu = TargetTriple::parse("x86_64-unknown-linux-gnu").expect("parse gnu");
        let musl = TargetTriple::parse("x86_64-unknown-linux-musl").expect("parse musl");
        assert_eq!(gnu.arch, musl.arch);
        assert_eq!(gnu.vendor, musl.vendor);
        assert_eq!(gnu.os, musl.os);
        assert_eq!(gnu.env, AbiEnv::Gnu);
        assert_eq!(musl.env, AbiEnv::Musl);
    }

    #[test]
    fn target_triple_conformance_windows_msvc() {
        let triple = TargetTriple::parse("x86_64-pc-windows-msvc").expect("parse windows msvc");
        assert_eq!(triple.arch, Arch::X86_64);
        assert_eq!(triple.vendor, Vendor::Pc);
        assert_eq!(triple.os, Os::Windows);
        assert_eq!(triple.env, AbiEnv::Msvc);
        assert_eq!(triple.to_string(), "x86_64-pc-windows-msvc");
    }

    #[test]
    fn os_dsl_platform_adapter_accepts_legacy_and_current_spellings() {
        assert_eq!(Os::parse_dsl_platform("Linux"), Os::Linux);
        assert_eq!(Os::parse_dsl_platform("Macos"), Os::Macos);
        assert_eq!(Os::parse_dsl_platform("MacOS"), Os::Macos);
        assert_eq!(Os::parse_dsl_platform("Windows"), Os::Windows);
    }

    #[test]
    fn os_dsl_platform_adapter_round_trips_canonical_variants() {
        assert_eq!(Os::Linux.to_dsl_platform_variant(), "Linux");
        assert_eq!(Os::Macos.to_dsl_platform_variant(), "Macos");
        assert_eq!(Os::Windows.to_dsl_platform_variant(), "Windows");
    }
}
