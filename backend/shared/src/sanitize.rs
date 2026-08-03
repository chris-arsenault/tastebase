use std::collections::HashSet;

/// Strip all HTML tags from user input, preserving text content.
pub fn clean(input: &str) -> String {
    let sanitized_html = ammonia::Builder::new()
        .tags(HashSet::new())
        .clean(input)
        .to_string();
    html_escape::decode_html_entities(&sanitized_html).into_owned()
}

/// Sanitize an optional string field.
pub fn clean_option(input: Option<&str>) -> Option<String> {
    input.map(clean)
}

/// Sanitize a string, returning empty string for None.
pub fn clean_or_empty(input: Option<&str>) -> String {
    input.map(clean).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::clean;

    #[test]
    fn plain_text_ampersands_are_not_stored_as_html_entities() {
        assert_eq!(clean("Art & Fear"), "Art & Fear");
        assert_eq!(clean("Feeling &amp; Knowing"), "Feeling & Knowing");
    }

    #[test]
    fn html_is_removed_while_its_safe_text_is_preserved() {
        assert_eq!(
            clean("Read <strong>this</strong><script>bad()</script> now"),
            "Read this now"
        );
    }
}
