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
        IssueLifecycleStage::Accepted => [IssueLifecycleStage::Implementing].as_slice(),
        IssueLifecycleStage::Implementing => [IssueLifecycleStage::CodeReview].as_slice(),
        IssueLifecycleStage::CodeReview => [IssueLifecycleStage::Testing].as_slice(),
        IssueLifecycleStage::Testing => [IssueLifecycleStage::Done].as_slice(),
        IssueLifecycleStage::Done => [].as_slice(),
        IssueLifecycleStage::TerminalFailed => [].as_slice(),
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
    fn transition_allows_new_stage_progression() {
        validate_stage_transition(
            IssueLifecycleStage::Accepted,
            IssueLifecycleStage::Implementing,
        )
        .expect("accepted -> implementing should be valid");
        validate_stage_transition(
            IssueLifecycleStage::Implementing,
            IssueLifecycleStage::CodeReview,
        )
        .expect("implementing -> code-review should be valid");
        validate_stage_transition(
            IssueLifecycleStage::CodeReview,
            IssueLifecycleStage::Testing,
        )
        .expect("code-review -> testing should be valid");
        validate_stage_transition(IssueLifecycleStage::Testing, IssueLifecycleStage::Done)
            .expect("testing -> done should be valid");
    }

    #[test]
    fn transition_rejects_out_of_order_progression() {
        let err =
            validate_stage_transition(IssueLifecycleStage::Design, IssueLifecycleStage::Accepted)
                .expect_err("design -> accepted should be invalid");
        assert!(err.contains("invalid stage transition"));
    }
}
