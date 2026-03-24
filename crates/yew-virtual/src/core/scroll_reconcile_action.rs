/// Result of one frame of programmatic scroll reconciliation.
///
/// Callers should keep scheduling animation frames while [`ScrollReconcileAction::Continue`] is returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollReconcileAction {
    /// Schedule another reconciliation frame on the next animation tick.
    ///
    /// # Details
    ///
    /// - Smooth scroll has not yet settled within tolerance.
    /// - Timeout has not been exceeded.
    Continue,

    /// The scroll position matched the target for enough consecutive frames.
    ///
    /// # Details
    ///
    /// - Internal [`ScrollState`](crate::core::scroll_state::ScrollState) is cleared.
    Done,

    /// Reconciliation gave up after [`VirtualizerOptions::scroll_reconciliation_timeout_ms`].
    ///
    /// # Details
    ///
    /// - Prevents infinite rAF loops if the browser never reaches the target offset.
    Timeout,
}
