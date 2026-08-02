use crate::error::AppError;
use crate::types::BookTag;
use std::collections::HashSet;

const MAX_NAME_LEN: usize = 1000;
const MAX_NOTES_LEN: usize = 4000;
const MAX_URL_LEN: usize = 2000;
const MAX_BOOK_SUMMARY_LEN: usize = 6000;
const MAX_BOOK_WRITEUP_LEN: usize = 6000;
const MAX_BOOK_TAGS: usize = 32;
const MAX_BOOK_TAG_KEY_LEN: usize = 64;
const MAX_BOOK_TAG_VALUE_LEN: usize = 120;
const MAX_BOOK_PAGE_COUNT: i32 = 100_000;
const MAX_BASE64_BYTES: usize = 10 * 1024 * 1024; // 10 MB

fn check_len(field: &str, value: &str, max: usize) -> Result<(), AppError> {
    if value.len() > max {
        return Err(AppError::BadRequest(format!(
            "{field} exceeds max length of {max}"
        )));
    }
    Ok(())
}

fn check_range(field: &str, value: i16, min: i16, max: i16) -> Result<(), AppError> {
    if value < min || value > max {
        return Err(AppError::BadRequest(format!(
            "{field} must be between {min} and {max}"
        )));
    }
    Ok(())
}

fn check_optional_range(
    field: &str,
    value: Option<i16>,
    min: i16,
    max: i16,
) -> Result<(), AppError> {
    if let Some(v) = value {
        check_range(field, v, min, max)?;
    }
    Ok(())
}

fn check_base64(field: &str, value: &str) -> Result<(), AppError> {
    // Base64 encodes 3 bytes as 4 chars; estimate decoded size
    let estimated_bytes = (value.len() * 3) / 4;
    if estimated_bytes > MAX_BASE64_BYTES {
        return Err(AppError::BadRequest(format!(
            "{field} exceeds max size of {} MB",
            MAX_BASE64_BYTES / (1024 * 1024)
        )));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn validate_tasting_input(
    name: Option<&str>,
    maker: Option<&str>,
    style: Option<&str>,
    score: Option<i16>,
    heat_user: Option<i16>,
    heat_vendor: Option<i16>,
    refreshing: Option<i16>,
    sweet: Option<i16>,
    tasting_notes_user: Option<&str>,
    tasting_notes_vendor: Option<&str>,
    product_url: Option<&str>,
) -> Result<(), AppError> {
    if let Some(v) = name {
        check_len("name", v, MAX_NAME_LEN)?;
    }
    if let Some(v) = maker {
        check_len("maker", v, MAX_NAME_LEN)?;
    }
    if let Some(v) = style {
        check_len("style", v, MAX_NAME_LEN)?;
    }
    if let Some(v) = tasting_notes_user {
        check_len("tastingNotesUser", v, MAX_NOTES_LEN)?;
    }
    if let Some(v) = tasting_notes_vendor {
        check_len("tastingNotesVendor", v, MAX_NOTES_LEN)?;
    }
    if let Some(v) = product_url {
        check_len("productUrl", v, MAX_URL_LEN)?;
    }
    check_optional_range("score", score, 0, 10)?;
    check_optional_range("heatUser", heat_user, 0, 10)?;
    check_optional_range("heatVendor", heat_vendor, 0, 10)?;
    check_optional_range("refreshing", refreshing, 1, 5)?;
    check_optional_range("sweet", sweet, 1, 5)?;
    Ok(())
}

pub fn validate_base64_fields(fields: &[(&str, Option<&str>)]) -> Result<(), AppError> {
    for (name, value) in fields {
        if let Some(v) = value
            && !v.is_empty()
        {
            check_base64(name, v)?;
        }
    }
    Ok(())
}

pub fn validate_recipe_input(
    title: &str,
    description: Option<&str>,
    base_servings: i32,
    notes: Option<&str>,
) -> Result<(), AppError> {
    check_len("title", title, MAX_NAME_LEN)?;
    if title.trim().is_empty() {
        return Err(AppError::BadRequest("title is required".into()));
    }
    if let Some(v) = description {
        check_len("description", v, MAX_NOTES_LEN)?;
    }
    if let Some(v) = notes {
        check_len("notes", v, MAX_NOTES_LEN)?;
    }
    if base_servings < 1 {
        return Err(AppError::BadRequest(
            "baseServings must be at least 1".into(),
        ));
    }
    Ok(())
}

pub fn validate_book_recommendation(
    title: &str,
    author: &str,
    summary: &str,
    why_recommended: &str,
) -> Result<(), AppError> {
    for (field, value, max) in [
        ("title", title, MAX_NAME_LEN),
        ("author", author, MAX_NAME_LEN),
        ("summary", summary, MAX_BOOK_SUMMARY_LEN),
        ("whyRecommended", why_recommended, MAX_BOOK_SUMMARY_LEN),
    ] {
        check_len(field, value, max)?;
        if value.trim().is_empty() {
            return Err(AppError::BadRequest(format!("{field} is required")));
        }
    }
    Ok(())
}

pub fn normalize_book_tags(tags: Vec<BookTag>) -> Result<Vec<BookTag>, AppError> {
    if tags.len() > MAX_BOOK_TAGS {
        return Err(AppError::BadRequest(format!(
            "tags must contain at most {MAX_BOOK_TAGS} entries"
        )));
    }

    let mut normalized = Vec::with_capacity(tags.len());
    let mut seen = HashSet::with_capacity(tags.len());
    for tag in tags {
        let key = crate::sanitize::clean(&tag.key).trim().to_lowercase();
        let value = crate::sanitize::clean(&tag.value).trim().to_lowercase();
        check_len("tag key", &key, MAX_BOOK_TAG_KEY_LEN)?;
        check_len("tag value", &value, MAX_BOOK_TAG_VALUE_LEN)?;
        if key.is_empty() || value.is_empty() {
            return Err(AppError::BadRequest(
                "tag key and value are required".into(),
            ));
        }
        if seen.insert((key.clone(), value.clone())) {
            normalized.push(BookTag { key, value });
        }
    }

    normalized.sort_by(|left, right| {
        left.key
            .cmp(&right.key)
            .then_with(|| left.value.cmp(&right.value))
    });
    Ok(normalized)
}

pub fn validate_book_metadata(
    page_count: Option<i32>,
    purchase_link: Option<&str>,
) -> Result<(), AppError> {
    if let Some(page_count) = page_count
        && !(1..=MAX_BOOK_PAGE_COUNT).contains(&page_count)
    {
        return Err(AppError::BadRequest(format!(
            "pageCount must be between 1 and {MAX_BOOK_PAGE_COUNT}"
        )));
    }

    if let Some(purchase_link) = purchase_link {
        check_len("purchaseLink", purchase_link, MAX_URL_LEN)?;
        let parsed = reqwest::Url::parse(purchase_link)
            .map_err(|_| AppError::BadRequest("purchaseLink must be a valid URL".into()))?;
        if !matches!(parsed.scheme(), "http" | "https") {
            return Err(AppError::BadRequest(
                "purchaseLink must use http or https".into(),
            ));
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn validate_book_recommendation_patch(
    title: Option<&str>,
    author: Option<&str>,
    summary: Option<&str>,
    why_recommended: Option<&str>,
    page_count: Option<Option<i32>>,
    purchase_link: Option<Option<&str>>,
) -> Result<(), AppError> {
    for (field, value, max) in [
        ("title", title, MAX_NAME_LEN),
        ("author", author, MAX_NAME_LEN),
        ("summary", summary, MAX_BOOK_SUMMARY_LEN),
        ("whyRecommended", why_recommended, MAX_BOOK_SUMMARY_LEN),
    ] {
        if let Some(value) = value {
            check_len(field, value, max)?;
            if value.trim().is_empty() {
                return Err(AppError::BadRequest(format!("{field} cannot be empty")));
            }
        }
    }

    if let Some(page_count) = page_count {
        validate_book_metadata(page_count, None)?;
    }
    if let Some(purchase_link) = purchase_link {
        validate_book_metadata(None, purchase_link)?;
    }
    Ok(())
}

pub fn validate_book_review(rating: i16, writeup: &str) -> Result<(), AppError> {
    check_range("rating", rating, 1, 5)?;
    check_len("writeup", writeup, MAX_BOOK_WRITEUP_LEN)?;
    if writeup.trim().is_empty() {
        return Err(AppError::BadRequest("writeup is required".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        normalize_book_tags, validate_book_metadata, validate_book_recommendation,
        validate_book_review,
    };
    use crate::types::BookTag;

    #[test]
    fn book_recommendations_require_all_reader_facing_fields() {
        let error = validate_book_recommendation("A title", "An author", "", "A reason")
            .expect_err("empty summaries must be rejected");
        assert_eq!(error.to_string(), "bad request: summary is required");
    }

    #[test]
    fn book_reviews_require_a_one_to_five_rating_and_writeup() {
        assert!(validate_book_review(1, "Not for me.").is_ok());
        assert!(validate_book_review(5, "Excellent.").is_ok());
        assert!(validate_book_review(0, "Too low.").is_err());
        assert!(validate_book_review(6, "Too high.").is_err());
        assert!(validate_book_review(4, "   ").is_err());
    }

    #[test]
    fn book_tags_are_normalized_and_deduplicated() {
        let tags = normalize_book_tags(vec![
            BookTag {
                key: " Category ".into(),
                value: "Music".into(),
            },
            BookTag {
                key: "category".into(),
                value: "music".into(),
            },
            BookTag {
                key: "Style".into(),
                value: "Academic".into(),
            },
        ])
        .expect("valid tags should normalize");

        assert_eq!(
            tags,
            vec![
                BookTag {
                    key: "category".into(),
                    value: "music".into(),
                },
                BookTag {
                    key: "style".into(),
                    value: "academic".into(),
                },
            ]
        );
    }

    #[test]
    fn book_metadata_requires_sensible_pages_and_http_links() {
        assert!(
            validate_book_metadata(Some(320), Some("https://www.amazon.com/dp/example")).is_ok()
        );
        assert!(validate_book_metadata(Some(0), None).is_err());
        assert!(validate_book_metadata(None, Some("not a link")).is_err());
        assert!(validate_book_metadata(None, Some("javascript:alert(1)")).is_err());
    }
}
