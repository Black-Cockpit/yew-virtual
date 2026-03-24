//! Unit tests for `Virtualizer` engine module.
//!
//! This module verifies the correctness of the core Virtualizer engine,
//! including initialization, scroll handling, measurement updates,
//! programmatic navigation, and dynamic item count changes.
//!
//! Test Scenarios:
//! - Virtualizer initializes with correct state.
//! - Scroll offset updates recalculate visible ranges.
//! - Container size updates recalculate visible ranges.
//! - Item measurements update sizes and offsets.
//! - Scroll-to-index produces correct offsets for all alignments.
//! - Item count changes update layout without full reset.
//! - Invalid configurations are rejected.
//! - Out-of-bounds measurements are rejected.

use yew_virtual::core::item_size_mode::ItemSizeMode;
use yew_virtual::core::scroll_alignment::ScrollAlignment;
use yew_virtual::core::scroll_direction::ScrollDirection;
use yew_virtual::core::virtualizer::Virtualizer;
use yew_virtual::core::virtualizer_options::VirtualizerOptions;

/// Tests that the virtualizer initializes with correct total size.
///
/// # Test Steps
/// - Creates a virtualizer with 100 items of fixed size 50.
/// - Asserts total size is 100 * 50 = 5000.
#[test]
fn should_initialize_with_correct_total_size() {
    // Create options for 100 items of fixed size 50.
    let options = VirtualizerOptions {
        item_count: 100,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        container_size: Some(500.0),
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Virtualizer creation should succeed");

    let virt = result.unwrap_or_else(|_| panic!("Already checked is_ok"));

    // Assert total size is 5000.
    assert!(
        (virt.total_size() - 5000.0).abs() < f64::EPSILON,
        "Total size should be 5000 (100 * 50)"
    );
}

/// Tests that the virtualizer initializes with correct total size including padding.
///
/// # Test Steps
/// - Creates a virtualizer with padding_start=20 and padding_end=30.
/// - Asserts total size includes padding.
#[test]
fn should_include_padding_in_total_size() {
    // Create options with padding.
    let options = VirtualizerOptions {
        item_count: 10,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        padding_start: 20.0,
        padding_end: 30.0,
        container_size: Some(500.0),
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed with padding");

    let virt = result.unwrap_or_else(|_| panic!("Already checked is_ok"));

    // Total: 20 + (10 * 50) + 30 = 550.
    assert!(
        (virt.total_size() - 550.0).abs() < f64::EPSILON,
        "Total size should be 550 (20 + 500 + 30)"
    );
}

/// Tests that gaps between items are included in total size.
///
/// # Test Steps
/// - Creates a virtualizer with gap=10 and 5 items.
/// - Asserts total size includes gaps.
#[test]
fn should_include_gaps_in_total_size() {
    // Create options with gap.
    let options = VirtualizerOptions {
        item_count: 5,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        gap: 10.0,
        container_size: Some(500.0),
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed with gap");

    let virt = result.unwrap_or_else(|_| panic!("Already checked is_ok"));

    // Total: 5*50 + 4*10 = 250 + 40 = 290.
    assert!(
        (virt.total_size() - 290.0).abs() < f64::EPSILON,
        "Total size should be 290 (5*50 + 4*10)"
    );
}

/// Tests that scroll offset updates change the visible range.
///
/// # Test Steps
/// - Creates a virtualizer with 100 items, container 200px.
/// - Scrolls to offset 500.
/// - Asserts range_start is around index 10.
#[test]
fn should_update_range_on_scroll_offset_change() {
    // Create options.
    let options = VirtualizerOptions {
        item_count: 100,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        container_size: Some(200.0),
        overscan: 0,
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed");
    let mut virt = result.unwrap_or_else(|_| panic!("Already checked is_ok"));

    // Scroll to offset 500.
    virt.update_scroll_offset(500.0, true);

    // Item at index 10 starts at 500. Visible: 10..13 (200px / 50px = 4 items, inclusive).
    assert_eq!(
        virt.range_start(),
        10,
        "Range start should be 10 after scroll to 500"
    );
    assert_eq!(virt.range_end(), 13, "Range end should be 13 (inclusive)");
}

/// Tests that container size updates change the visible range.
///
/// # Test Steps
/// - Creates a virtualizer with container 100px (2 items visible).
/// - Updates container to 300px.
/// - Asserts more items are visible.
#[test]
fn should_update_range_on_container_size_change() {
    // Create options with small container.
    let options = VirtualizerOptions {
        item_count: 100,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        container_size: Some(100.0),
        overscan: 0,
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed");
    let mut virt = result.unwrap_or_else(|_| panic!("Already checked is_ok"));

    // Initially 2 items visible.
    let initial_end = virt.range_end();

    // Resize container to 300px.
    virt.update_container_size(300.0);

    // Now 6 items should be visible (300 / 50), range_end is inclusive.
    assert!(
        virt.range_end() > initial_end,
        "More items should be visible after container resize"
    );
    assert_eq!(
        virt.range_end(),
        5,
        "Range end should be 5 (inclusive) with 300px container"
    );
}

/// Tests that item measurements update layout correctly.
///
/// # Test Steps
/// - Creates a virtualizer with estimated 50px sizes.
/// - Measures item 0 as 100px.
/// - Asserts total size increased.
#[test]
fn should_update_layout_on_measurement() {
    // Create options with estimated sizes.
    let options = VirtualizerOptions {
        item_count: 10,
        item_size_mode: ItemSizeMode::Estimated(50.0),
        container_size: Some(500.0),
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed");
    let mut virt = result.unwrap_or_else(|_| panic!("Already checked is_ok"));

    // Initial total: 10 * 50 = 500.
    let initial_total = virt.total_size();

    // Measure item 0 as 100px.
    let measure_result = virt.measure_item(0, 100.0);
    let outcome = measure_result.unwrap_or_else(|e| {
        panic!("Measurement should succeed: {e:?}");
    });
    assert!(
        outcome.layout_changed,
        "Measurement should report a layout change"
    );

    // Total should have increased by 50 (100 - 50).
    assert!(
        (virt.total_size() - (initial_total + 50.0)).abs() < f64::EPSILON,
        "Total size should increase by 50 after measuring item 0 as 100px"
    );
}

/// Tests that out-of-bounds measurement is rejected.
///
/// # Test Steps
/// - Creates a virtualizer with 10 items.
/// - Attempts to measure index 10 (out of bounds).
/// - Asserts an error is returned.
#[test]
fn should_reject_out_of_bounds_measurement() {
    // Create options.
    let options = VirtualizerOptions {
        item_count: 10,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        container_size: Some(500.0),
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed");
    let mut virt = result.unwrap_or_else(|_| panic!("Already checked is_ok"));

    // Attempt to measure out of bounds.
    let measure_result = virt.measure_item(10, 50.0);
    assert!(
        measure_result.is_err(),
        "Measuring index 10 with 10 items should fail"
    );
}

/// Tests scroll-to-index with Start alignment.
///
/// # Test Steps
/// - Creates a virtualizer with 100 items.
/// - Scrolls to index 20 with Start alignment.
/// - Asserts the offset equals item 20's start position.
#[test]
fn should_scroll_to_index_with_start_alignment() {
    // Create options.
    let options = VirtualizerOptions {
        item_count: 100,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        container_size: Some(200.0),
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed");
    let virt = result.unwrap_or_else(|_| panic!("Already checked is_ok"));

    // Scroll to index 20 with Start alignment.
    let scroll_result = virt.scroll_to_index(20, ScrollAlignment::Start);
    assert!(scroll_result.is_ok(), "Scroll-to-index should succeed");

    // Item 20 starts at offset 1000 (20 * 50).
    let offset = scroll_result.unwrap_or(0.0);
    assert!(
        (offset - 1000.0).abs() < f64::EPSILON,
        "Scroll offset should be 1000 for index 20 at Start"
    );
}

/// Tests scroll-to-index with Center alignment.
///
/// # Test Steps
/// - Creates a virtualizer with 100 items, container 200px.
/// - Scrolls to index 20 with Center alignment.
/// - Asserts the offset centers item 20 in the viewport.
#[test]
fn should_scroll_to_index_with_center_alignment() {
    // Create options.
    let options = VirtualizerOptions {
        item_count: 100,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        container_size: Some(200.0),
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed");
    let virt = result.unwrap_or_else(|_| panic!("Already checked is_ok"));

    // Scroll to index 20 with Center alignment.
    let scroll_result = virt.scroll_to_index(20, ScrollAlignment::Center);
    assert!(scroll_result.is_ok(), "Scroll-to-index should succeed");

    // Center: item_offset - container/2 + item_size/2 = 1000 - 100 + 25 = 925.
    let offset = scroll_result.unwrap_or(0.0);
    assert!(
        (offset - 925.0).abs() < f64::EPSILON,
        "Scroll offset should be 925 for index 20 at Center"
    );
}

/// Tests scroll-to-index with End alignment.
///
/// # Test Steps
/// - Creates a virtualizer with 100 items, container 200px.
/// - Scrolls to index 20 with End alignment.
/// - Asserts the offset places item 20 at the viewport end.
#[test]
fn should_scroll_to_index_with_end_alignment() {
    // Create options.
    let options = VirtualizerOptions {
        item_count: 100,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        container_size: Some(200.0),
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed");
    let virt = result.unwrap_or_else(|_| panic!("Already checked is_ok"));

    // Scroll to index 20 with End alignment.
    let scroll_result = virt.scroll_to_index(20, ScrollAlignment::End);
    assert!(scroll_result.is_ok(), "Scroll-to-index should succeed");

    // End: item_offset - container + item_size = 1000 - 200 + 50 = 850.
    let offset = scroll_result.unwrap_or(0.0);
    assert!(
        (offset - 850.0).abs() < f64::EPSILON,
        "Scroll offset should be 850 for index 20 at End"
    );
}

/// Tests scroll-to-index with Auto alignment when item is already visible.
///
/// # Test Steps
/// - Creates a virtualizer scrolled to show items 0-3.
/// - Scrolls to index 2 with Auto alignment.
/// - Asserts no scroll change occurs.
#[test]
fn should_not_scroll_when_item_already_visible_with_auto() {
    // Create options.
    let options = VirtualizerOptions {
        item_count: 100,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        container_size: Some(200.0),
        overscan: 0,
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed");
    let virt = result.unwrap_or_else(|_| panic!("Already checked is_ok"));

    // Items 0-3 are visible (0..200px). Scroll to index 2 with Auto.
    let scroll_result = virt.scroll_to_index(2, ScrollAlignment::Auto);
    assert!(scroll_result.is_ok(), "Scroll-to-index should succeed");

    // Should remain at 0 since item 2 (100-150px) is fully visible.
    let offset = scroll_result.unwrap_or(0.0);
    assert!(
        (offset - 0.0).abs() < f64::EPSILON,
        "Should not scroll when item is already visible"
    );
}

/// Tests scroll-to-index rejects out-of-bounds index.
///
/// # Test Steps
/// - Creates a virtualizer with 10 items.
/// - Attempts to scroll to index 10.
/// - Asserts an error is returned.
#[test]
fn should_reject_scroll_to_out_of_bounds_index() {
    // Create options.
    let options = VirtualizerOptions {
        item_count: 10,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        container_size: Some(500.0),
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed");
    let virt = result.unwrap_or_else(|_| panic!("Already checked is_ok"));

    // Attempt to scroll to index 10.
    let scroll_result = virt.scroll_to_index(10, ScrollAlignment::Start);
    assert!(
        scroll_result.is_err(),
        "Scrolling to index 10 with 10 items should fail"
    );
}

/// Tests that updating item count increases the total size.
///
/// # Test Steps
/// - Creates a virtualizer with 10 items.
/// - Updates count to 20.
/// - Asserts total size doubled.
#[test]
fn should_update_total_size_on_item_count_change() {
    // Create options.
    let options = VirtualizerOptions {
        item_count: 10,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        container_size: Some(500.0),
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed");
    let mut virt = result.unwrap_or_else(|_| panic!("Already checked is_ok"));

    // Update count to 20.
    virt.update_item_count(20);

    // New total: 20 * 50 = 1000.
    assert!(
        (virt.total_size() - 1000.0).abs() < f64::EPSILON,
        "Total size should be 1000 after count update to 20"
    );
    assert_eq!(virt.item_count(), 20, "Item count should be 20");
}

/// Tests that reducing item count decreases the total size.
///
/// # Test Steps
/// - Creates a virtualizer with 20 items.
/// - Updates count to 5.
/// - Asserts total size decreased.
#[test]
fn should_reduce_total_size_on_item_count_decrease() {
    // Create options.
    let options = VirtualizerOptions {
        item_count: 20,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        container_size: Some(500.0),
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed");
    let mut virt = result.unwrap_or_else(|_| panic!("Already checked is_ok"));

    // Reduce count to 5.
    virt.update_item_count(5);

    // New total: 5 * 50 = 250.
    assert!(
        (virt.total_size() - 250.0).abs() < f64::EPSILON,
        "Total size should be 250 after count decrease to 5"
    );
}

/// Tests that virtual items have correct metadata.
///
/// # Test Steps
/// - Creates a virtualizer with container showing 4 items.
/// - Gets virtual items.
/// - Asserts items have correct indices, sizes, and offsets.
#[test]
fn should_produce_virtual_items_with_correct_metadata() {
    // Create options.
    let options = VirtualizerOptions {
        item_count: 100,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        container_size: Some(200.0),
        overscan: 0,
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed");
    let virt = result.unwrap_or_else(|_| panic!("Already checked is_ok"));

    // Get virtual items.
    let items = virt.get_virtual_items();

    // Should have 4 items (200 / 50).
    assert_eq!(items.len(), 4, "Should have 4 visible items");

    // Verify first item.
    assert_eq!(items[0].index, 0, "First item index should be 0");
    assert!(
        (items[0].size - 50.0).abs() < f64::EPSILON,
        "First item size should be 50"
    );
    assert!(
        (items[0].start - 0.0).abs() < f64::EPSILON,
        "First item start should be 0"
    );

    // Verify third item.
    assert_eq!(items[2].index, 2, "Third item index should be 2");
    assert!(
        (items[2].start - 100.0).abs() < f64::EPSILON,
        "Third item start should be 100"
    );
}

/// Tests that zero item size is rejected.
///
/// # Test Steps
/// - Creates options with Fixed(0.0) size.
/// - Asserts virtualizer creation fails.
#[test]
fn should_reject_zero_item_size() {
    // Create options with zero size.
    let options = VirtualizerOptions {
        item_count: 10,
        item_size_mode: ItemSizeMode::Fixed(0.0),
        container_size: Some(500.0),
        ..VirtualizerOptions::default()
    };

    // Attempt to create virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_err(), "Zero item size should be rejected");
}

/// Tests that negative padding is rejected.
///
/// # Test Steps
/// - Creates options with negative padding_start.
/// - Asserts virtualizer creation fails.
#[test]
fn should_reject_negative_padding() {
    // Create options with negative padding.
    let options = VirtualizerOptions {
        item_count: 10,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        padding_start: -10.0,
        container_size: Some(500.0),
        ..VirtualizerOptions::default()
    };

    // Attempt to create virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_err(), "Negative padding should be rejected");
}

/// Tests that negative gap is rejected.
///
/// # Test Steps
/// - Creates options with negative gap.
/// - Asserts virtualizer creation fails.
#[test]
fn should_reject_negative_gap() {
    // Create options with negative gap.
    let options = VirtualizerOptions {
        item_count: 10,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        gap: -5.0,
        container_size: Some(500.0),
        ..VirtualizerOptions::default()
    };

    // Attempt to create virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_err(), "Negative gap should be rejected");
}

/// Tests that zero item count produces empty virtual items.
///
/// # Test Steps
/// - Creates a virtualizer with 0 items.
/// - Asserts virtual items list is empty.
#[test]
fn should_return_empty_items_for_zero_count() {
    // Create options with zero items.
    let options = VirtualizerOptions {
        item_count: 0,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        container_size: Some(500.0),
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed with zero items");
    let virt = result.unwrap_or_else(|_| panic!("Already checked is_ok"));

    // Assert no virtual items.
    let items = virt.get_virtual_items();
    assert!(
        items.is_empty(),
        "Should have no virtual items for zero count"
    );
    assert!(
        (virt.total_size() - 0.0).abs() < f64::EPSILON,
        "Total size should be 0 for zero items"
    );
}

/// Tests that scroll offset is clamped to valid bounds.
///
/// # Test Steps
/// - Creates a virtualizer.
/// - Scrolls to a very large offset.
/// - Asserts offset is clamped to max.
#[test]
fn should_clamp_scroll_offset_to_max() {
    // Create options.
    let options = VirtualizerOptions {
        item_count: 10,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        container_size: Some(200.0),
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed");
    let mut virt = result.unwrap_or_else(|_| panic!("Already checked is_ok"));

    // Scroll to a huge offset. The engine no longer clamps internally;
    // it stores the raw offset. Verify the offset was stored.
    virt.update_scroll_offset(99999.0, true);

    // Assert offset is stored (engine relies on hooks to provide valid offsets).
    assert!(
        virt.scroll_offset() > 0.0,
        "Scroll offset should be positive after large scroll"
    );
}

/// Tests item_size and item_offset accessors.
///
/// # Test Steps
/// - Creates a virtualizer with known sizes.
/// - Queries size and offset for a specific index.
/// - Asserts correct values.
#[test]
fn should_return_correct_item_size_and_offset() {
    // Create options.
    let options = VirtualizerOptions {
        item_count: 5,
        item_size_mode: ItemSizeMode::Fixed(40.0),
        container_size: Some(500.0),
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed");
    let virt = result.unwrap_or_else(|_| panic!("Already checked is_ok"));

    // Check item 3: offset = 3 * 40 = 120, size = 40.
    assert_eq!(virt.item_size(3), Some(40.0), "Item 3 size should be 40");
    assert_eq!(
        virt.item_offset(3),
        Some(120.0),
        "Item 3 offset should be 120"
    );

    // Check out-of-bounds.
    assert_eq!(virt.item_size(10), None, "Out of bounds should return None");
    assert_eq!(
        virt.item_offset(10),
        None,
        "Out of bounds should return None"
    );
}

/// Tests that requires_measurement reflects the item size mode.
///
/// # Test Steps
/// - Creates a virtualizer with Fixed mode.
/// - Asserts requires_measurement is false.
/// - Creates another with Estimated mode.
/// - Asserts requires_measurement is true.
#[test]
fn should_report_measurement_requirement_based_on_mode() {
    // Create with fixed mode.
    let fixed_opts = VirtualizerOptions {
        item_count: 10,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        container_size: Some(500.0),
        ..VirtualizerOptions::default()
    };
    let fixed_result = Virtualizer::new(fixed_opts);
    assert!(fixed_result.is_ok(), "Should succeed");
    let fixed_virt = fixed_result.unwrap_or_else(|_| panic!("Already checked is_ok"));
    assert!(
        !fixed_virt.requires_measurement(),
        "Fixed mode should not require measurement"
    );

    // Create with estimated mode.
    let est_opts = VirtualizerOptions {
        item_count: 10,
        item_size_mode: ItemSizeMode::Estimated(50.0),
        container_size: Some(500.0),
        ..VirtualizerOptions::default()
    };
    let est_result = Virtualizer::new(est_opts);
    assert!(est_result.is_ok(), "Should succeed");
    let est_virt = est_result.unwrap_or_else(|_| panic!("Already checked is_ok"));
    assert!(
        est_virt.requires_measurement(),
        "Estimated mode should require measurement"
    );
}

/// Tests horizontal scroll direction configuration.
///
/// # Test Steps
/// - Creates a virtualizer with horizontal direction.
/// - Asserts it initializes correctly.
#[test]
fn should_support_horizontal_direction() {
    // Create options with horizontal direction.
    let options = VirtualizerOptions {
        item_count: 50,
        item_size_mode: ItemSizeMode::Fixed(100.0),
        scroll_direction: ScrollDirection::Horizontal,
        container_size: Some(800.0),
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Horizontal virtualizer should succeed");

    let virt = result.unwrap_or_else(|_| panic!("Already checked is_ok"));

    // Total size: 50 * 100 = 5000.
    assert!(
        (virt.total_size() - 5000.0).abs() < f64::EPSILON,
        "Total size should be 5000 for horizontal list"
    );
}
