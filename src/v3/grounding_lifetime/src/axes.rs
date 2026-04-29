//! LanguageSpec axis discipline consumed by the analyzer (lane 6 vocabulary).
//!
//! R2 fold only **reads** which axes are load-bearing for fail-closed checks;
//! it does not author LanguageSpec rows (`t-ground-lifetime-analyzer.md` out-of-scope list).

/// Axes declared by the target `LanguageSpec` relevant to lifetime analysis.
///
/// Input surface is intentionally tiny: only fields needed for R2 tests and
/// `UnderRefined` on growability (`brief` test plan item 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LanguageSpecAxes {
    /// When true, `FreeMonoid<Char>` / string-like bindings must resolve
    /// `Growability` to `Yes` or `No` from use structure; absent evidence fails
    /// closed (`UnderRefined { axis: "growability" }`) per design-emission-model
    /// open caveat at ~558–559.
    pub string_growability_axis_load_bearing: bool,
}

impl LanguageSpecAxes {
    /// Default for tests mirroring a populated LanguageSpec with string family.
    pub fn example_rust_string_family() -> Self {
        Self {
            string_growability_axis_load_bearing: true,
        }
    }

    /// Axes where growability is not load-bearing — exposes ownership proof gaps
    /// that must not be papered over by `IndeterminateGrowability` alone.
    pub fn string_family_growability_not_load_bearing() -> Self {
        Self {
            string_growability_axis_load_bearing: false,
        }
    }
}
