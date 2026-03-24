//! Unit tests for `MeasurementCache` module.
//!
//! This module verifies the correctness of the measurement cache
//! used for storing and retrieving dynamic item sizes.
//! It ensures measurements are stored, averages are computed, and
//! invalid values are rejected.
//!
//! Test Scenarios:
//! - New cache starts empty with the initial estimate.
//! - Recording a measurement stores it correctly.
//! - Running average is computed after multiple measurements.
//! - Invalid measurements (NaN, negative, infinite) are rejected.
//! - Clearing the cache resets all state.
//! - Removing a specific measurement updates the average.

use yew_virtual::core::measurement_cache::MeasurementCache;
use yew_virtual::core::virtual_key::VirtualKey;

/// Tests that a new cache starts with zero measurements and the initial estimate.
///
/// # Test Steps
/// - Creates a new MeasurementCache with a known estimate.
/// - Asserts count is 0.
/// - Asserts average equals the initial estimate.
#[test]
fn should_start_empty_with_initial_estimate() {
    // Create a cache with 60px estimate.
    let cache = MeasurementCache::new(60.0);

    // Assert count is zero.
    assert_eq!(cache.count(), 0, "New cache should have zero measurements");

    // Assert average is the initial estimate.
    assert!(
        (cache.average() - 60.0).abs() < f64::EPSILON,
        "Average should be the initial estimate"
    );
}

/// Tests that recording a measurement stores it and updates the average.
///
/// # Test Steps
/// - Creates a cache and records a measurement.
/// - Asserts the measurement is retrievable.
/// - Asserts the average is updated.
#[test]
fn should_store_and_retrieve_measurement() {
    // Create a cache.
    let mut cache = MeasurementCache::new(50.0);

    // Record a measurement for index 0.
    let result = cache.record(VirtualKey::Index(0), 80.0);
    assert!(result.is_ok(), "Recording should succeed");
    assert!(
        result.unwrap_or(false),
        "First recording should be a change"
    );

    // Assert the measurement is retrievable.
    assert_eq!(
        cache.get(&VirtualKey::Index(0)),
        Some(80.0),
        "Should retrieve recorded value"
    );

    // Assert count is 1.
    assert_eq!(cache.count(), 1, "Count should be 1 after one recording");

    // Assert average updated.
    assert!(
        (cache.average() - 80.0).abs() < f64::EPSILON,
        "Average should be 80.0 with one measurement"
    );
}

/// Tests that recording the same value returns false for no change.
///
/// # Test Steps
/// - Records a measurement, then records the same value again.
/// - Asserts the second recording returns false.
#[test]
fn should_report_no_change_for_duplicate_measurement() {
    // Create a cache and record initial measurement.
    let mut cache = MeasurementCache::new(50.0);
    let _ = cache.record(VirtualKey::Index(0), 80.0);

    // Record the same value again.
    let result = cache.record(VirtualKey::Index(0), 80.0);
    assert!(result.is_ok(), "Recording should succeed");
    assert!(
        !result.unwrap_or(true),
        "Duplicate recording should not be a change"
    );
}

/// Tests that the running average is computed correctly with multiple measurements.
///
/// # Test Steps
/// - Records measurements for multiple indices.
/// - Asserts the average is the mean of all recorded values.
#[test]
fn should_compute_correct_running_average() {
    // Create a cache.
    let mut cache = MeasurementCache::new(50.0);

    // Record three measurements: 60, 80, 100.
    let _ = cache.record(VirtualKey::Index(0), 60.0);
    let _ = cache.record(VirtualKey::Index(1), 80.0);
    let _ = cache.record(VirtualKey::Index(2), 100.0);

    // Assert count is 3.
    assert_eq!(cache.count(), 3, "Count should be 3");

    // Assert average is (60 + 80 + 100) / 3 = 80.
    assert!(
        (cache.average() - 80.0).abs() < f64::EPSILON,
        "Average should be 80.0"
    );
}

/// Tests that NaN measurements are rejected.
///
/// # Test Steps
/// - Attempts to record a NaN measurement.
/// - Asserts that it returns an error.
#[test]
fn should_reject_nan_measurement() {
    // Create a cache.
    let mut cache = MeasurementCache::new(50.0);

    // Attempt to record NaN.
    let result = cache.record(VirtualKey::Index(0), f64::NAN);

    // Assert it failed.
    assert!(result.is_err(), "NaN measurement should be rejected");
}

/// Tests that negative measurements are rejected.
///
/// # Test Steps
/// - Attempts to record a negative measurement.
/// - Asserts that it returns an error.
#[test]
fn should_reject_negative_measurement() {
    // Create a cache.
    let mut cache = MeasurementCache::new(50.0);

    // Attempt to record a negative value.
    let result = cache.record(VirtualKey::Index(0), -10.0);

    // Assert it failed.
    assert!(result.is_err(), "Negative measurement should be rejected");
}

/// Tests that infinite measurements are rejected.
///
/// # Test Steps
/// - Attempts to record an infinite measurement.
/// - Asserts that it returns an error.
#[test]
fn should_reject_infinite_measurement() {
    // Create a cache.
    let mut cache = MeasurementCache::new(50.0);

    // Attempt to record infinity.
    let result = cache.record(VirtualKey::Index(0), f64::INFINITY);

    // Assert it failed.
    assert!(result.is_err(), "Infinite measurement should be rejected");
}

/// Tests that clearing the cache resets all state.
///
/// # Test Steps
/// - Records measurements, then clears the cache.
/// - Asserts count is zero and average is the new estimate.
#[test]
fn should_clear_all_measurements() {
    // Create a cache and record some measurements.
    let mut cache = MeasurementCache::new(50.0);
    let _ = cache.record(VirtualKey::Index(0), 80.0);
    let _ = cache.record(VirtualKey::Index(1), 90.0);

    // Clear with a new estimate.
    cache.clear(70.0);

    // Assert state is reset.
    assert_eq!(cache.count(), 0, "Count should be zero after clear");
    assert!(
        (cache.average() - 70.0).abs() < f64::EPSILON,
        "Average should be the new estimate after clear"
    );
    assert_eq!(
        cache.get(&VirtualKey::Index(0)),
        None,
        "Measurements should be gone after clear"
    );
}

/// Tests that removing a measurement updates the count and average.
///
/// # Test Steps
/// - Records two measurements, removes one.
/// - Asserts count and average are updated.
#[test]
fn should_update_average_after_removing_measurement() {
    // Create a cache and record measurements.
    let mut cache = MeasurementCache::new(50.0);
    let _ = cache.record(VirtualKey::Index(0), 60.0);
    let _ = cache.record(VirtualKey::Index(1), 100.0);

    // Remove index 0.
    let removed = cache.remove(&VirtualKey::Index(0));
    assert_eq!(removed, Some(60.0), "Should return the removed value");

    // Assert count is 1.
    assert_eq!(cache.count(), 1, "Count should be 1 after removal");

    // Assert average is now 100.
    assert!(
        (cache.average() - 100.0).abs() < f64::EPSILON,
        "Average should be 100.0 after removing 60.0"
    );
}

/// Tests that removing a non-existent index returns None.
///
/// # Test Steps
/// - Attempts to remove an index that was never recorded.
/// - Asserts it returns None.
#[test]
fn should_return_none_when_removing_nonexistent_index() {
    // Create a cache.
    let mut cache = MeasurementCache::new(50.0);

    // Attempt to remove a non-existent index.
    let removed = cache.remove(&VirtualKey::Index(99));

    // Assert it returned None.
    assert_eq!(
        removed, None,
        "Removing non-existent index should return None"
    );
}

/// Tests that getting a non-existent index returns None.
///
/// # Test Steps
/// - Queries a cache for an index that was never recorded.
/// - Asserts it returns None.
#[test]
fn should_return_none_for_unrecorded_index() {
    // Create a cache.
    let cache = MeasurementCache::new(50.0);

    // Query a non-existent index.
    let result = cache.get(&VirtualKey::Index(42));

    // Assert it returned None.
    assert_eq!(result, None, "Unrecorded index should return None");
}
