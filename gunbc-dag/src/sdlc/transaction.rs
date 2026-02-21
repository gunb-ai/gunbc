use gunbc_ir::transport::github::IssueLifecycleStage;

pub fn validate_stage_transition(
    current: IssueLifecycleStage,
    next: IssueLifecycleStage,
) -> Result<(), String> {
    if current == next {
        return Ok(());
    }
    let allowed = match current {
        IssueLifecycleStage::Idea => [IssueLifecycleStage::Design].as_slice(),
        IssueLifecycleStage::Design => [IssueLifecycleStage::DesignReview].as_slice(),
        IssueLifecycleStage::DesignReview => [IssueLifecycleStage::Accepted].as_slice(),
        IssueLifecycleStage::Accepted => [IssueLifecycleStage::Implementation].as_slice(),
        IssueLifecycleStage::Implementation => [IssueLifecycleStage::Closed].as_slice(),
        IssueLifecycleStage::Closed => [].as_slice(),
    };
    if allowed.contains(&next) {
        return Ok(());
    }
    Err(format!(
        "invalid stage transition `{}` -> `{}`",
        current.as_label(),
        next.as_label()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transition_allows_linear_progression() {
        validate_stage_transition(IssueLifecycleStage::Idea, IssueLifecycleStage::Design)
            .expect("idea -> design should be valid");
        validate_stage_transition(
            IssueLifecycleStage::DesignReview,
            IssueLifecycleStage::Accepted,
        )
        .expect("design-review -> accepted should be valid");
    }

    #[test]
    fn transition_rejects_out_of_order_progression() {
        let err = validate_stage_transition(IssueLifecycleStage::Design, IssueLifecycleStage::Accepted)
            .expect_err("design -> accepted should be invalid");
        assert!(err.contains("invalid stage transition"));
    }
}
