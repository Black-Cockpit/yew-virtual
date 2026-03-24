use std::sync::Arc;

/// Decides whether to adjust scroll when an item above the viewport resizes.
///
/// Arguments are item index, previous size, and new size in pixels.
pub type ShouldAdjustScrollOnResizeFn = Arc<dyn Fn(usize, f64, f64) -> bool + Send + Sync>;
