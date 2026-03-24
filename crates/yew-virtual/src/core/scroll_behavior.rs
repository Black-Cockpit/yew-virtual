/// Scroll behavior for programmatic scroll operations.
///
/// Controls the animation behavior when the virtualizer programmatically
/// scrolls to an offset or index. Maps to the browser's native
/// `ScrollBehavior` options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollBehavior {
    /// Let the browser determine the scroll behavior.
    Auto,

    /// Animate the scroll smoothly to the target position.
    Smooth,

    /// Jump immediately to the target position without animation.
    Instant,
}

impl Default for ScrollBehavior {
    /// Returns the default scroll behavior.
    ///
    /// # Returns
    ///
    /// - `ScrollBehavior::Auto`: The default behavior.
    fn default() -> Self {
        // Default to auto to match browser defaults.
        Self::Auto
    }
}
