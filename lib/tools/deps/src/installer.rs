//! Platform-specific installers.

use crate::manifest::PlatformInstall;
use crate::platform::Platform;
use std::process::Command;

/// Installation method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallMethod {
    Apt,
    Brew,
    Cargo,
    Script,
    GithubRelease,
    Unknown,
}

impl InstallMethod {
    /// Parse an install method from a string.
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "apt" => InstallMethod::Apt,
            "brew" => InstallMethod::Brew,
            "cargo" | "cargo_install" => InstallMethod::Cargo,
            "script" => InstallMethod::Script,
            "github_release" => InstallMethod::GithubRelease,
            _ => InstallMethod::Unknown,
        }
    }
}

/// Tool installer.
pub struct Installer {
    platform: Platform,
}

impl Installer {
    /// Create an installer for a specific platform.
    pub fn for_platform(platform: Platform) -> Self {
        Self { platform }
    }

    /// Get the current platform.
    pub fn platform(&self) -> Platform {
        self.platform
    }

    /// Check if a tool is installed by running the verify command.
    ///
    /// Note: This is part of the tool installation infrastructure - it's allowed
    /// to use Command::new because it's implementing the verify step.
    #[allow(clippy::disallowed_methods)]
    pub fn is_installed(&self, verify_cmd: &str) -> bool {
        let parts: Vec<&str> = verify_cmd.split_whitespace().collect();
        if parts.is_empty() {
            return false;
        }

        let result = Command::new(parts[0])
            .args(&parts[1..])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        matches!(result, Ok(status) if status.success())
    }

    /// Generate the install command for a platform install configuration.
    pub fn generate_install_cmd(&self, install: &PlatformInstall) -> Result<String, String> {
        let method = InstallMethod::parse(&install.method);

        match method {
            InstallMethod::Apt => {
                if install.packages.is_empty() {
                    return Err("apt install requires packages".to_string());
                }
                Ok(format!(
                    "sudo apt-get update && sudo apt-get install -y {}",
                    install.packages.join(" ")
                ))
            }
            InstallMethod::Brew => {
                if install.packages.is_empty() {
                    return Err("brew install requires packages".to_string());
                }
                Ok(format!("brew install {}", install.packages.join(" ")))
            }
            InstallMethod::Cargo => {
                if install.packages.is_empty() {
                    return Err("cargo install requires packages".to_string());
                }
                Ok(format!("cargo install {}", install.packages.join(" ")))
            }
            InstallMethod::Script => install
                .script
                .clone()
                .ok_or_else(|| "script install requires script field".to_string()),
            InstallMethod::GithubRelease => {
                // Would need more complex handling for downloading releases
                Err("github_release method not yet implemented".to_string())
            }
            InstallMethod::Unknown => Err(format!("unknown install method: {}", install.method)),
        }
    }

    /// Generate an idempotent install script that wraps with verify check.
    pub fn generate_idempotent_script(
        &self,
        tool_name: &str,
        verify_cmd: &str,
        install_cmd: &str,
    ) -> String {
        format!(
            r#"# Install {} if not present
if {} >/dev/null 2>&1; then
    echo '{} is already installed'
else
    echo 'Installing {}...'
    {}
    if {} >/dev/null 2>&1; then
        echo '{} installed successfully'
    else
        echo 'Failed to install {}'
        exit 1
    fi
fi
"#,
            tool_name,
            verify_cmd,
            tool_name,
            tool_name,
            install_cmd,
            verify_cmd,
            tool_name,
            tool_name
        )
    }
}

impl Default for Installer {
    fn default() -> Self {
        Self::for_platform(Platform::detect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_install_method() {
        assert_eq!(InstallMethod::parse("apt"), InstallMethod::Apt);
        assert_eq!(InstallMethod::parse("brew"), InstallMethod::Brew);
        assert_eq!(InstallMethod::parse("cargo"), InstallMethod::Cargo);
        assert_eq!(InstallMethod::parse("script"), InstallMethod::Script);
        assert_eq!(InstallMethod::parse("unknown"), InstallMethod::Unknown);
    }

    #[test]
    fn test_is_installed_with_common_tools() {
        let installer = Installer::for_platform(Platform::detect());
        // 'echo' should be available on all platforms
        assert!(installer.is_installed("echo test"));
    }

    #[test]
    fn test_generate_apt_cmd() {
        let installer = Installer::for_platform(Platform::Linux);
        let install = PlatformInstall {
            method: "apt".to_string(),
            packages: vec!["curl".to_string(), "wget".to_string()],
            script: None,
            url: None,
        };

        let cmd = installer.generate_install_cmd(&install).unwrap();
        assert!(cmd.contains("apt-get install"));
        assert!(cmd.contains("curl"));
        assert!(cmd.contains("wget"));
    }

    #[test]
    fn test_generate_idempotent_script() {
        let installer = Installer::for_platform(Platform::detect());
        let script = installer.generate_idempotent_script(
            "gh",
            "gh --version",
            "sudo apt-get install -y gh",
        );

        assert!(script.contains("gh --version"));
        assert!(script.contains("already installed"));
        assert!(script.contains("Installing"));
    }
}
