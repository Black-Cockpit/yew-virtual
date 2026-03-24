# yew-virtual

[![CI](https://github.com/Black-Cockpit/yew-virtual/actions/workflows/ci.yml/badge.svg)](https://github.com/Black-Cockpit/yew-virtual/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/yew-virtual.svg)](https://crates.io/crates/yew-virtual)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A headless virtualization engine for [Yew](https://yew.rs) applications, aligned in spirit with [TanStack Virtual](https://tanstack.com/virtual). It is built for very large lists and grids in WebAssembly: you keep full control of markup and styling while the engine computes visible ranges, overscan, and scroll alignment.

## Live demo

- [**Tailwind example**](https://black-cockpit.github.io/yew-virtual//example/) — vertical and horizontal virtual lists with Tailwind, live stats, and copy aimed at real integrations.

## Overview

yew-virtual splits a **headless core** (`Virtualizer`, options, and measurement math) from **Yew hooks** (`use_virtualizer`, `use_window_virtualizer`) that attach passive scroll listeners and resize observers. Use the core alone for tests or non-DOM simulations, or pair hooks with your own absolutely positioned rows or columns.

- **Headless engine** — Range extraction, overscan, padding, gap, dynamic measurement, scroll-to-index.
- **Container and window hooks** — Ref-based scroll container or document-level scroll.
- **Sizing modes** — Fixed, estimated, and dynamic strategies with optional per-item observers.
- **Safe Rust** — No `unsafe`, `unwrap()`, or `expect()` in the library crate.

## Features

| Area | Capabilities |
|------|----------------|
| **Layout** | Vertical and horizontal axes, RTL-aware horizontal `scrollLeft` |
| **Sizing** | `Fixed`, `Estimated`, `Dynamic`; optional runtime measurement |
| **Scrolling** | Overscan; programmatic scroll-to-index (Start / Center / End / Auto) |
| **Spacing** | Gap between items; padding at start and end |
| **Hooks** | Passive scroll listeners; `ResizeObserver` on the scroll container |
| **Quality** | Explicit error types; integration tests and CI coverage gate |

## Installation

```toml
[dependencies]
yew = { version = "0.21", features = ["csr"] }
yew-virtual = "0.1"
```

Enable [`yew`](https://docs.rs/yew)’s `csr` feature for browser (WASM) apps, or `ssr` / `hydration` when you follow Yew’s server-side rendering setup. `yew-virtual` does not define its own `csr` / `ssr` flags.

## Quick start

### Core engine (no DOM)

```rust
use yew_virtual::core::virtualizer::Virtualizer;
use yew_virtual::core::virtualizer_options::VirtualizerOptions;
use yew_virtual::core::item_size_mode::ItemSizeMode;

let options = VirtualizerOptions {
    item_count: 100_000,
    item_size_mode: ItemSizeMode::Fixed(50.0),
    container_size: Some(600.0),
    overscan: 5,
    ..VirtualizerOptions::default()
};

let mut virt = Virtualizer::new(options).expect("valid config");
virt.update_scroll_offset(500.0);

for item in virt.get_virtual_items() {
    println!("Index: {}, Offset: {}px, Size: {}px", item.index, item.start, item.size);
}
```

### With Yew hooks (WASM)

```rust
use yew::prelude::*;
use yew_virtual::hooks::use_virtualizer::use_virtualizer;
use yew_virtual::core::virtualizer_options::VirtualizerOptions;
use yew_virtual::core::item_size_mode::ItemSizeMode;

#[function_component(VirtualList)]
fn virtual_list() -> Html {
    let options = VirtualizerOptions {
        item_count: 100_000,
        item_size_mode: ItemSizeMode::Fixed(50.0),
        overscan: 5,
        ..VirtualizerOptions::default()
    };

    let (handle, container_ref) = use_virtualizer(options);

    html! {
        <div ref={container_ref} style="height: 600px; overflow-y: auto;">
            <div style={format!("height: {}px; position: relative;", handle.total_size())}>
                { for handle.get_virtual_items().iter().map(|item| html! {
                    <div
                        key={item.index}
                        style={format!(
                            "position: absolute; top: 0; left: 0; right: 0; height: {}px; transform: translateY({}px);",
                            item.size, item.start
                        )}
                    >
                        { format!("Item {}", item.index) }
                    </div>
                })}
            </div>
        </div>
    }
}
```

## Scroll alignment

When using `scroll_to_index`, four alignment modes are available:

| Alignment | Behavior |
|-----------|----------|
| `Start` | Item aligned to the viewport start |
| `Center` | Item centered in the viewport |
| `End` | Item aligned to the viewport end |
| `Auto` | Minimal scroll so the item becomes visible |

## Item size modes

| Mode | Description |
|------|-------------|
| `Fixed(f64)` | Uniform item size; no measurement |
| `Estimated(f64)` | Initial guess; refined when measured |
| `Dynamic(f64)` | Fallback size; measured at runtime |

## Running the example

The Tailwind demo uses [Trunk](https://trunkrs.dev):

```bash
cargo install trunk
cd examples/tailwind-virtual-list
trunk serve
```

## Running tests

```bash
cargo test -p yew-virtual
```

New tests should follow [TEST_RULES.md](TEST_RULES.md). CI runs [cargo-tarpaulin](https://github.com/xd009642/tarpaulin) on the `yew-virtual` package with **`--fail-under 90`** and uploads results to Codecov.

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request.

## License

Distributed under the [MIT](LICENSE) license.
