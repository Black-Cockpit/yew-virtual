use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use yew::prelude::*;

use crate::core::scroll_direction::ScrollDirection;
use crate::core::virtualizer::Virtualizer;
use crate::core::virtualizer_options::VirtualizerOptions;
use crate::hooks::item_measurement_registry::ItemMeasurementRegistry;
use crate::hooks::scroll_binding::ScrollBinding;
use crate::hooks::virtualizer_handle::VirtualizerHandle;

/// Yew hook that creates and manages a window-based virtualizer.
///
/// Uses the browser window as the scroll container instead of a specific
/// DOM element. Automatically handles window scroll and resize events
/// and triggers re-renders when the visible range changes. Supports both
/// vertical and horizontal scroll directions; horizontal RTL negates
/// `scrollX` consistently with the container hook.
///
/// Sets [`ScrollBinding::Window`] on the shared handle while the effect runs.
///
/// # Parameters
///
/// - `options`: Configuration options for the virtualizer.
///
/// # Returns
///
/// - `VirtualizerHandle`: The handle for reading state and triggering actions.
#[hook]
pub fn use_window_virtualizer(options: VirtualizerOptions) -> VirtualizerHandle {
    // Create a render trigger state to force re-renders.
    let render_trigger = use_state(|| 0u64);

    // Shared binding for window-level scrollTo helpers.
    let scroll_binding_state = use_state(|| Rc::new(RefCell::new(ScrollBinding::None)));
    let scroll_binding_rc = (*scroll_binding_state).clone();

    let item_registry_state = use_state(|| Rc::new(RefCell::new(ItemMeasurementRegistry::new())));
    let item_registry_rc = (*item_registry_state).clone();

    // Capture scroll direction and RTL for use in closures.
    let is_horizontal = options.scroll_direction == ScrollDirection::Horizontal;
    let is_rtl = options.is_rtl;

    // Create the virtualizer instance with window scroll enabled.
    let virtualizer = use_memo(options.clone(), |opts| {
        // Ensure the options are configured for window scrolling.
        let mut window_opts = opts.clone();
        window_opts.use_window_scroll = true;

        // Attempt to create the virtualizer with the provided options.
        let virt = match Virtualizer::new(window_opts) {
            Ok(v) => v,
            Err(_e) => {
                // Fall back to an empty virtualizer on failure.
                Virtualizer::empty()
            }
        };
        Rc::new(RefCell::new(virt))
    });

    // Set up window scroll and resize event listeners.
    let scroll_virtualizer = Rc::clone(&virtualizer);
    let scroll_trigger = render_trigger.clone();
    let binding_for_effect = Rc::clone(&scroll_binding_rc);

    use_effect_with(options.clone(), move |_opts| {
        // Get the window object.
        let window = web_sys::window();

        // Store cleanup closures (shared across setup and teardown).
        let scroll_closure: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
        let resize_closure: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
        let cleanup_window: Rc<RefCell<Option<web_sys::Window>>> = Rc::new(RefCell::new(None));

        if let Some(win) = window {
            // Mark the handle as window-scrolled for DOM helpers.
            *binding_for_effect.borrow_mut() = ScrollBinding::Window;

            // Read the initial window size based on scroll direction.
            let initial_size = if is_horizontal {
                win.inner_width()
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0)
            } else {
                win.inner_height()
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0)
            };

            // Seed the engine with the viewport dimension.
            scroll_virtualizer
                .borrow_mut()
                .update_container_size(initial_size);

            // Trigger initial render.
            let current = *scroll_trigger;
            scroll_trigger.set(current.wrapping_add(1));

            // Set up scroll event handler.
            let virt_for_scroll = Rc::clone(&scroll_virtualizer);
            let trigger_for_scroll = scroll_trigger.clone();
            let win_for_scroll = win.clone();

            let on_scroll = Closure::wrap(Box::new(move || {
                // Read the window scroll position based on direction and RTL.
                let scroll_pos = if is_horizontal {
                    let raw = win_for_scroll.scroll_x().unwrap_or(0.0);
                    if is_rtl { -raw } else { raw }
                } else {
                    win_for_scroll.scroll_y().unwrap_or(0.0)
                };

                // Update the virtualizer.
                virt_for_scroll
                    .borrow_mut()
                    .update_scroll_offset(scroll_pos, true);

                // Trigger re-render.
                let current = *trigger_for_scroll;
                trigger_for_scroll.set(current.wrapping_add(1));
            }) as Box<dyn FnMut()>);

            // Build passive event listener options.
            let listener_opts = web_sys::AddEventListenerOptions::new();
            listener_opts.set_passive(true);

            // Attach the scroll listener to the window with passive option.
            let _ = win.add_event_listener_with_callback_and_add_event_listener_options(
                "scroll",
                on_scroll.as_ref().unchecked_ref(),
                &listener_opts,
            );
            *scroll_closure.borrow_mut() = Some(on_scroll);

            // Set up resize event handler.
            let virt_for_resize = Rc::clone(&scroll_virtualizer);
            let trigger_for_resize = scroll_trigger.clone();
            let win_for_resize = win.clone();

            let on_resize = Closure::wrap(Box::new(move || {
                // Read the new window dimension based on direction.
                let new_size = if is_horizontal {
                    win_for_resize
                        .inner_width()
                        .ok()
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0)
                } else {
                    win_for_resize
                        .inner_height()
                        .ok()
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0)
                };

                // Update the virtualizer container size.
                virt_for_resize.borrow_mut().update_container_size(new_size);

                // Trigger re-render.
                let current = *trigger_for_resize;
                trigger_for_resize.set(current.wrapping_add(1));
            }) as Box<dyn FnMut()>);

            // Attach the resize listener to the window with passive option.
            let _ = win.add_event_listener_with_callback_and_add_event_listener_options(
                "resize",
                on_resize.as_ref().unchecked_ref(),
                &listener_opts,
            );
            *resize_closure.borrow_mut() = Some(on_resize);

            // Store the window for cleanup.
            *cleanup_window.borrow_mut() = Some(win);
        }

        // Single cleanup closure that handles both cases uniformly.
        let cleanup_scroll = scroll_closure;
        let cleanup_resize = resize_closure;
        let cleanup_win = cleanup_window;
        let binding_on_cleanup = Rc::clone(&binding_for_effect);

        move || {
            // Drop window binding before removing listeners.
            *binding_on_cleanup.borrow_mut() = ScrollBinding::None;

            if let Some(win) = cleanup_win.borrow().as_ref() {
                // Remove the scroll listener.
                if let Some(closure) = cleanup_scroll.borrow().as_ref() {
                    let _ = win.remove_event_listener_with_callback(
                        "scroll",
                        closure.as_ref().unchecked_ref(),
                    );
                }

                // Remove the resize listener.
                if let Some(closure) = cleanup_resize.borrow().as_ref() {
                    let _ = win.remove_event_listener_with_callback(
                        "resize",
                        closure.as_ref().unchecked_ref(),
                    );
                }
            }
        }
    });

    // Build and return the handle.
    VirtualizerHandle::new(
        Rc::clone(&virtualizer),
        render_trigger,
        Rc::clone(&scroll_binding_rc),
        Rc::clone(&item_registry_rc),
    )
}
