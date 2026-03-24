//! Horizontal virtual list demo component.
//!
//! Demonstrates a horizontally scrolling virtualized list with 50,000
//! gradient cards, configurable gap and padding, and live stats
//! showing the current visible range. Vertical wheel / trackpad deltas
//! map to horizontal scroll via a non-passive `wheel` listener.

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{AddEventListenerOptions, Element, WheelEvent};
use yew::prelude::*;

use yew_virtual::core::item_size_mode::ItemSizeMode;
use yew_virtual::core::scroll_direction::ScrollDirection;
use yew_virtual::core::virtualizer_options::VirtualizerOptions;

use crate::renderers::horizontal_item_renderer::render_horizontal_item;
use crate::utils::format_number::FormatNumber;
use crate::utils::virtualizer_helper::create_virtualizer_safe;

/// Stored non-passive wheel listener; dropped on effect cleanup.
type WheelListenerCell = Rc<RefCell<Option<Closure<dyn FnMut(WheelEvent)>>>>;

/// Horizontal item width in pixels.
const HORIZONTAL_ITEM_WIDTH: f64 = 180.0;

/// Horizontal container width in pixels.
const HORIZONTAL_CONTAINER_WIDTH: f64 = 600.0;

/// Horizontal list item count.
const HORIZONTAL_ITEM_COUNT: usize = 50_000;

/// Treat sub-pixel noise as identical scroll position when deduplicating scroll events.
const SCROLL_OFFSET_EPS: f64 = 0.5;

/// Horizontal virtual list demo component.
///
/// Renders a horizontally scrollable container with 50,000 virtualized
/// gradient cards, with 12px gaps and 16px padding on each side.
#[function_component(HorizontalListDemo)]
pub fn horizontal_list_demo() -> Html {
    // Create a reference for the scroll container DOM element.
    let container_ref = use_node_ref();

    // State counter used to trigger Yew re-renders on scroll.
    let render_counter = use_state(|| 0u64);

    // Create the virtualizer engine for horizontal scrolling.
    let virtualizer = use_memo((), |_| {
        // Configure the virtualizer for a horizontal fixed-size list with gaps.
        let options = VirtualizerOptions {
            item_count: HORIZONTAL_ITEM_COUNT,
            item_size_mode: ItemSizeMode::Fixed(HORIZONTAL_ITEM_WIDTH),
            scroll_direction: ScrollDirection::Horizontal,
            container_size: Some(HORIZONTAL_CONTAINER_WIDTH),
            overscan: 3,
            gap: 12.0,
            padding_start: 16.0,
            padding_end: 16.0,
            ..VirtualizerOptions::default()
        };

        // Create the virtualizer using the safe helper with automatic fallback.
        RefCell::new(create_virtualizer_safe(options))
    });

    // Borrow the virtualizer to read current state.
    let virt = virtualizer.borrow();
    let virtual_items = virt.get_virtual_items();
    let total_size = virt.total_size();
    let range_start = virt.range_start();
    let range_end = virt.range_end();
    drop(virt);

    // Set up the scroll event handler.
    let scroll_virt = virtualizer.clone();
    let scroll_trigger = render_counter.clone();
    let scroll_ref = container_ref.clone();

    let on_scroll = Callback::from(move |_: Event| {
        // Read the horizontal scroll position from the container element.
        if let Some(el) = scroll_ref.cast::<Element>() {
            // Get the horizontal scroll offset.
            let scroll_left = el.scroll_left() as f64;

            // Skip redundant updates when the DOM already matches the engine (for example after
            // programmatic scroll or when wheel and scroll events both fire for the same offset).
            let mut virt = scroll_virt.borrow_mut();
            if (scroll_left - virt.scroll_offset()).abs() <= SCROLL_OFFSET_EPS {
                return;
            }

            // Update the virtualizer with the new scroll position.
            virt.update_scroll_offset(scroll_left, true);
            drop(virt);

            // Trigger a Yew re-render.
            let current = *scroll_trigger;
            scroll_trigger.set(current.wrapping_add(1));
        }
    });

    // Map vertical wheel (and dominant trackpad axis) to horizontal scroll.
    use_effect_with(container_ref.clone(), move |cref| {
        let wheel_closure: WheelListenerCell = Rc::new(RefCell::new(None));
        let cleanup_el: Rc<RefCell<Option<Element>>> = Rc::new(RefCell::new(None));

        if let Some(el) = cref.cast::<Element>() {
            let el_w = el.clone();

            let closure = Closure::wrap(Box::new(move |e: WheelEvent| {
                let max_scroll = (el_w.scroll_width() - el_w.client_width()).max(0) as f64;
                if max_scroll <= f64::EPSILON {
                    return;
                }

                let dy = e.delta_y();
                let dx = e.delta_x();
                let delta = if dx.abs() > dy.abs() { dx } else { dy };
                if delta.abs() < f64::EPSILON {
                    return;
                }

                let current = el_w.scroll_left() as f64;
                let next = (current + delta).clamp(0.0, max_scroll);
                if (next - current).abs() < f64::EPSILON {
                    return;
                }

                e.prevent_default();
                el_w.set_scroll_left(next as i32);
            }) as Box<dyn FnMut(WheelEvent)>);

            let opts = AddEventListenerOptions::new();
            opts.set_passive(false);
            let _ = el.add_event_listener_with_callback_and_add_event_listener_options(
                "wheel",
                closure.as_ref().unchecked_ref(),
                &opts,
            );

            *wheel_closure.borrow_mut() = Some(closure);
            *cleanup_el.borrow_mut() = Some(el);
        }

        let wc = Rc::clone(&wheel_closure);
        let ce = Rc::clone(&cleanup_el);
        move || {
            if let (Some(el), Some(cl)) = (ce.borrow().as_ref(), wc.borrow().as_ref()) {
                let _ =
                    el.remove_event_listener_with_callback("wheel", cl.as_ref().unchecked_ref());
            }
            drop(wc.borrow_mut().take());
        }
    });

    // Reference render_counter to ensure it triggers re-renders.
    let _ = *render_counter;

    // Define the gradient color palette for horizontal cards.
    let colors = [
        ("from-blue-400 to-blue-600", "bg-blue-700/20"),
        ("from-emerald-400 to-emerald-600", "bg-emerald-700/20"),
        ("from-violet-400 to-violet-600", "bg-violet-700/20"),
        ("from-amber-400 to-amber-600", "bg-amber-700/20"),
        ("from-rose-400 to-rose-600", "bg-rose-700/20"),
        ("from-cyan-400 to-cyan-600", "bg-cyan-700/20"),
    ];

    // Render the horizontal list demo card.
    html! {
        <div data-testid="demo-horizontal" class="bg-white rounded-2xl shadow-lg shadow-slate-200/50 border border-slate-100 overflow-hidden">
            // Card header with title and stats.
            <div class="p-5 border-b border-slate-100">
                <div class="flex items-center justify-between mb-1 gap-2 flex-wrap">
                    <h2 class="text-lg font-semibold text-slate-800 flex items-center gap-2">
                        <span class="inline-flex h-6 min-w-[1.5rem] items-center justify-center rounded-md bg-blue-500/15 text-xs font-bold text-blue-700">
                            {"2"}
                        </span>
                        <span class="w-2 h-2 rounded-full bg-blue-500 inline-block shrink-0"></span>
                        {"Horizontal strip (gap + padding)"}
                    </h2>
                    <span class="text-xs font-medium text-slate-400 bg-slate-50 px-2 py-1 rounded-full">
                        {format!("Rendering {} of {}", range_end - range_start, HORIZONTAL_ITEM_COUNT.to_formatted())}
                    </span>
                </div>
                <p class="text-xs text-slate-500 leading-relaxed mb-1">
                    {"Use case: carousels, timelines, or chip rows. Set "}<code class="text-indigo-600 bg-slate-50 px-1 rounded">{"ScrollDirection::Horizontal"}</code>
                    {", tune "}<code class="text-indigo-600 bg-slate-50 px-1 rounded">{"gap"}</code>
                    {" / "}<code class="text-indigo-600 bg-slate-50 px-1 rounded">{"padding_*"}</code>
                    {"; vertical wheel deltas are mapped to horizontal scroll here."}
                </p>
                <p class="text-xs text-slate-400 leading-relaxed">
                    {"12px gap \u{2022} 16px start/end padding \u{2022} non-passive wheel listener"}
                </p>
            </div>

            // Horizontally scrollable container.
            <div
                ref={container_ref}
                onscroll={on_scroll}
                tabindex="0"
                class="overflow-x-auto overscroll-x-contain scrollbar-thin p-4 rounded-b-2xl focus:outline-none focus-visible:ring-2 focus-visible:ring-blue-400/40 focus-visible:ring-inset [overflow-anchor:none]"
                style="height: 240px;"
            >
                // Inner container sized to the full virtual width.
                <div
                    class="relative h-full"
                    style={format!("width: {}px;", total_size)}
                >
                    // Render only the visible cards using absolute positioning.
                    { for virtual_items.iter().map(|item| {
                        // Select gradient colors based on item index.
                        let color_idx = item.index % colors.len();
                        let (gradient, badge_bg) = colors[color_idx];
                        render_horizontal_item(item, gradient, badge_bg)
                    }) }
                </div>
            </div>
        </div>
    }
}
