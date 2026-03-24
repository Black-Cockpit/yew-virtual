/// Scroll direction configuration for the virtualizer.
///
/// Defines whether the virtualizer operates in vertical or horizontal mode,
/// which determines how item measurements and scroll positions are interpreted.
pub mod scroll_direction;

/// Size estimation strategy for virtual items.
///
/// Provides the different sizing strategies (fixed, estimated, dynamic)
/// that determine how the virtualizer calculates item dimensions.
pub mod item_size_mode;

/// Scroll alignment options for programmatic navigation.
///
/// Defines alignment behaviors (start, center, end, auto) used when
/// scrolling to a specific item by index.
pub mod scroll_alignment;

/// Scroll behavior options for programmatic scroll operations.
///
/// Controls the animation style (auto, smooth, instant) when the
/// virtualizer programmatically scrolls to an offset or item.
pub mod scroll_behavior;

/// Combined options for programmatic scroll operations.
///
/// Bundles alignment and behavior into a single options struct
/// used by scroll-to-index and scroll-to-offset methods.
pub mod scroll_to_options;

/// Dimensions of a scroll container or viewport.
///
/// Represents width and height of a rectangular area for tracking
/// scroll container measurements.
pub mod rect;

/// Visible range descriptor for the current viewport.
///
/// Contains start/end indices, overscan, and count for computing
/// which item indices should be rendered. Passed to custom range extractors.
pub mod visible_range;

/// Stable identity key for virtual items.
///
/// Supports numeric and string-based keys for custom key extractors
/// that produce stable identities across item reordering.
pub mod virtual_key;

/// Internal state for active programmatic scroll operations.
///
/// Tracks the target, alignment, behavior, and settling progress
/// of in-flight scroll reconciliation.
pub mod scroll_state;

/// Outcome of applying an item measurement update.
///
/// Used to report layout changes and DOM scroll compensation.
pub mod measure_item_outcome;

/// Action returned from programmatic scroll reconciliation ticks.
///
/// Controls whether the rAF loop continues or stops.
pub mod scroll_reconcile_action;

/// Type alias for scroll-adjustment policy on item resize.
///
/// See [`VirtualizerOptions`](virtualizer_options::VirtualizerOptions).
pub mod should_adjust_scroll_on_resize_fn;

/// Type alias for custom ResizeObserver-based item sizing.
///
/// See [`VirtualizerOptions`](virtualizer_options::VirtualizerOptions).
pub mod measure_element_fn;

/// Configuration options for the virtualizer engine.
///
/// Contains the complete set of parameters needed to initialize and
/// configure a virtualizer instance, including item count, sizing,
/// overscan, padding, gap, lanes, and scroll settings.
pub mod virtualizer_options;

/// Metadata for a single virtualized item.
///
/// Represents the computed layout information for an individual item
/// in the virtual list, including its index, size, offset position, key, and lane.
pub mod virtual_item;

/// The core virtualizer engine that drives all virtualization logic.
///
/// Implements range calculation, offset tracking, scroll handling,
/// measurement updates, multi-lane layout, and programmatic scroll navigation.
pub mod virtualizer;

/// Error types for the virtualization engine.
///
/// Defines all error conditions that can occur during virtualizer
/// operations such as invalid configuration or measurement failures.
pub mod virtualizer_error;

/// Measurement cache for storing dynamic item sizes.
///
/// Provides an efficient cache for storing and retrieving measured
/// item dimensions keyed by VirtualKey, supporting the dynamic sizing strategy.
pub mod measurement_cache;

/// Range calculation utilities for visible item determination.
///
/// Contains the logic for computing which items fall within the
/// visible viewport plus overscan buffer, with multi-lane support.
pub mod range_calculator;

/// Re-exports for convenient access to core types.
pub mod prelude;
