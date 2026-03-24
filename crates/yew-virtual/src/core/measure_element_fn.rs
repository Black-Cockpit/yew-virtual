use std::sync::Arc;

use crate::core::scroll_direction::ScrollDirection;

/// Custom size extraction from observed element dimensions.
///
/// Receives content-box width and height plus scroll direction; returns size along the scroll axis.
pub type MeasureElementFn = Arc<dyn Fn(f64, f64, ScrollDirection) -> f64 + Send + Sync>;
