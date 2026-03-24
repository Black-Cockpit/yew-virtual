/// Unit tests for core virtualization types and enums.
///
/// These tests validate the correctness of fundamental types like
/// ScrollDirection, ItemSizeMode, ScrollAlignment, and VirtualItem,
/// ensuring that:
/// - Default values are correct.
/// - Constructors produce valid instances.
/// - Enum variants behave as expected.
#[cfg(test)]
mod core_types_tests;

/// Unit tests for the measurement cache.
///
/// These tests validate the correctness of the MeasurementCache,
/// ensuring that:
/// - Measurements are stored and retrieved correctly.
/// - Running averages are computed accurately.
/// - Invalid measurements are rejected.
/// - Cache clearing works properly.
#[cfg(test)]
mod measurement_cache_tests;

/// Unit tests for the range calculator.
///
/// These tests validate the correctness of visible range calculation,
/// ensuring that:
/// - Binary search finds correct start and end indices.
/// - Overscan is applied correctly.
/// - Edge cases (empty dataset, zero container) are handled.
#[cfg(test)]
mod range_calculator_tests;

/// Unit tests for the virtualizer engine.
///
/// These tests validate the correctness of the Virtualizer,
/// ensuring that:
/// - Initialization with valid and invalid options works correctly.
/// - Scroll offset updates recalculate ranges.
/// - Item measurements update layout.
/// - Programmatic scroll navigation produces correct offsets.
/// - Dynamic item count changes are handled.
#[cfg(test)]
mod virtualizer_tests;

/// Unit tests for virtualizer options and configuration.
///
/// These tests validate the correctness of VirtualizerOptions,
/// ensuring that:
/// - Default options are valid.
/// - Invalid configurations are rejected.
/// - Options are properly propagated to the virtualizer.
#[cfg(test)]
mod virtualizer_options_tests;

/// Unit tests grouped under `coverage_gaps_tests/` per TEST_RULES.md.
///
/// These tests exercise error formatting, `VirtualKey`, scroll reconciliation,
/// multi-lane range calculation, and option validation branches via
/// `api_surface_tests.rs`.
#[cfg(test)]
mod coverage_gaps_tests;

/// Unit tests for new TanStack Virtual parity features.
///
/// These tests validate the correctness of all new features including:
/// - Multi-lane grid layout support.
/// - Per-index size estimation callbacks.
/// - Custom key extractors and VirtualKey types.
/// - ScrollBehavior, ScrollToOptions, and ScrollState types.
/// - Enabled/disabled toggle.
/// - Scroll padding and scroll margin.
/// - Dynamic option updates via set_options.
/// - Programmatic scroll operations (scroll_to_offset, scroll_by).
/// - get_virtual_item_for_offset lookup.
/// - Force re-measure via measure().
/// - is_scrolling and scroll direction tracking.
/// - Scroll adjustments on item resize above viewport.
/// - Custom range extractor for sticky headers.
/// - Initial measurements cache for SSR hydration.
#[cfg(test)]
mod new_features_tests;
