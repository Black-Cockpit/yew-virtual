/// Dimensions of a scroll container or viewport.
///
/// Represents the width and height of a rectangular area, used
/// for tracking scroll container dimensions and initial rect
/// configuration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// The width of the rectangle in pixels.
    pub width: f64,

    /// The height of the rectangle in pixels.
    pub height: f64,
}

impl Default for Rect {
    /// Returns a zero-sized rectangle.
    ///
    /// # Returns
    ///
    /// - `Rect`: A rectangle with zero width and height.
    fn default() -> Self {
        // Default to zero dimensions.
        Self {
            width: 0.0,
            height: 0.0,
        }
    }
}
