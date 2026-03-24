/// Core virtualization engine types and logic.
///
/// This module contains the fundamental virtualizer engine, item metadata,
/// range calculation, and all supporting types for headless virtualization.
pub mod core;

/// Yew integration hooks for virtualization.
///
/// Exposes [`crate::hooks::use_virtualizer::use_virtualizer`] and
/// [`crate::hooks::use_window_virtualizer::use_window_virtualizer`], which return a
/// [`crate::hooks::virtualizer_handle::VirtualizerHandle`] wired to browser scroll and resize
/// events, optional per-item `ResizeObserver` registration, and DOM `scrollTo` helpers.
#[cfg(target_arch = "wasm32")]
pub mod hooks;

/// Re-exports for convenient access to all public types.
///
/// This module provides a centralized prelude for importing the most
/// commonly used types, hooks, and enums from the crate.
pub mod prelude;
