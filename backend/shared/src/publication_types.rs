use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::types::BookTag;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum PublicationStatus {
    Recommended,
    Subscribed,
    Cancelled,
    NotInterested,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicationType {
    Magazine,
    Journal,
    Review,
    Newsletter,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AudienceLevel {
    General,
    InformedGeneralist,
    Professional,
    AcademicAdjacent,
    Academic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutlookRelationship {
    Aligned,
    Compatible,
    Neutral,
    Contrast,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionFormat {
    Print,
    Digital,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessModel {
    Subscription,
    Membership,
    Free,
    Institutional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PricePeriod {
    Month,
    Year,
    Issue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PriceType {
    Standard,
    Introductory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Cadence {
    pub label: String,
    #[serde(alias = "issues_per_year")]
    pub issues_per_year: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EditorialHome {
    pub country: Option<String>,
    pub city: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Outlook {
    pub summary: String,
    pub relationship: OutlookRelationship,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SubscriptionPrice {
    // Decimal serializes as a string so prices retain their exact value.
    pub amount: Decimal,
    pub currency: String,
    pub period: PricePeriod,
    #[serde(alias = "price_type")]
    pub price_type: Option<PriceType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SubscriptionOption {
    pub label: String,
    pub formats: Vec<SubscriptionFormat>,
    pub region: Option<String>,
    #[serde(alias = "access_model")]
    pub access_model: AccessModel,
    pub price: Option<SubscriptionPrice>,
    pub url: Option<String>,
    #[serde(alias = "verified_at")]
    pub verified_at: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublicationDetails {
    pub homepage: Option<String>,
    #[serde(alias = "publication_type")]
    pub publication_type: PublicationType,
    pub cadence: Cadence,
    #[serde(alias = "audience_level")]
    pub audience_level: AudienceLevel,
    #[serde(alias = "editorial_home")]
    pub editorial_home: Option<EditorialHome>,
    pub subscriptions: Vec<SubscriptionOption>,
    pub outlook: Option<Outlook>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PublicationRecommendation {
    pub id: Uuid,
    pub title: String,
    pub publisher: Option<String>,
    pub summary: String,
    pub why_recommended: String,
    #[serde(flatten)]
    pub details: Json<PublicationDetails>,
    pub tags: Json<Vec<BookTag>>,
    pub status: PublicationStatus,
    pub rating: Option<i16>,
    pub writeup: String,
    #[serde(with = "time::serde::rfc3339")]
    pub recommended_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicationInput {
    pub title: String,
    pub publisher: Option<String>,
    pub summary: String,
    #[serde(alias = "why_recommended")]
    pub why_recommended: String,
    #[serde(flatten)]
    pub details: PublicationDetails,
    #[serde(default)]
    pub tags: Option<Vec<BookTag>>,
}
