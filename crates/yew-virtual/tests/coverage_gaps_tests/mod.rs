/// Unit tests for API surface and scroll reconciliation coverage.
///
/// These tests validate public error and key types, option validation,
/// and programmatic scroll reconciliation branches, ensuring that:
/// - Display and Debug output stay stable for diagnostics.
/// - Lane and measurement edge cases do not panic.
#[cfg(test)]
mod api_surface_tests;
