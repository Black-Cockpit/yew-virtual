/// Strategy for determining the size of virtualized items.
///
/// Controls how the virtualizer calculates the dimensions of each item
/// in the list. The chosen mode affects initial layout estimation,
/// measurement behavior, and offset recalculation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ItemSizeMode {
    /// All items have a uniform fixed size.
    ///
    /// The virtualizer uses this exact value for every item without
    /// requiring runtime measurement. This is the most performant mode.
    Fixed(f64),

    /// Items have an estimated size that may be refined by measurement.
    ///
    /// The virtualizer uses the estimate for initial layout and
    /// recalculates offsets when actual measurements are provided.
    Estimated(f64),

    /// Item sizes are entirely unknown and must be measured at runtime.
    ///
    /// The virtualizer uses a default fallback size until actual
    /// measurements are provided for each item. The `f64` value
    /// serves as the initial fallback estimate.
    Dynamic(f64),
}

impl ItemSizeMode {
    /// Returns the base size value regardless of mode.
    ///
    /// # Returns
    ///
    /// - `f64`: The size value contained in the variant.
    pub fn base_size(&self) -> f64 {
        // Extract the inner size value from whichever variant is active.
        match self {
            Self::Fixed(size) => *size,
            Self::Estimated(size) => *size,
            Self::Dynamic(size) => *size,
        }
    }

    /// Checks whether items in this mode require runtime measurement.
    ///
    /// # Returns
    ///
    /// - `bool`: True if items may need measurement after rendering.
    pub fn requires_measurement(&self) -> bool {
        // Only fixed-size items never require measurement.
        !matches!(self, Self::Fixed(_))
    }
}

impl Default for ItemSizeMode {
    /// Returns the default item size mode.
    ///
    /// # Returns
    ///
    /// - `ItemSizeMode::Fixed(50.0)`: A reasonable default fixed size.
    fn default() -> Self {
        // Use a fixed size of 50 pixels as a sensible default.
        Self::Fixed(50.0)
    }
}
