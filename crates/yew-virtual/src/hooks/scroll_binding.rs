use web_sys::Element;

/// Identifies which DOM target receives programmatic scroll operations from [`VirtualizerHandle`](crate::hooks::virtualizer_handle::VirtualizerHandle).
///
/// Hooks transition from [`ScrollBinding::None`] to [`ScrollBinding::Element`] or
/// [`ScrollBinding::Window`] when effects attach listeners so `scrollTo` helpers
/// know where to apply offsets.
#[derive(Debug, Clone)]
pub enum ScrollBinding {
    /// No scroll target is bound yet (container not mounted or effect torn down).
    None,

    /// A scrollable element (for example `overflow-y: auto` on the list viewport).
    Element { element: Element },

    /// The browser window as the scroll container (`use_window_virtualizer`).
    Window,
}

impl Default for ScrollBinding {
    /// Returns [`ScrollBinding::None`] for hooks before the scroll target exists.
    fn default() -> Self {
        Self::None
    }
}
