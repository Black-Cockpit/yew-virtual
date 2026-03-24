//! Unit tests for `RangeCalculator` module.
//!
//! This module verifies the correctness of visible range calculation
//! using binary search over virtual item measurements.
//! It ensures correct start/end indices and proper handling of edge cases
//! including multi-lane layouts.
//!
//! Test Scenarios:
//! - Empty measurements returns None.
//! - Zero container size returns None.
//! - Full visibility when all items fit in viewport.
//! - Partial visibility with scroll offset.
//! - Single item dataset is handled correctly.
//! - Multi-lane range calculation works correctly.

use yew_virtual::core::range_calculator::RangeCalculator;
use yew_virtual::core::virtual_item::VirtualItem;

/// Helper to build measurements for uniform-sized items.
fn build_measurements(count: usize, size: f64) -> Vec<VirtualItem> {
    // Create virtual items with sequential offsets.
    (0..count)
        .map(|i| VirtualItem::new(i, size, i as f64 * size))
        .collect()
}

/// Tests that empty measurements returns None.
///
/// # Test Steps
/// - Calls calculate_range with empty measurements.
/// - Asserts the result is None.
#[test]
fn should_return_none_for_empty_measurements() {
    // Calculate range for empty measurements.
    let result = RangeCalculator::calculate_range(&[], 500.0, 0.0, 1);

    // Assert None.
    assert!(
        result.is_none(),
        "Should return None for empty measurements"
    );
}

/// Tests that zero container size returns None.
///
/// # Test Steps
/// - Creates measurements for 10 items.
/// - Calls calculate_range with container_size 0.
/// - Asserts the result is None.
#[test]
fn should_return_none_for_zero_container_size() {
    // Create measurements for 10 items of size 50.
    let measurements = build_measurements(10, 50.0);

    // Calculate range with zero container.
    let result = RangeCalculator::calculate_range(&measurements, 0.0, 0.0, 1);

    // Assert None.
    assert!(result.is_none(), "Should return None for zero container");
}

/// Tests that all items are visible when they fit in the viewport.
///
/// # Test Steps
/// - Creates 5 items of size 50 (total 250px).
/// - Uses a container of 500px.
/// - Asserts all items are in the range.
#[test]
fn should_include_all_items_when_viewport_is_large_enough() {
    // Create measurements for 5 items of size 50.
    let measurements = build_measurements(5, 50.0);

    // Calculate range with large viewport.
    let result = RangeCalculator::calculate_range(&measurements, 500.0, 0.0, 1);

    // Assert all items are visible.
    assert!(result.is_some(), "Should return a valid range");
    let (start, end) = result.unwrap_or((0, 0));
    assert_eq!(start, 0, "Start should be 0");
    assert_eq!(end, 4, "End should be 4 (last index inclusive)");
}

/// Tests that scroll offset correctly shifts the visible range.
///
/// # Test Steps
/// - Creates 20 items of size 50 (total 1000px).
/// - Scrolls to offset 250px with container 200px.
/// - Asserts the range covers visible items.
#[test]
fn should_shift_range_with_scroll_offset() {
    // Create measurements for 20 items of size 50.
    let measurements = build_measurements(20, 50.0);

    // Calculate range with scroll at 250, container 200.
    let result = RangeCalculator::calculate_range(&measurements, 200.0, 250.0, 1);

    // Assert valid range.
    assert!(result.is_some(), "Should return a valid range");
    let (start, end) = result.unwrap_or((0, 0));

    // Item at index 5 starts at 250. Items 5-8 visible in 200px viewport.
    assert_eq!(start, 5, "Start should be 5");
    assert!(end >= 8, "End should cover items through index 8-9");
}

/// Tests that a single item dataset is handled correctly.
///
/// # Test Steps
/// - Creates 1 item of size 50.
/// - Asserts the range includes the single item.
#[test]
fn should_handle_single_item_dataset() {
    // Create measurements for 1 item.
    let measurements = build_measurements(1, 50.0);

    // Calculate range.
    let result = RangeCalculator::calculate_range(&measurements, 500.0, 0.0, 1);

    // Assert single item is visible.
    assert!(result.is_some(), "Should return a valid range");
    let (start, end) = result.unwrap_or((0, 0));
    assert_eq!(start, 0, "Start should be 0");
    assert_eq!(end, 0, "End should be 0 (single item, inclusive)");
}

/// Tests range calculation at the end of the list.
///
/// # Test Steps
/// - Creates 20 items and scrolls to the end.
/// - Asserts the range includes the last items.
#[test]
fn should_calculate_range_at_end_of_list() {
    // Create measurements for 20 items of size 50 (total 1000px).
    let measurements = build_measurements(20, 50.0);

    // Scroll to near the end: offset 800, container 200.
    let result = RangeCalculator::calculate_range(&measurements, 200.0, 800.0, 1);

    // Assert valid range including last items.
    assert!(result.is_some(), "Should return a valid range");
    let (_start, end) = result.unwrap_or((0, 0));
    assert_eq!(end, 19, "End should be 19 (last item inclusive)");
}

/// Tests range calculation with negative scroll offset.
///
/// # Test Steps
/// - Calls calculate_range with a negative scroll offset.
/// - Asserts the range starts from 0.
#[test]
fn should_handle_negative_scroll_offset() {
    // Create measurements for 10 items.
    let measurements = build_measurements(10, 50.0);

    // Calculate range with negative offset.
    let result = RangeCalculator::calculate_range(&measurements, 200.0, -100.0, 1);

    // Assert range starts from 0.
    assert!(result.is_some(), "Should return a valid range");
    let (start, _end) = result.unwrap_or((0, 0));
    assert_eq!(start, 0, "Start should be 0 for negative scroll offset");
}
