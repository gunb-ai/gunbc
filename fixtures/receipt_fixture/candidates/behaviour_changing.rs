// ARM 2 of the behavioral receipt's control: BEHAVIOUR-CHANGING, at a boundary and nowhere else.
//
// `< 100` becomes `<= 100`, so exactly ONE input in all of i64 changes its answer: level = 100,
// which answers Mid here and High in the seed. Every other value is untouched.
//
// That is deliberate, and it is what makes this control discriminating rather than decorative.
// A corpus that sampled the Int domain would almost surely miss a single point. This one cannot,
// because the domain is not sampled: the literals `band_of` compares against cut the integers
// into classes, and the enumeration takes every boundary and its neighbours on both sides. If
// the boundary enumeration ever regresses to sampling or to a bounded window that happens to
// exclude 100, THIS ARM GOES GREEN AND THE CONTROL DIES SILENTLY -- so the selftest additionally
// requires that the reported divergence name `band_of` and the value 100.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    Low,
    Mid,
    High,
}

pub fn band_of(level: i64) -> Band {
    if level < 10 {
        Band::Low
    } else if level <= 100 {
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
