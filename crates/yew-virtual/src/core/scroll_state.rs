use crate::core::scroll_alignment::ScrollAlignment;
use crate::core::scroll_behavior::ScrollBehavior;

/// Internal state for an active programmatic scroll operation.
///
/// Tracks the target, alignment, behavior, and settling progress
/// of an in-flight scroll reconciliation loop. Used by the virtualizer
/// to determine when a smooth scroll has reached its destination.
#[derive(Debug, Clone, PartialEq)]
pub struct ScrollState {
    /// The target item index, if scrolling to a specific item.
    pub index: Option<usize>,

    /// The alignment used for this scroll operation.
    pub align: ScrollAlignment,

    /// The animation behavior for the scroll.
    pub behavior: ScrollBehavior,

    /// Timestamp (ms) when this scroll operation started.
    pub started_at: f64,

    /// The last computed target scroll offset.
    pub last_target_offset: f64,

    /// Number of consecutive frames where the scroll position
    /// matched the target (used for settling detection).
    pub stable_frames: u32,
}

impl ScrollState {
    /// Creates a new scroll state for a programmatic scroll operation.
    ///
    /// # Parameters
    ///
    /// - `index`: Optional target item index.
    /// - `align`: The alignment for the scroll.
    /// - `behavior`: The animation behavior.
    /// - `started_at`: Timestamp when the scroll started.
    /// - `last_target_offset`: The initial target scroll offset.
    pub fn new(
        index: Option<usize>,
        align: ScrollAlignment,
        behavior: ScrollBehavior,
        started_at: f64,
        last_target_offset: f64,
    ) -> Self {
        // Construct the initial scroll state with zero stable frames.
        Self {
            index,
            align,
            behavior,
            started_at,
            last_target_offset,
            stable_frames: 0,
        }
    }
}
