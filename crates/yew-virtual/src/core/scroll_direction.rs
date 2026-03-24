/// Direction of virtualization scrolling.
///
/// Determines whether the virtualizer measures items along the vertical
/// or horizontal axis. This affects how scroll positions, item sizes,
/// and container dimensions are interpreted by the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    /// Virtualize items along the vertical axis.
    ///
    /// Item sizes represent heights and scroll position tracks
    /// vertical offset within the container.
    Vertical,

    /// Virtualize items along the horizontal axis.
    ///
    /// Item sizes represent widths and scroll position tracks
    /// horizontal offset within the container.
    Horizontal,
}

impl Default for ScrollDirection {
    /// Returns the default scroll direction.
    ///
    /// # Returns
    ///
    /// - `ScrollDirection::Vertical`: The default direction.
    fn default() -> Self {
        // Default to vertical scrolling as it is the most common use case.
        Self::Vertical
    }
}
