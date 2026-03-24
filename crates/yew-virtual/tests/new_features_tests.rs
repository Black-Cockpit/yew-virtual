//! Unit tests for new TanStack Virtual parity features.
//!
//! This module verifies the correctness of all new features added
//! for TanStack Virtual parity including multi-lane support, per-index
//! estimation, custom keys, scroll behavior, scroll state, enabled toggle,
//! scroll padding, scroll margin, set_options, scroll_to_offset, scroll_by,
//! get_virtual_item_for_offset, measure, is_scrolling tracking, scroll
//! adjustments, custom range extractor, and initial measurements cache.
//!
//! Test Scenarios:
//! - Multi-lane layout distributes items across lanes.
//! - Per-index estimate_size callback is used for initial sizes.
//! - Custom get_item_key produces stable keys.
//! - ScrollBehavior and ScrollToOptions defaults are correct.
//! - Enabled toggle disables output without destroying state.
//! - Scroll padding affects scroll-to-index alignment.
//! - Scroll margin affects item offsets and total size.
//! - set_options updates configuration dynamically.
//! - prepare_scroll_to_offset computes correct offset.
//! - prepare_scroll_by computes correct relative offset.
//! - get_virtual_item_for_offset returns the correct item.
//! - measure() force re-measures all items.
//! - is_scrolling tracks scroll state.
//! - Scroll adjustments accumulate on item resize above viewport.
//! - Custom range_extractor controls rendered indices.
//! - initial_measurements_cache is used on first build.
//! - Zero lanes are rejected.
//! - Rect defaults to zero.
//! - VirtualKey supports multiple variants and Display.
//! - VisibleRange stores correct fields.
//! - ScrollState tracks reconciliation progress.

use std::sync::Arc;

use yew_virtual::core::item_size_mode::ItemSizeMode;
use yew_virtual::core::rect::Rect;
use yew_virtual::core::scroll_alignment::ScrollAlignment;
use yew_virtual::core::scroll_behavior::ScrollBehavior;
use yew_virtual::core::scroll_state::ScrollState;
use yew_virtual::core::scroll_to_options::ScrollToOptions;
use yew_virtual::core::virtual_item::VirtualItem;
use yew_virtual::core::virtual_key::VirtualKey;
use yew_virtual::core::virtualizer::Virtualizer;
use yew_virtual::core::virtualizer_options::VirtualizerOptions;
use yew_virtual::core::visible_range::VisibleRange;

/// Tests that multi-lane layout distributes items across lanes.
///
/// # Test Steps
/// - Creates a virtualizer with 2 lanes and 6 items.
/// - Asserts items are distributed across 2 lanes.
#[test]
fn should_distribute_items_across_lanes() {
    // Create options with 2 lanes.
    let options = VirtualizerOptions {
        item_count: 6,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        lanes: 2,
        container_size: Some(500.0),
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed with 2 lanes");

    let virt = result.unwrap_or_else(|_| Virtualizer::empty());

    // Get measurements and check lane assignments.
    let measurements = virt.measurements();
    assert_eq!(measurements.len(), 6, "Should have 6 measurements");

    // Items should be distributed across lanes 0 and 1.
    let lane_0_count = measurements.iter().filter(|m| m.lane == 0).count();
    let lane_1_count = measurements.iter().filter(|m| m.lane == 1).count();
    assert!(lane_0_count > 0, "Lane 0 should have items");
    assert!(lane_1_count > 0, "Lane 1 should have items");
}

/// Tests that per-index estimate_size callback is used.
///
/// # Test Steps
/// - Creates a virtualizer with a custom estimate_size that returns different sizes.
/// - Asserts items have different estimated sizes.
#[test]
fn should_use_per_index_estimate_size() {
    // Create options with per-index estimator.
    let options = VirtualizerOptions {
        item_count: 5,
        item_size_mode: ItemSizeMode::Estimated(50.0),
        container_size: Some(500.0),
        estimate_size: Some(Arc::new(|index| (index as f64 + 1.0) * 20.0)),
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed with estimate_size callback");

    let virt = result.unwrap_or_else(|_| Virtualizer::empty());

    // Item 0 should be 20px, item 1 should be 40px, etc.
    assert_eq!(virt.item_size(0), Some(20.0), "Item 0 should be 20px");
    assert_eq!(virt.item_size(1), Some(40.0), "Item 1 should be 40px");
    assert_eq!(virt.item_size(4), Some(100.0), "Item 4 should be 100px");
}

/// Tests that custom get_item_key produces stable keys.
///
/// # Test Steps
/// - Creates a virtualizer with a custom key extractor.
/// - Asserts virtual items have the custom keys.
#[test]
fn should_use_custom_item_keys() {
    // Create options with custom key extractor.
    let options = VirtualizerOptions {
        item_count: 3,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        container_size: Some(500.0),
        get_item_key: Some(Arc::new(|index| {
            VirtualKey::Named(format!("item-{}", index))
        })),
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed with custom keys");

    let virt = result.unwrap_or_else(|_| Virtualizer::empty());

    // Get virtual items and check keys.
    let items = virt.get_virtual_items();
    assert!(items.len() >= 3, "Should have at least 3 items");
    assert_eq!(
        items[0].key,
        VirtualKey::Named("item-0".to_string()),
        "Key should be custom"
    );
    assert_eq!(
        items[1].key,
        VirtualKey::Named("item-1".to_string()),
        "Key should be custom"
    );
}

/// Tests that ScrollBehavior defaults to Auto.
///
/// # Test Steps
/// - Creates a default ScrollBehavior.
/// - Asserts it is Auto.
#[test]
fn should_default_scroll_behavior_to_auto() {
    // Create the default behavior.
    let behavior = ScrollBehavior::default();

    // Assert it is Auto.
    assert_eq!(behavior, ScrollBehavior::Auto, "Default should be Auto");
}

/// Tests that ScrollToOptions defaults correctly.
///
/// # Test Steps
/// - Creates default ScrollToOptions.
/// - Asserts align is Auto and behavior is Auto.
#[test]
fn should_default_scroll_to_options() {
    // Create default options.
    let opts = ScrollToOptions::default();

    // Assert defaults.
    assert_eq!(
        opts.align,
        ScrollAlignment::Auto,
        "Default align should be Auto"
    );
    assert_eq!(
        opts.behavior,
        ScrollBehavior::Auto,
        "Default behavior should be Auto"
    );
}

/// Tests that enabled toggle disables output.
///
/// # Test Steps
/// - Creates a virtualizer with enabled=false.
/// - Asserts get_virtual_items returns empty.
#[test]
fn should_return_empty_items_when_disabled() {
    // Create options with enabled=false.
    let options = VirtualizerOptions {
        item_count: 100,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        container_size: Some(500.0),
        enabled: false,
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed when disabled");

    let virt = result.unwrap_or_else(|_| Virtualizer::empty());

    // Assert no virtual items.
    let items = virt.get_virtual_items();
    assert!(items.is_empty(), "Should have no items when disabled");
    assert!(!virt.is_enabled(), "Should report as disabled");
}

/// Tests that scroll padding affects scroll-to-index alignment.
///
/// # Test Steps
/// - Creates a virtualizer with scroll_padding_start=10.
/// - Scrolls to an item with Start alignment.
/// - Asserts the offset accounts for scroll padding.
#[test]
fn should_apply_scroll_padding_to_scroll_to_index() {
    // Create options with scroll padding.
    let options = VirtualizerOptions {
        item_count: 100,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        container_size: Some(200.0),
        scroll_padding_start: 10.0,
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed");

    let virt = result.unwrap_or_else(|_| Virtualizer::empty());

    // Scroll to index 20 with Start alignment.
    let offset_result = virt.scroll_to_index(20, ScrollAlignment::Start);
    assert!(offset_result.is_ok(), "Should succeed");

    // The offset should be item_start - scroll_padding_start = 1000 - 10 = 990.
    let offset = offset_result.unwrap_or(0.0);
    assert!(
        (offset - 990.0).abs() < f64::EPSILON,
        "Offset should account for scroll_padding_start"
    );
}

/// Tests that scroll margin affects item offsets.
///
/// # Test Steps
/// - Creates a virtualizer with scroll_margin=20.
/// - Asserts the first item starts at padding_start + scroll_margin.
#[test]
fn should_apply_scroll_margin_to_item_offsets() {
    // Create options with scroll margin.
    let options = VirtualizerOptions {
        item_count: 5,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        container_size: Some(500.0),
        scroll_margin: 20.0,
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed");

    let virt = result.unwrap_or_else(|_| Virtualizer::empty());

    // First item should start at scroll_margin (0 + 20 = 20).
    assert_eq!(
        virt.item_offset(0),
        Some(20.0),
        "First item should start at scroll_margin"
    );
}

/// Tests that set_options updates configuration dynamically.
///
/// # Test Steps
/// - Creates a virtualizer, then calls set_options with new count.
/// - Asserts the new count is reflected.
#[test]
fn should_update_options_dynamically() {
    // Create initial options.
    let options = VirtualizerOptions {
        item_count: 10,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        container_size: Some(500.0),
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed");
    let mut virt = result.unwrap_or_else(|_| Virtualizer::empty());

    // Update options with new count.
    let new_options = VirtualizerOptions {
        item_count: 20,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        container_size: Some(500.0),
        ..VirtualizerOptions::default()
    };
    let update_result = virt.set_options(new_options);
    assert!(update_result.is_ok(), "set_options should succeed");

    // Assert new count is reflected.
    assert_eq!(
        virt.item_count(),
        20,
        "Item count should be 20 after set_options"
    );
}

/// Tests that prepare_scroll_to_offset computes correct offset.
///
/// # Test Steps
/// - Creates a virtualizer and prepares a scroll to offset 500.
/// - Asserts the returned state has the correct target.
#[test]
fn should_prepare_scroll_to_offset() {
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
    let mut virt = result.unwrap_or_else(|_| Virtualizer::empty());

    // Prepare scroll to offset 500.
    let state = virt.prepare_scroll_to_offset(500.0, ScrollToOptions::default(), 0.0);

    // Assert the target offset.
    assert!(
        (state.last_target_offset - 500.0).abs() < 1.0,
        "Target offset should be close to 500"
    );
}

/// Tests that prepare_scroll_by computes correct relative offset.
///
/// # Test Steps
/// - Creates a virtualizer at offset 100, then prepares scroll_by 200.
/// - Asserts the target is 300.
#[test]
fn should_prepare_scroll_by() {
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
    let mut virt = result.unwrap_or_else(|_| Virtualizer::empty());

    // Set initial scroll offset.
    virt.update_scroll_offset(100.0, false);

    // Prepare scroll by 200.
    let state = virt.prepare_scroll_by(200.0, ScrollBehavior::Auto, 0.0);

    // Assert the target offset is 300.
    assert!(
        (state.last_target_offset - 300.0).abs() < f64::EPSILON,
        "Target should be 300 (100 + 200)"
    );
}

/// Tests that get_virtual_item_for_offset returns the correct item.
///
/// # Test Steps
/// - Creates a virtualizer with 10 items of size 50.
/// - Queries for offset 125 which is in the middle of item 2.
/// - Asserts item 2 is returned.
#[test]
fn should_find_item_for_offset() {
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
    let virt = result.unwrap_or_else(|_| Virtualizer::empty());

    // Query for offset 125 (in the middle of item 2: 100-150).
    let item = virt.get_virtual_item_for_offset(125.0);
    assert!(item.is_some(), "Should find an item");
    assert_eq!(item.map(|i| i.index), Some(2), "Should be item 2");
}

/// Tests that measure() force re-measures all items.
///
/// # Test Steps
/// - Creates a virtualizer, measures an item, then calls measure().
/// - Asserts the measurement is cleared.
#[test]
fn should_force_remeasure_all_items() {
    // Create options.
    let options = VirtualizerOptions {
        item_count: 5,
        item_size_mode: ItemSizeMode::Estimated(50.0),
        container_size: Some(500.0),
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed");
    let mut virt = result.unwrap_or_else(|_| Virtualizer::empty());

    // Measure item 0 as 100.
    let _ = virt.measure_item(0, 100.0);
    assert_eq!(
        virt.item_size(0),
        Some(100.0),
        "Item 0 should be 100 after measure"
    );

    // Force re-measure.
    virt.measure();

    // Item 0 should revert to the estimated size.
    assert_eq!(
        virt.item_size(0),
        Some(50.0),
        "Item 0 should revert to estimate after measure()"
    );
}

/// Tests that is_scrolling tracks scroll state.
///
/// # Test Steps
/// - Creates a virtualizer, asserts is_scrolling is false.
/// - Updates scroll offset with is_scrolling=true.
/// - Asserts is_scrolling is true.
/// - Sets is_scrolling to false.
/// - Asserts is_scrolling is false.
#[test]
fn should_track_is_scrolling_state() {
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
    let mut virt = result.unwrap_or_else(|_| Virtualizer::empty());

    // Initially not scrolling.
    assert!(!virt.is_scrolling(), "Should not be scrolling initially");

    // Scroll with is_scrolling=true.
    virt.update_scroll_offset(100.0, true);
    assert!(
        virt.is_scrolling(),
        "Should be scrolling after scroll event"
    );

    // Set scrolling to false.
    virt.set_is_scrolling(false);
    assert!(!virt.is_scrolling(), "Should not be scrolling after reset");
}

/// Tests scroll direction (forward/backward) tracking.
///
/// # Test Steps
/// - Scrolls forward, asserts direction is Some(true).
/// - Scrolls backward, asserts direction is Some(false).
#[test]
fn should_track_scroll_direction() {
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
    let mut virt = result.unwrap_or_else(|_| Virtualizer::empty());

    // Scroll forward.
    virt.update_scroll_offset(100.0, true);
    assert_eq!(
        virt.is_scroll_forward(),
        Some(true),
        "Should be scrolling forward"
    );

    // Scroll backward.
    virt.update_scroll_offset(50.0, true);
    assert_eq!(
        virt.is_scroll_forward(),
        Some(false),
        "Should be scrolling backward"
    );
}

/// Tests that custom range extractor controls rendered indices.
///
/// # Test Steps
/// - Creates a virtualizer with a custom range extractor that always includes index 0.
/// - Asserts index 0 is always in the virtual items.
#[test]
fn should_use_custom_range_extractor() {
    // Create options with custom range extractor (always include index 0 as sticky).
    let options = VirtualizerOptions {
        item_count: 100,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        container_size: Some(200.0),
        range_extractor: Some(Arc::new(|range| {
            // Always include index 0 (sticky header).
            let mut indices: Vec<usize> = Vec::new();
            indices.push(0);
            let start = range.start_index.saturating_sub(range.overscan);
            let end = (range.end_index + range.overscan).min(range.count.saturating_sub(1));
            for i in start..=end {
                if i != 0 {
                    indices.push(i);
                }
            }
            indices
        })),
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed");
    let mut virt = result.unwrap_or_else(|_| Virtualizer::empty());

    // Scroll far down.
    virt.update_scroll_offset(2000.0, true);

    // Get virtual items.
    let items = virt.get_virtual_items();

    // Index 0 should be present (sticky header).
    let has_zero = items.iter().any(|i| i.index == 0);
    assert!(
        has_zero,
        "Index 0 should always be present due to custom extractor"
    );
}

/// Tests that zero lanes are rejected.
///
/// # Test Steps
/// - Creates options with lanes=0.
/// - Asserts virtualizer creation fails.
#[test]
fn should_reject_zero_lanes() {
    // Create options with zero lanes.
    let options = VirtualizerOptions {
        item_count: 10,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        container_size: Some(500.0),
        lanes: 0,
        ..VirtualizerOptions::default()
    };

    // Attempt to create virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_err(), "Zero lanes should be rejected");
}

/// Tests that Rect defaults to zero.
///
/// # Test Steps
/// - Creates a default Rect.
/// - Asserts width and height are zero.
#[test]
fn should_default_rect_to_zero() {
    // Create default rect.
    let rect = Rect::default();

    // Assert zero dimensions.
    assert!((rect.width - 0.0).abs() < f64::EPSILON, "Width should be 0");
    assert!(
        (rect.height - 0.0).abs() < f64::EPSILON,
        "Height should be 0"
    );
}

/// Tests VirtualKey variants and Display.
///
/// # Test Steps
/// - Creates Index and Named keys.
/// - Asserts Display formatting.
/// - Asserts From conversions.
#[test]
fn should_support_virtual_key_variants() {
    // Create an index key.
    let key_idx = VirtualKey::Index(42);
    assert_eq!(
        format!("{}", key_idx),
        "42",
        "Index key should display as number"
    );

    // Create a named key.
    let key_named = VirtualKey::Named("item-1".to_string());
    assert_eq!(
        format!("{}", key_named),
        "item-1",
        "Named key should display as string"
    );

    // From conversions.
    let from_usize: VirtualKey = 5usize.into();
    assert_eq!(from_usize, VirtualKey::Index(5), "From<usize> should work");

    let from_str: VirtualKey = "test".into();
    assert_eq!(
        from_str,
        VirtualKey::Named("test".to_string()),
        "From<&str> should work"
    );
}

/// Tests VisibleRange stores correct fields.
///
/// # Test Steps
/// - Creates a VisibleRange with known values.
/// - Asserts all fields are correct.
#[test]
fn should_store_visible_range_fields() {
    // Create a visible range.
    let range = VisibleRange {
        start_index: 5,
        end_index: 15,
        overscan: 3,
        count: 100,
    };

    // Assert all fields.
    assert_eq!(range.start_index, 5, "start_index should be 5");
    assert_eq!(range.end_index, 15, "end_index should be 15");
    assert_eq!(range.overscan, 3, "overscan should be 3");
    assert_eq!(range.count, 100, "count should be 100");
}

/// Tests ScrollState tracks reconciliation progress.
///
/// # Test Steps
/// - Creates a ScrollState.
/// - Asserts initial stable_frames is 0.
#[test]
fn should_create_scroll_state_with_zero_stable_frames() {
    // Create a scroll state.
    let state = ScrollState::new(
        Some(10),
        ScrollAlignment::Center,
        ScrollBehavior::Smooth,
        1000.0,
        500.0,
    );

    // Assert fields.
    assert_eq!(state.index, Some(10), "Index should be Some(10)");
    assert_eq!(
        state.align,
        ScrollAlignment::Center,
        "Align should be Center"
    );
    assert_eq!(
        state.behavior,
        ScrollBehavior::Smooth,
        "Behavior should be Smooth"
    );
    assert!(
        (state.started_at - 1000.0).abs() < f64::EPSILON,
        "started_at should be 1000"
    );
    assert!(
        (state.last_target_offset - 500.0).abs() < f64::EPSILON,
        "last_target_offset should be 500"
    );
    assert_eq!(
        state.stable_frames, 0,
        "stable_frames should be 0 initially"
    );
}

/// Tests that initial_offset is applied on creation.
///
/// # Test Steps
/// - Creates a virtualizer with initial_offset=100.
/// - Asserts the scroll_offset starts at 100.
#[test]
fn should_apply_initial_offset() {
    // Create options with initial offset.
    let options = VirtualizerOptions {
        item_count: 100,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        container_size: Some(200.0),
        initial_offset: 100.0,
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed");
    let virt = result.unwrap_or_else(|_| Virtualizer::empty());

    // Assert initial scroll offset.
    assert!(
        (virt.scroll_offset() - 100.0).abs() < f64::EPSILON,
        "Scroll offset should start at 100"
    );
}

/// Tests that new default options have correct new field values.
///
/// # Test Steps
/// - Creates default VirtualizerOptions.
/// - Asserts all new fields have expected defaults.
#[test]
fn should_have_correct_new_default_values() {
    // Create default options.
    let opts = VirtualizerOptions::default();

    // Assert new field defaults.
    assert_eq!(opts.lanes, 1, "Default lanes should be 1");
    assert!(
        (opts.scroll_margin - 0.0).abs() < f64::EPSILON,
        "Default scroll_margin should be 0"
    );
    assert!(
        (opts.scroll_padding_start - 0.0).abs() < f64::EPSILON,
        "Default scroll_padding_start should be 0"
    );
    assert!(
        (opts.scroll_padding_end - 0.0).abs() < f64::EPSILON,
        "Default scroll_padding_end should be 0"
    );
    assert!(
        (opts.initial_offset - 0.0).abs() < f64::EPSILON,
        "Default initial_offset should be 0"
    );
    assert!(opts.enabled, "Default enabled should be true");
    assert!(!opts.is_rtl, "Default is_rtl should be false");
    assert_eq!(
        opts.index_attribute, "data-index",
        "Default index_attribute should be data-index"
    );
    assert_eq!(
        opts.is_scrolling_reset_delay, 150,
        "Default is_scrolling_reset_delay should be 150"
    );
    assert!(
        !opts.use_scrollend_event,
        "Default use_scrollend_event should be false"
    );
    assert!(
        !opts.use_animation_frame_with_resize_observer,
        "Default should be false"
    );
    assert!(
        opts.estimate_size.is_none(),
        "Default estimate_size should be None"
    );
    assert!(
        opts.get_item_key.is_none(),
        "Default get_item_key should be None"
    );
    assert!(
        opts.range_extractor.is_none(),
        "Default range_extractor should be None"
    );
    assert!(
        opts.initial_measurements_cache.is_empty(),
        "Default initial_measurements_cache should be empty"
    );
}

/// Tests that scroll adjustments accumulate on item resize above viewport.
///
/// # Test Steps
/// - Creates a virtualizer with estimated sizes.
/// - Scrolls to an offset so items 0-2 are above viewport.
/// - Measures item 0 larger. Asserts scroll_adjustments increased.
#[test]
fn should_accumulate_scroll_adjustments_on_resize_above_viewport() {
    // Create options.
    let options = VirtualizerOptions {
        item_count: 20,
        item_size_mode: ItemSizeMode::Estimated(50.0),
        container_size: Some(200.0),
        overscan: 0,
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed");
    let mut virt = result.unwrap_or_else(|_| Virtualizer::empty());

    // Scroll to 500 (items 0-9 are above/at viewport).
    virt.update_scroll_offset(500.0, false);

    // Measure item 0 as 100px (was 50px, delta=50).
    let _ = virt.measure_item(0, 100.0);

    // Scroll adjustments should be 50 (item 0 was above viewport).
    assert!(
        (virt.scroll_adjustments() - 50.0).abs() < f64::EPSILON,
        "Scroll adjustments should be 50 after resizing item above viewport"
    );
}

/// Tests prepare_scroll_to_index with full options.
///
/// # Test Steps
/// - Creates a virtualizer and prepares scroll to index 20.
/// - Asserts the returned state has correct fields.
#[test]
fn should_prepare_scroll_to_index_with_options() {
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
    let mut virt = result.unwrap_or_else(|_| Virtualizer::empty());

    // Prepare scroll to index 20 with smooth behavior.
    let scroll_opts = ScrollToOptions {
        align: ScrollAlignment::Start,
        behavior: ScrollBehavior::Smooth,
    };
    let state_result = virt.prepare_scroll_to_index(20, scroll_opts, 0.0);
    assert!(state_result.is_ok(), "Should succeed");

    let state = state_result.unwrap_or_else(|_| {
        ScrollState::new(None, ScrollAlignment::Auto, ScrollBehavior::Auto, 0.0, 0.0)
    });

    // Assert state fields.
    assert_eq!(state.index, Some(20), "Index should be Some(20)");
    assert_eq!(
        state.behavior,
        ScrollBehavior::Smooth,
        "Behavior should be Smooth"
    );

    // Active scroll state should be set.
    assert!(
        virt.scroll_state().is_some(),
        "Should have active scroll state"
    );

    // Clear it.
    virt.clear_scroll_state();
    assert!(
        virt.scroll_state().is_none(),
        "Should have no scroll state after clear"
    );
}

/// Tests that initial_measurements_cache is used on first build.
///
/// # Test Steps
/// - Creates a virtualizer with pre-populated measurements.
/// - Asserts the items use the cached sizes.
#[test]
fn should_use_initial_measurements_cache() {
    // Create pre-populated measurements.
    let initial_cache = vec![
        VirtualItem::new(0, 100.0, 0.0),
        VirtualItem::new(1, 75.0, 100.0),
        VirtualItem::new(2, 50.0, 175.0),
    ];

    // Create options with initial cache.
    let options = VirtualizerOptions {
        item_count: 3,
        item_size_mode: ItemSizeMode::Estimated(50.0),
        container_size: Some(500.0),
        initial_measurements_cache: initial_cache,
        ..VirtualizerOptions::default()
    };

    // Create the virtualizer.
    let result = Virtualizer::new(options);
    assert!(result.is_ok(), "Should succeed with initial cache");

    let virt = result.unwrap_or_else(|_| Virtualizer::empty());

    // Items should use cached sizes.
    assert_eq!(
        virt.item_size(0),
        Some(100.0),
        "Item 0 should use cached size 100"
    );
    assert_eq!(
        virt.item_size(1),
        Some(75.0),
        "Item 1 should use cached size 75"
    );
    assert_eq!(
        virt.item_size(2),
        Some(50.0),
        "Item 2 should use cached size 50"
    );
}
