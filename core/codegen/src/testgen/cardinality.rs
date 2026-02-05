//! Test-specific cardinality helpers.

use gunbc_ir::Cardinality;

/// Representative "many" count for test generation.
///
/// This value is intentionally small to keep generated tests lightweight
/// while still exercising list fan-in and multi-element behavior.
pub const FERMI_MANY: u32 = 3;

/// Return boundary test cases with a small, test-only "many" case when needed.
pub fn fermi_test_cases(cardinality: Cardinality) -> Vec<u32> {
    let mut cases = cardinality.test_cases();

    if cardinality.allows_many() {
        let has_many = cases.iter().any(|&n| n > 1);
        if !has_many {
            let mut candidate = FERMI_MANY.max(2);
            if candidate < cardinality.min {
                candidate = cardinality.min;
            }
            if let Some(max) = cardinality.max {
                if candidate > max {
                    candidate = max;
                }
            }
            if cardinality.allows_count(candidate) && !cases.contains(&candidate) {
                cases.push(candidate);
            }
        }
    }

    cases.sort();
    cases.dedup();
    cases
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fermi_adds_many_for_zero_or_more() {
        assert_eq!(
            fermi_test_cases(Cardinality::ZERO_OR_MORE),
            vec![0, 1, FERMI_MANY]
        );
    }

    #[test]
    fn fermi_keeps_one_or_more_boundary_cases() {
        assert_eq!(fermi_test_cases(Cardinality::ONE_OR_MORE), vec![1, 2]);
    }

    #[test]
    fn fermi_respects_bounded_ranges() {
        assert_eq!(
            fermi_test_cases(Cardinality::new(0, Some(2))),
            vec![0, 1, 2]
        );
    }
}
