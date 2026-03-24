//! Vertical virtual list demo component.
//!
//! Demonstrates a vertically scrolling virtualized list with 100,000 items,
//! scroll-to-index controls with Start alignment, and live stats showing
//! the current scroll offset, visible range, and total scrollable size.

use std::cell::RefCell;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::Element;
use yew::prelude::*;

use yew_virtual::core::item_size_mode::ItemSizeMode;
use yew_virtual::core::scroll_alignment::ScrollAlignment;
use yew_virtual::core::scroll_direction::ScrollDirection;
use yew_virtual::core::virtualizer_options::VirtualizerOptions;

use crate::renderers::vertical_item_renderer::render_vertical_item;
use crate::utils::format_number::FormatNumber;
use crate::utils::virtualizer_helper::create_virtualizer_safe;

/// Total number of items in the vertical demo list.
const ITEM_COUNT: usize = 100_000;

/// Fixed item height in pixels.
const ITEM_HEIGHT: f64 = 64.0;

/// Container height in pixels.
const CONTAINER_HEIGHT: f64 = 600.0;

/// Overscan item count.
const OVERSCAN: usize = 5;

/// Vertical virtual list demo component.
///
/// Renders a scrollable container with 100,000 virtualized rows,
/// a scroll-to-index input, and a live stats bar.
#[function_component(VirtualListDemo)]
pub fn virtual_list_demo() -> Html {
    // Create a reference for the scroll container DOM element.
    let container_ref = use_node_ref();

    // State counter used to trigger Yew re-renders on scroll.
    let render_counter = use_state(|| 0u64);

    // State for the scroll-to-index input value.
    let scroll_target = use_state(String::new);

    // Latest scrollTop from the browser; rAF callback applies it with the Yew update (see on_scroll).
    let pending_scroll_top = use_mut_ref(|| 0.0f64);

    // When true, a requestAnimationFrame callback is already queued for this list.
    let scroll_raf_armed = use_mut_ref(|| false);

    // Create the virtualizer engine, memoized to avoid recreation.
    let virtualizer = use_memo((), |_| {
        // Configure the virtualizer for a vertical fixed-size list.
        let options = VirtualizerOptions {
            item_count: ITEM_COUNT,
            item_size_mode: ItemSizeMode::Fixed(ITEM_HEIGHT),
            scroll_direction: ScrollDirection::Vertical,
            container_size: Some(CONTAINER_HEIGHT),
            overscan: OVERSCAN,
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
    let current_scroll = virt.scroll_offset();
    drop(virt);

    // Set up the scroll event handler.
    let scroll_virt = virtualizer.clone();
    let scroll_trigger = render_counter.clone();
    let scroll_ref = container_ref.clone();
    let pending_top_ref = pending_scroll_top.clone();
    let raf_armed_ref = scroll_raf_armed.clone();

    let on_scroll = Callback::from(move |_: Event| {
        // Read the scroll position from the container element.
        let Some(el) = scroll_ref.cast::<Element>() else {
            return;
        };

        // Store the latest offset; multiple scroll events in one frame collapse to one update.
        let scroll_top = el.scroll_top() as f64;
        *pending_top_ref.borrow_mut() = scroll_top;

        // Avoid scheduling more than one rAF per frame — prevents synchronous Yew patches during
        // compositor scrolling, which commonly shows up as jump or shake in virtualized lists.
        if *raf_armed_ref.borrow() {
            return;
        }
        *raf_armed_ref.borrow_mut() = true;

        let pending_for_raf = pending_top_ref.clone();
        let raf_for_raf = raf_armed_ref.clone();
        let virt_for_raf = scroll_virt.clone();
        let trigger_for_raf = scroll_trigger.clone();

        let closure = Closure::wrap(Box::new(move || {
            *raf_for_raf.borrow_mut() = false;
            let y = *pending_for_raf.borrow();
            virt_for_raf.borrow_mut().update_scroll_offset(y, true);
            let current = *trigger_for_raf;
            trigger_for_raf.set(current.wrapping_add(1));
        }) as Box<dyn FnMut()>);

        let schedule_ok = web_sys::window().is_some_and(|w| {
            w.request_animation_frame(closure.as_ref().unchecked_ref())
                .is_ok()
        });

        if schedule_ok {
            closure.forget();
        } else {
            *raf_armed_ref.borrow_mut() = false;
            scroll_virt
                .borrow_mut()
                .update_scroll_offset(*pending_top_ref.borrow(), true);
            let current = *scroll_trigger;
            scroll_trigger.set(current.wrapping_add(1));
        }
    });

    // Set up the scroll-to-index button handler.
    let goto_virt = virtualizer.clone();
    let goto_ref = container_ref.clone();
    let goto_target = scroll_target.clone();
    let goto_trigger = render_counter.clone();

    let on_goto = Callback::from(move |_: MouseEvent| {
        // Parse the user-entered index from the input field.
        if let Ok(index) = goto_target.parse::<usize>() {
            // Calculate the required scroll offset for the target index.
            let virt = goto_virt.borrow();
            if let Ok(offset) = virt.scroll_to_index(index, ScrollAlignment::Start) {
                drop(virt);

                // Match the engine to the integer scroll position the DOM actually stores.
                let scroll_top_i32 = offset as i32;
                let scroll_top_dom = scroll_top_i32 as f64;

                // Apply the scroll position to the DOM container.
                if let Some(el) = goto_ref.cast::<Element>() {
                    el.set_scroll_top(scroll_top_i32);
                }

                // Synchronize the virtualizer with the same value so scroll events do not fight
                // a fractional engine offset against an integer scrollTop.
                goto_virt
                    .borrow_mut()
                    .update_scroll_offset(scroll_top_dom, false);

                // Trigger a Yew re-render.
                let current = *goto_trigger;
                goto_trigger.set(current.wrapping_add(1));
            }
        }
    });

    // Set up the input change handler for the scroll-to-index field.
    let input_target = scroll_target.clone();
    let on_input = Callback::from(move |e: InputEvent| {
        // Extract the input value from the event target.
        if let Some(input) = e
            .target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        {
            input_target.set(input.value());
        }
    });

    // Reference render_counter to ensure it triggers re-renders.
    let _ = *render_counter;

    // Render the vertical list demo card.
    html! {
        <div data-testid="demo-vertical" class="bg-white rounded-2xl shadow-lg shadow-slate-200/50 border border-slate-100 overflow-hidden">
            // Card header with title, controls, and stats.
            <div class="p-5 border-b border-slate-100">
                <div class="flex items-center justify-between mb-3 gap-2 flex-wrap">
                    <h2 class="text-lg font-semibold text-slate-800 flex items-center gap-2">
                        <span class="inline-flex h-6 min-w-[1.5rem] items-center justify-center rounded-md bg-emerald-500/15 text-xs font-bold text-emerald-700">
                            {"1"}
                        </span>
                        <span class="w-2 h-2 rounded-full bg-green-500 inline-block shrink-0"></span>
                        {"Vertical list (container scroll)"}
                    </h2>
                    <span class="text-xs font-medium text-slate-400 bg-slate-50 px-2 py-1 rounded-full tabular-nums min-w-[10.5rem] text-right inline-block">
                        {format!("Rendering {} of {}", range_end - range_start, ITEM_COUNT.to_formatted())}
                    </span>
                </div>
                <p class="text-xs text-slate-500 mb-3 leading-relaxed">
                    {"Use case: feed, inbox, or log lines. Wire "}<code class="text-indigo-600 bg-slate-50 px-1 rounded">{"use_virtualizer"}</code>
                    {" on a div with overflow-y; set "}<code class="text-indigo-600 bg-slate-50 px-1 rounded">{"ItemSizeMode::Fixed"}</code>
                    {" when row height is constant (this demo: "}{ITEM_HEIGHT}{"px)."}
                </p>

                // Scroll-to-index input and button.
                <div class="flex gap-2">
                    <input
                        type="number"
                        placeholder={format!("Go to index (0-{})", ITEM_COUNT - 1)}
                        class="flex-1 px-3 py-1.5 text-sm border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-400 transition-all"
                        oninput={on_input}
                        value={(*scroll_target).clone()}
                    />
                    <button
                        type="button"
                        onclick={on_goto}
                        class="px-4 py-1.5 text-sm font-medium text-white bg-indigo-500 hover:bg-indigo-600 rounded-lg transition-colors active:scale-95 transform"
                    >
                        {"Go"}
                    </button>
                </div>

                // Live stats bar.
                <div class="mt-3 flex flex-wrap gap-x-4 gap-y-1 text-xs text-slate-400 tabular-nums">
                    <span class="inline-block min-w-[7.5rem]">{format!("Scroll: {:.0}px", current_scroll)}</span>
                    <span class="inline-block min-w-[11rem]">{format!("Range: {}..{}", range_start, range_end)}</span>
                    <span class="inline-block min-w-[9rem]">{format!("Total: {:.0}px", total_size)}</span>
                </div>
            </div>

            // Scrollable container for the virtual list.
            <div
                ref={container_ref}
                onscroll={on_scroll}
                class="overflow-y-auto overscroll-y-contain scrollbar-thin [overflow-anchor:none] [scrollbar-gutter:stable]"
                style={format!("height: {}px;", CONTAINER_HEIGHT)}
            >
                // Inner container sized to the full virtual height.
                <div
                    class="relative w-full isolate [contain:layout]"
                    style={format!("height: {}px;", total_size)}
                >
                    // Render only the visible items using absolute positioning.
                    { for virtual_items.iter().map(render_vertical_item) }
                </div>
            </div>
        </div>
    }
}
