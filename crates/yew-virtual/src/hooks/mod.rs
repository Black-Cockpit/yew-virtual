/// Scroll target binding for DOM programmatic scrolling.
///
/// Defines whether the virtualizer scrolls an element, the window, or nothing
/// until the hook attaches a target.
#[cfg(target_arch = "wasm32")]
pub mod scroll_binding;

/// Browser scroll helpers (`scrollTo`, reading offsets, compensation).
///
/// Maps logical scroll offsets to `web_sys` APIs for element and window targets.
#[cfg(target_arch = "wasm32")]
pub mod scroll_application;

/// Per-item `ResizeObserver` registry for dynamic row/column measurement.
///
/// Keeps observer closures alive and forwards sizes into the core virtualizer.
#[cfg(target_arch = "wasm32")]
pub mod item_measurement_registry;

/// Yew hook for container-based virtualization.
///
/// Provides the `use_virtualizer` hook that manages a virtualizer instance
/// bound to a scroll container element, handling scroll events, resize
/// observations, and item measurements automatically.
#[cfg(target_arch = "wasm32")]
pub mod use_virtualizer;

/// Yew hook for window-based virtualization.
///
/// Provides the `use_window_virtualizer` hook that uses the browser window
/// as the scroll container instead of a specific DOM element.
#[cfg(target_arch = "wasm32")]
pub mod use_window_virtualizer;

/// Handle type returned by virtualizer hooks.
///
/// Provides the interface for reading virtual items, total size, DOM scrolling,
/// and triggering measurement updates or programmatic scroll navigation.
#[cfg(target_arch = "wasm32")]
pub mod virtualizer_handle;

/// Re-exports for convenient access to hook types.
#[cfg(target_arch = "wasm32")]
pub mod prelude;
