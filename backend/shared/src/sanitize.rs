/// Strip all HTML tags from user input, preserving text content.
pub fn clean(input: &str) -> String {
    ammonia::clean(input)
}

/// Sanitize an optional string field.
pub fn clean_option(input: Option<&str>) -> Option<String> {
    input.map(clean)
}

/// Sanitize a string, returning empty string for None.
pub fn clean_or_empty(input: Option<&str>) -> String {
    input.map(clean).unwrap_or_default()
}
