//! Panel summarizing common integration scenarios for yew-virtual.

use yew::prelude::*;

/// Renders a concise guide for developers wiring yew-virtual into an app.
///
/// Highlights headless core vs hooks, sizing modes, and scroll targets so
/// integrators can map demos to their own layouts.
#[function_component(UseCasesPanel)]
pub fn use_cases_panel() -> Html {
    html! {
        <section
            class="mb-10 rounded-2xl border border-slate-200 bg-white/90 shadow-sm shadow-slate-200/40 p-6 md:p-8"
            aria-labelledby="use-cases-heading"
        >
            <h2 id="use-cases-heading" class="text-xl font-semibold text-slate-800 mb-4">
                {"Integration guide"}
            </h2>
            <p class="text-sm text-slate-600 mb-6 leading-relaxed">
                {"yew-virtual is "}<strong>{"headless"}</strong>
                {": the core engine computes item offsets and visible ranges; "}
                {"you render DOM. Use "}<code class="text-indigo-700 bg-slate-50 px-1 rounded">{"yew_virtual::core"}</code>
                {" for tests and non-UI tools, and "}<code class="text-indigo-700 bg-slate-50 px-1 rounded">{"yew_virtual::hooks"}</code>
                {" in WASM to bind scroll and resize observers."}
            </p>
            <ul class="grid gap-4 md:grid-cols-2 text-sm text-slate-600">
                <li class="rounded-xl border border-slate-100 bg-slate-50/80 p-4">
                    <h3 class="font-semibold text-slate-800 mb-1">{"Long vertical lists"}</h3>
                    <p class="leading-relaxed">
                        {"Default pattern: "}<code class="text-indigo-700 bg-white px-1 rounded">{"use_virtualizer"}</code>
                        {" + overflow container. Use "}<code class="text-indigo-700 bg-white px-1 rounded">{"ItemSizeMode::Fixed"}</code>
                        {" when every row shares height; "}<code class="text-indigo-700 bg-white px-1 rounded">{"Estimated"}</code>
                        {" / "}<code class="text-indigo-700 bg-white px-1 rounded">{"Dynamic"}</code>
                        {" when heights vary and you measure after paint."}
                    </p>
                </li>
                <li class="rounded-xl border border-slate-100 bg-slate-50/80 p-4">
                    <h3 class="font-semibold text-slate-800 mb-1">{"Window-level scrolling"}</h3>
                    <p class="leading-relaxed">
                        {"Full-page feeds: "}<code class="text-indigo-700 bg-white px-1 rounded">{"use_window_virtualizer"}</code>
                        {" tracks the browser viewport instead of an inner div. Set "}
                        <code class="text-indigo-700 bg-white px-1 rounded">{"use_window_scroll: true"}</code>
                        {" in options when driving logic from the core directly."}
                    </p>
                </li>
                <li class="rounded-xl border border-slate-100 bg-slate-50/80 p-4">
                    <h3 class="font-semibold text-slate-800 mb-1">{"Horizontal lanes"}</h3>
                    <p class="leading-relaxed">
                        {"Carousels and wide tables: "}<code class="text-indigo-700 bg-white px-1 rounded">{"ScrollDirection::Horizontal"}</code>
                        {". Combine "}<code class="text-indigo-700 bg-white px-1 rounded">{"gap"}</code>
                        {" and "}<code class="text-indigo-700 bg-white px-1 rounded">{"padding_start"}</code>
                        {" / "}<code class="text-indigo-700 bg-white px-1 rounded">{"padding_end"}</code>
                        {" for spacing; map wheel events if you need vertical gesture to scroll sideways."}
                    </p>
                </li>
                <li class="rounded-xl border border-slate-100 bg-slate-50/80 p-4">
                    <h3 class="font-semibold text-slate-800 mb-1">{"Programmatic scroll"}</h3>
                    <p class="leading-relaxed">
                        {"Jump to an item: "}<code class="text-indigo-700 bg-white px-1 rounded">{"scroll_to_index"}</code>
                        {" with "}<code class="text-indigo-700 bg-white px-1 rounded">{"ScrollAlignment"}</code>
                        {" (Start / Center / End / Auto). Hooks expose DOM "}<code class="text-indigo-700 bg-white px-1 rounded">{"scrollTo"}</code>
                        {" helpers; the core stays testable without a browser."}
                    </p>
                </li>
                <li class="rounded-xl border border-slate-100 bg-slate-50/80 p-4 md:col-span-2">
                    <h3 class="font-semibold text-slate-800 mb-1">{"Grids and multi-column layouts"}</h3>
                    <p class="leading-relaxed">
                        {"Set "}<code class="text-indigo-700 bg-white px-1 rounded">{"lanes > 1"}</code>
                        {" to distribute indices across columns or rows, or compose multiple virtualizers (TanStack-style) for two axes."}
                    </p>
                </li>
            </ul>
        </section>
    }
}
