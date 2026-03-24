//! Root application component for the Tailwind virtual list example.
//!
//! Renders a gradient shell, an integration guide, live demos, and footer links.

use yew::prelude::*;

use crate::components::horizontal_list_demo::HorizontalListDemo;
use crate::components::use_cases_panel::UseCasesPanel;
use crate::components::virtual_list_demo::VirtualListDemo;
use crate::utils::format_number::FormatNumber;

/// Item count referenced in the vertical demo subtitle.
const ITEM_COUNT: usize = 100_000;

/// Root application component that renders the Tailwind-based virtual list demo.
#[function_component(App)]
pub fn app() -> Html {
    html! {
        <div
            class="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 p-4 md:p-8 text-slate-900"
            data-testid="app-root"
        >
            <div class="max-w-5xl mx-auto">
                <header class="mb-8 flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
                    <div class="text-center sm:text-left flex-1">
                        <h1 class="text-4xl font-bold bg-gradient-to-r from-blue-600 to-indigo-600 bg-clip-text text-transparent mb-2">
                            {"yew-virtual"}
                        </h1>
                        <p class="text-slate-600 text-lg">
                            {"Headless virtualization for Yew — TanStack Virtual style \u{2022} "}
                            <span class="font-semibold text-indigo-600">
                                {format!("{} items in demo 1", ITEM_COUNT.to_formatted())}
                            </span>
                        </p>
                        <p class="text-slate-500 text-sm mt-1">
                            {"Below: when to use the crate, then two live references you can copy from."}
                        </p>
                    </div>
                    <a
                        href="https://github.com/Black-Cockpit/yew-virtual"
                        class="inline-flex items-center justify-center gap-2 px-4 py-2 rounded-xl text-sm font-medium border border-slate-200 bg-white/90 text-slate-700 hover:bg-white shadow-sm transition-colors shrink-0"
                        target="_blank"
                        rel="noopener noreferrer"
                    >
                        <span aria-hidden="true">{"↗"}</span>
                        {"Repository"}
                    </a>
                </header>

                <UseCasesPanel />

                <h2 class="text-lg font-semibold text-slate-800 mb-4">
                    {"Live examples"}
                </h2>
                <div class="grid gap-6 md:grid-cols-2">
                    <VirtualListDemo />
                    <HorizontalListDemo />
                </div>

                <footer class="mt-10 text-center text-sm text-slate-500">
                    {"Crate path: "}<code class="text-indigo-700 bg-white/80 px-1 rounded border border-slate-200">{"crates/yew-virtual"}</code>
                    {" · "}
                    <a class="underline decoration-indigo-400/50 hover:text-indigo-600" href="https://docs.rs/yew-virtual">
                        {"docs.rs"}
                    </a>
                    {" · "}
                    <a class="underline decoration-indigo-400/50 hover:text-indigo-600" href="https://yew.rs">
                        {"Yew"}
                    </a>
                </footer>
            </div>
        </div>
    }
}
