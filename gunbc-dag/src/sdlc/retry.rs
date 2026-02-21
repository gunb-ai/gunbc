use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryState {
    pub attempts: u32,
    pub budget: u32,
    pub next_retry_at_epoch_ms: Option<u128>,
    pub last_error: Option<String>,
}

impl Default for RetryState {
    fn default() -> Self {
        Self {
            attempts: 0,
            budget: 3,
            next_retry_at_epoch_ms: None,
            last_error: None,
        }
    }
}

pub fn retry_ready(state: &RetryState, now_epoch_ms: u128) -> bool {
    match state.next_retry_at_epoch_ms {
        None => true,
        Some(next) => now_epoch_ms >= next,
    }
}

pub fn register_retry_failure(
    state: &mut RetryState,
    now_epoch_ms: u128,
    base_backoff_ms: u128,
    error: String,
) -> bool {
    state.attempts = state.attempts.saturating_add(1);
    state.last_error = Some(error);
    if state.attempts >= state.budget {
        state.next_retry_at_epoch_ms = None;
        return false;
    }
    let exponent = state.attempts.saturating_sub(1);
    let multiplier = 2u128.saturating_pow(exponent);
    let delay = base_backoff_ms.saturating_mul(multiplier);
    state.next_retry_at_epoch_ms = Some(now_epoch_ms.saturating_add(delay));
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_register_failure_sets_exponential_backoff() {
        let mut state = RetryState {
            budget: 4,
            ..RetryState::default()
        };
        assert!(register_retry_failure(
            &mut state,
            1000,
            200,
            "network".to_string()
        ));
        assert_eq!(state.attempts, 1);
        assert_eq!(state.next_retry_at_epoch_ms, Some(1200));
        assert!(retry_ready(&state, 1200));
    }

    #[test]
    fn retry_register_failure_exhausts_budget_fail_closed() {
        let mut state = RetryState {
            budget: 2,
            ..RetryState::default()
        };
        assert!(register_retry_failure(
            &mut state,
            1000,
            100,
            "first".to_string()
        ));
        assert!(!register_retry_failure(
            &mut state,
            1100,
            100,
            "second".to_string()
        ));
        assert_eq!(state.attempts, 2);
        assert_eq!(state.next_retry_at_epoch_ms, None);
    }
}
