use crate::core::virtual_item::VirtualItem;

/// Utility for calculating visible item ranges within a viewport.
///
/// Provides pure functions that determine which items fall within the
/// visible area of a scroll container, accounting for overscan buffers
/// and multi-lane layouts. This is the core logic that the virtualizer
/// uses to decide which items to render.
pub struct RangeCalculator;

impl RangeCalculator {
    /// Calculates the range of visible items given scroll state and measurements.
    ///
    /// For single-lane lists, uses binary search to find the first and last
    /// visible items. For multi-lane layouts, expands the range to cover all
    /// lanes' visible items.
    ///
    /// # Parameters
    ///
    /// - `measurements`: The precomputed measurement cache of virtual items.
    /// - `outer_size`: Size of the visible viewport in pixels.
    /// - `scroll_offset`: Current scroll position in pixels.
    /// - `lanes`: Number of parallel lanes in the layout.
    ///
    /// # Returns
    ///
    /// - `Option<(usize, usize)>`: Tuple of (start_index, end_index) both inclusive,
    ///   or None if there are no measurements or the viewport has zero size.
    pub fn calculate_range(
        measurements: &[VirtualItem],
        outer_size: f64,
        scroll_offset: f64,
        lanes: usize,
    ) -> Option<(usize, usize)> {
        // Handle empty measurements.
        if measurements.is_empty() {
            return None;
        }

        // Handle zero viewport size.
        if outer_size <= 0.0 {
            return None;
        }

        let last_index = measurements.len() - 1;

        // Handle case when item count is less than or equal to lanes.
        if measurements.len() <= lanes {
            return Some((0, last_index));
        }

        // Find the nearest start index using binary search.
        let mut start_index =
            Self::find_nearest_binary_search(0, last_index, measurements, scroll_offset);

        // Find the end index by expanding forward.
        let mut end_index = start_index;

        if lanes == 1 {
            // Single lane: expand forward until items exceed viewport.
            while end_index < last_index {
                if let Some(item) = measurements.get(end_index) {
                    if item.end >= scroll_offset + outer_size {
                        break;
                    }
                } else {
                    break;
                }
                end_index += 1;
            }
        } else {
            // Multi-lane: expand forward until all lanes are covered.
            let mut end_per_lane = vec![0.0f64; lanes];
            while end_index < last_index
                && end_per_lane
                    .iter()
                    .any(|pos| *pos < scroll_offset + outer_size)
            {
                if let Some(item) = measurements.get(end_index)
                    && item.lane < lanes
                {
                    end_per_lane[item.lane] = item.end;
                }
                end_index += 1;
            }

            // Expand backward until all lanes' visible items closer to the top are included.
            let mut start_per_lane = vec![scroll_offset + outer_size; lanes];
            while start_index > 0 && start_per_lane.iter().any(|pos| *pos >= scroll_offset) {
                if let Some(item) = measurements.get(start_index)
                    && item.lane < lanes
                {
                    start_per_lane[item.lane] = item.start;
                }
                start_index -= 1;
            }

            // Align start to lane boundary.
            start_index = start_index.saturating_sub(start_index % lanes);

            // Align end to lane boundary.
            let remainder = end_index % lanes;
            if remainder != 0 {
                end_index = (end_index + (lanes - 1 - remainder)).min(last_index);
            }
        }

        Some((start_index, end_index))
    }

    /// Finds the nearest item index whose start offset is closest to the target.
    ///
    /// # Parameters
    ///
    /// - `low`: Lower bound of the search range.
    /// - `high`: Upper bound of the search range.
    /// - `measurements`: The virtual item measurements.
    /// - `value`: The target offset to search for.
    ///
    /// # Returns
    ///
    /// - `usize`: The index of the nearest item.
    fn find_nearest_binary_search(
        mut low: usize,
        mut high: usize,
        measurements: &[VirtualItem],
        value: f64,
    ) -> usize {
        // Binary search for the nearest start offset.
        while low <= high {
            let middle = low + (high - low) / 2;

            // Get the start offset of the middle item.
            let current_value = measurements.get(middle).map_or(0.0, |item| item.start);

            if current_value < value {
                // Check for underflow before incrementing.
                if middle == high {
                    break;
                }
                low = middle + 1;
            } else if current_value > value {
                // Check for underflow before decrementing.
                if middle == 0 {
                    break;
                }
                high = middle - 1;
            } else {
                return middle;
            }
        }

        // Return the closest index, backing up one if we overshot.
        if low > 0 {
            low - 1
        } else {
            0
        }
    }
}
