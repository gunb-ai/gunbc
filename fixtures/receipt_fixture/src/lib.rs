// The SEED half of the behavioral receipt's own control.
//
// Hand-authored on purpose: a controlled fixture independently authors its input and its expected
// outcome (DESIGN §5). If this file were emitted, the control's expectation and the thing under
// test would share a producer, and the arms could agree because both were wrong.
//
// Its declared surface is `../authority.dag`, which is what the mode derives the corpus from.
// The two files in `../candidates/` are installed over THIS file, one per arm.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    Low,
    Mid,
    High,
}

pub fn band_of(level: i64) -> Band {
    if level < 10 {
        Band::Low
    } else if level < 100 {
        Band::Mid
    } else {
        Band::High
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
