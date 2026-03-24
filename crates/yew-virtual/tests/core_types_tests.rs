//! Unit tests for core virtualization types.
//!
//! This module verifies the correctness of fundamental types including
//! ScrollDirection, ItemSizeMode, ScrollAlignment, and VirtualItem.
//! It ensures default values, constructors, and enum behaviors are correct.
//!
//! Test Scenarios:
//! - Default scroll direction is vertical.
//! - Item size mode returns correct base sizes.
//! - Virtual items compute end positions correctly.
//! - Scroll alignment defaults to auto.

use yew_virtual::core::item_size_mode::ItemSizeMode;
use yew_virtual::core::scroll_alignment::ScrollAlignment;
use yew_virtual::core::scroll_direction::ScrollDirection;
use yew_virtual::core::virtual_item::VirtualItem;
use yew_virtual::core::virtual_key::VirtualKey;

/// Tests that the default scroll direction is vertical.
///
/// # Test Steps
/// - Creates a default ScrollDirection.
/// - Asserts that it equals Vertical.
#[test]
fn should_default_to_vertical_scroll_direction() {
    // Create the default scroll direction.
    let direction = ScrollDirection::default();

    // Assert it is vertical.
    assert_eq!(
        direction,
        ScrollDirection::Vertical,
        "Default scroll direction should be Vertical"
    );
}

/// Tests that scroll direction variants are distinct.
///
/// # Test Steps
/// - Creates both Vertical and Horizontal variants.
/// - Asserts that they are not equal.
#[test]
fn should_distinguish_vertical_and_horizontal_directions() {
    // Create both direction variants.
    let vertical = ScrollDirection::Vertical;
    let horizontal = ScrollDirection::Horizontal;

    // Assert they are not equal.
    assert_ne!(
        vertical, horizontal,
        "Vertical and Horizontal should be distinct"
    );
}

/// Tests that fixed item size mode returns the correct base size.
///
/// # Test Steps
/// - Creates a Fixed item size mode with a known value.
/// - Asserts that base_size returns the value.
/// - Asserts that requires_measurement returns false.
#[test]
fn should_return_correct_base_size_for_fixed_mode() {
    // Create a fixed mode with 100px.
    let mode = ItemSizeMode::Fixed(100.0);

    // Assert the base size is correct.
    assert!(
        (mode.base_size() - 100.0).abs() < f64::EPSILON,
        "Fixed mode base size should be 100.0"
    );

    // Assert measurement is not required.
    assert!(
        !mode.requires_measurement(),
        "Fixed mode should not require measurement"
    );
}

/// Tests that estimated item size mode requires measurement.
///
/// # Test Steps
/// - Creates an Estimated item size mode.
/// - Asserts that base_size returns the estimate.
/// - Asserts that requires_measurement returns true.
#[test]
fn should_require_measurement_for_estimated_mode() {
    // Create an estimated mode with 75px.
    let mode = ItemSizeMode::Estimated(75.0);

    // Assert the base size is the estimate.
    assert!(
        (mode.base_size() - 75.0).abs() < f64::EPSILON,
        "Estimated mode base size should be 75.0"
    );

    // Assert measurement is required.
    assert!(
        mode.requires_measurement(),
        "Estimated mode should require measurement"
    );
}

/// Tests that dynamic item size mode requires measurement.
///
/// # Test Steps
/// - Creates a Dynamic item size mode.
/// - Asserts that base_size returns the fallback.
/// - Asserts that requires_measurement returns true.
#[test]
fn should_require_measurement_for_dynamic_mode() {
    // Create a dynamic mode with 40px fallback.
    let mode = ItemSizeMode::Dynamic(40.0);

    // Assert the base size is the fallback.
    assert!(
        (mode.base_size() - 40.0).abs() < f64::EPSILON,
        "Dynamic mode base size should be 40.0"
    );

    // Assert measurement is required.
    assert!(
        mode.requires_measurement(),
        "Dynamic mode should require measurement"
    );
}

/// Tests that the default item size mode is Fixed(50.0).
///
/// # Test Steps
/// - Creates a default ItemSizeMode.
/// - Asserts it is Fixed with value 50.0.
#[test]
fn should_default_to_fixed_fifty_for_item_size_mode() {
    // Create the default mode.
    let mode = ItemSizeMode::default();

    // Assert it is Fixed(50.0).
    assert_eq!(
        mode,
        ItemSizeMode::Fixed(50.0),
        "Default item size mode should be Fixed(50.0)"
    );
}

/// Tests that scroll alignment defaults to auto.
///
/// # Test Steps
/// - Creates a default ScrollAlignment.
/// - Asserts that it equals Auto.
#[test]
fn should_default_to_auto_scroll_alignment() {
    // Create the default alignment.
    let alignment = ScrollAlignment::default();

    // Assert it is Auto.
    assert_eq!(
        alignment,
        ScrollAlignment::Auto,
        "Default scroll alignment should be Auto"
    );
}

/// Tests that virtual item computes end position correctly.
///
/// # Test Steps
/// - Creates a VirtualItem with known index, size, and start.
/// - Asserts that end equals start + size.
/// - Asserts that key equals index.
/// - Asserts that lane is 0.
#[test]
fn should_compute_virtual_item_end_position() {
    // Create a virtual item at index 5, size 30, offset 150.
    let item = VirtualItem::new(5, 30.0, 150.0);

    // Assert end is start + size.
    assert!(
        (item.end - 180.0).abs() < f64::EPSILON,
        "End should be start + size (150 + 30 = 180)"
    );

    // Assert key equals index.
    assert_eq!(item.key, VirtualKey::Index(5), "Key should equal index");

    // Assert lane is 0 for standard items.
    assert_eq!(item.lane, 0, "Lane should be 0 for non-grid items");
}

/// Tests that virtual item with key and lane sets them correctly.
///
/// # Test Steps
/// - Creates a VirtualItem with a specific key and lane.
/// - Asserts that lane and key are set to the provided values.
#[test]
fn should_create_virtual_item_with_key_and_lane() {
    // Create a virtual item with lane 2 and a custom key.
    let item = VirtualItem::with_key_and_lane(3, 50.0, 100.0, VirtualKey::Index(3), 2);

    // Assert lane is set correctly.
    assert_eq!(item.lane, 2, "Lane should be 2");

    // Assert other fields are correct.
    assert_eq!(item.index, 3, "Index should be 3");
    assert!(
        (item.size - 50.0).abs() < f64::EPSILON,
        "Size should be 50.0"
    );
    assert!(
        (item.start - 100.0).abs() < f64::EPSILON,
        "Start should be 100.0"
    );
}
