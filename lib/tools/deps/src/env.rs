//! Environment ops for deps graphs.

use crate::platform::Platform;
use gunbc_exec::{env_single_output, EnvNode, ExecError};
use gunbc_ir::Value;
use std::collections::HashMap;

/// Environment toggle enabling strict dry-run behavior for deps env nodes.
pub const STRICT_DRY_RUN_ENV: &str = "GUNBC_STRICT_DRY_RUN";

/// Platform environment — detects current platform.
#[derive(Debug, Clone, Copy)]
pub struct PlatformEnv;

impl PlatformEnv {
    pub fn output_port(&self) -> &'static str {
        "platform"
    }

    pub fn mock_outputs(&self) -> HashMap<String, Value> {
        self.mock_output_map()
    }

    fn outputs(&self) -> HashMap<String, Value> {
        let platform = Platform::detect();
        env_single_output(self.output_port(), platform)
    }

    fn mock_output_map(&self) -> HashMap<String, Value> {
        let platform = if strict_dry_run_enabled() {
            Platform::Unknown
        } else {
            Platform::Linux
        };
        env_single_output(self.output_port(), platform)
    }
}

impl EnvNode for PlatformEnv {
    fn env_outputs(&self) -> Result<HashMap<String, Value>, ExecError> {
        Ok(self.outputs())
    }

    fn mock_outputs(&self) -> HashMap<String, Value> {
        self.mock_output_map()
    }
}

pub fn strict_dry_run_enabled() -> bool {
    std::env::var(STRICT_DRY_RUN_ENV)
        .ok()
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    #[test]
    fn strict_dry_run_flag_controls_platform_mock_default() {
        with_env_lock(|| {
            let env = PlatformEnv;

            std::env::remove_var(STRICT_DRY_RUN_ENV);
            let non_strict = env.mock_outputs();
            assert_eq!(
                non_strict
                    .get(env.output_port())
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
                "linux"
            );

            std::env::set_var(STRICT_DRY_RUN_ENV, "true");
            let strict = env.mock_outputs();
            assert_eq!(
                strict
                    .get(env.output_port())
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
                "unknown"
            );
        });
    }

    fn with_env_lock<F>(f: F)
    where
        F: FnOnce() + std::panic::UnwindSafe,
    {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let result = std::panic::catch_unwind(f);
        std::env::remove_var(STRICT_DRY_RUN_ENV);
        if let Err(panic) = result {
            std::panic::resume_unwind(panic);
        }
    }
}
