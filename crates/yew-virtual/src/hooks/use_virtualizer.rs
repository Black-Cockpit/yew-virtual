use std::cell::RefCell;
use std::rc::Rc;

use js_sys;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::Element;
use yew::prelude::*;

use crate::core::scroll_direction::ScrollDirection;
use crate::core::virtualizer::Virtualizer;
use crate::core::virtualizer_options::VirtualizerOptions;
use crate::hooks::item_measurement_registry::ItemMeasurementRegistry;
use crate::hooks::scroll_binding::ScrollBinding;
use crate::hooks::virtualizer_handle::VirtualizerHandle;

/// Yew hook that creates and manages a container-based virtualizer.
///
/// Binds the virtualizer to a scroll container element, automatically
/// handling scroll events, resize observations, and triggering re-renders
/// when the visible range changes. Supports both vertical and horizontal
/// scroll directions and RTL horizontal scroll offsets.
///
/// The returned [`VirtualizerHandle`] shares stable `Rc` registries: the hook
/// sets [`ScrollBinding::Element`] when the container ref resolves so DOM
/// scroll helpers and measurement compensation work correctly.
///
/// # Parameters
///
/// - `options`: Configuration options for the virtualizer.
///
/// # Returns
///
/// - `(VirtualizerHandle, NodeRef)`: The handle for reading state and the
///   `NodeRef` to attach to the scroll container element.
#[hook]
pub fn use_virtualizer(options: VirtualizerOptions) -> (VirtualizerHandle, NodeRef) {
    // Create a NodeRef for the scroll container element.
    let container_ref = use_node_ref();

    // Create a render trigger state to force re-renders.
    let render_trigger = use_state(|| 0u64);

    // Stable binding updated when the container mounts (for programmatic scroll).
    let scroll_binding_state = use_state(|| Rc::new(RefCell::new(ScrollBinding::None)));
    let scroll_binding_rc = (*scroll_binding_state).clone();

    // Stable registry for optional per-item ResizeObserver usage via the handle.
    let item_registry_state = use_state(|| Rc::new(RefCell::new(ItemMeasurementRegistry::new())));
    let item_registry_rc = (*item_registry_state).clone();

    // Capture scroll direction for use in closures.
    let is_horizontal = options.scroll_direction == ScrollDirection::Horizontal;
    let is_rtl = options.is_rtl;

    // Create or update the virtualizer instance when options identity changes.
    let virtualizer = use_memo(options.clone(), |opts| {
        // Attempt to create the virtualizer with the provided options.
        let virt = match Virtualizer::new(opts.clone()) {
            Ok(v) => v,
            Err(_e) => {
                // Fall back to an empty virtualizer on failure.
                Virtualizer::empty()
            }
        };
        Rc::new(RefCell::new(virt))
    });

    // Set up scroll event listener and ResizeObserver on the container.
    let scroll_virtualizer = Rc::clone(&virtualizer);
    let scroll_trigger = render_trigger.clone();
    let scroll_container_ref = container_ref.clone();
    let binding_for_effect = Rc::clone(&scroll_binding_rc);

    use_effect_with(
        (scroll_container_ref.clone(), options.clone()),
        move |(_container_ref, _opts)| {
            // Get the scroll container element.
            let container = _container_ref.cast::<Element>();

            // Store cleanup closures (shared across setup and teardown).
            let scroll_closure: Rc<RefCell<Option<Closure<dyn FnMut()>>>> =
                Rc::new(RefCell::new(None));
            let resize_observer: Rc<RefCell<Option<web_sys::ResizeObserver>>> =
                Rc::new(RefCell::new(None));
            let resize_closure: Rc<
                RefCell<Option<Closure<dyn FnMut(js_sys::Array, web_sys::ResizeObserver)>>>,
            > = Rc::new(RefCell::new(None));
            let cleanup_element: Rc<RefCell<Option<Element>>> = Rc::new(RefCell::new(None));

            if let Some(el) = container {
                // Publish the element to the handle for DOM scroll APIs.
                *binding_for_effect.borrow_mut() = ScrollBinding::Element {
                    element: el.clone(),
                };

                // Read the initial container size based on scroll direction.
                let container_size = if is_horizontal {
                    el.client_width() as f64
                } else {
                    el.client_height() as f64
                };

                // Push the viewport size into the engine.
                scroll_virtualizer
                    .borrow_mut()
                    .update_container_size(container_size);

                // Increment render trigger to reflect initial state.
                let current = *scroll_trigger;
                scroll_trigger.set(current.wrapping_add(1));

                // Set up scroll event handler.
                let virt_for_scroll = Rc::clone(&scroll_virtualizer);
                let trigger_for_scroll = scroll_trigger.clone();
                let el_for_scroll = el.clone();

                let closure = Closure::wrap(Box::new(move || {
                    // Read the current scroll position based on direction.
                    let scroll_pos = if is_horizontal {
                        let raw = el_for_scroll.scroll_left() as f64;
                        if is_rtl {
                            -raw
                        } else {
                            raw
                        }
                    } else {
                        el_for_scroll.scroll_top() as f64
                    };

                    // Update the virtualizer with the new scroll offset.
                    virt_for_scroll
                        .borrow_mut()
                        .update_scroll_offset(scroll_pos, true);

                    // Trigger a re-render.
                    let current = *trigger_for_scroll;
                    trigger_for_scroll.set(current.wrapping_add(1));
                }) as Box<dyn FnMut()>);

                // Build passive event listener options for smoother scrolling.
                let listener_opts = web_sys::AddEventListenerOptions::new();
                listener_opts.set_passive(true);

                // Attach the scroll listener with passive option.
                let _ = el.add_event_listener_with_callback_and_add_event_listener_options(
                    "scroll",
                    closure.as_ref().unchecked_ref(),
                    &listener_opts,
                );

                // Store the closure to prevent it from being dropped.
                *scroll_closure.borrow_mut() = Some(closure);

                // Set up ResizeObserver for container size changes.
                let virt_for_resize = Rc::clone(&scroll_virtualizer);
                let trigger_for_resize = scroll_trigger.clone();

                let resize_cb = Closure::wrap(Box::new(
                    move |entries: js_sys::Array, _observer: web_sys::ResizeObserver| {
                        // Process the first resize entry.
                        if let Some(entry) =
                            entries.get(0).dyn_ref::<web_sys::ResizeObserverEntry>()
                        {
                            let rect: web_sys::DomRectReadOnly = entry.content_rect();

                            // Read the appropriate dimension based on scroll direction.
                            let new_size = if is_horizontal {
                                rect.width()
                            } else {
                                rect.height()
                            };

                            // Update the virtualizer container size.
                            virt_for_resize.borrow_mut().update_container_size(new_size);

                            // Trigger a re-render.
                            let current = *trigger_for_resize;
                            trigger_for_resize.set(current.wrapping_add(1));
                        }
                    },
                )
                    as Box<dyn FnMut(js_sys::Array, web_sys::ResizeObserver)>);

                // Create and start the ResizeObserver.
                if let Ok(observer) =
                    web_sys::ResizeObserver::new(resize_cb.as_ref().unchecked_ref())
                {
                    observer.observe(&el);
                    *resize_observer.borrow_mut() = Some(observer);
                }

                // Store the resize callback to prevent it from being dropped.
                *resize_closure.borrow_mut() = Some(resize_cb);

                // Store the element for cleanup.
                *cleanup_element.borrow_mut() = Some(el);
            }

            // Single cleanup closure that handles all cases uniformly.
            let cleanup_scroll = scroll_closure;
            let cleanup_resize = resize_observer;
            let cleanup_resize_cb = resize_closure;
            let cleanup_el = cleanup_element;
            let binding_on_cleanup = Rc::clone(&binding_for_effect);

            move || {
                // Clear the published scroll target so handles do not reference stale DOM.
                *binding_on_cleanup.borrow_mut() = ScrollBinding::None;

                // Remove the scroll listener if element and closure exist.
                if let (Some(el), Some(closure)) = (
                    cleanup_el.borrow().as_ref(),
                    cleanup_scroll.borrow().as_ref(),
                ) {
                    let _ = el.remove_event_listener_with_callback(
                        "scroll",
                        closure.as_ref().unchecked_ref(),
                    );
                }

                // Disconnect the resize observer.
                if let Some(observer) = cleanup_resize.borrow().as_ref() {
                    observer.disconnect();
                }

                // Drop the resize callback explicitly.
                drop(cleanup_resize_cb);
            }
        },
    );

    // Build the handle for the component to use.
    let handle = VirtualizerHandle::new(
        Rc::clone(&virtualizer),
        render_trigger,
        Rc::clone(&scroll_binding_rc),
        Rc::clone(&item_registry_rc),
    );

    (handle, container_ref)
}
