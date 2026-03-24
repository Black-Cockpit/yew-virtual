use std::cell::RefCell;
use std::rc::Rc;

use yew::prelude::UseStateHandle;

use crate::core::measure_item_outcome::MeasureItemOutcome;
use crate::core::scroll_alignment::ScrollAlignment;
use crate::core::scroll_behavior::ScrollBehavior;
use crate::core::scroll_direction::ScrollDirection;
use crate::core::scroll_reconcile_action::ScrollReconcileAction;
use crate::core::scroll_to_options::ScrollToOptions;
use crate::core::virtual_item::VirtualItem;
use crate::core::virtualizer::Virtualizer;
use crate::core::virtualizer_error::VirtualizerError;
use crate::core::virtualizer_options::VirtualizerOptions;
use crate::hooks::item_measurement_registry::ItemMeasurementRegistry;
use crate::hooks::scroll_application::{
    apply_scroll_compensation, apply_scroll_offset, read_element_scroll, read_window_scroll,
};
use crate::hooks::scroll_binding::ScrollBinding;

/// Handle returned by virtualizer hooks for reading state and triggering actions.
///
/// Bridges the headless [`Virtualizer`](crate::core::virtualizer::Virtualizer) to the browser:
/// it applies scroll compensation and programmatic `scrollTo` calls, registers item
/// resize observers, and bumps Yew render state when layout or range changes.
///
/// Constructed only by [`use_virtualizer`](crate::hooks::use_virtualizer::use_virtualizer) or
/// [`use_window_virtualizer`](crate::hooks::use_window_virtualizer::use_window_virtualizer);
/// hooks inject the shared `scroll_binding` and `item_measurement` registries.
#[derive(Clone)]
pub struct VirtualizerHandle {
    /// Shared reference to the virtualizer engine.
    virtualizer: Rc<RefCell<Virtualizer>>,

    /// State handle used to trigger Yew re-renders.
    render_trigger: UseStateHandle<u64>,

    /// DOM scroll target (element or window) updated by the hook lifecycle.
    scroll_binding: Rc<RefCell<ScrollBinding>>,

    /// Per-item ResizeObserver registry; keeps closures alive for WASM.
    item_measurement: Rc<RefCell<ItemMeasurementRegistry>>,
}

impl VirtualizerHandle {
    /// Creates a new virtualizer handle.
    ///
    /// Hooks pass stable `Rc` handles so effects can update `scroll_binding` when the
    /// container mounts and so item observers share the same registry instance.
    ///
    /// # Parameters
    ///
    /// - `virtualizer`: Shared engine instance (`Rc<RefCell<Virtualizer>>`).
    /// - `render_trigger`: Yew counter incremented to force a re-render.
    /// - `scroll_binding`: Shared binding set to [`ScrollBinding::Element`] or [`ScrollBinding::Window`].
    /// - `item_measurement`: Shared registry for [`observe_item_element`](Self::observe_item_element).
    pub fn new(
        virtualizer: Rc<RefCell<Virtualizer>>,
        render_trigger: UseStateHandle<u64>,
        scroll_binding: Rc<RefCell<ScrollBinding>>,
        item_measurement: Rc<RefCell<ItemMeasurementRegistry>>,
    ) -> Self {
        // Store all shared handles for the component lifetime.
        Self {
            virtualizer,
            render_trigger,
            scroll_binding,
            item_measurement,
        }
    }

    /// Returns metadata for items that should be rendered for the current viewport.
    ///
    /// Delegates to the core engine; respects range extractors, overscan, and `enabled`.
    ///
    /// # Returns
    ///
    /// - `Vec<VirtualItem>`: Cloned layout entries for visible (plus overscan) indices.
    pub fn get_virtual_items(&self) -> Vec<VirtualItem> {
        // Borrow the engine immutably and clone the visible slice.
        let virt = self.virtualizer.borrow();
        virt.get_virtual_items()
    }

    /// Returns the total scrollable size of the virtualized content in pixels.
    ///
    /// # Returns
    ///
    /// - `f64`: Sum of item geometry, padding, gap, and scroll margin semantics from options.
    pub fn total_size(&self) -> f64 {
        // Read the precomputed total from the engine.
        let virt = self.virtualizer.borrow();
        virt.total_size()
    }

    /// Returns the current logical scroll offset along the virtualizer axis.
    ///
    /// # Returns
    ///
    /// - `f64`: Last offset passed to `update_scroll_offset` on the core instance.
    pub fn scroll_offset(&self) -> f64 {
        // Delegate to the core scroll offset field.
        let virt = self.virtualizer.borrow();
        virt.scroll_offset()
    }

    /// Returns the configured number of items in the dataset.
    ///
    /// # Returns
    ///
    /// - `usize`: Current `item_count` from options.
    pub fn item_count(&self) -> usize {
        // Read count from the live options snapshot.
        let virt = self.virtualizer.borrow();
        virt.item_count()
    }

    /// Returns the first visible item index (inclusive).
    ///
    /// # Returns
    ///
    /// - `usize`: Range start from the last range calculation.
    pub fn range_start(&self) -> usize {
        // Forward to the core visible range.
        let virt = self.virtualizer.borrow();
        virt.range_start()
    }

    /// Returns the last visible item index (inclusive).
    ///
    /// # Returns
    ///
    /// - `usize`: Range end from the last range calculation.
    pub fn range_end(&self) -> usize {
        // Forward to the core visible range.
        let virt = self.virtualizer.borrow();
        virt.range_end()
    }

    /// Returns whether user-driven scrolling is considered active.
    ///
    /// # Returns
    ///
    /// - `bool`: Mirrors the core `is_scrolling` flag (hooks may debounce this).
    pub fn is_scrolling(&self) -> bool {
        // Read scrolling state from the engine.
        let virt = self.virtualizer.borrow();
        virt.is_scrolling()
    }

    /// Returns whether the last scroll delta moved forward along the axis.
    ///
    /// # Returns
    ///
    /// - `Option<bool>`: `Some(true)` forward, `Some(false)` backward, `None` if idle or unknown.
    pub fn is_scroll_forward(&self) -> Option<bool> {
        // Delegate to core scroll direction bookkeeping.
        let virt = self.virtualizer.borrow();
        virt.is_scroll_forward()
    }

    /// Returns accumulated scroll adjustment from resizes above the viewport.
    ///
    /// # Returns
    ///
    /// - `f64`: Value maintained by the core during measurement updates.
    pub fn scroll_adjustments(&self) -> f64 {
        // Read the adjustment accumulator from the engine.
        let virt = self.virtualizer.borrow();
        virt.scroll_adjustments()
    }

    /// Returns the viewport size along the scroll axis used for range math.
    ///
    /// # Returns
    ///
    /// - `f64`: Container or window viewport size in pixels.
    pub fn container_size(&self) -> f64 {
        // Return the core container dimension.
        let virt = self.virtualizer.borrow();
        virt.container_size()
    }

    /// Records a measured size for one item and updates layout.
    ///
    /// When the core reports [`MeasureItemOutcome::scroll_compensation`], applies an instant
    /// `scrollTo` delta on the bound element or window so content does not jump when items
    /// above the viewport resize.
    ///
    /// # Parameters
    ///
    /// - `index`: Item index to update.
    /// - `size`: Measured size in pixels along the scroll axis.
    ///
    /// # Returns
    ///
    /// - `Ok(MeasureItemOutcome)`: Layout change and compensation metadata.
    /// - `Err(VirtualizerError)`: Invalid index or invalid measurement.
    ///
    /// # Errors
    ///
    /// - `IndexOutOfBounds` when `index` is not in range.
    /// - `MeasurementError` when `size` is not acceptable to the cache.
    pub fn measure_item(
        &self,
        index: usize,
        size: f64,
    ) -> Result<MeasureItemOutcome, VirtualizerError> {
        // Apply the measurement on the core engine first.
        let outcome = self.virtualizer.borrow_mut().measure_item(index, size)?;

        // Compensate the DOM scroll position when the engine requests it.
        if outcome.scroll_compensation.abs() > f64::EPSILON {
            // Read axis and RTL from options for the scroll helper.
            let virt = self.virtualizer.borrow();
            let horizontal = virt.options().scroll_direction == ScrollDirection::Horizontal;
            let is_rtl = virt.options().is_rtl;
            drop(virt);

            // Apply instant scroll delta on the bound target (best-effort).
            let binding = self.scroll_binding.borrow();
            let _ = apply_scroll_compensation(
                &binding,
                outcome.scroll_compensation,
                horizontal,
                is_rtl,
            );
        }

        // Schedule a Yew render when layout actually changed.
        if outcome.layout_changed {
            self.trigger_render();
        }

        Ok(outcome)
    }

    /// Attaches a `ResizeObserver` to an item element for dynamic sizing.
    ///
    /// Uses [`VirtualizerOptions::measure_element`] when set; otherwise reads
    /// [`ResizeObserverEntry::content_rect`](web_sys::ResizeObserverEntry::content_rect)
    /// width or height based on [`ScrollDirection`].
    ///
    /// # Parameters
    ///
    /// - `index`: Item index this element represents.
    /// - `element`: DOM node to observe (typically the row/cell root).
    ///
    /// # Returns
    ///
    /// - `Ok(())` when observation was registered.
    /// - `Err(VirtualizerError)` when the observer could not be created.
    ///
    /// # Errors
    ///
    /// - `MeasurementError` when `ResizeObserver::new` fails or the registry rejects the call.
    pub fn observe_item_element(
        &self,
        index: usize,
        element: &web_sys::Element,
    ) -> Result<(), VirtualizerError> {
        // Snapshot options so the observer closure sees a consistent configuration.
        let opts = self.virtualizer.borrow().options().clone();

        // Share the engine with the registry callback.
        let virt = Rc::clone(&self.virtualizer);

        // Build a render callback that increments the Yew counter.
        let rt = self.render_trigger.clone();
        let tr: Rc<dyn Fn()> = Rc::new(move || {
            let current = *rt;
            rt.set(current.wrapping_add(1));
        });

        // Register or replace the observer for this index.
        self.item_measurement
            .borrow_mut()
            .observe_item(index, element, virt, opts, tr)
            .map_err(VirtualizerError::MeasurementError)?;

        Ok(())
    }

    /// Disconnects any `ResizeObserver` previously registered for `index`.
    ///
    /// # Parameters
    ///
    /// - `index`: Item index whose observation should stop.
    pub fn unobserve_item_element(&self, index: usize) {
        // Remove and disconnect the observer if present.
        self.item_measurement.borrow_mut().unobserve_item(index);
    }

    /// Sets a new total item count and rebuilds measurements.
    ///
    /// # Parameters
    ///
    /// - `count`: New dataset length.
    pub fn update_item_count(&self, count: usize) {
        // Mutate the engine and bump the render generation.
        self.virtualizer.borrow_mut().update_item_count(count);
        self.trigger_render();
    }

    /// Replaces options in place while preserving measurement state where possible.
    ///
    /// # Parameters
    ///
    /// - `options`: Full new option set (same shape as construction).
    ///
    /// # Returns
    ///
    /// - `Ok(())` when validation succeeds.
    /// - `Err(VirtualizerError)` when the new options are invalid.
    ///
    /// # Errors
    ///
    /// - Same validation failures as [`Virtualizer::new`](crate::core::virtualizer::Virtualizer::new).
    pub fn set_options(&self, options: VirtualizerOptions) -> Result<(), VirtualizerError> {
        // Apply validated options on the core instance.
        self.virtualizer.borrow_mut().set_options(options)?;
        self.trigger_render();
        Ok(())
    }

    /// Computes the target scroll offset to bring an item into view (no DOM I/O).
    ///
    /// # Parameters
    ///
    /// - `index`: Target item index.
    /// - `alignment`: How to align the item in the viewport.
    ///
    /// # Returns
    ///
    /// - `Ok(f64)` with the logical offset to scroll to.
    /// - `Err(VirtualizerError)` when the index is invalid.
    ///
    /// # Errors
    ///
    /// - `IndexOutOfBounds` when `index >= item_count`.
    pub fn scroll_to_index(
        &self,
        index: usize,
        alignment: ScrollAlignment,
    ) -> Result<f64, VirtualizerError> {
        // Pure calculation on the headless engine.
        let virt = self.virtualizer.borrow();
        virt.scroll_to_index(index, alignment)
    }

    /// Scrolls the bound target so the given item aligns per `options`.
    ///
    /// Prepares internal [`ScrollState`](crate::core::scroll_state::ScrollState) with
    /// [`js_sys::Date::now`] and invokes `scrollTo` (or [`VirtualizerOptions::scroll_to_fn`]).
    ///
    /// # Parameters
    ///
    /// - `index`: Item index to scroll into view.
    /// - `options`: Alignment and [`ScrollBehavior`].
    ///
    /// # Returns
    ///
    /// - `Ok(())` when scroll was applied.
    /// - `Err(VirtualizerError)` when the index is invalid or the DOM target is missing.
    ///
    /// # Errors
    ///
    /// - `IndexOutOfBounds` for bad indices.
    /// - `ScrollContainerUnavailable` when no scroll target is bound or scroll fails.
    pub fn scroll_to_index_dom(
        &self,
        index: usize,
        options: ScrollToOptions,
    ) -> Result<(), VirtualizerError> {
        // Capture a timestamp for reconciliation timeouts on the core state.
        let now = js_sys::Date::now();

        // Prepare programmatic scroll state and read the resolved offset.
        let target = {
            let mut v = self.virtualizer.borrow_mut();
            let st = v.prepare_scroll_to_index(index, options.clone(), now)?;
            st.last_target_offset
        };

        // Apply the scroll on the element or window.
        self.apply_scroll_offset_to_dom(options.behavior, target, options.clone())?;

        // Ensure the component re-reads handle getters.
        self.trigger_render();

        Ok(())
    }

    /// Scrolls the bound target to a logical pixel offset.
    ///
    /// # Parameters
    ///
    /// - `to_offset`: Desired scroll position along the virtualizer axis.
    /// - `options`: Alignment (for clamping math) and [`ScrollBehavior`].
    ///
    /// # Returns
    ///
    /// - `Ok(())` on success.
    /// - `Err(VirtualizerError)` when the scroll target is unavailable.
    ///
    /// # Errors
    ///
    /// - `ScrollContainerUnavailable` when binding is [`ScrollBinding::None`] or DOM calls fail.
    pub fn scroll_to_offset_dom(
        &self,
        to_offset: f64,
        options: ScrollToOptions,
    ) -> Result<(), VirtualizerError> {
        // Record the current time for smooth-scroll reconciliation bookkeeping.
        let now = js_sys::Date::now();

        // Store scroll state and obtain the clamped target offset.
        let target = {
            let mut v = self.virtualizer.borrow_mut();
            let st = v.prepare_scroll_to_offset(to_offset, options.clone(), now);
            st.last_target_offset
        };

        // Perform the browser scroll operation.
        self.apply_scroll_offset_to_dom(options.behavior, target, options.clone())?;
        self.trigger_render();

        Ok(())
    }

    /// Scrolls by a relative delta from the current logical offset.
    ///
    /// # Parameters
    ///
    /// - `delta`: Pixels to add to the current scroll offset (positive moves content forward).
    /// - `behavior`: Animation style for `scrollTo`.
    ///
    /// # Returns
    ///
    /// - `Ok(())` when the scroll command was issued.
    /// - `Err(VirtualizerError)` when the scroll target is missing.
    ///
    /// # Errors
    ///
    /// - `ScrollContainerUnavailable` when no scroll target is bound.
    pub fn scroll_by_dom(
        &self,
        delta: f64,
        behavior: ScrollBehavior,
    ) -> Result<(), VirtualizerError> {
        // Timestamp the operation for reconciliation.
        let now = js_sys::Date::now();

        // Compute the new target from the current engine offset plus delta.
        let target = {
            let mut v = self.virtualizer.borrow_mut();
            let st = v.prepare_scroll_by(delta, behavior, now);
            st.last_target_offset
        };

        // Build minimal options for the custom scroll hook path.
        let opts = ScrollToOptions {
            align: ScrollAlignment::Start,
            behavior,
        };

        self.apply_scroll_offset_to_dom(behavior, target, opts)?;
        self.trigger_render();

        Ok(())
    }

    /// Returns whether a smooth programmatic scroll is still being reconciled.
    ///
    /// # Returns
    ///
    /// - `true` when active [`ScrollState`](crate::core::scroll_state::ScrollState) uses [`ScrollBehavior::Smooth`].
    pub fn needs_scroll_reconciliation(&self) -> bool {
        // Inspect core scroll state for smooth behavior.
        self.virtualizer
            .borrow()
            .scroll_state()
            .map(|s| s.behavior == ScrollBehavior::Smooth)
            .unwrap_or(false)
    }

    /// Performs one frame of smooth-scroll reconciliation (call from `requestAnimationFrame`).
    ///
    /// Refreshes the target offset from layout, nudges the DOM with instant scroll,
    /// then runs [`Virtualizer::scroll_reconciliation_tick`](crate::core::virtualizer::Virtualizer::scroll_reconciliation_tick).
    ///
    /// # Parameters
    ///
    /// - `now_ms`: Current time in milliseconds (same epoch as [`js_sys::Date::now`]).
    ///
    /// # Returns
    ///
    /// - `ScrollReconcileAction::Continue` while the animation should keep scheduling frames.
    /// - `Done` or `Timeout` when reconciliation should stop.
    pub fn reconciliation_step(&self, now_ms: f64) -> ScrollReconcileAction {
        // Recompute target offset if scrolling to a dynamic index.
        {
            let mut v = self.virtualizer.borrow_mut();
            v.refresh_programmatic_scroll_target();
        }

        // Read the latest desired offset from programmatic state.
        let target = self
            .virtualizer
            .borrow()
            .scroll_state()
            .map(|s| s.last_target_offset);

        // Snap the DOM to the refreshed target using instant behavior.
        if let Some(t) = target {
            let _ = self.apply_scroll_offset_to_dom(
                ScrollBehavior::Instant,
                t,
                ScrollToOptions {
                    align: ScrollAlignment::Start,
                    behavior: ScrollBehavior::Instant,
                },
            );
        }

        // Observe the actual scroll position after the nudge.
        let current = self.read_dom_scroll_offset().unwrap_or(0.0);

        // Advance reconciliation state machine on the core.
        let action = self
            .virtualizer
            .borrow_mut()
            .scroll_reconciliation_tick(current, now_ms);

        // Re-render when reconciliation completes or times out.
        if action != ScrollReconcileAction::Continue {
            self.trigger_render();
        }

        action
    }

    /// Reads the logical scroll offset from the bound DOM target.
    ///
    /// # Returns
    ///
    /// - `Ok(f64)` with the offset along the virtualizer axis (RTL-aware for horizontal).
    /// - `Err(VirtualizerError)` when nothing is bound or the window is missing.
    ///
    /// # Errors
    ///
    /// - `ScrollContainerUnavailable` for [`ScrollBinding::None`] or failed window access.
    pub fn read_dom_scroll_offset(&self) -> Result<f64, VirtualizerError> {
        // Resolve axis and RTL from live options.
        let virt = self.virtualizer.borrow();
        let horizontal = virt.options().scroll_direction == ScrollDirection::Horizontal;
        let is_rtl = virt.options().is_rtl;
        drop(virt);

        // Dispatch to element or window readers.
        let binding = self.scroll_binding.borrow();
        match &*binding {
            ScrollBinding::None => Err(VirtualizerError::ScrollContainerUnavailable(
                "scroll target not bound".to_string(),
            )),
            ScrollBinding::Element { element } => read_element_scroll(element, horizontal, is_rtl),
            ScrollBinding::Window => read_window_scroll(horizontal, is_rtl),
        }
    }

    /// Prepares scroll-to-index state and returns only the target offset.
    ///
    /// Equivalent to the headless [`Virtualizer::prepare_scroll_to_index`](crate::core::virtualizer::Virtualizer::prepare_scroll_to_index)
    /// with `now_ms` from [`js_sys::Date::now`]. Does not scroll the DOM.
    ///
    /// # Parameters
    ///
    /// - `index`: Target item index.
    /// - `options`: Scroll options including behavior for state tracking.
    ///
    /// # Returns
    ///
    /// - `Ok(f64)` with `last_target_offset` from the prepared state.
    /// - `Err(VirtualizerError)` when the index is invalid.
    ///
    /// # Errors
    ///
    /// - `IndexOutOfBounds` when `index` is out of range.
    pub fn prepare_scroll_to_index(
        &self,
        index: usize,
        options: ScrollToOptions,
    ) -> Result<f64, VirtualizerError> {
        // Use wall-clock time for reconciliation metadata.
        let now = js_sys::Date::now();

        // Store state on the engine and return the numeric target.
        let state = self
            .virtualizer
            .borrow_mut()
            .prepare_scroll_to_index(index, options, now)?;

        Ok(state.last_target_offset)
    }

    /// Prepares scroll-to-offset and returns the clamped target (no DOM scroll).
    ///
    /// # Parameters
    ///
    /// - `to_offset`: Raw desired offset.
    /// - `options`: Alignment and behavior for the prepared state.
    ///
    /// # Returns
    ///
    /// - `f64`: `last_target_offset` after alignment and clamping.
    pub fn prepare_scroll_to_offset(&self, to_offset: f64, options: ScrollToOptions) -> f64 {
        // Timestamp for programmatic scroll lifecycle.
        let now = js_sys::Date::now();

        // Mutate core state and read the stored target.
        self.virtualizer
            .borrow_mut()
            .prepare_scroll_to_offset(to_offset, options, now)
            .last_target_offset
    }

    /// Prepares a relative scroll and returns the clamped target (no DOM scroll).
    ///
    /// # Parameters
    ///
    /// - `delta`: Pixels to add to the current offset.
    /// - `behavior`: Stored on the prepared scroll state.
    ///
    /// # Returns
    ///
    /// - `f64`: Resulting `last_target_offset`.
    pub fn prepare_scroll_by(&self, delta: f64, behavior: ScrollBehavior) -> f64 {
        // Wall-clock start time for reconciliation.
        let now = js_sys::Date::now();

        self.virtualizer
            .borrow_mut()
            .prepare_scroll_by(delta, behavior, now)
            .last_target_offset
    }

    /// Clears dynamic size cache and rebuilds all measurements from estimates.
    pub fn measure(&self) {
        // Force a full remeasure pass on the core.
        self.virtualizer.borrow_mut().measure();
        self.trigger_render();
    }

    /// Returns the item whose span contains the given scroll offset.
    ///
    /// # Parameters
    ///
    /// - `offset`: Logical scroll position to probe.
    ///
    /// # Returns
    ///
    /// - `Some(VirtualItem)` when a matching measurement exists.
    /// - `None` when the list is empty or offset is outside known layout.
    pub fn get_virtual_item_for_offset(&self, offset: f64) -> Option<VirtualItem> {
        // Clone the hit item from the measurement array.
        let virt = self.virtualizer.borrow();
        virt.get_virtual_item_for_offset(offset).cloned()
    }

    /// Returns whether the current sizing mode expects runtime measurement.
    ///
    /// # Returns
    ///
    /// - `bool`: Delegates to [`ItemSizeMode::requires_measurement`](crate::core::item_size_mode::ItemSizeMode::requires_measurement).
    pub fn requires_measurement(&self) -> bool {
        let virt = self.virtualizer.borrow();
        virt.requires_measurement()
    }

    /// Returns whether the virtualizer is currently enabled.
    ///
    /// When disabled, the engine yields no virtual items.
    ///
    /// # Returns
    ///
    /// - `bool`: `VirtualizerOptions::enabled` snapshot.
    pub fn is_enabled(&self) -> bool {
        let virt = self.virtualizer.borrow();
        virt.is_enabled()
    }

    /// Returns the shared `Rc` wrapping the core engine.
    ///
    /// # Returns
    ///
    /// - `Rc<RefCell<Virtualizer>>`: Same instance the hook mutates from effects.
    pub fn virtualizer_ref(&self) -> Rc<RefCell<Virtualizer>> {
        Rc::clone(&self.virtualizer)
    }

    /// Returns the shared scroll binding so advanced callers can inspect the target.
    ///
    /// # Returns
    ///
    /// - `Rc<RefCell<ScrollBinding>>`: Updated by hooks when the container mounts.
    pub fn scroll_binding_rc(&self) -> Rc<RefCell<ScrollBinding>> {
        Rc::clone(&self.scroll_binding)
    }

    /// Applies `scrollTo` (or custom `scroll_to_fn`) using current options and binding.
    ///
    /// # Parameters
    ///
    /// - `behavior`: Passed to `ScrollToOptions` for the DOM API.
    /// - `offset`: Logical offset along the virtualizer axis.
    /// - `opts`: Full scroll options forwarded to a custom scroll function when set.
    ///
    /// # Returns
    ///
    /// - `Ok(())` when a scroll command was applied or delegated.
    /// - `Err(VirtualizerError)` when the binding is missing or the element cannot scroll.
    ///
    /// # Errors
    ///
    /// - `ScrollContainerUnavailable` for unbound targets or non-`HtmlElement` elements.
    fn apply_scroll_offset_to_dom(
        &self,
        behavior: ScrollBehavior,
        offset: f64,
        opts: ScrollToOptions,
    ) -> Result<(), VirtualizerError> {
        // Read scroll axis, RTL, and optional injectable scroll fn.
        let virt = self.virtualizer.borrow();
        let horizontal = virt.options().scroll_direction == ScrollDirection::Horizontal;
        let is_rtl = virt.options().is_rtl;
        let scroll_to_fn = virt.options().scroll_to_fn.clone();
        drop(virt);

        // Delegate to the web_sys helper module.
        let binding = self.scroll_binding.borrow();
        apply_scroll_offset(
            &binding,
            offset,
            horizontal,
            is_rtl,
            behavior,
            scroll_to_fn.as_ref(),
            opts,
        )
    }

    /// Increments the Yew render counter so the next frame re-queries the handle.
    fn trigger_render(&self) {
        // Wrap the counter to avoid panics on overflow in long sessions.
        let current = *self.render_trigger;
        self.render_trigger.set(current.wrapping_add(1));
    }
}
