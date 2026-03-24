/// Result of recording a runtime item measurement.
///
/// Describes whether layout changed and how much the scroll position
/// should be compensated on the DOM when an item above the viewport resizes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeasureItemOutcome {
    /// Whether the measurement altered cached sizes or layout.
    pub layout_changed: bool,

    /// Pixels to add to the scroll offset on the scroll container immediately.
    ///
    /// Non-zero when an item above the viewport grew or shrank and scroll
    /// adjustment is enabled, matching TanStack Virtual scroll compensation.
    pub scroll_compensation: f64,
}

impl MeasureItemOutcome {
    /// Sentinel value: no layout change and no DOM scroll compensation.
    ///
    /// Use as a fallback when composing measurement results or in tests.
    pub const UNCHANGED: Self = Self {
        layout_changed: false,
        scroll_compensation: 0.0,
    };
}
