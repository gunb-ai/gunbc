// ARM 1 of the behavioral receipt's control: BEHAVIOUR-PRESERVING.
//
// `band_of` is rewritten to test the upper bound first. Every byte of the function body moves;
// no input changes its answer. The receipt must report EQUIVALENT, and an EQUIVALENT that is not
// accompanied by moved bytes proves nothing -- which is why this file differs from the seed.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    Low,
    Mid,
    High,
}

pub fn band_of(level: i64) -> Band {
    if level >= 100 {
        Band::High
    } else if level >= 10 {
        Band::Mid
    } else {
        Band::Low
    }
}

pub fn escalate(band: Band, urgent: bool) -> Band {
    if urgent {
        match band {
            Band::Low => Band::Mid,
            Band::Mid => Band::High,
            Band::High => Band::High,
        }
    } else {
        band
    }
}
