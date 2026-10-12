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
        include_str!("../../../db/migrations/016_publication_issues.sql"),
    ] {
        sqlx::raw_sql(migration).execute(&pool).await.unwrap();
    }

    let input: PublicationInput = serde_json::from_value(json!({
        "title":"Journal", "summary":"Summary", "why_recommended":"Reason", "publication_type":"journal",
        "cadence":{"label":"quarterly"}, "audience_level":"academic", "subscriptions":[],
        "tags":[{"key":"outlook", "value":"humanist"}, {"key":"outlook", "value":"rationalist"}]
    })).unwrap();
    let saved = publications::save(&pool, vec![input.clone()], false)
        .await
        .unwrap();
    let id = saved[0].id;
    let serialized = serde_json::to_value(&saved[0]).unwrap();
    assert_eq!(serialized["publicationType"], "journal");
    assert_eq!(serialized["whyRecommended"], "Reason");
    assert_eq!(serialized["tags"].as_array().unwrap().len(), 2);
    sqlx::query("UPDATE publication_recommendations SET status = 'cancelled', rating = 4, writeup = 'Useful' WHERE id = $1")
        .bind(id).execute(&pool).await.unwrap();
    let original = publications::get(&pool, id).await.unwrap();
    let mut changed_input = input.clone();
    changed_input.title = "JOURNAL".into();
    changed_input.summary = "Changed summary".into();
    changed_input.tags = Some(vec![]);
    let mut new_input = input.clone();
    new_input.title = "Should roll back".into();
    assert!(
        publications::save(&pool, vec![new_input, changed_input.clone()], false)
            .await
            .is_err()
    );
    let unchanged = publications::get(&pool, id).await.unwrap();
    assert_eq!(unchanged.summary, original.summary);
    assert_eq!(unchanged.tags.len(), 2);
    assert_eq!(unchanged.updated_at, original.updated_at);
    assert_eq!(publications::list(&pool, None).await.unwrap().len(), 1);
    let fields = json!({"summary":"Should not change"});
    assert!(
        publications::patch(&pool, id, fields.as_object().unwrap().clone(), false)
            .await
            .is_err()
    );
    let resaved = publications::save(&pool, vec![changed_input], true)
        .await
        .unwrap();
    assert_eq!(resaved[0].id, id);
    assert_eq!(resaved[0].rating, Some(4));
    assert_eq!(resaved[0].writeup, "Useful");
    assert_eq!(resaved[0].summary, "Changed summary");
    // Restore the corpus for the remaining assertions.
    publications::save(&pool, vec![input], true).await.unwrap();
    let fields = json!({"publisher":"New publisher", "subscriptions":[{
        "label":"Digital", "formats":["digital"], "access_model":"subscription",
        "price":{"amount":"12.34", "currency":"USD", "period":"year"}, "verified_at":"2026-09-12"
    }]});
    let patched = publications::patch(&pool, id, fields.as_object().unwrap().clone(), true)
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
    let cleared = publications::patch(&pool, id, fields.as_object().unwrap().clone(), true)
        .await
        .unwrap();
    assert!(cleared.publisher.is_none());
    assert!(cleared.tags.is_empty());
    assert_eq!(cleared.details.subscriptions.len(), 1);
    exercise_issue_reviews(&pool, id).await;
    exercise_book_reviews(&pool).await;
    sqlx::query("INSERT INTO book_recommendations (title, author, summary, why_recommended, is_public) VALUES ('Legacy private book', 'Author', 'Summary', 'Reason', false)")
        .execute(&pool).await.unwrap();
    assert_eq!(
        books::list_recommendations(&pool, None)
            .await
            .unwrap()
            .len(),
        2
    );
    pool.close().await;
    sqlx::raw_sql(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&admin)
        .await
        .unwrap();
}

fn review(rating: i16, writeup: &str) -> crate::shelf_reviews::ReviewInput {
    crate::shelf_reviews::ReviewInput {
        rating,
        writeup: writeup.into(),
    }
}

async fn exercise_issue_reviews(pool: &sqlx::PgPool, publication_id: uuid::Uuid) {
    use crate::publication_issues::{self as issues, IssueInput};
    let issue = issues::save(pool, publication_id, serde_json::from_value(json!({
        "label":"October 2026", "metadata":{"title":"Special issue", "volume":"12", "issue_number":"10", "publication_date":"2026-10-01", "page_count":96}
    })).unwrap()).await.unwrap();
    let second = issues::save(
        pool,
        publication_id,
        IssueInput {
            label: "November 2026".into(),
            metadata: Default::default(),
        },
    )
    .await
    .unwrap();
    let saved = issues::save_review(pool, issue.id, review(5, "<b>Excellent essays</b>"))
        .await
        .unwrap();
    assert_eq!(saved.writeup, "Excellent essays");
    assert!(
        issues::save_review(pool, issue.id, review(6, "Invalid"))
            .await
            .is_err()
    );
    assert!(
        issues::save_review(pool, issue.id, review(3, "<script>blank</script>"))
            .await
            .is_err()
    );
    assert!(matches!(
        issues::save_review(pool, uuid::Uuid::new_v4(), review(4, "Missing")).await,
        Err(crate::error::AppError::NotFound)
    ));
    let updated = issues::save(
        pool,
        publication_id,
        serde_json::from_value(json!({
            "label":"OCTOBER 2026", "metadata":{"title":null,"summary":"Essays"}
        }))
        .unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(updated.id, issue.id);
    assert_eq!(updated.rating, Some(5));
    assert_eq!(updated.writeup, saved.writeup);
    assert!(updated.metadata.title.is_none());
    assert_eq!(updated.metadata.volume.as_deref(), Some("12"));
    let listed = issues::list(pool, publication_id).await.unwrap();
    assert_eq!(listed.len(), 2);
    assert!(
        listed
            .iter()
            .find(|i| i.id == second.id)
            .unwrap()
            .rating
            .is_none()
    );
    let publication = publications::get(pool, publication_id).await.unwrap();
    assert_eq!(publication.rating, Some(4));
    assert_eq!(publication.writeup, "Useful");
    assert!(matches!(
        publication.status,
        crate::publication_types::PublicationStatus::Cancelled
    ));
    assert_eq!(publication.issues.len(), 2);
    assert_eq!(
        publication
            .issues
            .iter()
            .find(|i| i.id == issue.id)
            .unwrap()
            .metadata
            .issue_number
            .as_deref(),
        Some("10")
    );
    publications::save_review(pool, publication_id, review(3, "Journal overall"))
        .await
        .unwrap();
    let fields = json!({"publication_type":"magazine"});
    publications::patch(
        pool,
        publication_id,
        fields.as_object().unwrap().clone(),
        true,
    )
    .await
    .unwrap();
    assert!(
        publications::save_review(pool, publication_id, review(3, "Must be per issue"))
            .await
            .is_err()
    );
    assert!(matches!(
        issues::save(
            pool,
            uuid::Uuid::new_v4(),
            IssueInput {
                label: "Missing".into(),
                metadata: Default::default()
            }
        )
        .await,
        Err(crate::error::AppError::NotFound)
    ));
}

fn book_input(title: &str, summary: &str) -> books::PreparedBookRecommendation {
    books::PreparedBookRecommendation {
        title: title.into(),
        author: "Author".into(),
        summary: summary.into(),
        why_recommended: "Original reason".into(),
        page_count: Some(120),
        purchase_link: None,
        tags: Some(vec![crate::types::BookTag {
            key: "fit".into(),
            value: "high".into(),
        }]),
    }
}

async fn exercise_book_reviews(pool: &sqlx::PgPool) {
    let saved =
        books::save_recommendations(pool, vec![book_input("Book", "Original summary")], false)
            .await
            .unwrap();
    let id = saved[0].id;
    let reviewed = crate::shelf_reviews::save_book_review(pool, id, review(4, "Enjoyed this"))
        .await
        .unwrap();
    assert!(matches!(reviewed.status, crate::types::BookStatus::Read));
    assert!(reviewed.read_at.is_some());
    assert_eq!(reviewed.summary, "Original summary");
    assert!(
        books::save_recommendations(
            pool,
            vec![book_input("Rollback", "New"), book_input("BOOK", "Changed")],
            false
        )
        .await
        .is_err()
    );
    let unchanged = books::get_recommendation(pool, id).await.unwrap().unwrap();
    assert_eq!(unchanged.summary, "Original summary");
    assert_eq!(unchanged.updated_at, reviewed.updated_at);
    assert_eq!(
        books::list_recommendations(pool, None).await.unwrap().len(),
        1
    );
    let updated = books::save_recommendations(pool, vec![book_input("Book", "Changed")], true)
        .await
        .unwrap();
    assert_eq!(updated[0].summary, "Changed");
    assert_eq!(updated[0].rating, Some(4));
    assert_eq!(updated[0].writeup, "Enjoyed this");
    sqlx::query(
        "UPDATE book_recommendations SET status = 'did_not_finish', read_at = NULL WHERE id = $1",
    )
    .bind(id)
    .execute(pool)
    .await
    .unwrap();
    let unfinished = crate::shelf_reviews::save_book_review(pool, id, review(2, "Stopped halfway"))
        .await
        .unwrap();
    assert!(matches!(
        unfinished.status,
        crate::types::BookStatus::DidNotFinish
    ));
    assert!(unfinished.read_at.is_none());
    assert!(
        crate::shelf_reviews::save_book_review(pool, id, review(0, "Invalid"))
            .await
            .is_err()
    );
    assert!(matches!(
        crate::shelf_reviews::save_book_review(pool, uuid::Uuid::new_v4(), review(4, "Missing"))
            .await,
        Err(crate::error::AppError::NotFound)
    ));
}
