use crate::core::virtual_key::VirtualKey;

/// Metadata for a single virtualized item.
///
/// Represents the computed layout information that the virtualizer
/// produces for each item within the visible range. Components use
/// this metadata to position and render items in the scroll container.
#[derive(Debug, Clone, PartialEq)]
pub struct VirtualItem {
    /// The index of this item in the full dataset.
    pub index: usize,

    /// The measured or estimated size of this item in pixels.
    ///
    /// Represents height for vertical lists or width for horizontal lists.
    pub size: f64,

    /// The offset in pixels from the start of the virtualized region.
    ///
    /// Represents the top offset for vertical lists or the left offset
    /// for horizontal lists.
    pub start: f64,

    /// The end position in pixels (start + size).
    pub end: f64,

    /// A stable identity key for this item.
    ///
    /// When a custom `get_item_key` is provided, this key follows the
    /// item across reorders. Otherwise it equals the index.
    pub key: VirtualKey,

    /// The lane index for grid layouts (0 for single-column lists).
    pub lane: usize,
}

impl VirtualItem {
    /// Creates a new virtual item with the given layout parameters.
    ///
    /// # Parameters
    ///
    /// - `index`: The item's position in the full dataset.
    /// - `size`: The item's size in pixels along the scroll axis.
    /// - `start`: The item's offset from the start of the virtual region.
    pub fn new(index: usize, size: f64, start: f64) -> Self {
        // Compute the end position from start and size.
        let end = start + size;

        // Construct the virtual item with the computed layout.
        Self {
            index,
            size,
            start,
            end,
            key: VirtualKey::Index(index),
            lane: 0,
        }
    }

    /// Creates a new virtual item with a specific key and lane.
    ///
    /// # Parameters
    ///
    /// - `index`: The item's position in the full dataset.
    /// - `size`: The item's size in pixels along the scroll axis.
    /// - `start`: The item's offset from the start of the virtual region.
    /// - `key`: The stable identity key for this item.
    /// - `lane`: The lane (column/row) index for grid layouts.
    pub fn with_key_and_lane(
        index: usize,
        size: f64,
        start: f64,
        key: VirtualKey,
        lane: usize,
    ) -> Self {
        // Compute the end position from start and size.
        let end = start + size;

        // Construct the virtual item with key and lane information.
        Self {
            index,
            size,
            start,
            end,
            key,
            lane,
        }
    }
}
