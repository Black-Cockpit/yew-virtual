/// Options for programmatic scroll operations.
///
/// Combines alignment and behavior settings for scroll-to-index
/// and scroll-to-offset operations. This mirrors the scroll options
/// interface from TanStack Virtual.
use crate::core::scroll_alignment::ScrollAlignment;
use crate::core::scroll_behavior::ScrollBehavior;

/// Configuration for a programmatic scroll operation.
///
/// Used by `scroll_to_index`, `scroll_to_offset`, and `scroll_by`
/// to control where and how the scroll occurs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollToOptions {
    /// How to align the target within the viewport.
    pub align: ScrollAlignment,

    /// The animation behavior for the scroll.
    pub behavior: ScrollBehavior,
}

impl Default for ScrollToOptions {
    /// Returns default scroll-to options.
    ///
    /// # Returns
    ///
    /// - `ScrollToOptions`: Auto alignment with auto behavior.
    fn default() -> Self {
        // Use auto alignment and auto behavior as defaults.
        Self {
            align: ScrollAlignment::Auto,
            behavior: ScrollBehavior::Auto,
        }
    }
}
