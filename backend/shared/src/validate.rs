use crate::error::AppError;

const MAX_NAME_LEN: usize = 1000;
const MAX_NOTES_LEN: usize = 4000;
const MAX_URL_LEN: usize = 2000;
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

fn check_optional_range(field: &str, value: Option<i16>, min: i16, max: i16) -> Result<(), AppError> {
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
    if let Some(v) = name { check_len("name", v, MAX_NAME_LEN)?; }
    if let Some(v) = maker { check_len("maker", v, MAX_NAME_LEN)?; }
    if let Some(v) = style { check_len("style", v, MAX_NAME_LEN)?; }
    if let Some(v) = tasting_notes_user { check_len("tastingNotesUser", v, MAX_NOTES_LEN)?; }
    if let Some(v) = tasting_notes_vendor { check_len("tastingNotesVendor", v, MAX_NOTES_LEN)?; }
    if let Some(v) = product_url { check_len("productUrl", v, MAX_URL_LEN)?; }
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
            && !v.is_empty() {
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
    if let Some(v) = description { check_len("description", v, MAX_NOTES_LEN)?; }
    if let Some(v) = notes { check_len("notes", v, MAX_NOTES_LEN)?; }
    if base_servings < 1 {
        return Err(AppError::BadRequest("baseServings must be at least 1".into()));
    }
    Ok(())
}
