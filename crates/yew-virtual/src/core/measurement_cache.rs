use std::collections::HashMap;

use crate::core::virtual_key::VirtualKey;
use crate::core::virtualizer_error::VirtualizerError;

/// Cache for storing measured item sizes keyed by VirtualKey.
///
/// Maintains a mapping from item keys to their actual measured
/// dimensions. Using keys instead of indices means measured sizes
/// follow items across reorders when a custom key extractor is used.
#[derive(Debug, Clone)]
pub struct MeasurementCache {
    /// Map of item key to measured size in pixels.
    measurements: HashMap<VirtualKey, f64>,

    /// Running average of all measured sizes for estimation.
    average_size: f64,

    /// Total number of measurements recorded.
    measurement_count: usize,
}

impl MeasurementCache {
    /// Creates a new empty measurement cache.
    ///
    /// # Parameters
    ///
    /// - `initial_estimate`: The default size estimate used before measurements arrive.
    pub fn new(initial_estimate: f64) -> Self {
        // Initialize with the provided estimate and empty storage.
        Self {
            measurements: HashMap::new(),
            average_size: initial_estimate,
            measurement_count: 0,
        }
    }

    /// Records a measured size for the given item key.
    ///
    /// Updates the running average and stores the measurement. If a
    /// measurement already exists for this key, it is replaced.
    ///
    /// # Parameters
    ///
    /// - `key`: The item key to record a measurement for.
    /// - `size`: The measured size in pixels.
    ///
    /// # Returns
    ///
    /// - `Ok(bool)`: True if the measurement changed the stored value.
    /// - `Err(VirtualizerError)`: If the size is invalid.
    ///
    /// # Errors
    ///
    /// - Returns `MeasurementError` if size is negative, NaN, or infinite.
    pub fn record(&mut self, key: VirtualKey, size: f64) -> Result<bool, VirtualizerError> {
        // Validate the measured size.
        if size.is_nan() || size.is_infinite() || size < 0.0 {
            return Err(VirtualizerError::MeasurementError(format!(
                "Invalid measurement size {} for key {}",
                size, key
            )));
        }

        // Check if the measurement is different from the existing one.
        let changed = self
            .measurements
            .get(&key)
            .is_none_or(|existing| (*existing - size).abs() > f64::EPSILON);

        // Store the measurement.
        self.measurements.insert(key, size);

        // Recalculate the running average if the value changed.
        if changed {
            self.measurement_count = self.measurements.len();
            let total: f64 = self.measurements.values().sum();
            if self.measurement_count > 0 {
                self.average_size = total / self.measurement_count as f64;
            }
        }

        Ok(changed)
    }

    /// Retrieves the measured size for a given key.
    ///
    /// # Parameters
    ///
    /// - `key`: The item key to look up.
    ///
    /// # Returns
    ///
    /// - `Option<f64>`: The measured size if available.
    pub fn get(&self, key: &VirtualKey) -> Option<f64> {
        // Look up the key in the measurement map.
        self.measurements.get(key).copied()
    }

    /// Returns the current average of all measured sizes.
    ///
    /// # Returns
    ///
    /// - `f64`: The running average size.
    pub fn average(&self) -> f64 {
        // Return the computed running average.
        self.average_size
    }

    /// Returns the total number of stored measurements.
    ///
    /// # Returns
    ///
    /// - `usize`: The count of recorded measurements.
    pub fn count(&self) -> usize {
        // Return the current measurement count.
        self.measurement_count
    }

    /// Clears all stored measurements and resets the average.
    ///
    /// # Parameters
    ///
    /// - `initial_estimate`: The new default size estimate after clearing.
    pub fn clear(&mut self, initial_estimate: f64) {
        // Remove all stored measurements.
        self.measurements.clear();

        // Reset the average to the provided estimate.
        self.average_size = initial_estimate;

        // Reset the count.
        self.measurement_count = 0;
    }

    /// Removes the measurement for a specific key.
    ///
    /// # Parameters
    ///
    /// - `key`: The item key whose measurement should be removed.
    ///
    /// # Returns
    ///
    /// - `Option<f64>`: The removed measurement if it existed.
    pub fn remove(&mut self, key: &VirtualKey) -> Option<f64> {
        // Remove the measurement entry.
        let removed = self.measurements.remove(key);

        // Recalculate the average if a value was removed.
        if removed.is_some() {
            self.measurement_count = self.measurements.len();
            if self.measurement_count > 0 {
                let total: f64 = self.measurements.values().sum();
                self.average_size = total / self.measurement_count as f64;
            }
        }

        removed
    }
}

impl Default for MeasurementCache {
    /// Returns a default measurement cache with a 50px estimate.
    fn default() -> Self {
        // Use a sensible default estimate.
        Self::new(50.0)
    }
}
