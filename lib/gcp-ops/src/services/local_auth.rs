//! Local developer auth service interface.
//!
//! Models interactive `gcloud auth login --update-adc` as a typed service call
//! while still executing via the existing Shell transport boundary.

use gunbc_ir::transport::ShellRequest;

/// CLI behavior metadata for local auth methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CliMethodMeta {
    /// CLI binary name.
    pub command: &'static str,
    /// Base CLI args (without optional flags).
    pub args: &'static [&'static str],
    /// Whether the command expects interactive user involvement.
    pub interactive: bool,
    /// Whether retrying is idempotent.
    pub idempotent: bool,
}

/// Metadata for `gcloud auth login --update-adc`.
pub const LOGIN_UPDATE_ADC_META: CliMethodMeta = CliMethodMeta {
    command: "gcloud",
    args: &["auth", "login", "--update-adc"],
    interactive: true,
    idempotent: true,
};

/// Options for interactive gcloud login.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GcloudLoginOptions {
    /// Stream login prompts and URLs directly to the terminal.
    pub passthrough_stdio: bool,
}

impl Default for GcloudLoginOptions {
    fn default() -> Self {
        Self {
            passthrough_stdio: true,
        }
    }
}

impl GcloudLoginOptions {
    /// Resolve options from environment overrides.
    ///
    /// - `GUNBC_GCLOUD_PASSTHROUGH=0|false|no|off` (default: passthrough on)
    pub fn from_env() -> Self {
        let passthrough_stdio = read_bool_env("GUNBC_GCLOUD_PASSTHROUGH").unwrap_or(true);
        Self { passthrough_stdio }
    }
}

/// Local auth service interface.
pub trait LocalAuthService {
    /// Build an interactive login request that updates ADC credentials.
    fn login_update_adc(&self, options: GcloudLoginOptions) -> ShellRequest;
}

/// gcloud CLI implementation of local auth.
#[derive(Debug, Clone, Copy, Default)]
pub struct GcloudCli;

impl LocalAuthService for GcloudCli {
    fn login_update_adc(&self, options: GcloudLoginOptions) -> ShellRequest {
        let mut req = ShellRequest::new(LOGIN_UPDATE_ADC_META.command)
            .args(LOGIN_UPDATE_ADC_META.args.iter().copied())
            .passthrough(options.passthrough_stdio);

        // In remote IDE terminals, BROWSER often points to a helper that can
        // open the host browser. Mirror it into CLOUDSDK_BROWSER when unset.
        let has_cloudsdk_browser = std::env::var("CLOUDSDK_BROWSER")
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false);
        if !has_cloudsdk_browser {
            if let Ok(browser) = std::env::var("BROWSER") {
                let browser = browser.trim();
                if !browser.is_empty() {
                    req = req.env("CLOUDSDK_BROWSER", browser);
                }
            }
        }

        req
    }
}

fn read_bool_env(name: &str) -> Option<bool> {
    std::env::var(name)
        .ok()
        .as_deref()
        .map(str::trim)
        .and_then(parse_bool)
}

fn parse_bool(raw: &str) -> Option<bool> {
    if raw.eq_ignore_ascii_case("1")
        || raw.eq_ignore_ascii_case("true")
        || raw.eq_ignore_ascii_case("yes")
        || raw.eq_ignore_ascii_case("on")
    {
        return Some(true);
    }
    if raw.eq_ignore_ascii_case("0")
        || raw.eq_ignore_ascii_case("false")
        || raw.eq_ignore_ascii_case("no")
        || raw.eq_ignore_ascii_case("off")
    {
        return Some(false);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bool_accepts_common_true_values() {
        assert_eq!(parse_bool("1"), Some(true));
        assert_eq!(parse_bool("true"), Some(true));
        assert_eq!(parse_bool("YES"), Some(true));
        assert_eq!(parse_bool("On"), Some(true));
    }

    #[test]
    fn parse_bool_accepts_common_false_values() {
        assert_eq!(parse_bool("0"), Some(false));
        assert_eq!(parse_bool("false"), Some(false));
        assert_eq!(parse_bool("NO"), Some(false));
        assert_eq!(parse_bool("off"), Some(false));
    }

    #[test]
    fn gcloud_login_request_defaults_to_passthrough() {
        let req = GcloudCli.login_update_adc(GcloudLoginOptions::default());
        assert_eq!(req.command, "gcloud");
        assert_eq!(req.args[0..3], ["auth", "login", "--update-adc"]);
        assert!(!req.args.iter().any(|a| a == "--no-launch-browser"));
        assert!(req.passthrough);
    }
}
