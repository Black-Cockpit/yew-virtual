/// Alignment behavior when programmatically scrolling to an item.
///
/// Controls where the target item is positioned within the viewport
/// after a scroll-to-index operation. This mirrors the alignment
/// options found in TanStack Virtual.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollAlignment {
    /// Position the item at the start of the viewport.
    Start,

    /// Position the item at the center of the viewport.
    Center,

    /// Position the item at the end of the viewport.
    End,

    /// Automatically determine the minimal scroll to make the item visible.
    ///
    /// If the item is already fully visible, no scroll occurs. Otherwise,
    /// the viewport scrolls the minimum amount needed to bring the item
    /// into view.
    Auto,
}

impl Default for ScrollAlignment {
    /// Returns the default scroll alignment.
    ///
    /// # Returns
    ///
    /// - `ScrollAlignment::Auto`: The default alignment behavior.
    fn default() -> Self {
        // Default to auto alignment for minimal scroll disruption.
        Self::Auto
    }
}
