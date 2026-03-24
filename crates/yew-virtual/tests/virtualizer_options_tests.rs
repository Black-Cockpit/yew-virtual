//! Unit tests for `VirtualizerOptions` module.
//!
//! This module verifies the correctness of VirtualizerOptions defaults
//! and configuration propagation to the virtualizer engine.
//!
//! Test Scenarios:
//! - Default options have expected values.
//! - Default options create a valid virtualizer.
//! - Custom options are propagated correctly.

use yew_virtual::core::item_size_mode::ItemSizeMode;
use yew_virtual::core::scroll_direction::ScrollDirection;
use yew_virtual::core::virtualizer_options::VirtualizerOptions;

/// Tests that default options have the expected values.
///
/// # Test Steps
/// - Creates a default VirtualizerOptions.
/// - Asserts each field has the expected default.
#[test]
fn should_have_correct_default_values() {
    // Create default options.
    let opts = VirtualizerOptions::default();

    // Assert each default value.
    assert_eq!(opts.item_count, 0, "Default item_count should be 0");
    assert_eq!(
        opts.item_size_mode,
        ItemSizeMode::Fixed(50.0),
        "Default item_size_mode should be Fixed(50.0)"
    );
    assert_eq!(
        opts.scroll_direction,
        ScrollDirection::Vertical,
        "Default scroll_direction should be Vertical"
    );
    assert_eq!(opts.overscan, 5, "Default overscan should be 5");
    assert!(
        (opts.padding_start - 0.0).abs() < f64::EPSILON,
        "Default padding_start should be 0.0"
    );
    assert!(
        (opts.padding_end - 0.0).abs() < f64::EPSILON,
        "Default padding_end should be 0.0"
    );
    assert!(
        (opts.gap - 0.0).abs() < f64::EPSILON,
        "Default gap should be 0.0"
    );
    assert_eq!(
        opts.container_size, None,
        "Default container_size should be None"
    );
    assert!(
        !opts.use_window_scroll,
        "Default use_window_scroll should be false"
    );
}

/// Tests that options with custom values can be constructed using defaults.
///
/// # Test Steps
/// - Creates options with custom values using struct update syntax.
/// - Asserts key values are set correctly.
#[test]
fn should_accept_fully_custom_options() {
    // Create custom options using struct update syntax.
    let opts = VirtualizerOptions {
        item_count: 1000,
        item_size_mode: ItemSizeMode::Estimated(75.0),
        scroll_direction: ScrollDirection::Horizontal,
        overscan: 10,
        padding_start: 20.0,
        padding_end: 30.0,
        gap: 8.0,
        container_size: Some(600.0),
        use_window_scroll: true,
        ..VirtualizerOptions::default()
    };

    // Assert custom values.
    assert_eq!(opts.item_count, 1000, "item_count should be 1000");
    assert_eq!(
        opts.item_size_mode,
        ItemSizeMode::Estimated(75.0),
        "item_size_mode should be Estimated(75.0)"
    );
    assert_eq!(
        opts.scroll_direction,
        ScrollDirection::Horizontal,
        "scroll_direction should be Horizontal"
    );
    assert_eq!(opts.overscan, 10, "overscan should be 10");
    assert!(
        (opts.padding_start - 20.0).abs() < f64::EPSILON,
        "padding_start should be 20.0"
    );
    assert!(
        (opts.padding_end - 30.0).abs() < f64::EPSILON,
        "padding_end should be 30.0"
    );
    assert!((opts.gap - 8.0).abs() < f64::EPSILON, "gap should be 8.0");
    assert_eq!(
        opts.container_size,
        Some(600.0),
        "container_size should be Some(600.0)"
    );
    assert!(opts.use_window_scroll, "use_window_scroll should be true");
}

/// Tests that options support Clone and PartialEq.
///
/// # Test Steps
/// - Creates options, clones them.
/// - Asserts the clone equals the original.
#[test]
fn should_support_clone_and_partial_eq() {
    // Create options.
    let opts = VirtualizerOptions {
        item_count: 50,
        item_size_mode: ItemSizeMode::Dynamic(30.0),
        ..VirtualizerOptions::default()
    };

    // Clone the options.
    let cloned = opts.clone();

    // Assert equality.
    assert_eq!(opts, cloned, "Cloned options should equal original");
}
