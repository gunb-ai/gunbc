// Generated tests -- do not edit.
// Source module: extdeps.systemd.systemctl

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
async fn test_systemd___systemctl_stop() {
        // Signature: Systemd__Systemctl.Stop(unit: String) -> (bool, String, String)
        let service = Systemd__Systemctl::new(crate::dry_run::DryRunMode(true));
        let result = service.stop(compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Systemd__Systemctl.Stop should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_systemd___systemctl_stop_user_unit() {
        // Signature: Systemd__Systemctl.StopUserUnit(unit: String) -> (ShellOutcome, bool, String)
        let service = Systemd__Systemctl::new(crate::dry_run::DryRunMode(true));
        let result = service.stop_user_unit(compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Systemd__Systemctl.StopUserUnit should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_systemd___systemctl_reset_failed_user_unit() {
        // Signature: Systemd__Systemctl.ResetFailedUserUnit(unit: String) -> (ShellOutcome, bool, String)
        let service = Systemd__Systemctl::new(crate::dry_run::DryRunMode(true));
        let result = service.reset_failed_user_unit(compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Systemd__Systemctl.ResetFailedUserUnit should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_systemd___systemctl_reset_failed_unit() {
        // Signature: Systemd__Systemctl.ResetFailedUnit(unit: String) -> (bool, String)
        let service = Systemd__Systemctl::new(crate::dry_run::DryRunMode(true));
        let result = service.reset_failed_unit(compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Systemd__Systemctl.ResetFailedUnit should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_systemd___systemctl_kill_unit_term() {
        // Signature: Systemd__Systemctl.KillUnitTerm(unit: String) -> (bool, String)
        let service = Systemd__Systemctl::new(crate::dry_run::DryRunMode(true));
        let result = service.kill_unit_term(compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Systemd__Systemctl.KillUnitTerm should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_systemd___systemctl_mask_now() {
        // Signature: Systemd__Systemctl.MaskNow(unit: String) -> (bool, String, String)
        let service = Systemd__Systemctl::new(crate::dry_run::DryRunMode(true));
        let result = service.mask_now(compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Systemd__Systemctl.MaskNow should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_systemd___systemctl_mask_runtime() {
        // Signature: Systemd__Systemctl.MaskRuntime(unit: String) -> (bool, String, String)
        let service = Systemd__Systemctl::new(crate::dry_run::DryRunMode(true));
        let result = service.mask_runtime(compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Systemd__Systemctl.MaskRuntime should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_systemd___systemctl_reboot() {
        // Signature: Systemd__Systemctl.Reboot() -> (bool, String)
        let service = Systemd__Systemctl::new(crate::dry_run::DryRunMode(true));
        let result = service.reboot().await;
        assert!(result.is_ok(), "Systemd__Systemctl.Reboot should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_systemd___systemctl_set_property() {
        // Signature: Systemd__Systemctl.SetProperty(unit: String, property: String, value: String) -> (bool, String)
        let service = Systemd__Systemctl::new(crate::dry_run::DryRunMode(true));
        let result = service.set_property(compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Systemd__Systemctl.SetProperty should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_systemd___systemctl_is_active() {
        // Signature: Systemd__Systemctl.IsActive(unit: String) -> (String, bool)
        let service = Systemd__Systemctl::new(crate::dry_run::DryRunMode(true));
        let result = service.is_active(compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Systemd__Systemctl.IsActive should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_systemd___systemctl_show_property() {
        // Signature: Systemd__Systemctl.ShowProperty(unit: String, property: String) -> (String, bool)
        let service = Systemd__Systemctl::new(crate::dry_run::DryRunMode(true));
        let result = service.show_property(compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Systemd__Systemctl.ShowProperty should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_systemd___systemctl_show_user_property() {
        // Signature: Systemd__Systemctl.ShowUserProperty(unit: String, property: String) -> (ShellOutcome, String, bool)
        let service = Systemd__Systemctl::new(crate::dry_run::DryRunMode(true));
        let result = service.show_user_property(compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Systemd__Systemctl.ShowUserProperty should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_systemd___systemctl_show_load_state() {
        // Signature: Systemd__Systemctl.ShowLoadState(unit: String) -> (String, bool)
        let service = Systemd__Systemctl::new(crate::dry_run::DryRunMode(true));
        let result = service.show_load_state(compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Systemd__Systemctl.ShowLoadState should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_systemd___systemctl_list_units() {
        // Signature: Systemd__Systemctl.ListUnits(pattern: String, unit_type: String, state: String) -> (String, bool)
        let service = Systemd__Systemctl::new(crate::dry_run::DryRunMode(true));
        let result = service.list_units(compile_error!("no test default for type: NonEmptyStr"), String::new(), String::new()).await;
        assert!(result.is_ok(), "Systemd__Systemctl.ListUnits should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_systemd___systemctl_list_units_all_loaded() {
        // Signature: Systemd__Systemctl.ListUnitsAllLoaded(pattern: String, unit_type: String) -> (String, bool)
        let service = Systemd__Systemctl::new(crate::dry_run::DryRunMode(true));
        let result = service.list_units_all_loaded(compile_error!("no test default for type: NonEmptyStr"), String::new()).await;
        assert!(result.is_ok(), "Systemd__Systemctl.ListUnitsAllLoaded should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_systemd___systemctl_list_units_user_all_loaded() {
        // Signature: Systemd__Systemctl.ListUnitsUserAllLoaded(pattern: String, unit_type: String) -> (String, bool)
        let service = Systemd__Systemctl::new(crate::dry_run::DryRunMode(true));
        let result = service.list_units_user_all_loaded(compile_error!("no test default for type: NonEmptyStr"), String::new()).await;
        assert!(result.is_ok(), "Systemd__Systemctl.ListUnitsUserAllLoaded should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_systemd___systemctl_status() {
        // Signature: Systemd__Systemctl.Status(unit: String) -> (String, bool)
        let service = Systemd__Systemctl::new(crate::dry_run::DryRunMode(true));
        let result = service.status(compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Systemd__Systemctl.Status should succeed in dry-run mode: {:?}", result.err());
}
}
