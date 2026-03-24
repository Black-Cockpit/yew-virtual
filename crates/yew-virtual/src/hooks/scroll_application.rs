//! Low-level helpers for reading and writing browser scroll positions.
//!
//! Translates the headless virtualizer’s logical offsets into `web_sys` calls
//! on [`HtmlElement`](web_sys::HtmlElement) or [`Window`](web_sys::Window),
//! including RTL horizontal scroll sign conventions.

use wasm_bindgen::JsCast;
use web_sys::{
    window, Element, HtmlElement, ScrollBehavior as WebScrollBehavior, ScrollToOptions, Window,
};

use crate::core::scroll_behavior::ScrollBehavior;
use crate::core::scroll_to_options::ScrollToOptions as CoreScrollToOptions;
use crate::core::virtualizer_error::VirtualizerError;
use crate::hooks::scroll_binding::ScrollBinding;

/// Converts the crate’s scroll behavior enum to the browser’s `ScrollBehavior`.
///
/// # Parameters
///
/// - `b`: Core behavior variant to map.
///
/// # Returns
///
/// - `WebScrollBehavior`: Value accepted by `ScrollToOptions::set_behavior`.
fn to_web_behavior(b: ScrollBehavior) -> WebScrollBehavior {
    // Map each core variant to the web_sys counterpart.
    match b {
        ScrollBehavior::Auto => WebScrollBehavior::Auto,
        ScrollBehavior::Smooth => WebScrollBehavior::Smooth,
        ScrollBehavior::Instant => WebScrollBehavior::Instant,
    }
}

/// Reads the logical scroll offset from a scrollable element.
///
/// Horizontal RTL uses a negated `scrollLeft` so positive offsets match LTR semantics.
///
/// # Parameters
///
/// - `el`: The DOM element with `overflow` scroll.
/// - `horizontal`: When true, reads `scrollLeft`; otherwise `scrollTop`.
/// - `is_rtl`: When true with horizontal, negates the raw `scrollLeft`.
///
/// # Returns
///
/// - `Ok(f64)`: Logical offset in pixels.
pub fn read_element_scroll(
    el: &Element,
    horizontal: bool,
    is_rtl: bool,
) -> Result<f64, VirtualizerError> {
    if horizontal {
        // Read horizontal scroll; invert sign for RTL writing modes.
        let raw = el.scroll_left() as f64;
        Ok(if is_rtl { -raw } else { raw })
    } else {
        // Vertical lists use scrollTop directly.
        Ok(el.scroll_top() as f64)
    }
}

/// Reads the logical scroll offset from the browser window.
///
/// # Parameters
///
/// - `horizontal`: When true, uses `scrollX`; otherwise `scrollY`.
/// - `is_rtl`: When true with horizontal, negates `scrollX`.
///
/// # Returns
///
/// - `Ok(f64)`: Logical window scroll offset.
/// - `Err(VirtualizerError)` when `window` is unavailable.
///
/// # Errors
///
/// - `ScrollContainerUnavailable` if `web_sys::window()` returns `None`.
pub fn read_window_scroll(horizontal: bool, is_rtl: bool) -> Result<f64, VirtualizerError> {
    // Obtain the global window handle.
    let win = web_sys::window().ok_or_else(|| {
        VirtualizerError::ScrollContainerUnavailable("window missing".to_string())
    })?;

    if horizontal {
        // Match element RTL handling for document-level horizontal scroll.
        let raw = win.scroll_x().unwrap_or(0.0);
        Ok(if is_rtl { -raw } else { raw })
    } else {
        Ok(win.scroll_y().unwrap_or(0.0))
    }
}

/// Scrolls the bound target to a logical offset.
///
/// When `scroll_to_fn` is set on options, invokes it and skips native `scrollTo`.
///
/// # Parameters
///
/// - `binding`: Element, window, or none.
/// - `offset`: Logical position along the virtualizer axis.
/// - `horizontal`: Whether the virtualizer scrolls on X.
/// - `is_rtl`: RTL inversion for horizontal writes.
/// - `behavior`: Animation mode for native `scrollTo`.
/// - `scroll_to_fn`: Optional injectable scroll implementation.
/// - `align_opts`: Forwarded to `scroll_to_fn` when used.
///
/// # Returns
///
/// - `Ok(())` after delegating to a custom function or native scroll.
/// - `Err(VirtualizerError)` when the target is missing or invalid.
///
/// # Errors
///
/// - `ScrollContainerUnavailable` for unbound targets, missing window, or non-HTML elements.
pub fn apply_scroll_offset(
    binding: &ScrollBinding,
    offset: f64,
    horizontal: bool,
    is_rtl: bool,
    behavior: ScrollBehavior,
    scroll_to_fn: Option<&std::sync::Arc<dyn Fn(f64, CoreScrollToOptions) + Send + Sync>>,
    align_opts: CoreScrollToOptions,
) -> Result<(), VirtualizerError> {
    // Build scroll options with the requested animation behavior.
    let opts = ScrollToOptions::new();
    opts.set_behavior(to_web_behavior(behavior));

    // Convert logical offset to the raw value expected by the DOM in RTL horizontal mode.
    let raw_offset = if horizontal && is_rtl {
        -offset
    } else {
        offset
    };

    if let Some(f) = scroll_to_fn {
        // User-provided scroll completely replaces native scrolling.
        f(offset, align_opts);
        return Ok(());
    }

    match binding {
        ScrollBinding::None => Err(VirtualizerError::ScrollContainerUnavailable(
            "scroll target not bound".to_string(),
        )),
        ScrollBinding::Element { element } => {
            // scrollTo exists on HtmlElement in web_sys.
            let html = element.dyn_ref::<HtmlElement>().ok_or_else(|| {
                VirtualizerError::ScrollContainerUnavailable(
                    "element is not HtmlElement".to_string(),
                )
            })?;

            if horizontal {
                opts.set_left(raw_offset);
            } else {
                opts.set_top(raw_offset);
            }

            html.scroll_to_with_scroll_to_options(&opts);
            Ok(())
        }
        ScrollBinding::Window => {
            let win = window().ok_or_else(|| {
                VirtualizerError::ScrollContainerUnavailable("window missing".to_string())
            })?;
            scroll_window_to(&win, raw_offset, horizontal, behavior)
        }
    }
}

/// Scrolls the window to an offset using `scrollTo`.
///
/// # Parameters
///
/// - `win`: Active window reference.
/// - `raw`: Already RTL-adjusted pixel value for the active axis.
/// - `horizontal`: Whether to set `left` or `top`.
/// - `behavior`: Scroll animation mode.
///
/// # Returns
///
/// - `Ok(())` after invoking `scroll_to_with_scroll_to_options`.
fn scroll_window_to(
    win: &Window,
    raw: f64,
    horizontal: bool,
    behavior: ScrollBehavior,
) -> Result<(), VirtualizerError> {
    let opts = ScrollToOptions::new();
    opts.set_behavior(to_web_behavior(behavior));

    if horizontal {
        opts.set_left(raw);
    } else {
        opts.set_top(raw);
    }

    win.scroll_to_with_scroll_to_options(&opts);
    Ok(())
}

/// Adds a delta to the current scroll position using an instant `scrollTo`.
///
/// Used when item resizes above the viewport must shift the scroll offset without animation.
///
/// # Parameters
///
/// - `binding`: Active scroll target.
/// - `delta`: Pixels to add to the current logical offset.
/// - `horizontal`: Virtualizer axis.
/// - `is_rtl`: RTL flag for reading and writing.
///
/// # Returns
///
/// - `Ok(())` when compensation scroll was applied.
/// - `Err(VirtualizerError)` when the binding is invalid or read fails.
///
/// # Errors
///
/// - `ScrollContainerUnavailable` when the target cannot be read or scrolled.
pub fn apply_scroll_compensation(
    binding: &ScrollBinding,
    delta: f64,
    horizontal: bool,
    is_rtl: bool,
) -> Result<(), VirtualizerError> {
    // Read the current logical position from the same target we will write to.
    let current = match binding {
        ScrollBinding::None => {
            return Err(VirtualizerError::ScrollContainerUnavailable(
                "scroll target not bound".to_string(),
            ))
        }
        ScrollBinding::Element { element } => read_element_scroll(element, horizontal, is_rtl)?,
        ScrollBinding::Window => read_window_scroll(horizontal, is_rtl)?,
    };

    // Apply instant scroll to the summed offset without a custom scroll hook path.
    let target = current + delta;
    apply_scroll_offset(
        binding,
        target,
        horizontal,
        is_rtl,
        ScrollBehavior::Instant,
        None,
        CoreScrollToOptions::default(),
    )
}
