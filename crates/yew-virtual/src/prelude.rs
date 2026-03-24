//! Re-exports for convenient access to all public yew-virtual types.
//!
//! This module provides a single import point for the most commonly used
//! types, enums, hooks, and handles in the yew-virtual crate. Import
//! this prelude to get started quickly with virtualization in Yew.

pub use crate::core::prelude::*;

#[cfg(target_arch = "wasm32")]
pub use crate::hooks::prelude::*;
