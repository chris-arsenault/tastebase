use shared::types::{NutritionFacts, ProductType};
use uuid::Uuid;

use crate::llm::{
    build_vision_prompt, clamp_score_i16, invoke_claude, normalize_number, parse_json_from_text,
};
use crate::Ctx;

/// Result of image analysis for product identification.
pub struct ImageExtraction {
    pub product_type: Option<ProductType>,
    pub name: Option<String>,
    pub maker: Option<String>,
    pub style: Option<String>,
}

/// Result of ingredients label extraction.
pub struct IngredientsExtraction {
    pub ingredients: Option<Vec<String>>,
}

/// Result of nutrition facts panel extraction.
pub struct NutritionExtraction {
    pub nutrition_facts: Option<NutritionFacts>,
}

/// Analyze a product image to identify type, name, maker, style.
pub async fn run_image_extraction(
    ctx: &Ctx,
    image_base64: &str,
    image_mime_type: &str,
) -> Result<ImageExtraction, Box<dyn std::error::Error + Send + Sync>> {
    let instructions = r#"Analyze this product image and extract information.

First, determine the product type:
- "sauce" = hot sauce, pepper sauce, chili sauce, salsa
- "drink" = non-alcoholic beverage (kombucha, juice, soda, sparkling water, tea, coffee, NA beer, mocktail, smoothie, sports drink)

Return JSON with these keys:
{
  "product_type": "sauce" or "drink",
  "name": "product name",
  "maker": "brand/manufacturer",
  "style": "style category",
  "keywords": ["relevant", "search", "keywords"]
}

Style examples:
- For sauces: Habanero, Ghost Pepper, Chipotle, Cayenne, Carolina Reaper, Sriracha, Buffalo
- For drinks: Kombucha, NA Beer, Mocktail, Juice, Soda, Sparkling Water, Tea, Coffee, Smoothie, Sports Drink

Use null for any field you cannot determine."#;

    let payload = build_vision_prompt(instructions, image_base64, image_mime_type);
    let text = invoke_claude(&ctx.bedrock, &ctx.bedrock_model_id, &payload).await?;
    let parsed = match parse_json_from_text(&text) {
        Some(p) => p,
        None => return Ok(ImageExtraction { product_type: None, name: None, maker: None, style: None }),
    };

    let product_type = match parsed.get("product_type").and_then(|v| v.as_str()) {
        Some("drink") => Some(ProductType::Drink),
        _ => Some(ProductType::Sauce),
    };

    Ok(ImageExtraction {
        product_type,
        name: parsed.get("name").and_then(|v| v.as_str()).map(String::from),
        maker: parsed.get("maker").and_then(|v| v.as_str()).map(String::from),
        style: parsed.get("style").and_then(|v| v.as_str()).map(String::from),
    })
}

/// Extract ingredients list from a label image.
pub async fn run_ingredients_extraction(
    ctx: &Ctx,
    image_base64: &str,
    image_mime_type: &str,
) -> Result<IngredientsExtraction, Box<dyn std::error::Error + Send + Sync>> {
    let instructions = r#"Extract the ingredients list from this hot sauce label image.

Return JSON with exactly these keys:
{
  "ingredients": ["ingredient1", "ingredient2", ...]
}

Guidelines:
- Find the ingredients list (usually starts with "Ingredients:")
- List each ingredient separately, in order shown on label
- Simplify ingredient names: "Red Habanero Peppers" not "Red Habanero Peppers (Capsicum chinense)"
- If no ingredients list visible, set ingredients to empty array"#;

    let payload = build_vision_prompt(instructions, image_base64, image_mime_type);
    let text = invoke_claude(&ctx.bedrock, &ctx.bedrock_model_id, &payload).await?;
    let parsed = match parse_json_from_text(&text) {
        Some(p) => p,
        None => return Ok(IngredientsExtraction { ingredients: None }),
    };

    let ingredients = parsed
        .get("ingredients")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    let s = item.as_str()?.trim();
                    if s.is_empty() { None } else { Some(s.to_string()) }
                })
                .collect::<Vec<_>>()
        });

    Ok(IngredientsExtraction { ingredients })
}

/// Extract nutrition facts from a label image.
pub async fn run_nutrition_extraction(
    ctx: &Ctx,
    image_base64: &str,
    image_mime_type: &str,
) -> Result<NutritionExtraction, Box<dyn std::error::Error + Send + Sync>> {
    let instructions = r#"Extract nutrition facts from this hot sauce label image.

Return JSON with exactly these keys:
{
  "nutrition_facts": {
    "serving_size": "e.g. 1 tsp (5g)",
    "calories": number or null,
    "total_fat": "e.g. 0g",
    "sodium": "e.g. 190mg",
    "total_carbs": "e.g. 1g",
    "sugars": "e.g. 0g",
    "protein": "e.g. 0g"
  }
}

Guidelines:
- Look for "Nutrition Facts" panel - extract serving size and per-serving values
- Include units with values (mg, g, etc.) except calories which is just a number
- Use null for any nutrition value not visible or legible
- If no nutrition panel visible, set nutrition_facts to null"#;

    let payload = build_vision_prompt(instructions, image_base64, image_mime_type);
    let text = invoke_claude(&ctx.bedrock, &ctx.bedrock_model_id, &payload).await?;
    let parsed = match parse_json_from_text(&text) {
        Some(p) => p,
        None => return Ok(NutritionExtraction { nutrition_facts: None }),
    };

    let nutrition_facts = parsed.get("nutrition_facts").and_then(|raw| {
        let obj = raw.as_object()?;
        Some(NutritionFacts {
            serving_size: obj.get("serving_size").and_then(|v| v.as_str()).map(String::from),
            calories: obj.get("calories").and_then(|v| normalize_number(v)).map(|n| n as i32),
            total_fat: obj.get("total_fat").and_then(|v| v.as_str()).map(String::from),
            sodium: obj.get("sodium").and_then(|v| v.as_str()).map(String::from),
            total_carbs: obj.get("total_carbs").and_then(|v| v.as_str()).map(String::from),
            sugars: obj.get("sugars").and_then(|v| v.as_str()).map(String::from),
            protein: obj.get("protein").and_then(|v| v.as_str()).map(String::from),
        })
    });

    Ok(NutritionExtraction { nutrition_facts })
}

// -- Database enrichment helpers --

/// Apply image extraction results to the database, merging without overwriting
/// user-provided values.
pub async fn apply_image_enrichment(
    db: &sqlx::PgPool,
    id: Uuid,
    extraction: &ImageExtraction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Fetch current values to respect user-provided data
    let row: Option<CurrentImageFields> = sqlx::query_as(
        "SELECT product_type, name, maker, style FROM tastings WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(db)
    .await?;

    let current = match row {
        Some(r) => r,
        None => return Ok(()),
    };

    let product_type = if current.product_type.is_none() {
        extraction.product_type
    } else {
        current.product_type
    };

    let name = merge_string_field(&current.name, extraction.name.as_deref());
    let maker = merge_string_field(&current.maker, extraction.maker.as_deref());
    let style = merge_string_field(&current.style, extraction.style.as_deref());

    sqlx::query(
        "UPDATE tastings SET product_type = $2, name = $3, maker = $4, style = $5, updated_at = now() WHERE id = $1",
    )
    .bind(id)
    .bind(product_type)
    .bind(&name)
    .bind(&maker)
    .bind(&style)
    .execute(db)
    .await?;

    Ok(())
}

#[derive(sqlx::FromRow)]
struct CurrentImageFields {
    product_type: Option<ProductType>,
    name: String,
    maker: String,
    style: String,
}

/// Apply ingredients extraction to the database.
pub async fn apply_ingredients_enrichment(
    db: &sqlx::PgPool,
    id: Uuid,
    extraction: &IngredientsExtraction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(ref ingredients) = extraction.ingredients {
        // Only set if current is NULL
        let result = sqlx::query(
            "UPDATE tastings SET ingredients = $2, updated_at = now() WHERE id = $1 AND ingredients IS NULL",
        )
        .bind(id)
        .bind(ingredients)
        .execute(db)
        .await?;

        tracing::info!(
            record_id = %id,
            ingredient_count = ingredients.len(),
            rows_affected = result.rows_affected(),
            "ingredients enrichment applied"
        );
    }
    Ok(())
}

/// Apply nutrition extraction to the database.
pub async fn apply_nutrition_enrichment(
    db: &sqlx::PgPool,
    id: Uuid,
    extraction: &NutritionExtraction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(ref nf) = extraction.nutrition_facts {
        let json_value = serde_json::to_value(nf)?;
        // Only set if current is NULL
        sqlx::query(
            "UPDATE tastings SET nutrition_facts = $2, updated_at = now() WHERE id = $1 AND nutrition_facts IS NULL",
        )
        .bind(id)
        .bind(json_value)
        .execute(db)
        .await?;
    }
    Ok(())
}

/// Merge a string field: keep the current value if it's non-empty, otherwise use the candidate.
fn merge_string_field(current: &str, candidate: Option<&str>) -> String {
    if !current.trim().is_empty() {
        return current.to_string();
    }
    candidate
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(current)
        .to_string()
}

// -- Voice metric helpers --

/// Extract score and heat_user from a voice transcript via Bedrock.
pub async fn extract_voice_metrics(
    ctx: &Ctx,
    transcript: &str,
) -> Result<VoiceMetrics, Box<dyn std::error::Error + Send + Sync>> {
    if transcript.is_empty() {
        return Ok(VoiceMetrics { score: None, heat_user: None });
    }

    let instructions =
        "Extract user tasting details from this transcript. Return JSON only with keys: score, heat_user. Use null for unknowns.";
    let payload = crate::llm::build_text_prompt(instructions, transcript);
    let text = invoke_claude(&ctx.bedrock, &ctx.bedrock_model_id, &payload).await?;
    let parsed = match parse_json_from_text(&text) {
        Some(p) => p,
        None => return Ok(VoiceMetrics { score: None, heat_user: None }),
    };

    Ok(VoiceMetrics {
        score: clamp_score_i16(
            parsed.get("score").and_then(|v| normalize_number(v)),
        ),
        heat_user: clamp_score_i16(
            parsed.get("heat_user").and_then(|v| normalize_number(v)),
        ),
    })
}

pub struct VoiceMetrics {
    pub score: Option<i16>,
    pub heat_user: Option<i16>,
}

/// Apply voice metrics to the database. If force is true, overwrite existing values.
pub async fn apply_voice_metrics(
    db: &sqlx::PgPool,
    id: Uuid,
    metrics: &VoiceMetrics,
    force: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if force {
        sqlx::query(
            "UPDATE tastings SET score = COALESCE($2, score), heat_user = COALESCE($3, heat_user), updated_at = now() WHERE id = $1",
        )
        .bind(id)
        .bind(metrics.score)
        .bind(metrics.heat_user)
        .execute(db)
        .await?;
    } else {
        // Only set if current is NULL
        if let Some(score) = metrics.score {
            sqlx::query(
                "UPDATE tastings SET score = $2, updated_at = now() WHERE id = $1 AND score IS NULL",
            )
            .bind(id)
            .bind(score)
            .execute(db)
            .await?;
        }
        if let Some(heat_user) = metrics.heat_user {
            sqlx::query(
                "UPDATE tastings SET heat_user = $2, updated_at = now() WHERE id = $1 AND heat_user IS NULL",
            )
            .bind(id)
            .bind(heat_user)
            .execute(db)
            .await?;
        }
    }
    Ok(())
}
