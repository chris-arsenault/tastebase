//! Runs only against an explicitly supplied disposable PostgreSQL database.
use crate::{books, publication_types::PublicationInput, publications};
use serde_json::json;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use std::str::FromStr;

#[tokio::test]
#[ignore = "requires TASTEBASE_TEST_DATABASE_URL pointing to a disposable PostgreSQL database"]
async fn publication_database_round_trip() {
    let url =
        std::env::var("TASTEBASE_TEST_DATABASE_URL").expect("disposable database URL required");
    let options = PgConnectOptions::from_str(&url).unwrap();
    let schema = format!("shelf_test_{}", uuid::Uuid::new_v4().simple());
    let search_path = format!("SET search_path TO {schema}, public");
    let admin = PgPoolOptions::new()
        .max_connections(1)
        .connect_with(options.clone())
        .await
        .unwrap();
    sqlx::raw_sql(&format!("CREATE SCHEMA {schema}"))
        .execute(&admin)
        .await
        .unwrap();
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .after_connect(move |connection, _| {
            let sql = search_path.clone();
            Box::pin(async move {
                sqlx::query(&sql).execute(connection).await?;
                Ok(())
            })
        })
        .connect_with(options)
        .await
        .unwrap();
    sqlx::raw_sql("CREATE TABLE users (id UUID PRIMARY KEY)")
        .execute(&pool)
        .await
        .unwrap();
    for migration in [
        include_str!("../../../db/migrations/011_create_book_recommendations.sql"),
        include_str!("../../../db/migrations/012_single_owner_books.sql"),
        include_str!("../../../db/migrations/013_drop_book_user_id.sql"),
        include_str!("../../../db/migrations/014_book_metadata_and_tags.sql"),
        include_str!("../../../db/migrations/015_public_books_and_publications.sql"),
    ] {
        sqlx::raw_sql(migration).execute(&pool).await.unwrap();
    }

    let input: PublicationInput = serde_json::from_value(json!({
        "title":"Journal", "summary":"Summary", "why_recommended":"Reason", "publication_type":"journal",
        "cadence":{"label":"quarterly"}, "audience_level":"academic", "subscriptions":[],
        "tags":[{"key":"outlook", "value":"humanist"}, {"key":"outlook", "value":"rationalist"}]
    })).unwrap();
    let saved = publications::save(&pool, vec![input.clone()])
        .await
        .unwrap();
    let id = saved[0].id;
    let serialized = serde_json::to_value(&saved[0]).unwrap();
    assert_eq!(serialized["publicationType"], "journal");
    assert_eq!(serialized["whyRecommended"], "Reason");
    assert_eq!(serialized["tags"].as_array().unwrap().len(), 2);
    sqlx::query("UPDATE publication_recommendations SET status = 'cancelled', rating = 4, writeup = 'Useful' WHERE id = $1")
        .bind(id).execute(&pool).await.unwrap();
    let resaved = publications::save(&pool, vec![input]).await.unwrap();
    assert_eq!(resaved[0].id, id);
    assert_eq!(resaved[0].rating, Some(4));
    assert_eq!(resaved[0].writeup, "Useful");
    let fields = json!({"publisher":"New publisher", "subscriptions":[{
        "label":"Digital", "formats":["digital"], "access_model":"subscription",
        "price":{"amount":"12.34", "currency":"USD", "period":"year"}, "verified_at":"2026-09-12"
    }]});
    let patched = publications::patch(&pool, id, fields.as_object().unwrap().clone())
        .await
        .unwrap();
    assert_eq!(patched.tags.len(), 2);
    assert_eq!(patched.rating, Some(4));
    assert_eq!(patched.publisher.as_deref(), Some("New publisher"));
    assert_eq!(
        patched.details.subscriptions[0]
            .price
            .as_ref()
            .unwrap()
            .amount
            .to_string(),
        "12.34"
    );
    let corpus = books::tag_corpus(&pool).await.unwrap();
    assert_eq!(corpus.len(), 2);
    assert_eq!(corpus[0].publication_count, 1);
    assert_eq!(corpus[0].book_count, 0);
    assert_eq!(corpus[0].item_count, 1);
    assert_eq!(
        publications::list(
            &pool,
            Some(crate::publication_types::PublicationStatus::Cancelled)
        )
        .await
        .unwrap()
        .len(),
        1
    );
    let fields = json!({"publisher":null, "tags":[]});
    let cleared = publications::patch(&pool, id, fields.as_object().unwrap().clone())
        .await
        .unwrap();
    assert!(cleared.publisher.is_none());
    assert!(cleared.tags.is_empty());
    assert_eq!(cleared.details.subscriptions.len(), 1);
    sqlx::query("INSERT INTO book_recommendations (title, author, summary, why_recommended, is_public) VALUES ('Legacy private book', 'Author', 'Summary', 'Reason', false)")
        .execute(&pool).await.unwrap();
    assert_eq!(
        books::list_recommendations(&pool, None)
            .await
            .unwrap()
            .len(),
        1
    );
    pool.close().await;
    sqlx::raw_sql(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&admin)
        .await
        .unwrap();
}
