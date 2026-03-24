//! Render helper for horizontal virtual list items.
//!
//! Produces the Tailwind-styled HTML for a single card in the
//! horizontal virtualized list, including gradient backgrounds,
//! index badges, and offset labels.

use yew::prelude::*;

use yew_virtual::core::virtual_item::VirtualItem;

/// Renders a single horizontal list card as a positioned element.
///
/// # Parameters
///
/// - `item`: The virtual item metadata containing index, size, and offset.
/// - `gradient`: Tailwind gradient classes for the card background.
/// - `badge_bg`: Tailwind background class for the index badge overlay.
///
/// # Returns
///
/// - `Html`: The Yew HTML for the positioned horizontal card.
pub fn render_horizontal_item(item: &VirtualItem, gradient: &str, badge_bg: &str) -> Html {
    // Render the absolutely positioned card element.
    html! {
        <div
            key={item.index}
            class={format!(
                "absolute top-0 h-full rounded-xl bg-gradient-to-br {} text-white p-4 flex flex-col justify-between shadow-md hover:shadow-lg transition-shadow",
                gradient
            )}
            style={format!("width: {}px; transform: translateX({}px);", item.size, item.start)}
        >
            // Top section with badge and title.
            <div>
                <div class={format!("inline-block text-xs font-mono px-2 py-0.5 rounded {} mb-2", badge_bg)}>
                    {format!("#{}", item.index)}
                </div>
                <div class="font-semibold text-sm">
                    {format!("Card {}", item.index)}
                </div>
            </div>

            // Bottom section with offset label.
            <div class="text-xs opacity-75">
                {format!("{:.0}px", item.start)}
            </div>
        </div>
    }
}
