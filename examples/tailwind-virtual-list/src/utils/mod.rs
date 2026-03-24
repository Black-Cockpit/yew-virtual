/// Utility trait for formatting numbers with thousands separators.
///
/// Provides the `FormatNumber` trait used across components to
/// display large item counts in a human-readable format.
pub mod format_number;

/// Helper utilities for safe virtualizer creation.
///
/// Provides a fallback function that creates a valid virtualizer
/// instance without using `unwrap()`, `expect()`, or `panic!()`.
pub mod virtualizer_helper;
