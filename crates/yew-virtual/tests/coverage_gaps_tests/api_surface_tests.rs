//! Unit tests for the `api_surface_tests` module.
//!
//! This module verifies auxiliary public APIs and layout edge branches.
//! It ensures consistent formatting and predictable scroll reconciliation for integrators.
//!
//! Test Scenarios:
//! - VirtualKey conversions and Display output.
//! - VirtualizerError and VirtualizerOptions surface behavior.
//! - Scroll reconciliation, lane changes, and multi-lane range calculation.

use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;

use yew_virtual::core::item_size_mode::ItemSizeMode;
use yew_virtual::core::measure_item_outcome::MeasureItemOutcome;
use yew_virtual::core::measurement_cache::MeasurementCache;
use yew_virtual::core::range_calculator::RangeCalculator;
use yew_virtual::core::rect::Rect;
use yew_virtual::core::scroll_behavior::ScrollBehavior;
use yew_virtual::core::scroll_direction::ScrollDirection;
use yew_virtual::core::scroll_reconcile_action::ScrollReconcileAction;
use yew_virtual::core::scroll_to_options::ScrollToOptions;
use yew_virtual::core::virtual_item::VirtualItem;
use yew_virtual::core::virtual_key::VirtualKey;
use yew_virtual::core::virtualizer::Virtualizer;
use yew_virtual::core::virtualizer_error::VirtualizerError;
use yew_virtual::core::virtualizer_options::VirtualizerOptions;

/// Tests VirtualKey default, From conversions, and Display formatting.
///
/// # Test Steps
/// - Builds keys from usize, String, and string slices.
/// - Asserts Default, equality, and Display strings for each variant.
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn should_resolve_virtual_key_defaults_and_display() {
    // Exercise Default for VirtualKey.
    assert_eq!(VirtualKey::default(), VirtualKey::Index(0));

    // Build a key from a usize via From.
    let k: VirtualKey = 7usize.into();
    assert_eq!(k, VirtualKey::Index(7));

    // Build named keys from owned String and from str slice.
    let named: VirtualKey = "row-a".to_string().into();
    let named2: VirtualKey = "row-b".into();
    assert_eq!(named, VirtualKey::Named("row-a".into()));
    assert_eq!(named2, VirtualKey::Named("row-b".into()));

    // Format both variants with Display.
    assert_eq!(format!("{}", VirtualKey::Index(42)), "42");
    assert_eq!(format!("{}", VirtualKey::Named("id".into())), "id");
}

/// Tests VirtualizerOptions Debug formatting and PartialEq inequality.
///
/// # Test Steps
/// - Formats default options with Debug.
/// - Builds differing scalar fields and asserts inequality against defaults.
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn should_format_virtualizer_options_debug_and_detect_inequality() {
    // Format default options and check struct name appears in output.
    let opts = VirtualizerOptions::default();
    let dbg = format!("{:?}", opts);
    assert!(
        dbg.contains("VirtualizerOptions") && dbg.contains("item_count"),
        "Debug should name the struct and scalar fields"
    );

    // Compare defaults to options with different scroll padding.
    let a = VirtualizerOptions::default();
    let b = VirtualizerOptions {
        scroll_padding_start: 1.0,
        ..VirtualizerOptions::default()
    };
    assert_ne!(a, b);

    // Compare defaults to RTL-flagged options.
    let b = VirtualizerOptions {
        is_rtl: true,
        ..VirtualizerOptions::default()
    };
    assert_ne!(a, b);

    // Compare defaults to scrollend-enabled options.
    let b = VirtualizerOptions {
        use_scrollend_event: true,
        ..VirtualizerOptions::default()
    };
    assert_ne!(a, b);
}

/// Tests Display text for every VirtualizerError variant.
///
/// # Test Steps
/// - Instantiates each error variant with sample payloads.
/// - Asserts formatted messages contain distinguishing substrings.
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn should_format_all_virtualizer_error_variants() {
    // Invalid item count message.
    let e = VirtualizerError::InvalidItemCount("n".into());
    assert!(e.to_string().contains("Invalid item count"));

    // Invalid item size message.
    let e = VirtualizerError::InvalidItemSize("s".into());
    assert!(e.to_string().contains("Invalid item size"));

    // Invalid overscan message.
    let e = VirtualizerError::InvalidOverscan("o".into());
    assert!(e.to_string().contains("Invalid overscan"));

    // Index out of bounds embeds requested and total counts.
    let e = VirtualizerError::IndexOutOfBounds {
        requested: 9,
        total: 3,
    };
    let msg = e.to_string();
    assert!(msg.contains("9") && msg.contains("3"));

    // Measurement error message.
    let e = VirtualizerError::MeasurementError("m".into());
    assert!(e.to_string().contains("Measurement error"));

    // Scroll container unavailable message.
    let e = VirtualizerError::ScrollContainerUnavailable("c".into());
    assert!(e.to_string().contains("Scroll container unavailable"));

    // Invalid configuration message.
    let e = VirtualizerError::InvalidConfiguration("x".into());
    assert!(e.to_string().contains("Invalid configuration"));
}

/// Tests MeasureItemOutcome::UNCHANGED matches a zero-compensation sentinel.
///
/// # Test Steps
/// - Compares UNCHANGED to an explicit struct with false layout_changed and zero compensation.
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn should_define_measure_item_outcome_unchanged_sentinel() {
    // UNCHANGED must match explicit zero-outcome layout.
    assert_eq!(
        MeasureItemOutcome::UNCHANGED,
        MeasureItemOutcome {
            layout_changed: false,
            scroll_compensation: 0.0,
        }
    );
}

/// Tests Virtualizer::empty yields zero items and no virtual rows.
///
/// # Test Steps
/// - Calls empty().
/// - Asserts item_count is zero and get_virtual_items is empty.
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn should_create_empty_virtualizer_without_items() {
    // Construct the empty fallback virtualizer.
    let virt = Virtualizer::empty();
    // Assert zero dataset and no rendered virtual items.
    assert_eq!(virt.item_count(), 0);
    assert!(virt.get_virtual_items().is_empty());
}

/// Tests Virtualizer::new rejects invalid sizes and padding.
///
/// # Test Steps
/// - Attempts construction with zero base size, negative padding, and NaN size.
/// - Asserts Err with expected error categories.
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn should_reject_invalid_virtualizer_new_options() {
    // Zero base size must yield InvalidItemSize.
    let bad = VirtualizerOptions {
        item_size_mode: ItemSizeMode::Fixed(0.0),
        ..VirtualizerOptions::default()
    };
    assert!(matches!(
        Virtualizer::new(bad),
        Err(VirtualizerError::InvalidItemSize(_))
    ));

    // Negative padding must yield InvalidConfiguration.
    let bad = VirtualizerOptions {
        padding_start: -1.0,
        ..VirtualizerOptions::default()
    };
    assert!(matches!(
        Virtualizer::new(bad),
        Err(VirtualizerError::InvalidConfiguration(_))
    ));

    // NaN base size must yield InvalidItemSize.
    let bad = VirtualizerOptions {
        item_size_mode: ItemSizeMode::Fixed(f64::NAN),
        ..VirtualizerOptions::default()
    };
    assert!(matches!(
        Virtualizer::new(bad),
        Err(VirtualizerError::InvalidItemSize(_))
    ));
}

/// Tests initial_offset_fn overrides scalar initial_offset on construction.
///
/// # Test Steps
/// - Supplies initial_offset_fn returning 80.0 with initial_offset 0.0.
/// - Asserts scroll_offset equals the callback result.
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn should_apply_initial_offset_fn_on_construction() {
    // Build options where the callback wins over the scalar offset.
    let opts = VirtualizerOptions {
        item_count: 5,
        item_size_mode: ItemSizeMode::Fixed(40.0),
        container_size: Some(200.0),
        initial_offset: 0.0,
        initial_offset_fn: Some(Arc::new(|| 80.0)),
        ..VirtualizerOptions::default()
    };
    // Construct and read scroll offset from the engine.
    let virt = Virtualizer::new(opts).expect("valid options");
    assert_eq!(virt.scroll_offset(), 80.0);
}

/// Tests horizontal container size falls back to initial_rect.width.
///
/// # Test Steps
/// - Omits container_size for a horizontal virtualizer.
/// - Asserts container_size resolves from initial_rect width.
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn should_resolve_horizontal_container_from_initial_rect_width() {
    // Horizontal list without explicit container uses initial_rect.width.
    let opts = VirtualizerOptions {
        item_count: 3,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        scroll_direction: ScrollDirection::Horizontal,
        container_size: None,
        initial_rect: Rect {
            width: 333.0,
            height: 0.0,
        },
        ..VirtualizerOptions::default()
    };
    let virt = Virtualizer::new(opts).expect("valid options");
    assert_eq!(virt.container_size(), 333.0);
}

/// Tests set_options clears lane-related caches when lane count decreases.
///
/// # Test Steps
/// - Builds a two-lane virtualizer and re-applies options to record prev_lanes.
/// - Reduces lanes to one and asserts the option sticks.
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn should_clear_lane_caches_when_lane_count_changes() {
    // Start with two lanes and a modest dataset.
    let opts_two = VirtualizerOptions {
        item_count: 10,
        item_size_mode: ItemSizeMode::Fixed(30.0),
        container_size: Some(300.0),
        lanes: 2,
        ..VirtualizerOptions::default()
    };
    let mut virt = Virtualizer::new(opts_two.clone()).expect("valid");

    // Re-apply the same options so prev_lanes is recorded.
    virt.set_options(opts_two).expect("re-apply same lanes");

    // Drop to a single lane and expect successful update.
    let mut opts_one = virt.options().clone();
    opts_one.lanes = 1;
    virt.set_options(opts_one)
        .expect("lane reduction should apply");

    assert_eq!(virt.options().lanes, 1);
}

/// Tests scroll reconciliation times out and clears scroll state.
///
/// # Test Steps
/// - Prepares programmatic scroll with a short reconciliation timeout.
/// - Advances time far into the future and expects Timeout plus cleared state.
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn should_timeout_programmatic_scroll_reconciliation() {
    // Tight timeout and many stable frames so timeout wins first.
    let opts = VirtualizerOptions {
        item_count: 5,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        container_size: Some(200.0),
        scroll_reconciliation_timeout_ms: 50,
        scroll_reconciliation_stable_frames: 100,
        ..VirtualizerOptions::default()
    };
    let mut virt = Virtualizer::new(opts).expect("valid");

    virt.prepare_scroll_to_offset(10.0, ScrollToOptions::default(), 0.0);

    // Large now_ms crosses the timeout threshold.
    let action = virt.scroll_reconciliation_tick(10.0, 1_000_000.0);
    assert_eq!(action, ScrollReconcileAction::Timeout);
    assert!(virt.scroll_state().is_none());
}

/// Tests scroll reconciliation completes when scroll matches target quickly.
///
/// # Test Steps
/// - Sets stable_frames to one for immediate settle.
/// - Ticks with current scroll equal to target offset and expects Done.
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn should_finish_reconciliation_when_scroll_matches_target() {
    // Single stable frame requirement for a fast Done path.
    let opts = VirtualizerOptions {
        item_count: 5,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        container_size: Some(200.0),
        scroll_reconciliation_stable_frames: 1,
        ..VirtualizerOptions::default()
    };
    let mut virt = Virtualizer::new(opts).expect("valid");

    virt.prepare_scroll_by(0.0, ScrollBehavior::Auto, 0.0);
    let target = virt.scroll_state().expect("state").last_target_offset;

    let a = virt.scroll_reconciliation_tick(target, 10.0);
    assert_eq!(a, ScrollReconcileAction::Done);
}

/// Tests scroll reconciliation returns Continue while far from the target.
///
/// # Test Steps
/// - Prepares a distant target with many required stable frames.
/// - Ticks with a mismatched current scroll and expects Continue.
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn should_continue_reconciliation_while_scroll_differs_from_target() {
    // High stable frame count keeps reconciliation active.
    let opts = VirtualizerOptions {
        item_count: 5,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        container_size: Some(200.0),
        scroll_reconciliation_stable_frames: 50,
        scroll_reconciliation_timeout_ms: 60_000,
        ..VirtualizerOptions::default()
    };
    let mut virt = Virtualizer::new(opts).expect("valid");

    virt.prepare_scroll_to_offset(500.0, ScrollToOptions::default(), 0.0);

    let action = virt.scroll_reconciliation_tick(0.0, 100.0);
    assert_eq!(action, ScrollReconcileAction::Continue);
}

/// Tests scroll_reconciliation_tick returns Done when no scroll state exists.
///
/// # Test Steps
/// - Constructs a virtualizer without programmatic scroll.
/// - Calls scroll_reconciliation_tick and expects Done.
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn should_return_done_from_reconciliation_tick_without_scroll_state() {
    let mut virt = Virtualizer::new(VirtualizerOptions {
        item_count: 2,
        item_size_mode: ItemSizeMode::Fixed(20.0),
        container_size: Some(100.0),
        ..VirtualizerOptions::default()
    })
    .expect("valid");

    assert_eq!(
        virt.scroll_reconciliation_tick(0.0, 0.0),
        ScrollReconcileAction::Done
    );
}

/// Tests refresh_programmatic_scroll_target does nothing without active scroll state.
///
/// # Test Steps
/// - Builds a virtualizer with no programmatic scroll.
/// - Calls refresh_programmatic_scroll_target and asserts scroll_state stays None.
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn should_noop_refresh_programmatic_target_without_scroll_state() {
    let mut virt = Virtualizer::new(VirtualizerOptions {
        item_count: 3,
        item_size_mode: ItemSizeMode::Fixed(40.0),
        container_size: Some(200.0),
        ..VirtualizerOptions::default()
    })
    .expect("valid");

    virt.refresh_programmatic_scroll_target();
    assert!(virt.scroll_state().is_none());
}

/// Tests get_virtual_item_for_offset returns None when the engine has no measurements.
///
/// # Test Steps
/// - Uses Virtualizer::empty which has an empty measurement cache.
/// - Queries offset zero and expects None.
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn should_return_none_for_virtual_item_lookup_when_empty() {
    let virt = Virtualizer::empty();
    assert!(virt.get_virtual_item_for_offset(0.0).is_none());
}

/// Tests MeasurementCache::record reports no change when size matches existing entry.
///
/// # Test Steps
/// - Records a size for a key twice with identical values.
/// - Asserts the second call returns Ok(false).
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn should_report_false_when_rerecording_identical_measurement() {
    let mut cache = MeasurementCache::new(10.0);
    let key = VirtualKey::Index(0);
    cache.record(key.clone(), 25.0).expect("first record");
    let again = cache.record(key, 25.0).expect("second record");
    assert!(!again, "Identical size should not count as a change");
}

/// Tests MeasurementCache::default matches a new cache with fifty-pixel estimate.
///
/// # Test Steps
/// - Constructs Default and explicit new(50.0) caches.
/// - Asserts average and count match.
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn should_match_default_measurement_cache_to_fifty_pixel_estimate() {
    let a = MeasurementCache::default();
    let b = MeasurementCache::new(50.0);
    assert_eq!(a.average(), b.average());
    assert_eq!(a.count(), b.count());
}

/// Tests RangeCalculator supports multi-lane measurement layouts.
///
/// # Test Steps
/// - Builds twelve items alternating lanes zero and one.
/// - Invokes calculate_range with two lanes and asserts a valid inclusive range.
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn should_calculate_visible_range_for_multi_lane_measurements() {
    let measurements: Vec<VirtualItem> = (0..12usize)
        .map(|i| {
            let row = (i / 2) as f64;
            let start = row * 40.0;
            let size = 36.0;
            VirtualItem {
                index: i,
                size,
                start,
                end: start + size,
                key: VirtualKey::Index(i),
                lane: i % 2,
            }
        })
        .collect();

    let range = RangeCalculator::calculate_range(&measurements, 200.0, 0.0, 2);
    assert!(range.is_some());
    let (start, end) = range.expect("range");
    assert!(start <= end);
    assert!(end < measurements.len());
}
