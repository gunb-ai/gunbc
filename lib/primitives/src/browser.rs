//! Browser-open command resolution utilities.

use gunbc_ir::transport::ShellRequest;
use gunbc_ir::{ExecutionEnv, Os, RuntimePlatform};
use std::path::Path;

/// Resolve a browser-open shell request for a runtime platform.
///
/// Returns `None` for no-browser environments (CI/container/emulator).
pub fn browser_open_request(file_path: &str, runtime: &RuntimePlatform) -> Option<ShellRequest> {
    if matches!(
        runtime.env,
        ExecutionEnv::Ci | ExecutionEnv::Container | ExecutionEnv::Emulator
    ) {
        return None;
    }

    if runtime.env == ExecutionEnv::Wsl {
        let abs_path = absolutize(file_path);
        return Some(ShellRequest::new("wslview").arg(abs_path.to_string_lossy().into_owned()));
    }

    let request = match runtime.host.os {
        Os::Macos => ShellRequest::new("open").arg(file_path),
        Os::Windows => ShellRequest::new("cmd")
            .arg("/C")
            .arg("start")
            .arg("")
            .arg(file_path),
        _ => ShellRequest::new("xdg-open").arg(file_path),
    };
    Some(request)
}

fn absolutize(path: &str) -> std::path::PathBuf {
    let p = Path::new(path);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(p))
            .unwrap_or_else(|_| p.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{AbiEnv, Arch, TargetTriple, Vendor};

    fn runtime(os: Os, env: ExecutionEnv) -> RuntimePlatform {
        RuntimePlatform::new(
            TargetTriple::new(Arch::X86_64, Vendor::Unknown, os, AbiEnv::Gnu),
            env,
        )
    }

    #[test]
    fn resolves_wsl_to_wslview() {
        let req =
            browser_open_request("target/report.html", &runtime(Os::Linux, ExecutionEnv::Wsl))
                .expect("wsl should produce request");
        assert_eq!(req.command, "wslview");
        assert_eq!(req.args.len(), 1);
        assert!(req.args[0].contains("report.html"));
    }

    #[test]
    fn resolves_macos_to_open() {
        let req = browser_open_request(
            "/tmp/report.html",
            &runtime(Os::Macos, ExecutionEnv::Native),
        )
        .expect("macOS should produce request");
        assert_eq!(req.command, "open");
        assert_eq!(req.args, vec!["/tmp/report.html".to_string()]);
    }

    #[test]
    fn resolves_linux_to_xdg_open() {
        let req = browser_open_request(
            "/tmp/report.html",
            &runtime(Os::Linux, ExecutionEnv::Native),
        )
        .expect("linux should produce request");
        assert_eq!(req.command, "xdg-open");
        assert_eq!(req.args, vec!["/tmp/report.html".to_string()]);
    }

    #[test]
    fn resolves_windows_to_cmd_start() {
        let req = browser_open_request(
            "C:\\tmp\\report.html",
            &runtime(Os::Windows, ExecutionEnv::Native),
        )
        .expect("windows should produce request");
        assert_eq!(req.command, "cmd");
        assert_eq!(
            req.args,
            vec![
                "/C".to_string(),
                "start".to_string(),
                "".to_string(),
                "C:\\tmp\\report.html".to_string()
            ]
        );
    }

    #[test]
    fn skips_no_browser_environments() {
        assert!(
            browser_open_request("/tmp/report.html", &runtime(Os::Linux, ExecutionEnv::Ci))
                .is_none()
        );
        assert!(browser_open_request(
            "/tmp/report.html",
            &runtime(Os::Linux, ExecutionEnv::Container)
        )
        .is_none());
        assert!(browser_open_request(
            "/tmp/report.html",
            &runtime(Os::Linux, ExecutionEnv::Emulator)
        )
        .is_none());
    }
}
