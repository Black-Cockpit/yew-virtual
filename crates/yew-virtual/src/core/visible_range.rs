/// Describes the visible range of items in a virtualized list.
///
/// Contains the start and end indices of the visible viewport along
/// with the overscan and total count used to compute the rendered
/// index set. This type is passed to custom range extractors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisibleRange {
    /// The first visible item index (inclusive).
    pub start_index: usize,

    /// The last visible item index (inclusive).
    pub end_index: usize,

    /// The number of extra items beyond the viewport.
    pub overscan: usize,

    /// The total number of items in the dataset.
    pub count: usize,
}
