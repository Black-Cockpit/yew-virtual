use std::sync::Arc;

use crate::core::item_size_mode::ItemSizeMode;
use crate::core::measure_element_fn::MeasureElementFn;
use crate::core::rect::Rect;
use crate::core::scroll_direction::ScrollDirection;
use crate::core::scroll_to_options::ScrollToOptions;
use crate::core::should_adjust_scroll_on_resize_fn::ShouldAdjustScrollOnResizeFn;
use crate::core::virtual_item::VirtualItem;
use crate::core::virtual_key::VirtualKey;
use crate::core::visible_range::VisibleRange;

/// Configuration options for initializing a virtualizer instance.
///
/// Contains all parameters the virtualizer needs to compute visible ranges,
/// item positions, and total scrollable size. Developers configure these
/// options before creating a virtualizer.
#[derive(Clone)]
pub struct VirtualizerOptions {
    /// Total number of items in the dataset.
    pub item_count: usize,

    /// Strategy for determining item sizes.
    pub item_size_mode: ItemSizeMode,

    /// Scroll direction (vertical or horizontal).
    pub scroll_direction: ScrollDirection,

    /// Number of extra items to render beyond the visible viewport.
    pub overscan: usize,

    /// Padding in pixels before the first item.
    pub padding_start: f64,

    /// Padding in pixels after the last item.
    pub padding_end: f64,

    /// Extra scroll padding at the start that affects scroll-to alignment
    /// calculations but does not affect total size.
    pub scroll_padding_start: f64,

    /// Extra scroll padding at the end that affects scroll-to alignment
    /// calculations but does not affect total size.
    pub scroll_padding_end: f64,

    /// Gap in pixels between consecutive items.
    pub gap: f64,

    /// Number of parallel lanes (columns for vertical, rows for horizontal).
    ///
    /// When greater than 1, items are distributed across lanes using a
    /// shortest-lane-first algorithm. Enables grid-style layouts.
    pub lanes: usize,

    /// Offset applied to item positions to account for elements above
    /// the virtualizer (e.g., sticky headers). Affects total size and
    /// measurement start offsets.
    pub scroll_margin: f64,

    /// Optional fixed size of the scroll container viewport in pixels.
    ///
    /// When `None`, the container size must be provided via measurement
    /// or set dynamically at runtime.
    pub container_size: Option<f64>,

    /// Whether to use window-level scrolling instead of a scroll container.
    pub use_window_scroll: bool,

    /// Initial scroll offset in pixels. Applied when the virtualizer
    /// is first created to restore scroll position.
    pub initial_offset: f64,

    /// Initial scroll container dimensions before measurement is available.
    pub initial_rect: Rect,

    /// Whether the virtualizer is enabled. When disabled, clears all
    /// measurements and returns empty items without destroying the instance.
    pub enabled: bool,

    /// Whether the scroll container uses right-to-left direction.
    /// Inverts horizontal scroll offset when true.
    pub is_rtl: bool,

    /// DOM attribute name used to look up item indices on measured
    /// elements. Defaults to `"data-index"`.
    pub index_attribute: String,

    /// Delay in milliseconds before `is_scrolling` resets to false
    /// after scrolling stops. Defaults to 150.
    pub is_scrolling_reset_delay: u32,

    /// Whether to use the native `scrollend` event instead of debounce
    /// for detecting scroll stop.
    pub use_scrollend_event: bool,

    /// Whether to wrap ResizeObserver callbacks in `requestAnimationFrame`
    /// to avoid layout thrashing.
    pub use_animation_frame_with_resize_observer: bool,

    /// Per-index size estimation callback. When provided, overrides
    /// the scalar value from `item_size_mode` for initial estimates.
    /// Each call receives an item index and returns the estimated size.
    pub estimate_size: Option<Arc<dyn Fn(usize) -> f64 + Send + Sync>>,

    /// Custom key extractor callback. When provided, returns a stable
    /// identity key for each item index. Defaults to using the index.
    pub get_item_key: Option<Arc<dyn Fn(usize) -> VirtualKey + Send + Sync>>,

    /// Custom range extractor callback. When provided, returns the
    /// list of item indices to render given the visible range.
    /// Enables sticky headers, pinned rows, and other custom layouts.
    pub range_extractor: Option<Arc<dyn Fn(VisibleRange) -> Vec<usize> + Send + Sync>>,

    /// Pre-populated measurements cache for SSR hydration or state
    /// restoration. When provided, these cached items are used instead
    /// of estimates on the first measurement pass.
    pub initial_measurements_cache: Vec<VirtualItem>,

    /// When set, overrides `initial_offset` on construction with the callback result.
    pub initial_offset_fn: Option<Arc<dyn Fn() -> f64 + Send + Sync>>,

    /// Invoked when range, scroll state, or layout affecting visible items changes.
    ///
    /// Framework hooks use this to schedule renders without polling.
    pub on_change: Option<Arc<dyn Fn() + Send + Sync>>,

    /// Custom programmatic scroll implementation. When `None`, hooks use the
    /// browser default (`scrollTo` on the container or window).
    ///
    /// Receives the logical scroll offset and full scroll options.
    pub scroll_to_fn: Option<Arc<dyn Fn(f64, ScrollToOptions) + Send + Sync>>,

    /// When set, decides whether to adjust scroll when an item above the
    /// viewport resizes. Arguments are `(index, previous_size, new_size)`.
    pub should_adjust_scroll_position_on_item_size_change: Option<ShouldAdjustScrollOnResizeFn>,

    /// When false, scroll compensation for resizes above the viewport is skipped
    /// while a smooth programmatic scroll is active.
    pub adjust_scroll_on_resize_during_smooth_scroll: bool,

    /// Custom measurement from observed element size. Arguments are
    /// `(border_width, border_height, scroll_direction)`; return size along the scroll axis.
    pub measure_element: Option<MeasureElementFn>,

    /// When false, hooks may skip attaching ResizeObserver work during active scroll.
    pub measure_during_scroll: bool,

    /// Maximum duration in milliseconds for smooth-scroll reconciliation.
    pub scroll_reconciliation_timeout_ms: u32,

    /// Consecutive matching frames required before smooth scroll is considered settled.
    pub scroll_reconciliation_stable_frames: u32,
}

impl std::fmt::Debug for VirtualizerOptions {
    /// Formats the options for debug output, omitting closure fields.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Format all non-closure fields.
        f.debug_struct("VirtualizerOptions")
            .field("item_count", &self.item_count)
            .field("item_size_mode", &self.item_size_mode)
            .field("scroll_direction", &self.scroll_direction)
            .field("overscan", &self.overscan)
            .field("padding_start", &self.padding_start)
            .field("padding_end", &self.padding_end)
            .field("scroll_padding_start", &self.scroll_padding_start)
            .field("scroll_padding_end", &self.scroll_padding_end)
            .field("gap", &self.gap)
            .field("lanes", &self.lanes)
            .field("scroll_margin", &self.scroll_margin)
            .field("container_size", &self.container_size)
            .field("use_window_scroll", &self.use_window_scroll)
            .field("initial_offset", &self.initial_offset)
            .field("initial_rect", &self.initial_rect)
            .field("enabled", &self.enabled)
            .field("is_rtl", &self.is_rtl)
            .field("index_attribute", &self.index_attribute)
            .field("is_scrolling_reset_delay", &self.is_scrolling_reset_delay)
            .field("use_scrollend_event", &self.use_scrollend_event)
            .field(
                "use_animation_frame_with_resize_observer",
                &self.use_animation_frame_with_resize_observer,
            )
            .field(
                "estimate_size",
                &self.estimate_size.as_ref().map(|_| "Fn(usize) -> f64"),
            )
            .field(
                "get_item_key",
                &self
                    .get_item_key
                    .as_ref()
                    .map(|_| "Fn(usize) -> VirtualKey"),
            )
            .field(
                "range_extractor",
                &self
                    .range_extractor
                    .as_ref()
                    .map(|_| "Fn(VisibleRange) -> Vec<usize>"),
            )
            .field(
                "initial_measurements_cache",
                &self.initial_measurements_cache.len(),
            )
            .field(
                "initial_offset_fn",
                &self.initial_offset_fn.as_ref().map(|_| "Fn() -> f64"),
            )
            .field("on_change", &self.on_change.as_ref().map(|_| "Fn()"))
            .field(
                "scroll_to_fn",
                &self
                    .scroll_to_fn
                    .as_ref()
                    .map(|_| "Fn(f64, ScrollToOptions)"),
            )
            .field(
                "should_adjust_scroll_position_on_item_size_change",
                &self
                    .should_adjust_scroll_position_on_item_size_change
                    .as_ref()
                    .map(|_| "Fn(usize, f64, f64) -> bool"),
            )
            .field(
                "adjust_scroll_on_resize_during_smooth_scroll",
                &self.adjust_scroll_on_resize_during_smooth_scroll,
            )
            .field(
                "measure_element",
                &self
                    .measure_element
                    .as_ref()
                    .map(|_| "Fn(f64, f64, ScrollDirection) -> f64"),
            )
            .field("measure_during_scroll", &self.measure_during_scroll)
            .field(
                "scroll_reconciliation_timeout_ms",
                &self.scroll_reconciliation_timeout_ms,
            )
            .field(
                "scroll_reconciliation_stable_frames",
                &self.scroll_reconciliation_stable_frames,
            )
            .finish()
    }
}

impl PartialEq for VirtualizerOptions {
    /// Compares options for equality, ignoring closure fields.
    fn eq(&self, other: &Self) -> bool {
        // Compare all non-closure fields.
        self.item_count == other.item_count
            && self.item_size_mode == other.item_size_mode
            && self.scroll_direction == other.scroll_direction
            && self.overscan == other.overscan
            && self.padding_start == other.padding_start
            && self.padding_end == other.padding_end
            && self.scroll_padding_start == other.scroll_padding_start
            && self.scroll_padding_end == other.scroll_padding_end
            && self.gap == other.gap
            && self.lanes == other.lanes
            && self.scroll_margin == other.scroll_margin
            && self.container_size == other.container_size
            && self.use_window_scroll == other.use_window_scroll
            && self.initial_offset == other.initial_offset
            && self.initial_rect == other.initial_rect
            && self.enabled == other.enabled
            && self.is_rtl == other.is_rtl
            && self.index_attribute == other.index_attribute
            && self.is_scrolling_reset_delay == other.is_scrolling_reset_delay
            && self.use_scrollend_event == other.use_scrollend_event
            && self.use_animation_frame_with_resize_observer
                == other.use_animation_frame_with_resize_observer
            && self.adjust_scroll_on_resize_during_smooth_scroll
                == other.adjust_scroll_on_resize_during_smooth_scroll
            && self.measure_during_scroll == other.measure_during_scroll
            && self.scroll_reconciliation_timeout_ms == other.scroll_reconciliation_timeout_ms
            && self.scroll_reconciliation_stable_frames == other.scroll_reconciliation_stable_frames
    }
}

impl Default for VirtualizerOptions {
    /// Returns sensible default virtualizer options.
    ///
    /// # Returns
    ///
    /// - `VirtualizerOptions`: Defaults with zero items, fixed 50px size,
    ///   vertical direction, overscan of 5, and no padding or gap.
    fn default() -> Self {
        // Construct defaults suitable for a basic vertical list.
        Self {
            item_count: 0,
            item_size_mode: ItemSizeMode::default(),
            scroll_direction: ScrollDirection::default(),
            overscan: 5,
            padding_start: 0.0,
            padding_end: 0.0,
            scroll_padding_start: 0.0,
            scroll_padding_end: 0.0,
            gap: 0.0,
            lanes: 1,
            scroll_margin: 0.0,
            container_size: None,
            use_window_scroll: false,
            initial_offset: 0.0,
            initial_rect: Rect::default(),
            enabled: true,
            is_rtl: false,
            index_attribute: "data-index".to_string(),
            is_scrolling_reset_delay: 150,
            use_scrollend_event: false,
            use_animation_frame_with_resize_observer: false,
            estimate_size: None,
            get_item_key: None,
            range_extractor: None,
            initial_measurements_cache: Vec::new(),
            initial_offset_fn: None,
            on_change: None,
            scroll_to_fn: None,
            should_adjust_scroll_position_on_item_size_change: None,
            adjust_scroll_on_resize_during_smooth_scroll: false,
            measure_element: None,
            measure_during_scroll: true,
            scroll_reconciliation_timeout_ms: 5000,
            scroll_reconciliation_stable_frames: 16,
        }
    }
}
