use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::error::AppError;

pub type VariationCondition = BTreeMap<String, String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariationOption {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariationGroup {
    pub id: String,
    pub label: String,
    pub default: String,
    pub options: Vec<VariationOption>,
}

fn invalid(message: impl Into<String>) -> AppError {
    AppError::BadRequest(message.into())
}

pub fn validate_groups(groups: &[VariationGroup]) -> Result<(), AppError> {
    let mut ids = HashSet::new();
    for group in groups {
        if group.id.trim().is_empty() || group.label.trim().is_empty() {
            return Err(invalid("variation groups require non-empty id and label"));
        }
        if !ids.insert(&group.id) {
            return Err(invalid(format!("duplicate variation group: {}", group.id)));
        }
        let mut options = HashSet::new();
        for option in &group.options {
            if option.id.trim().is_empty() || option.label.trim().is_empty() {
                return Err(invalid("variation options require non-empty id and label"));
            }
            if !options.insert(&option.id) {
                return Err(invalid(format!("duplicate option in group {}", group.id)));
            }
        }
        if !options.contains(&group.default) {
            return Err(invalid(format!(
                "group {} requires a default option from its options",
                group.id
            )));
        }
    }
    Ok(())
}

pub fn validate_condition(
    groups: &[VariationGroup],
    condition: &VariationCondition,
) -> Result<(), AppError> {
    for (group_id, option_id) in condition {
        let group = groups
            .iter()
            .find(|group| group.id == *group_id)
            .ok_or_else(|| invalid(format!("unknown variation group: {group_id}")))?;
        if !group.options.iter().any(|option| option.id == *option_id) {
            return Err(invalid(format!(
                "unknown option {option_id} for group {group_id}"
            )));
        }
    }
    Ok(())
}

/// Validate all content together, including rows retained by a partial update.
/// An ingredient referenced by a step must be present whenever that step is.
pub fn validate_content<'a>(
    groups: &[VariationGroup],
    ingredients: impl IntoIterator<Item = (&'a str, &'a VariationCondition)>,
    steps: impl IntoIterator<Item = (&'a str, &'a str, &'a VariationCondition)>,
) -> Result<(), AppError> {
    validate_groups(groups)?;
    let mut ingredient_map = BTreeMap::new();
    for (id, condition) in ingredients {
        validate_condition(groups, condition)?;
        if ingredient_map.insert(id, condition).is_some() {
            return Err(invalid(format!("duplicate ingredient id: {id}")));
        }
    }
    let mut step_ids = HashSet::new();
    for (id, content, condition) in steps {
        validate_condition(groups, condition)?;
        if !step_ids.insert(id) {
            return Err(invalid(format!("duplicate step id: {id}")));
        }
        // Match the frontend's {ingredient_id} token syntax.
        for suffix in content.split('{').skip(1) {
            let Some((token, _)) = suffix.split_once('}') else {
                continue;
            };
            if token.is_empty() || !token.chars().all(|c| c.is_alphanumeric() || c == '_') {
                continue;
            }
            // Legacy recipes may contain literal brace text. Only validate known tokens.
            if let Some(ingredient_condition) = ingredient_map.get(token)
                && ingredient_condition
                    .iter()
                    .any(|(key, value)| condition.get(key) != Some(value))
            {
                return Err(invalid(format!(
                    "step {id} references ingredient {token} that is hidden for some selections"
                )));
            }
        }
    }
    Ok(())
}

pub fn groups_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "array",
        "description": "Independent dropdowns, e.g. style (greek/skyr) and flavor (plain/peach/cherry). Each group needs a default option ID. Do not enumerate combinations. Omit to preserve on update; [] clears groups if no retained content uses them.",
        "items": {
            "type": "object", "additionalProperties": false,
            "required": ["id", "label", "default", "options"],
            "properties": {
                "id": {"type": "string"}, "label": {"type": "string"},
                "default": {"type": "string"},
                "options": {"type": "array", "minItems": 1, "items": {
                    "type": "object", "additionalProperties": false,
                    "required": ["id", "label"],
                    "properties": {"id": {"type": "string"}, "label": {"type": "string"}}
                }}
            }
        }
    })
}

pub fn condition_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object", "additionalProperties": {"type": "string"},
        "description": "Show this row only when ALL group IDs match the option IDs, e.g. {\"style\":\"skyr\"} or {\"style\":\"skyr\",\"flavor\":\"peach\"}. Omitted or {} means always show. Keep common rows once, give every row a unique id, and put alternatives at their intended position in the ordered list. Steps referencing conditional ingredients must include those conditions."
    })
}
