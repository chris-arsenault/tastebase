use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

// -- Users --

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CognitoUser {
    pub cognito_sub: String,
    pub user_id: Uuid,
    pub email: String,
    #[serde(with = "time::serde::rfc3339")]
    pub linked_at: OffsetDateTime,
}

// -- Tastings --

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "product_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ProductType {
    Sauce,
    Drink,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "processing_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ProcessingStatus {
    Pending,
    ImageExtracted,
    IngredientsExtracted,
    NutritionExtracted,
    VoiceTranscribed,
    VoiceExtracted,
    NotesFormatted,
    BackExtracted,
    Complete,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NutritionFacts {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serving_size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calories: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_fat: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sodium: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_carbs: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sugars: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protein: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Tasting {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub product_type: Option<ProductType>,
    pub name: String,
    pub maker: String,
    pub date: time::Date,
    pub score: Option<i16>,
    pub style: String,
    pub heat_user: Option<i16>,
    pub heat_vendor: Option<i16>,
    pub refreshing: Option<i16>,
    pub sweet: Option<i16>,
    pub tasting_notes_user: String,
    pub tasting_notes_vendor: String,
    pub product_url: String,
    pub image_url: Option<String>,
    pub image_key: Option<String>,
    pub ingredients_image_url: Option<String>,
    pub ingredients_image_key: Option<String>,
    pub nutrition_image_url: Option<String>,
    pub nutrition_image_key: Option<String>,
    pub nutrition_facts: Option<sqlx::types::Json<NutritionFacts>>,
    pub ingredients: Option<Vec<String>>,
    pub voice_key: Option<String>,
    pub voice_transcript: Option<String>,
    pub status: ProcessingStatus,
    pub processing_error: Option<String>,
    pub needs_attention: bool,
    pub attention_reason: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

/// Public-facing tasting (strips internal fields like voice_key).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TastingPublic {
    pub id: Uuid,
    pub product_type: Option<ProductType>,
    pub name: String,
    pub maker: String,
    pub date: time::Date,
    pub score: Option<i16>,
    pub style: String,
    pub heat_user: Option<i16>,
    pub heat_vendor: Option<i16>,
    pub refreshing: Option<i16>,
    pub sweet: Option<i16>,
    pub tasting_notes_user: String,
    pub tasting_notes_vendor: String,
    pub product_url: String,
    pub image_url: Option<String>,
    pub ingredients_image_url: Option<String>,
    pub nutrition_image_url: Option<String>,
    pub nutrition_facts: Option<NutritionFacts>,
    pub ingredients: Option<Vec<String>>,
    pub status: ProcessingStatus,
    pub processing_error: Option<String>,
    pub needs_attention: bool,
    pub attention_reason: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl From<Tasting> for TastingPublic {
    fn from(t: Tasting) -> Self {
        Self {
            id: t.id,
            product_type: t.product_type,
            name: t.name,
            maker: t.maker,
            date: t.date,
            score: t.score,
            style: t.style,
            heat_user: t.heat_user,
            heat_vendor: t.heat_vendor,
            refreshing: t.refreshing,
            sweet: t.sweet,
            tasting_notes_user: t.tasting_notes_user,
            tasting_notes_vendor: t.tasting_notes_vendor,
            product_url: t.product_url,
            image_url: t.image_url,
            ingredients_image_url: t.ingredients_image_url,
            nutrition_image_url: t.nutrition_image_url,
            nutrition_facts: t.nutrition_facts.map(|j| j.0),
            ingredients: t.ingredients,
            status: t.status,
            processing_error: t.processing_error,
            needs_attention: t.needs_attention,
            attention_reason: t.attention_reason,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }
}

// -- Recipes --

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "recipe_source", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum RecipeSource {
    Claude,
    Manual,
    Import,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "unit_type")]
#[serde(rename_all = "lowercase")]
pub enum UnitType {
    #[sqlx(rename = "g")]
    G,
    #[sqlx(rename = "kg")]
    Kg,
    #[sqlx(rename = "ml")]
    Ml,
    #[sqlx(rename = "l")]
    L,
    #[sqlx(rename = "tsp")]
    Tsp,
    #[sqlx(rename = "tbsp")]
    Tbsp,
    #[sqlx(rename = "cup")]
    Cup,
    #[sqlx(rename = "fl_oz")]
    FlOz,
    #[sqlx(rename = "oz")]
    Oz,
    #[sqlx(rename = "lb")]
    Lb,
    #[sqlx(rename = "pinch")]
    Pinch,
    #[sqlx(rename = "piece")]
    Piece,
    #[sqlx(rename = "")]
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Recipe {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub base_servings: i32,
    pub notes: Option<String>,
    pub source: RecipeSource,
    pub source_meta: Option<sqlx::types::Json<serde_json::Value>>,
    pub version: i32,
    pub version_group_id: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RecipeWithThumb {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub base_servings: i32,
    pub notes: Option<String>,
    pub source: RecipeSource,
    pub source_meta: Option<sqlx::types::Json<serde_json::Value>>,
    pub version: i32,
    pub version_group_id: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub thumbnail_url: Option<String>,
    pub latest_score: Option<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RecipeIngredient {
    pub id: Uuid,
    pub recipe_id: Uuid,
    pub widget_id: String,
    pub name: String,
    pub short_name: String,
    pub amount: rust_decimal::Decimal,
    pub unit: UnitType,
    pub sort_order: i32,
    pub linked_recipe_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RecipeStep {
    pub id: Uuid,
    pub recipe_id: Uuid,
    pub widget_id: String,
    pub title: String,
    pub content: String,
    pub timer_seconds: Option<i32>,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RecipeReview {
    pub id: Uuid,
    pub recipe_id: Uuid,
    pub voice_key: Option<String>,
    pub voice_transcript: Option<String>,
    pub notes: String,
    pub score: Option<i16>,
    pub status: ProcessingStatus,
    pub processing_error: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RecipeImage {
    pub id: Uuid,
    pub recipe_id: Uuid,
    pub image_url: String,
    pub image_key: String,
    pub caption: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

/// Full recipe with nested ingredients, steps, reviews, and images.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeFull {
    #[serde(flatten)]
    pub recipe: Recipe,
    pub ingredients: Vec<RecipeIngredient>,
    pub steps: Vec<RecipeStep>,
    pub reviews: Vec<RecipeReview>,
    pub images: Vec<RecipeImage>,
}

// -- Auth --

#[derive(Debug, Clone)]
pub struct UserContext {
    pub sub: String,
    pub email: Option<String>,
    pub user_id: Option<Uuid>,
}
