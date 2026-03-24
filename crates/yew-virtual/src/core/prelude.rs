//! Re-exports for convenient access to core virtualization types.
//!
//! This module provides a centralized location for importing commonly used
//! core types, enums, and the virtualizer engine. Import this prelude to get
//! access to the most frequently used items in the core module.

pub use super::item_size_mode::ItemSizeMode;
pub use super::measure_item_outcome::MeasureItemOutcome;
pub use super::measurement_cache::MeasurementCache;
pub use super::range_calculator::RangeCalculator;
pub use super::rect::Rect;
pub use super::scroll_alignment::ScrollAlignment;
pub use super::scroll_behavior::ScrollBehavior;
pub use super::scroll_direction::ScrollDirection;
pub use super::scroll_reconcile_action::ScrollReconcileAction;
pub use super::scroll_state::ScrollState;
pub use super::scroll_to_options::ScrollToOptions;
pub use super::virtual_item::VirtualItem;
pub use super::virtual_key::VirtualKey;
pub use super::virtualizer::Virtualizer;
pub use super::virtualizer_error::VirtualizerError;
pub use super::virtualizer_options::VirtualizerOptions;
pub use super::visible_range::VisibleRange;
