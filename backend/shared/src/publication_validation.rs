use crate::{
    error::AppError,
    publication_types::{PublicationInput, SubscriptionOption},
};
use rust_decimal::Decimal;

fn text(value: &mut String, field: &str, required: bool, max: usize) -> Result<(), AppError> {
    *value = crate::sanitize::clean(value).trim().to_owned();
    if (required && value.is_empty()) || value.len() > max {
        return Err(AppError::BadRequest(format!(
            "{field} must contain {} to {max} bytes",
            usize::from(required)
        )));
    }
    Ok(())
}

fn optional(value: &mut Option<String>, field: &str, max: usize) -> Result<(), AppError> {
    if let Some(value) = value {
        text(value, field, true, max)?;
    }
    Ok(())
}

fn url(value: &Option<String>) -> Result<(), AppError> {
    crate::validate::validate_book_metadata(None, value.as_deref())
}

fn subscription(option: &mut SubscriptionOption) -> Result<(), AppError> {
    text(&mut option.label, "subscription label", true, 1000)?;
    optional(&mut option.region, "region", 120)?;
    optional(&mut option.notes, "subscription notes", 6000)?;
    optional(&mut option.url, "subscription URL", 2000)?;
    url(&option.url)?;
    if option.formats.is_empty() || option.formats.len() > 2 {
        return Err(AppError::BadRequest(
            "formats must contain print, digital, or both".into(),
        ));
    }
    let formats = serde_json::to_value(&option.formats).expect("formats serialize");
    if option.formats.len() == 2 && formats[0] == formats[1] {
        return Err(AppError::BadRequest("formats must be unique".into()));
    }
    if let Some(price) = &mut option.price {
        if price.amount < Decimal::ZERO {
            return Err(AppError::BadRequest(
                "price amount cannot be negative".into(),
            ));
        }
        price.currency = price.currency.trim().to_ascii_uppercase();
        if price.currency.len() != 3 || !price.currency.bytes().all(|b| b.is_ascii_uppercase()) {
            return Err(AppError::BadRequest(
                "currency must be a three-letter currency code".into(),
            ));
        }
    }
    if let Some(date) = &option.verified_at {
        let format =
            time::format_description::parse("[year]-[month]-[day]").expect("valid date format");
        if date.len() != 10 || time::Date::parse(date, &format).is_err() {
            return Err(AppError::BadRequest(
                "verifiedAt must be a valid YYYY-MM-DD date".into(),
            ));
        }
    }
    Ok(())
}

pub fn prepare(mut input: PublicationInput) -> Result<PublicationInput, AppError> {
    text(&mut input.title, "title", true, 1000)?;
    optional(&mut input.publisher, "publisher", 1000)?;
    text(&mut input.summary, "summary", true, 6000)?;
    text(&mut input.why_recommended, "whyRecommended", true, 6000)?;
    let details = &mut input.details;
    optional(&mut details.homepage, "homepage", 2000)?;
    url(&details.homepage)?;
    text(&mut details.cadence.label, "cadence label", true, 120)?;
    if details.cadence.issues_per_year.is_some_and(|n| n <= 0) {
        return Err(AppError::BadRequest(
            "issuesPerYear must be positive".into(),
        ));
    }
    if let Some(home) = &mut details.editorial_home {
        optional(&mut home.country, "country", 120)?;
        optional(&mut home.city, "city", 120)?;
        if home.country.is_none() && home.city.is_none() {
            return Err(AppError::BadRequest(
                "editorialHome requires a city or country".into(),
            ));
        }
    }
    if let Some(outlook) = &mut details.outlook {
        text(&mut outlook.summary, "outlook summary", true, 6000)?;
    }
    if details.subscriptions.len() > 20 {
        return Err(AppError::BadRequest(
            "at most 20 subscription options are allowed".into(),
        ));
    }
    for option in &mut details.subscriptions {
        subscription(option)?;
    }
    input.tags = input
        .tags
        .map(crate::validate::normalize_book_tags)
        .transpose()?;
    Ok(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    pub fn example() -> serde_json::Value {
        json!({"title":"Example", "publisher":"Publisher", "summary":"Summary", "why_recommended":"A useful contrast",
            "publication_type":"journal", "cadence":{"label":"monthly", "issues_per_year":12},
            "audience_level":"academic-adjacent", "subscriptions":[{"label":"Annual print",
                "formats":["print"], "access_model":"subscription", "price":{"amount":"49.95", "currency":"GBP", "period":"year"},
                "verified_at":"2026-09-12"}], "outlook":{"summary":"Humanist", "relationship":"contrast"},
            "tags":[{"key":"fit", "value":"high"}, {"key":"outlook", "value":"humanist"}]})
    }

    #[test]
    fn accepts_mcp_names_and_serializes_shared_camel_case_fields() {
        let prepared = prepare(serde_json::from_value(example()).unwrap()).unwrap();
        let json = serde_json::to_value(prepared).unwrap();
        assert_eq!(json["whyRecommended"], "A useful contrast");
        assert_eq!(json["publicationType"], "journal");
        assert_eq!(json["subscriptions"][0]["price"]["amount"], "49.95");
        assert_eq!(json["outlook"]["relationship"], "contrast");
    }

    #[test]
    fn rejects_invalid_dates_prices_links_and_cadence() {
        for (pointer, bad) in [
            ("/subscriptions/0/verified_at", json!("2026-02-30")),
            ("/subscriptions/0/price/amount", json!("-1")),
            ("/subscriptions/0/price/currency", json!("dollars")),
            ("/cadence/issues_per_year", json!(0)),
        ] {
            let mut value = example();
            *value.pointer_mut(pointer).unwrap() = bad;
            assert!(
                prepare(serde_json::from_value(value).unwrap()).is_err(),
                "{pointer}"
            );
        }
        let mut value = example();
        value["homepage"] = json!("javascript:alert(1)");
        assert!(prepare(serde_json::from_value(value).unwrap()).is_err());
    }

    #[test]
    fn patch_preserves_omissions_clears_optional_fields_and_rejects_feedback() {
        let current =
            serde_json::to_value(prepare(serde_json::from_value(example()).unwrap()).unwrap())
                .unwrap();
        let patch = json!({"publisher": null, "outlook": null});
        let updated =
            crate::publications::merge_patch(current.clone(), patch.as_object().unwrap().clone())
                .unwrap();
        assert!(updated.publisher.is_none());
        assert!(updated.details.outlook.is_none());
        assert_eq!(updated.details.subscriptions.len(), 1);
        let forbidden = json!({"rating": 5});
        assert!(
            crate::publications::merge_patch(current, forbidden.as_object().unwrap().clone())
                .is_err()
        );
    }
}
