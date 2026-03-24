//! Helper utilities for safe virtualizer creation.
//!
//! Provides a fallback function that creates a valid virtualizer
//! instance using only explicit `match` error handling. No `unwrap()`,
//! `expect()`, or `panic!()` is used anywhere.

use yew_virtual::core::virtualizer::Virtualizer;
use yew_virtual::core::virtualizer_options::VirtualizerOptions;

/// Creates a virtualizer with the given options, falling back to an empty instance.
///
/// Attempts creation with the provided options first. On failure, falls
/// back to `Virtualizer::empty()` which is infallible and always returns
/// a valid zero-item virtualizer.
///
/// # Parameters
///
/// - `options`: The primary options to attempt first.
///
/// # Returns
///
/// - `Virtualizer`: A working virtualizer instance (possibly empty on failure).
pub fn create_virtualizer_safe(options: VirtualizerOptions) -> Virtualizer {
    // Attempt to create with the provided options.
    match Virtualizer::new(options) {
        Ok(v) => v,
        Err(_e) => {
            // Return an infallible empty fallback on failure.
            Virtualizer::empty()
        }
    }
}
