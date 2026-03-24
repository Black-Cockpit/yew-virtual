/// Trait for formatting numbers with thousands separators.
///
/// Provides a method to convert numeric values into comma-separated
/// strings for human-readable display of large numbers in the UI.
pub trait FormatNumber {
    /// Formats the number with comma separators.
    ///
    /// # Returns
    ///
    /// - `String`: The formatted number string with commas.
    fn to_formatted(&self) -> String;
}

impl FormatNumber for usize {
    /// Formats a `usize` value with comma separators.
    ///
    /// # Returns
    ///
    /// - `String`: The formatted number string.
    fn to_formatted(&self) -> String {
        // Convert the number to a string.
        let s = self.to_string();
        let bytes = s.as_bytes();
        let len = bytes.len();

        // Build the formatted string with commas every three digits.
        let mut result = String::with_capacity(len + len / 3);
        for (i, &byte) in bytes.iter().enumerate() {
            // Insert a comma before every group of three digits.
            if i > 0 && (len - i).is_multiple_of(3) {
                result.push(',');
            }
            result.push(byte as char);
        }
        result
    }
}
