/// Stable identity key for a virtual item.
///
/// Supports both numeric and string-based keys to allow custom
/// key extractors that produce stable identities across item
/// reordering. When items are reordered, their measured sizes
/// follow the key rather than the index.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VirtualKey {
    /// A numeric key, typically derived from the item index.
    Index(usize),

    /// A string-based key for custom identity strategies.
    Named(String),
}

impl Default for VirtualKey {
    /// Returns a default key of index zero.
    ///
    /// # Returns
    ///
    /// - `VirtualKey::Index(0)`: The default key.
    fn default() -> Self {
        // Default to index zero.
        Self::Index(0)
    }
}

impl From<usize> for VirtualKey {
    /// Creates a key from a numeric index.
    ///
    /// # Parameters
    ///
    /// - `index`: The numeric index value.
    fn from(index: usize) -> Self {
        // Wrap the index in the Index variant.
        Self::Index(index)
    }
}

impl From<String> for VirtualKey {
    /// Creates a key from a string.
    ///
    /// # Parameters
    ///
    /// - `name`: The string key value.
    fn from(name: String) -> Self {
        // Wrap the string in the Named variant.
        Self::Named(name)
    }
}

impl From<&str> for VirtualKey {
    /// Creates a key from a string slice.
    ///
    /// # Parameters
    ///
    /// - `name`: The string slice key value.
    fn from(name: &str) -> Self {
        // Convert to owned string and wrap.
        Self::Named(name.to_string())
    }
}

impl std::fmt::Display for VirtualKey {
    /// Formats the key for display.
    ///
    /// # Parameters
    ///
    /// - `f`: The formatter.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Delegate formatting based on variant.
        match self {
            Self::Index(i) => write!(f, "{}", i),
            Self::Named(s) => write!(f, "{}", s),
        }
    }
}
