use std::cell::RefCell;
use std::collections::HashMap;

use crate::core::measure_item_outcome::MeasureItemOutcome;
use crate::core::measurement_cache::MeasurementCache;
use crate::core::range_calculator::RangeCalculator;
use crate::core::scroll_alignment::ScrollAlignment;
use crate::core::scroll_behavior::ScrollBehavior;
use crate::core::scroll_reconcile_action::ScrollReconcileAction;
use crate::core::scroll_state::ScrollState;
use crate::core::scroll_to_options::ScrollToOptions;
use crate::core::virtual_item::VirtualItem;
use crate::core::virtual_key::VirtualKey;
use crate::core::virtualizer_error::VirtualizerError;
use crate::core::virtualizer_options::VirtualizerOptions;
use crate::core::visible_range::VisibleRange;

/// The core virtualization engine.
///
/// Maintains all state needed for virtualization: measurements cache,
/// visible ranges, scroll position, lane assignments, and scroll state.
/// It produces `VirtualItem` metadata that Yew components use to render
/// only the items within (and near) the visible viewport.
///
/// This engine is headless -- it does not interact with the DOM directly.
/// Instead, it accepts inputs (scroll offset, container size, measurements)
/// and produces outputs (virtual items, total size, scroll targets).
#[derive(Debug, Clone)]
pub struct Virtualizer {
    /// The current configuration options.
    options: VirtualizerOptions,

    /// Precomputed measurements for all items.
    measurements_cache: Vec<VirtualItem>,

    /// Cache for dynamically measured item sizes keyed by VirtualKey.
    item_size_cache: MeasurementCache,

    /// Lane assignments cache mapping item index to lane.
    lane_assignments: HashMap<usize, usize>,

    /// Indices of items with pending measurement changes.
    pending_measured_cache_indexes: Vec<usize>,

    /// Previous lanes value for detecting lane count changes.
    prev_lanes: Option<usize>,

    /// Current scroll offset in pixels.
    scroll_offset: f64,

    /// Current container viewport size in pixels.
    container_size: f64,

    /// Start index of the current visible range (inclusive).
    range_start: usize,

    /// End index of the current visible range (inclusive).
    range_end: usize,

    /// Total scrollable size in pixels.
    total_size: f64,

    /// Whether the user is currently scrolling.
    is_scrolling: bool,

    /// The direction of the last scroll event.
    ///
    /// True means forward (scrolling down/right), false means backward.
    /// None means no scroll has occurred.
    scroll_forward: Option<bool>,

    /// Accumulated scroll adjustments for maintaining position during
    /// item resizes above the viewport.
    scroll_adjustments: f64,

    /// Active programmatic scroll state, if any.
    scroll_state: Option<ScrollState>,

    /// Bumps when measurements or layout affecting item positions change.
    measurement_version: u64,

    /// Memoized virtual item list for repeated reads within the same layout epoch.
    virtual_items_cache: RefCell<Option<(u64, Vec<VirtualItem>)>>,
}

impl Virtualizer {
    /// Creates an empty virtualizer with zero items and no scrollable content.
    ///
    /// This constructor is infallible and always returns a valid instance.
    /// It is intended as a safe fallback when normal construction fails,
    /// ensuring no `unwrap()`, `expect()`, or `panic!()` is needed.
    ///
    /// # Returns
    ///
    /// - `Virtualizer`: An empty virtualizer that produces no virtual items.
    pub fn empty() -> Self {
        // Construct the virtualizer directly without validation.
        Self {
            options: VirtualizerOptions::default(),
            measurements_cache: Vec::new(),
            item_size_cache: MeasurementCache::new(1.0),
            lane_assignments: HashMap::new(),
            pending_measured_cache_indexes: Vec::new(),
            prev_lanes: None,
            scroll_offset: 0.0,
            container_size: 0.0,
            range_start: 0,
            range_end: 0,
            total_size: 0.0,
            is_scrolling: false,
            scroll_forward: None,
            scroll_adjustments: 0.0,
            scroll_state: None,
            measurement_version: 0,
            virtual_items_cache: RefCell::new(None),
        }
    }

    /// Creates a new virtualizer with the given options.
    ///
    /// Validates configuration, initializes internal arrays, and
    /// computes the initial layout.
    ///
    /// # Parameters
    ///
    /// - `options`: Configuration for the virtualizer.
    ///
    /// # Returns
    ///
    /// - `Ok(Virtualizer)`: A fully initialized virtualizer.
    /// - `Err(VirtualizerError)`: If configuration is invalid.
    ///
    /// # Errors
    ///
    /// - Returns `InvalidItemSize` if the base size is zero or negative.
    /// - Returns `InvalidConfiguration` if padding, gap, or lanes is invalid.
    pub fn new(options: VirtualizerOptions) -> Result<Self, VirtualizerError> {
        // Validate item size.
        let base_size = options.item_size_mode.base_size();
        if base_size <= 0.0 || base_size.is_nan() || base_size.is_infinite() {
            return Err(VirtualizerError::InvalidItemSize(format!(
                "Base size must be positive and finite, got {}",
                base_size
            )));
        }

        // Validate padding values.
        if options.padding_start < 0.0 || options.padding_end < 0.0 {
            return Err(VirtualizerError::InvalidConfiguration(
                "Padding values must be non-negative".to_string(),
            ));
        }

        // Validate gap value.
        if options.gap < 0.0 {
            return Err(VirtualizerError::InvalidConfiguration(
                "Gap must be non-negative".to_string(),
            ));
        }

        // Validate lanes.
        if options.lanes == 0 {
            return Err(VirtualizerError::InvalidConfiguration(
                "Lanes must be at least 1".to_string(),
            ));
        }

        // Initialize the measurement cache with the base size estimate.
        let item_size_cache = MeasurementCache::new(base_size);

        // Resolve the container size from options or initial rect.
        let container_size = options.container_size.unwrap_or_else(|| {
            if options.scroll_direction
                == crate::core::scroll_direction::ScrollDirection::Horizontal
            {
                options.initial_rect.width
            } else {
                options.initial_rect.height
            }
        });

        // Resolve the initial scroll offset from callback or scalar option.
        let scroll_offset = if let Some(ref f) = options.initial_offset_fn {
            f()
        } else {
            options.initial_offset
        };

        // Build the virtualizer.
        let mut virt = Self {
            options,
            measurements_cache: Vec::new(),
            item_size_cache,
            lane_assignments: HashMap::new(),
            pending_measured_cache_indexes: Vec::new(),
            prev_lanes: None,
            scroll_offset,
            container_size,
            range_start: 0,
            range_end: 0,
            total_size: 0.0,
            is_scrolling: false,
            scroll_forward: None,
            scroll_adjustments: 0.0,
            scroll_state: None,
            measurement_version: 0,
            virtual_items_cache: RefCell::new(None),
        };

        // Compute the initial measurements.
        virt.rebuild_measurements();

        // Compute the initial range.
        virt.recalculate_range();

        Ok(virt)
    }

    /// Updates all options at runtime without recreating the virtualizer.
    ///
    /// Validates the new configuration and recalculates layout. Measurement
    /// state is preserved across option changes.
    ///
    /// # Parameters
    ///
    /// - `options`: The new configuration options.
    ///
    /// # Returns
    ///
    /// - `Ok(())`: If options were applied successfully.
    /// - `Err(VirtualizerError)`: If the new configuration is invalid.
    ///
    /// # Errors
    ///
    /// - Returns `InvalidItemSize` if the base size is zero or negative.
    /// - Returns `InvalidConfiguration` if padding, gap, or lanes is invalid.
    pub fn set_options(&mut self, options: VirtualizerOptions) -> Result<(), VirtualizerError> {
        // Validate item size.
        let base_size = options.item_size_mode.base_size();
        if base_size <= 0.0 || base_size.is_nan() || base_size.is_infinite() {
            return Err(VirtualizerError::InvalidItemSize(format!(
                "Base size must be positive and finite, got {}",
                base_size
            )));
        }

        // Validate padding values.
        if options.padding_start < 0.0 || options.padding_end < 0.0 {
            return Err(VirtualizerError::InvalidConfiguration(
                "Padding values must be non-negative".to_string(),
            ));
        }

        // Validate gap value.
        if options.gap < 0.0 {
            return Err(VirtualizerError::InvalidConfiguration(
                "Gap must be non-negative".to_string(),
            ));
        }

        // Validate lanes.
        if options.lanes == 0 {
            return Err(VirtualizerError::InvalidConfiguration(
                "Lanes must be at least 1".to_string(),
            ));
        }

        // Detect lane count changes.
        if let Some(prev) = self.prev_lanes {
            if prev != options.lanes {
                // Clear lane assignments and size cache on lane change.
                self.lane_assignments.clear();
                self.item_size_cache.clear(base_size);
                self.measurements_cache.clear();
                self.pending_measured_cache_indexes.clear();
            }
        }

        // Store the previous lanes for future detection.
        self.prev_lanes = Some(options.lanes);

        // Apply the new options.
        self.options = options;

        // Rebuild measurements and range.
        self.rebuild_measurements();
        self.recalculate_range();

        self.notify_change();

        Ok(())
    }

    /// Resolves the estimated size for a given item index.
    ///
    /// # Parameters
    ///
    /// - `index`: The item index.
    ///
    /// # Returns
    ///
    /// - `f64`: The estimated size for this item.
    fn estimate_size_for_index(&self, index: usize) -> f64 {
        // Use the per-index callback if provided.
        if let Some(ref estimate_fn) = self.options.estimate_size {
            return estimate_fn(index);
        }

        // Fall back to the base size from item_size_mode.
        self.options.item_size_mode.base_size()
    }

    /// Resolves the key for a given item index.
    ///
    /// # Parameters
    ///
    /// - `index`: The item index.
    ///
    /// # Returns
    ///
    /// - `VirtualKey`: The key for this item.
    fn get_key_for_index(&self, index: usize) -> VirtualKey {
        // Use the custom key extractor if provided.
        if let Some(ref key_fn) = self.options.get_item_key {
            return key_fn(index);
        }

        // Default to using the index as the key.
        VirtualKey::Index(index)
    }

    /// Rebuilds the full measurements cache from scratch or incrementally.
    fn rebuild_measurements(&mut self) {
        // If disabled, clear everything.
        if !self.options.enabled {
            self.measurements_cache.clear();
            self.item_size_cache
                .clear(self.options.item_size_mode.base_size());
            self.lane_assignments.clear();
            self.total_size = 0.0;
            self.measurement_version = self.measurement_version.wrapping_add(1);
            self.virtual_items_cache.borrow_mut().take();
            return;
        }

        let count = self.options.item_count;
        let lanes = self.options.lanes;
        let padding_start = self.options.padding_start;
        let scroll_margin = self.options.scroll_margin;
        let gap = self.options.gap;

        // Clean up stale lane cache entries when count decreases.
        if self.lane_assignments.len() > count {
            self.lane_assignments.retain(|&index, _| index < count);
        }

        // Determine the minimum index to rebuild from.
        let min = if self.pending_measured_cache_indexes.is_empty() {
            0
        } else {
            self.pending_measured_cache_indexes
                .iter()
                .copied()
                .min()
                .unwrap_or(0)
        };
        self.pending_measured_cache_indexes.clear();

        // Populate from initial_measurements_cache if we have no cache yet.
        if self.measurements_cache.is_empty() && !self.options.initial_measurements_cache.is_empty()
        {
            self.measurements_cache = self.options.initial_measurements_cache.clone();
            for item in &self.measurements_cache {
                let _ = self.item_size_cache.record(item.key.clone(), item.size);
            }
        }

        // Truncate if min is less than current cache length.
        if min < self.measurements_cache.len() {
            self.measurements_cache.truncate(min);
        }

        // Track last item index per lane for O(1) lookup.
        let mut lane_last_index: Vec<Option<usize>> = vec![None; lanes];

        // Initialize from existing measurements (before min).
        for m in 0..self.measurements_cache.len().min(min) {
            if let Some(item) = self.measurements_cache.get(m) {
                if item.lane < lanes {
                    lane_last_index[item.lane] = Some(m);
                }
            }
        }

        // Build measurements for indices from min to count.
        for i in min..count {
            let key = self.get_key_for_index(i);

            // Check for cached lane assignment.
            let cached_lane = self.lane_assignments.get(&i).copied();

            let lane: usize;
            let start: f64;

            if let Some(cl) = cached_lane {
                if lanes > 1 {
                    // Use cached lane with O(1) lookup for previous item in same lane.
                    lane = cl;
                    let prev_index = lane_last_index.get(lane).copied().flatten();
                    let prev_end = prev_index
                        .and_then(|pi| self.measurements_cache.get(pi))
                        .map(|item| item.end);
                    start = prev_end
                        .map(|e| e + gap)
                        .unwrap_or(padding_start + scroll_margin);
                } else {
                    // Single lane: use previous item.
                    lane = 0;
                    let prev_item = if i > 0 {
                        self.measurements_cache.get(i - 1)
                    } else {
                        None
                    };
                    start = prev_item
                        .map(|item| item.end + gap)
                        .unwrap_or(padding_start + scroll_margin);
                }
            } else {
                // No cache: find the shortest lane.
                if lanes == 1 {
                    lane = 0;
                    let prev_item = if i > 0 {
                        self.measurements_cache.get(i - 1)
                    } else {
                        None
                    };
                    start = prev_item
                        .map(|item| item.end + gap)
                        .unwrap_or(padding_start + scroll_margin);
                } else {
                    // Find the lane with the smallest end value.
                    let furthest = self.get_furthest_measurement(i);
                    start = furthest
                        .as_ref()
                        .map(|item| item.end + gap)
                        .unwrap_or(padding_start + scroll_margin);
                    lane = furthest.as_ref().map(|item| item.lane).unwrap_or(i % lanes);

                    // Cache the lane assignment for multi-lane.
                    if lanes > 1 {
                        self.lane_assignments.insert(i, lane);
                    }
                }
            }

            // Resolve the size from cache or estimate.
            let measured_size = self.item_size_cache.get(&key);
            let size = measured_size.unwrap_or_else(|| self.estimate_size_for_index(i));

            // Build the virtual item.
            let item = VirtualItem::with_key_and_lane(i, size, start, key, lane);

            // Store or update in measurements cache.
            if i < self.measurements_cache.len() {
                self.measurements_cache[i] = item;
            } else {
                self.measurements_cache.push(item);
            }

            // Update lane's last item index.
            if lane < lanes {
                lane_last_index[lane] = Some(i);
            }
        }

        // Truncate any excess items beyond count.
        self.measurements_cache.truncate(count);

        // Recompute total size.
        self.recompute_total_size();
    }

    /// Finds the measurement in the shortest lane for multi-lane assignment.
    ///
    /// # Parameters
    ///
    /// - `index`: The index to find the furthest measurement before.
    ///
    /// # Returns
    ///
    /// - `Option<VirtualItem>`: The measurement with the smallest end value.
    fn get_furthest_measurement(&self, index: usize) -> Option<VirtualItem> {
        let lanes = self.options.lanes;
        let mut furthest_found: HashMap<usize, bool> = HashMap::new();
        let mut furthest_measurements: HashMap<usize, VirtualItem> = HashMap::new();

        // Walk backwards through measurements to find the shortest lane.
        let mut m = index;
        while m > 0 {
            m -= 1;
            if let Some(measurement) = self.measurements_cache.get(m) {
                if furthest_found.contains_key(&measurement.lane) {
                    continue;
                }

                let prev = furthest_measurements.get(&measurement.lane);
                if prev.is_none() || measurement.end > prev.map_or(0.0, |p| p.end) {
                    furthest_measurements.insert(measurement.lane, measurement.clone());
                } else if measurement.end < prev.map_or(0.0, |p| p.end) {
                    furthest_found.insert(measurement.lane, true);
                }

                if furthest_found.len() == lanes {
                    break;
                }
            }
        }

        // Return the lane with the smallest end value if all lanes are covered.
        if furthest_measurements.len() == lanes {
            furthest_measurements
                .values()
                .min_by(|a, b| {
                    a.end
                        .partial_cmp(&b.end)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then(a.index.cmp(&b.index))
                })
                .cloned()
        } else {
            None
        }
    }

    /// Recomputes the total scrollable size from measurements.
    fn recompute_total_size(&mut self) {
        let measurements = &self.measurements_cache;
        let lanes = self.options.lanes;

        // Find the maximum end value.
        let end = if measurements.is_empty() {
            self.options.padding_start
        } else if lanes == 1 {
            measurements.last().map_or(0.0, |item| item.end)
        } else {
            // Find the maximum end value across all lanes.
            let mut end_by_lane: Vec<Option<f64>> = vec![None; lanes];
            let mut idx = measurements.len();
            while idx > 0 && end_by_lane.iter().any(|v| v.is_none()) {
                idx -= 1;
                if let Some(item) = measurements.get(idx) {
                    if item.lane < lanes && end_by_lane[item.lane].is_none() {
                        end_by_lane[item.lane] = Some(item.end);
                    }
                }
            }
            end_by_lane.iter().filter_map(|v| *v).fold(0.0f64, f64::max)
        };

        // Total = end - scroll_margin + padding_end.
        self.total_size = (end - self.options.scroll_margin + self.options.padding_end).max(0.0);

        // Invalidate memoized virtual items after measurement rebuild.
        self.measurement_version = self.measurement_version.wrapping_add(1);
        self.virtual_items_cache.borrow_mut().take();
    }

    /// Computes a fingerprint for the virtual items memoization cache.
    fn virtual_items_cache_key(&self) -> u64 {
        // Mix range, count, layout epoch, and enabled flag into one key.
        let mut h: u64 = self.range_start as u64;
        h ^= (self.range_end as u64).wrapping_shl(12);
        h ^= (self.options.item_count as u64).wrapping_shl(24);
        h ^= self.measurement_version.wrapping_shl(32);
        if !self.options.enabled {
            h ^= 1u64 << 63;
        }
        h
    }

    /// Notifies the optional `on_change` callback.
    fn notify_change(&self) {
        // Invoke the consumer hook when virtualizer-visible state changes.
        if let Some(ref cb) = self.options.on_change {
            cb();
        }
    }

    /// Updates the scroll offset and recalculates the visible range.
    ///
    /// This is the primary method called during scroll events.
    ///
    /// # Parameters
    ///
    /// - `scroll_offset`: The new scroll position in pixels.
    /// - `is_scrolling`: Whether the user is actively scrolling.
    pub fn update_scroll_offset(&mut self, scroll_offset: f64, is_scrolling: bool) {
        // Track scroll direction.
        if is_scrolling {
            self.scroll_forward = Some(scroll_offset > self.scroll_offset);
        } else {
            self.scroll_forward = None;
        }

        // Update scrolling state.
        self.is_scrolling = is_scrolling;

        // Reset scroll adjustments on new offset.
        self.scroll_adjustments = 0.0;

        // Store the new scroll offset.
        self.scroll_offset = scroll_offset;

        // Recalculate the visible range.
        self.recalculate_range();

        // Drop memoized items because the visible range may have changed.
        self.virtual_items_cache.borrow_mut().take();
        self.notify_change();
    }

    /// Updates the container viewport size and recalculates the visible range.
    ///
    /// Called when the scroll container is resized.
    ///
    /// # Parameters
    ///
    /// - `container_size`: The new viewport size in pixels.
    pub fn update_container_size(&mut self, container_size: f64) {
        // Store the new container size, ensuring non-negative.
        self.container_size = container_size.max(0.0);

        // Recalculate the visible range with the new size.
        self.recalculate_range();

        self.virtual_items_cache.borrow_mut().take();
        self.notify_change();
    }

    /// Records a measurement for a specific item and recalculates layout if changed.
    ///
    /// # Parameters
    ///
    /// - `index`: The index of the item that was measured.
    /// - `size`: The measured size in pixels.
    ///
    /// # Returns
    ///
    /// - `Ok(MeasureItemOutcome)`: Layout change flag and DOM scroll compensation.
    /// - `Err(VirtualizerError)`: If the index is out of bounds or size is invalid.
    ///
    /// # Errors
    ///
    /// - Returns `IndexOutOfBounds` if index >= item_count.
    /// - Returns `MeasurementError` if size is invalid.
    pub fn measure_item(
        &mut self,
        index: usize,
        size: f64,
    ) -> Result<MeasureItemOutcome, VirtualizerError> {
        // Validate the index.
        if index >= self.options.item_count {
            return Err(VirtualizerError::IndexOutOfBounds {
                requested: index,
                total: self.options.item_count,
            });
        }

        // Get the key for this item.
        let key = self.get_key_for_index(index);

        // Check the existing cached size.
        let existing_size = self
            .item_size_cache
            .get(&key)
            .or_else(|| self.measurements_cache.get(index).map(|item| item.size));

        let prior = existing_size.unwrap_or(size);
        let delta = size - prior;

        // Record the measurement in the cache.
        let changed = self.item_size_cache.record(key, size)?;

        let mut scroll_compensation = 0.0;

        // Recalculate layout if the measurement changed.
        if changed {
            let smooth_active = self
                .scroll_state
                .as_ref()
                .map(|s| s.behavior == ScrollBehavior::Smooth)
                .unwrap_or(false);

            let allow_resize_adjust =
                !smooth_active || self.options.adjust_scroll_on_resize_during_smooth_scroll;

            let mut should_adjust = true;
            if let Some(ref cb) = self
                .options
                .should_adjust_scroll_position_on_item_size_change
            {
                should_adjust = cb(index, prior, size);
            }

            // Adjust scroll position if the item is above the viewport.
            if allow_resize_adjust && should_adjust {
                if let Some(item) = self.measurements_cache.get(index) {
                    if item.start < self.scroll_offset + self.scroll_adjustments {
                        self.scroll_adjustments += delta;
                        scroll_compensation = delta;
                    }
                }
            }

            // Mark this index as pending for incremental rebuild.
            self.pending_measured_cache_indexes.push(index);

            // Rebuild measurements and range.
            self.rebuild_measurements();
            self.recalculate_range();
            self.notify_change();
        }

        Ok(MeasureItemOutcome {
            layout_changed: changed,
            scroll_compensation,
        })
    }

    /// Updates the total item count and recalculates layout.
    ///
    /// Handles dataset size changes (insertions, removals) without
    /// full reinitialization.
    ///
    /// # Parameters
    ///
    /// - `new_count`: The new total number of items.
    pub fn update_item_count(&mut self, new_count: usize) {
        // Update the options.
        self.options.item_count = new_count;

        // Rebuild measurements and range.
        self.rebuild_measurements();
        self.recalculate_range();

        self.notify_change();
    }

    /// Calculates the scroll offset needed to bring a specific item into view.
    ///
    /// Accounts for scroll padding and alignment. Does not actually scroll.
    ///
    /// # Parameters
    ///
    /// - `index`: The index of the target item.
    /// - `align`: How to align the item within the viewport.
    ///
    /// # Returns
    ///
    /// - `Option<(f64, ScrollAlignment)>`: The target scroll offset and resolved alignment,
    ///   or None if the index has no measurement.
    pub fn get_offset_for_index(
        &self,
        index: usize,
        align: ScrollAlignment,
    ) -> Option<(f64, ScrollAlignment)> {
        // Clamp the index to valid range.
        let clamped_index = if self.options.item_count == 0 {
            return None;
        } else {
            index.min(self.options.item_count - 1)
        };

        // Look up the item measurement.
        let item = self.measurements_cache.get(clamped_index)?;

        let size = self.container_size;
        let scroll_offset = self.scroll_offset;

        // Resolve auto alignment.
        let mut resolved_align = align;
        if resolved_align == ScrollAlignment::Auto {
            if item.end >= scroll_offset + size - self.options.scroll_padding_end {
                resolved_align = ScrollAlignment::End;
            } else if item.start <= scroll_offset + self.options.scroll_padding_start {
                resolved_align = ScrollAlignment::Start;
            } else {
                // Item is already visible.
                return Some((scroll_offset, resolved_align));
            }
        }

        // Compute the target offset.
        let to_offset = if resolved_align == ScrollAlignment::End {
            item.end + self.options.scroll_padding_end
        } else {
            item.start - self.options.scroll_padding_start
        };

        // Apply alignment.
        let aligned_offset = self.get_offset_for_alignment(to_offset, resolved_align, item.size);

        Some((aligned_offset, resolved_align))
    }

    /// Calculates the aligned scroll offset for a given target.
    ///
    /// # Parameters
    ///
    /// - `to_offset`: The raw target offset.
    /// - `align`: The alignment strategy.
    /// - `item_size`: The size of the target item (used for center alignment).
    ///
    /// # Returns
    ///
    /// - `f64`: The clamped and aligned scroll offset.
    pub fn get_offset_for_alignment(
        &self,
        to_offset: f64,
        align: ScrollAlignment,
        item_size: f64,
    ) -> f64 {
        let size = self.container_size;

        // Adjust offset based on alignment.
        let adjusted = match align {
            ScrollAlignment::Center => to_offset + (item_size - size) / 2.0,
            ScrollAlignment::End => to_offset - size,
            _ => to_offset,
        };

        // Clamp to valid bounds.
        let max_offset = (self.total_size - self.container_size).max(0.0);
        adjusted.clamp(0.0, max_offset)
    }

    /// Calculates the scroll offset to navigate to a specific item.
    ///
    /// This is a convenience wrapper that returns just the offset.
    ///
    /// # Parameters
    ///
    /// - `index`: The index of the target item.
    /// - `alignment`: How to align the item within the viewport.
    ///
    /// # Returns
    ///
    /// - `Ok(f64)`: The scroll offset to navigate to.
    /// - `Err(VirtualizerError)`: If the index is out of bounds.
    ///
    /// # Errors
    ///
    /// - Returns `IndexOutOfBounds` if index >= item_count.
    pub fn scroll_to_index(
        &self,
        index: usize,
        alignment: ScrollAlignment,
    ) -> Result<f64, VirtualizerError> {
        // Validate the index.
        if index >= self.options.item_count {
            return Err(VirtualizerError::IndexOutOfBounds {
                requested: index,
                total: self.options.item_count,
            });
        }

        // Get the offset for the index.
        let result = self.get_offset_for_index(index, alignment);

        // Return the offset or fall back to current scroll offset.
        Ok(result.map_or(self.scroll_offset, |(offset, _)| offset))
    }

    /// Prepares a scroll-to-index operation with full options.
    ///
    /// Returns the target offset, resolved alignment, and behavior for
    /// the hook layer to apply to the DOM.
    ///
    /// # Parameters
    ///
    /// - `index`: The target item index.
    /// - `options`: Scroll options including alignment and behavior.
    /// - `now_ms`: Monotonic or wall time in milliseconds for reconciliation timeouts.
    ///
    /// # Returns
    ///
    /// - `Ok(ScrollState)`: The scroll state to reconcile.
    /// - `Err(VirtualizerError)`: If the index is out of bounds.
    ///
    /// # Errors
    ///
    /// - Returns `IndexOutOfBounds` if index >= item_count.
    pub fn prepare_scroll_to_index(
        &mut self,
        index: usize,
        options: ScrollToOptions,
        now_ms: f64,
    ) -> Result<ScrollState, VirtualizerError> {
        // Validate the index.
        if index >= self.options.item_count {
            return Err(VirtualizerError::IndexOutOfBounds {
                requested: index,
                total: self.options.item_count,
            });
        }

        // Get the offset for the index.
        let result = self.get_offset_for_index(index, options.align);
        let (offset, _align) = result.unwrap_or((self.scroll_offset, options.align));

        // Create a scroll state for reconciliation.
        let state = ScrollState::new(Some(index), options.align, options.behavior, now_ms, offset);

        // Store the scroll state.
        self.scroll_state = Some(state.clone());
        self.notify_change();

        Ok(state)
    }

    /// Prepares a scroll-to-offset operation.
    ///
    /// Returns the target scroll state for the hook layer to apply.
    ///
    /// # Parameters
    ///
    /// - `to_offset`: The target scroll offset in pixels.
    /// - `options`: Scroll options including alignment and behavior.
    /// - `now_ms`: Timestamp in milliseconds for reconciliation timeouts.
    ///
    /// # Returns
    ///
    /// - `ScrollState`: The scroll state to reconcile.
    pub fn prepare_scroll_to_offset(
        &mut self,
        to_offset: f64,
        options: ScrollToOptions,
        now_ms: f64,
    ) -> ScrollState {
        // Compute the aligned offset.
        let offset = self.get_offset_for_alignment(to_offset, options.align, 0.0);

        // Create a scroll state for reconciliation.
        let state = ScrollState::new(None, options.align, options.behavior, now_ms, offset);

        // Store the scroll state.
        self.scroll_state = Some(state.clone());
        self.notify_change();

        state
    }

    /// Prepares a scroll-by operation (relative scroll).
    ///
    /// # Parameters
    ///
    /// - `delta`: The number of pixels to scroll by.
    /// - `behavior`: The scroll animation behavior.
    /// - `now_ms`: Timestamp in milliseconds for reconciliation timeouts.
    ///
    /// # Returns
    ///
    /// - `ScrollState`: The scroll state to reconcile.
    pub fn prepare_scroll_by(
        &mut self,
        delta: f64,
        behavior: ScrollBehavior,
        now_ms: f64,
    ) -> ScrollState {
        // Compute the target offset.
        let offset = self.scroll_offset + delta;

        // Create a scroll state for reconciliation.
        let state = ScrollState::new(None, ScrollAlignment::Start, behavior, now_ms, offset);

        // Store the scroll state.
        self.scroll_state = Some(state.clone());
        self.notify_change();

        state
    }

    /// Recomputes the programmatic scroll target after measurements change.
    ///
    /// Call this during smooth scroll reconciliation so the target offset
    /// tracks dynamic item sizes.
    pub fn refresh_programmatic_scroll_target(&mut self) {
        // Read index and alignment without holding a mutable borrow across get_offset.
        let (idx, align) = {
            let Some(s) = self.scroll_state.as_ref() else {
                return;
            };
            (s.index, s.align)
        };

        if let Some(i) = idx {
            if let Some((off, _)) = self.get_offset_for_index(i, align) {
                if let Some(s) = self.scroll_state.as_mut() {
                    s.last_target_offset = off;
                }
            }
        }
    }

    /// Advances programmatic scroll reconciliation for one animation frame.
    ///
    /// # Parameters
    ///
    /// - `current_scroll`: Observed scroll offset from the DOM.
    /// - `now_ms`: Current time in milliseconds.
    ///
    /// # Returns
    ///
    /// - `ScrollReconcileAction`: Whether to keep scheduling frames.
    pub fn scroll_reconciliation_tick(
        &mut self,
        current_scroll: f64,
        now_ms: f64,
    ) -> ScrollReconcileAction {
        // Stop immediately when no programmatic scroll is active.
        let Some(state) = self.scroll_state.as_mut() else {
            return ScrollReconcileAction::Done;
        };

        let timeout_ms = self.options.scroll_reconciliation_timeout_ms as f64;
        if now_ms - state.started_at > timeout_ms {
            self.scroll_state = None;
            self.notify_change();
            return ScrollReconcileAction::Timeout;
        }

        let target = state.last_target_offset;
        let diff = (current_scroll - target).abs();
        if diff < 1.0 {
            state.stable_frames = state.stable_frames.saturating_add(1);
        } else {
            state.stable_frames = 0;
        }

        let needed = self.options.scroll_reconciliation_stable_frames;
        if state.stable_frames >= needed {
            self.scroll_state = None;
            self.notify_change();
            return ScrollReconcileAction::Done;
        }

        ScrollReconcileAction::Continue
    }

    /// Forces a re-measure of all items by clearing the size cache.
    pub fn measure(&mut self) {
        // Clear the size cache to force re-measurement.
        self.item_size_cache
            .clear(self.options.item_size_mode.base_size());

        // Clear lane assignments for full re-layout.
        self.lane_assignments.clear();

        // Rebuild everything.
        self.measurements_cache.clear();
        self.rebuild_measurements();
        self.recalculate_range();
        self.notify_change();
    }

    /// Returns the virtual item at a given scroll offset.
    ///
    /// # Parameters
    ///
    /// - `offset`: The scroll offset to look up.
    ///
    /// # Returns
    ///
    /// - `Option<&VirtualItem>`: The item at the given offset.
    pub fn get_virtual_item_for_offset(&self, offset: f64) -> Option<&VirtualItem> {
        // Handle empty measurements.
        if self.measurements_cache.is_empty() {
            return None;
        }

        // Binary search for the item closest to the offset.
        let last = self.measurements_cache.len() - 1;
        let mut low = 0usize;
        let mut high = last;

        while low <= high {
            let middle = low + (high - low) / 2;
            let current_value = self.measurements_cache.get(middle).map_or(0.0, |i| i.start);

            if current_value < offset {
                if middle == high {
                    break;
                }
                low = middle + 1;
            } else if current_value > offset {
                if middle == 0 {
                    break;
                }
                high = middle - 1;
            } else {
                return self.measurements_cache.get(middle);
            }
        }

        // Return the closest item.
        let idx = if low > 0 { low - 1 } else { 0 };
        self.measurements_cache.get(idx)
    }

    /// Returns the list of virtual items in the current visible range.
    ///
    /// Uses the custom range extractor if provided, otherwise returns
    /// items in the contiguous range with overscan applied.
    ///
    /// # Returns
    ///
    /// - `Vec<VirtualItem>`: Metadata for each item in the visible range.
    pub fn get_virtual_items(&self) -> Vec<VirtualItem> {
        // Handle disabled state.
        if !self.options.enabled {
            return Vec::new();
        }

        let key = self.virtual_items_cache_key();

        // Return a clone from the memo cache when the fingerprint matches.
        {
            let cache = self.virtual_items_cache.borrow();
            if let Some((cached_key, cached_vec)) = cache.as_ref() {
                if *cached_key == key {
                    return cached_vec.clone();
                }
            }
        }

        // Get the rendered indices.
        let indexes = self.get_virtual_indexes();

        // Map indices to measurements.
        let mut items = Vec::with_capacity(indexes.len());
        for i in indexes {
            if let Some(measurement) = self.measurements_cache.get(i) {
                items.push(measurement.clone());
            }
        }

        *self.virtual_items_cache.borrow_mut() = Some((key, items.clone()));

        items
    }

    /// Returns the indices that should be rendered.
    ///
    /// # Returns
    ///
    /// - `Vec<usize>`: The item indices to render.
    fn get_virtual_indexes(&self) -> Vec<usize> {
        // Build the visible range descriptor.
        let visible_range = VisibleRange {
            start_index: self.range_start,
            end_index: self.range_end,
            overscan: self.options.overscan,
            count: self.options.item_count,
        };

        // Use custom range extractor if provided.
        if let Some(ref extractor) = self.options.range_extractor {
            return extractor(visible_range);
        }

        // Default range extraction with overscan.
        Self::default_range_extractor(visible_range)
    }

    /// Default range extractor that applies overscan to the visible range.
    ///
    /// # Parameters
    ///
    /// - `range`: The visible range descriptor.
    ///
    /// # Returns
    ///
    /// - `Vec<usize>`: The contiguous set of indices to render.
    fn default_range_extractor(range: VisibleRange) -> Vec<usize> {
        // Handle empty range.
        if range.count == 0 {
            return Vec::new();
        }

        // Apply overscan to start and end.
        let start = range.start_index.saturating_sub(range.overscan);
        let end = (range.end_index + range.overscan).min(range.count.saturating_sub(1));

        // Build the index list.
        (start..=end).collect()
    }

    /// Returns the total scrollable size of the virtualized region.
    ///
    /// # Returns
    ///
    /// - `f64`: Total size in pixels including padding.
    pub fn total_size(&self) -> f64 {
        // Return the precomputed total size.
        self.total_size
    }

    /// Returns the current scroll offset.
    ///
    /// # Returns
    ///
    /// - `f64`: Current scroll position in pixels.
    pub fn scroll_offset(&self) -> f64 {
        // Return the current scroll offset.
        self.scroll_offset
    }

    /// Returns the current container viewport size.
    ///
    /// # Returns
    ///
    /// - `f64`: Current container size in pixels.
    pub fn container_size(&self) -> f64 {
        // Return the current container size.
        self.container_size
    }

    /// Returns the start index of the visible range (inclusive).
    ///
    /// # Returns
    ///
    /// - `usize`: The first index in the visible range.
    pub fn range_start(&self) -> usize {
        // Return the range start.
        self.range_start
    }

    /// Returns the end index of the visible range (inclusive).
    ///
    /// # Returns
    ///
    /// - `usize`: The last index in the visible range.
    pub fn range_end(&self) -> usize {
        // Return the range end.
        self.range_end
    }

    /// Returns the current virtualizer options.
    ///
    /// # Returns
    ///
    /// - `&VirtualizerOptions`: Reference to the current options.
    pub fn options(&self) -> &VirtualizerOptions {
        // Return a reference to the options.
        &self.options
    }

    /// Returns the item count.
    ///
    /// # Returns
    ///
    /// - `usize`: The total number of items.
    pub fn item_count(&self) -> usize {
        // Return the item count from options.
        self.options.item_count
    }

    /// Returns the measurement cache.
    ///
    /// # Returns
    ///
    /// - `&MeasurementCache`: Reference to the measurement cache.
    pub fn measurement_cache(&self) -> &MeasurementCache {
        // Return a reference to the cache.
        &self.item_size_cache
    }

    /// Returns the full measurements array.
    ///
    /// # Returns
    ///
    /// - `&[VirtualItem]`: The precomputed measurements for all items.
    pub fn measurements(&self) -> &[VirtualItem] {
        // Return the measurements cache slice.
        &self.measurements_cache
    }

    /// Returns whether the virtualizer requires item measurement.
    ///
    /// # Returns
    ///
    /// - `bool`: True if the item size mode requires runtime measurement.
    pub fn requires_measurement(&self) -> bool {
        // Delegate to the item size mode.
        self.options.item_size_mode.requires_measurement()
    }

    /// Returns whether the user is currently scrolling.
    ///
    /// # Returns
    ///
    /// - `bool`: True if a scroll event is in progress.
    pub fn is_scrolling(&self) -> bool {
        // Return the current scrolling state.
        self.is_scrolling
    }

    /// Returns whether the last scroll was forward (down/right).
    ///
    /// # Returns
    ///
    /// - `Option<bool>`: True if forward, false if backward, None if no scroll.
    pub fn is_scroll_forward(&self) -> Option<bool> {
        // Return the scroll direction.
        self.scroll_forward
    }

    /// Returns the accumulated scroll adjustments.
    ///
    /// # Returns
    ///
    /// - `f64`: The accumulated adjustments in pixels.
    pub fn scroll_adjustments(&self) -> f64 {
        // Return the current adjustments.
        self.scroll_adjustments
    }

    /// Returns the active scroll state, if any.
    ///
    /// # Returns
    ///
    /// - `Option<&ScrollState>`: The active scroll operation state.
    pub fn scroll_state(&self) -> Option<&ScrollState> {
        // Return a reference to the scroll state.
        self.scroll_state.as_ref()
    }

    /// Clears the active scroll state.
    pub fn clear_scroll_state(&mut self) {
        // Remove the active scroll state.
        self.scroll_state = None;
        self.notify_change();
    }

    /// Returns whether the virtualizer is enabled.
    ///
    /// # Returns
    ///
    /// - `bool`: True if the virtualizer is enabled.
    pub fn is_enabled(&self) -> bool {
        // Return the enabled state.
        self.options.enabled
    }

    /// Sets the is_scrolling state.
    ///
    /// # Parameters
    ///
    /// - `is_scrolling`: The new scrolling state.
    pub fn set_is_scrolling(&mut self, is_scrolling: bool) {
        // Update the scrolling state.
        self.is_scrolling = is_scrolling;

        // Clear direction when scrolling stops.
        if !is_scrolling {
            self.scroll_forward = None;
        }

        self.notify_change();
    }

    /// Returns the size of a specific item.
    ///
    /// # Parameters
    ///
    /// - `index`: The item index.
    ///
    /// # Returns
    ///
    /// - `Option<f64>`: The item size if the index is valid.
    pub fn item_size(&self, index: usize) -> Option<f64> {
        // Look up from measurements.
        self.measurements_cache.get(index).map(|item| item.size)
    }

    /// Returns the offset of a specific item.
    ///
    /// # Parameters
    ///
    /// - `index`: The item index.
    ///
    /// # Returns
    ///
    /// - `Option<f64>`: The item offset if the index is valid.
    pub fn item_offset(&self, index: usize) -> Option<f64> {
        // Look up from measurements.
        self.measurements_cache.get(index).map(|item| item.start)
    }

    /// Recalculates the visible range from current scroll state.
    fn recalculate_range(&mut self) {
        // Delegate to the range calculator.
        let result = RangeCalculator::calculate_range(
            &self.measurements_cache,
            self.container_size,
            self.scroll_offset,
            self.options.lanes,
        );

        // Update the range if a valid result was returned.
        if let Some((start, end)) = result {
            self.range_start = start;
            self.range_end = end;
        } else {
            self.range_start = 0;
            self.range_end = 0;
        }
    }
}
