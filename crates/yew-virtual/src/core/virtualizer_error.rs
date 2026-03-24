use thiserror::Error;

/// Errors that can occur during virtualizer operations.
///
/// Covers all failure modes in the virtualization engine, including
/// invalid configuration, measurement issues, scroll container problems,
/// and index-related errors.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum VirtualizerError {
    /// The provided item count is invalid.
    ///
    /// # Details
    /// - Occurs when the item count overflows internal calculations.
    #[error("Invalid item count: {0}")]
    InvalidItemCount(String),

    /// The provided item size is invalid.
    ///
    /// # Details
    /// - Occurs when a zero or negative size is specified.
    #[error("Invalid item size: {0}")]
    InvalidItemSize(String),

    /// The overscan value is invalid.
    ///
    /// # Details
    /// - Occurs when overscan is negative.
    #[error("Invalid overscan value: {0}")]
    InvalidOverscan(String),

    /// The requested index is out of bounds.
    ///
    /// # Details
    /// - Occurs when scrolling to or measuring an item beyond the dataset.
    #[error("Index out of bounds: requested {requested}, total {total}")]
    IndexOutOfBounds {
        /// The index that was requested.
        requested: usize,
        /// The total number of items.
        total: usize,
    },

    /// A measurement update failed.
    ///
    /// # Details
    /// - Occurs when a measured size is invalid (e.g., negative or NaN).
    #[error("Measurement error: {0}")]
    MeasurementError(String),

    /// The scroll container reference is unavailable.
    ///
    /// # Details
    /// - Occurs when the DOM element for the scroll container cannot be accessed.
    #[error("Scroll container unavailable: {0}")]
    ScrollContainerUnavailable(String),

    /// A configuration parameter is invalid.
    ///
    /// # Details
    /// - Occurs for general configuration issues not covered by more specific variants.
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
}
