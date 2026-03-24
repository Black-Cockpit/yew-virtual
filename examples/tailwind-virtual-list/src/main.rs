//! Application entry point for the Tailwind virtual list example.

mod app;
mod components;
mod renderers;
mod utils;

use crate::app::App;

/// Application entry point.
fn main() {
    // Mount the Yew application to the document body.
    yew::Renderer::<App>::new().render();
}
