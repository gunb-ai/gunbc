// Generated tests -- do not edit.
// Source module: extdeps.package_managers.apt

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
async fn test_apt___package_manager_show_exact() {
        // Signature: Apt__PackageManager.ShowExact(selection: String) -> (bool, String, String)
        let service = Apt__PackageManager::new(crate::dry_run::DryRunMode(true));
        let result = service.show_exact(compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Apt__PackageManager.ShowExact should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_apt___package_manager_simulate_install() {
        // Signature: Apt__PackageManager.SimulateInstall(selection: String) -> (bool, String, String)
        let service = Apt__PackageManager::new(crate::dry_run::DryRunMode(true));
        let result = service.simulate_install(compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Apt__PackageManager.SimulateInstall should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_apt___package_manager_install_exact_non_removing() {
        // Signature: Apt__PackageManager.InstallExactNonRemoving(selection: String) -> (bool, String, String)
        let service = Apt__PackageManager::new(crate::dry_run::DryRunMode(true));
        let result = service.install_exact_non_removing(compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Apt__PackageManager.InstallExactNonRemoving should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_apt___package_manager_remove() {
        // Signature: Apt__PackageManager.Remove(package: String) -> (bool, String, String)
        let service = Apt__PackageManager::new(crate::dry_run::DryRunMode(true));
        let result = service.remove(compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Apt__PackageManager.Remove should succeed in dry-run mode: {:?}", result.err());
}
}
