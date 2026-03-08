//! Platform-specific installers.

use crate::manifest::PlatformInstall;
use crate::package_manager::PackageManagerId;
use crate::platform::Platform;

/// Backward-compatible alias for install method identity.
pub type InstallMethod = PackageManagerId;

/// Typed installation plan for one dependency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallPlan {
    pub package_manager: PackageManagerId,
    pub packages: Vec<String>,
    pub script: Option<String>,
    pub url: Option<String>,
}

impl InstallPlan {
    /// Strict parse from legacy `PlatformInstall`.
    pub fn from_platform_install_strict(install: &PlatformInstall) -> Result<Self, String> {
        let package_manager = PackageManagerId::parse_strict(&install.method)?;
        let plan = Self {
            package_manager,
            packages: install.packages.clone(),
            script: install.script.clone(),
            url: install.url.clone(),
        };
        plan.validate()?;
        Ok(plan)
    }

    /// Compatibility adapter for legacy manifests (lossy parse boundary).
    pub fn from_platform_install_compat(install: &PlatformInstall) -> Option<Self> {
        let package_manager = PackageManagerId::parse_compat(&install.method)?;
        let plan = Self {
            package_manager,
            packages: install.packages.clone(),
            script: install.script.clone(),
            url: install.url.clone(),
        };
        plan.validate().ok()?;
        Some(plan)
    }

    fn validate(&self) -> Result<(), String> {
        match self.package_manager {
            PackageManagerId::Apt
            | PackageManagerId::Apk
            | PackageManagerId::Brew
            | PackageManagerId::Cargo => {
                if self.packages.is_empty() {
                    return Err(format!(
                        "{} install requires non-empty packages",
                        self.package_manager
                    ));
                }
            }
            PackageManagerId::Script => {
                if self.script.is_none() {
                    return Err("script install requires script field".to_string());
                }
            }
            PackageManagerId::GithubRelease => {
                if self.url.is_none() {
                    return Err("github_release install requires url field".to_string());
                }
            }
        }
        Ok(())
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

    /// Generate the install command for a strict typed install plan.
    pub fn generate_install_cmd_for_plan(&self, plan: &InstallPlan) -> Result<String, String> {
        match plan.package_manager {
            PackageManagerId::Apt => Ok(format!(
                "sudo apt-get update && sudo apt-get install -y {}",
                plan.packages.join(" ")
            )),
            PackageManagerId::Apk => Ok(format!(
                "sudo apk add --no-cache {}",
                plan.packages.join(" ")
            )),
            PackageManagerId::Brew => Ok(format!("brew install {}", plan.packages.join(" "))),
            PackageManagerId::Cargo => match &plan.url {
                Some(url) => Ok(format!(
                    "cargo install --git {} {}",
                    url,
                    plan.packages.join(" ")
                )),
                None => Ok(format!("cargo install {}", plan.packages.join(" "))),
            },
            PackageManagerId::Script => plan
                .script
                .clone()
                .ok_or_else(|| "script install requires script field".to_string()),
            PackageManagerId::GithubRelease => Err(format!(
                "github_release install requires dedicated downloader implementation (url={})",
                plan.url.clone().unwrap_or_default()
            )),
        }
    }

    /// Generate install command from legacy manifest install config (strict parse path).
    pub fn generate_install_cmd(&self, install: &PlatformInstall) -> Result<String, String> {
        let plan = InstallPlan::from_platform_install_strict(install)?;
        self.generate_install_cmd_for_plan(&plan)
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
    fn strict_plan_parse_fails_closed_for_unknown_method() {
        let install = PlatformInstall {
            method: "nix".to_string(),
            packages: vec!["jq".to_string()],
            script: None,
            url: None,
        };
        assert!(InstallPlan::from_platform_install_strict(&install).is_err());
    }

    #[test]
    fn generate_apt_cmd() {
        let installer = Installer::for_platform(Platform::Linux);
        let install = PlatformInstall {
            method: "apt".to_string(),
            packages: vec!["curl".to_string(), "wget".to_string()],
            script: None,
            url: None,
        };

        let cmd = installer
            .generate_install_cmd(&install)
            .expect("apt command");
        assert!(cmd.contains("apt-get install"));
        assert!(cmd.contains("curl"));
        assert!(cmd.contains("wget"));
    }

    #[test]
    fn preserve_required_script_field() {
        let install = PlatformInstall {
            method: "script".to_string(),
            packages: vec![],
            script: Some("echo install".to_string()),
            url: None,
        };
        let plan = InstallPlan::from_platform_install_strict(&install).expect("script plan");
        assert_eq!(plan.script.as_deref(), Some("echo install"));
    }

    #[test]
    fn script_plan_without_script_fails() {
        let install = PlatformInstall {
            method: "script".to_string(),
            packages: vec![],
            script: None,
            url: None,
        };
        assert!(InstallPlan::from_platform_install_strict(&install).is_err());
    }

    #[test]
    fn generate_idempotent_script() {
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
