use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use js_sys::Array;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{Element, ResizeObserver, ResizeObserverEntry};

use crate::core::scroll_direction::ScrollDirection;
use crate::core::virtualizer::Virtualizer;
use crate::core::virtualizer_options::VirtualizerOptions;

/// Owns per-index `ResizeObserver` instances for dynamic item sizing.
///
/// Closures are stored in a side vector so they are not dropped while observers
/// are active. Used by [`VirtualizerHandle::observe_item_element`](crate::hooks::virtualizer_handle::VirtualizerHandle::observe_item_element).
pub struct ItemMeasurementRegistry {
    /// Active observers keyed by item index.
    observers: HashMap<usize, ResizeObserver>,

    /// Keeps `Closure` values alive for the lifetime of the registry.
    closures: Vec<Closure<dyn FnMut(Array, ResizeObserver)>>,
}

impl ItemMeasurementRegistry {
    /// Creates an empty registry with no observers.
    pub fn new() -> Self {
        Self {
            observers: HashMap::new(),
            closures: Vec::new(),
        }
    }

    /// Registers or replaces a `ResizeObserver` for one item index.
    ///
    /// On resize, computes size via [`VirtualizerOptions::measure_element`] or the
    /// default content-box dimension along [`ScrollDirection`], then calls
    /// [`Virtualizer::measure_item`]. Respects [`VirtualizerOptions::measure_during_scroll`].
    ///
    /// # Parameters
    ///
    /// - `index`: Dataset index for this element.
    /// - `element`: DOM node to observe.
    /// - `virtualizer`: Shared engine updated on resize.
    /// - `options`: Snapshot of options (cloned for the closure).
    /// - `trigger_render`: Invoked when a measurement changes layout.
    ///
    /// # Returns
    ///
    /// - `Ok(())` when observation started.
    /// - `Err(String)` when `ResizeObserver` construction fails.
    pub fn observe_item(
        &mut self,
        index: usize,
        element: &Element,
        virtualizer: Rc<RefCell<Virtualizer>>,
        options: VirtualizerOptions,
        trigger_render: Rc<dyn Fn()>,
    ) -> Result<(), String> {
        // Tear down any previous observer for the same index.
        if let Some(old) = self.observers.remove(&index) {
            old.disconnect();
        }

        let virt = Rc::clone(&virtualizer);
        let opts = options.clone();
        let trigger = Rc::clone(&trigger_render);

        let closure = Closure::wrap(Box::new(move |entries: Array, _obs: ResizeObserver| {
            // Optionally skip work while the user is actively scrolling.
            if !opts.measure_during_scroll && virt.borrow().is_scrolling() {
                return;
            }

            // Keep the first entry alive across content_rect access.
            let first = entries.get(0);
            let Some(entry) = first.dyn_ref::<ResizeObserverEntry>() else {
                return;
            };

            let rect = entry.content_rect();
            let w = rect.width();
            let h = rect.height();

            // Resolve size along the scroll axis (or custom measurer).
            let size = if let Some(ref f) = opts.measure_element {
                f(w, h, opts.scroll_direction)
            } else {
                match opts.scroll_direction {
                    ScrollDirection::Vertical => h,
                    ScrollDirection::Horizontal => w,
                }
            };

            let mut v = virt.borrow_mut();
            let outcome = v.measure_item(index, size);
            drop(v);

            if let Ok(o) = outcome {
                if o.layout_changed {
                    trigger();
                }
            }
        }) as Box<dyn FnMut(Array, ResizeObserver)>);

        let observer = ResizeObserver::new(closure.as_ref().unchecked_ref())
            .map_err(|_| "ResizeObserver::new failed".to_string())?;

        observer.observe(element);

        self.observers.insert(index, observer);
        self.closures.push(closure);

        Ok(())
    }

    /// Disconnects and removes the observer for `index`, if any.
    ///
    /// # Parameters
    ///
    /// - `index`: Item index to stop observing.
    pub fn unobserve_item(&mut self, index: usize) {
        if let Some(old) = self.observers.remove(&index) {
            old.disconnect();
        }
    }

    /// Disconnects every observer and drops all closures.
    pub fn disconnect_all(&mut self) {
        for (_, o) in self.observers.drain() {
            o.disconnect();
        }
        self.closures.clear();
    }
}

impl Default for ItemMeasurementRegistry {
    fn default() -> Self {
        Self::new()
    }
}
