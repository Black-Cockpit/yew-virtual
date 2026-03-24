//! Render helper for vertical virtual list items.
//!
//! Produces the Tailwind-styled HTML for a single row in the
//! vertical virtualized list, including color accent bars,
//! item content, and index badges.

use yew::prelude::*;

use yew_virtual::core::virtual_item::VirtualItem;

/// Renders a single vertical list item as a positioned row.
///
/// # Parameters
///
/// - `item`: The virtual item metadata containing index, size, and offset.
///
/// # Returns
///
/// - `Html`: The Yew HTML for the positioned list row.
pub fn render_vertical_item(item: &VirtualItem) -> Html {
    // Determine row background styling based on index parity.
    let bg_class = if item.index.is_multiple_of(2) {
        "bg-white"
    } else {
        "bg-slate-50/50"
    };

    // Determine accent color based on index modulo.
    let accent = match item.index % 5 {
        0 => "bg-blue-500",
        1 => "bg-emerald-500",
        2 => "bg-violet-500",
        3 => "bg-amber-500",
        _ => "bg-rose-500",
    };

    // Render the absolutely positioned row element.
    html! {
        <div
            key={item.index}
            class={format!(
                "absolute left-0 right-0 flex items-center px-5 {} hover:bg-indigo-50/50 transition-colors border-b border-slate-50",
                bg_class
            )}
            style={format!(
                "height: {}px; transform: translate3d(0, {}px, 0); backface-visibility: hidden;",
                item.size,
                item.start
            )}
        >
            // Color accent bar.
            <div class={format!("w-1 h-8 rounded-full mr-4 {}", accent)}></div>

            // Item content area.
            <div class="flex-1 min-w-0">
                <div class="font-medium text-sm text-slate-700 truncate">
                    {format!("Item #{}", item.index)}
                </div>
                <div class="text-xs text-slate-400 truncate">
                    {format!("Row {} \u{2022} Offset {:.0}px \u{2022} Size {:.0}px", item.index, item.start, item.size)}
                </div>
            </div>

            // Index badge.
            <div class="text-xs font-mono text-slate-300 bg-slate-50 px-2 py-0.5 rounded">
                {format!("#{}", item.index)}
            </div>
        </div>
    }
}
