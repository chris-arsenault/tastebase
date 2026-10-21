use super::*;
use serde_json::json;

fn yogurt() -> serde_json::Value {
    json!({
        "title": "Yogurt", "description": "Culture and flavor choices", "base_servings": 4,
        "notes": "Fixture notes.",
        "variation_groups": [
            {"id":"style", "label":"Style", "default":"greek", "options":[
                {"id":"greek", "label":"Greek"}, {"id":"skyr", "label":"Skyr"}
            ]},
            {"id":"flavor", "label":"Flavor", "default":"plain", "options":[
                {"id":"plain", "label":"Plain"}, {"id":"peach", "label":"Peach"},
                {"id":"cherry", "label":"Cherry"}
            ]}
        ],
        "ingredients": [
            {"id":"milk", "name":"Milk", "short_name":"milk", "amount":1, "unit":"l"},
            {"id":"peaches", "name":"Peaches", "short_name":"peaches", "amount":100, "unit":"g", "when":{"flavor":"peach"}}
        ],
        "steps": [
            {"id":"heat", "title":"Heat", "content":"Heat {milk}."},
            {"id":"greek", "title":"Greek culture", "content":"Use Greek culture instructions.", "when":{"style":"greek"}},
            {"id":"skyr", "title":"Skyr culture", "content":"Use Skyr culture instructions.", "when":{"style":"skyr"}},
            {"id":"peach", "title":"Peach flavor", "content":"Add {peaches}.", "when":{"flavor":"peach"}}
        ]
    })
}

#[test]
fn schemas_expose_groups_and_conditions_on_both_write_tools() {
    for tool in [save_recipe_tool_def(), update_recipe_tool_def()] {
        let properties = &tool["inputSchema"]["properties"];
        assert_eq!(properties["variation_groups"]["type"], "array");
        for rows in ["ingredients", "steps"] {
            assert_eq!(
                properties[rows]["items"]["properties"]["when"]["type"],
                "object"
            );
        }
    }
    let mut legacy = yogurt();
    legacy.as_object_mut().unwrap().remove("variation_groups");
    let params: SaveRecipeParams = serde_json::from_value(legacy).unwrap();
    assert!(params.variation_groups.is_empty());
    assert!(params.ingredients[0].when.is_empty());
    let omitted: UpdateRecipeParams = serde_json::from_value(json!({"recipe_id":"x"})).unwrap();
    let cleared: UpdateRecipeParams =
        serde_json::from_value(json!({"recipe_id":"x", "variation_groups":[]})).unwrap();
    assert!(omitted.variation_groups.is_none());
    assert!(cleared.variation_groups.unwrap().is_empty());
}

#[test]
fn rejects_invalid_groups_conditions_and_hidden_ingredient_references() {
    let input: SaveRecipeParams = serde_json::from_value(yogurt()).unwrap();
    let validate = |input: &SaveRecipeParams| {
        recipe_variations::validate_content(
            &input.variation_groups,
            input
                .ingredients
                .iter()
                .map(|ing| (ing.id.as_str(), &ing.when)),
            input
                .steps
                .iter()
                .map(|step| (step.id.as_str(), step.content.as_str(), &step.when)),
        )
    };
    validate(&input).unwrap();
    for broken in [
        json!({"style":"missing"}),
        json!({"unknown":"peach"}),
        json!({}),
    ] {
        let mut data = yogurt();
        data["steps"][3]["when"] = broken;
        assert!(validate(&serde_json::from_value(data).unwrap()).is_err());
    }
    let mut data = yogurt();
    data["variation_groups"][0]["default"] = json!("missing");
    assert!(validate(&serde_json::from_value(data).unwrap()).is_err());
    let mut data = yogurt();
    data["variation_groups"][0]["options"][1]["id"] = json!("greek");
    assert!(validate(&serde_json::from_value(data).unwrap()).is_err());
    let mut data = yogurt();
    data["variation_groups"][1]["id"] = json!("style");
    assert!(validate(&serde_json::from_value(data).unwrap()).is_err());
}

async fn update(
    state: &AppState,
    user: &shared::types::UserContext,
    args: serde_json::Value,
) -> Result<serde_json::Value, AppError> {
    update_recipe(state, user, serde_json::from_value(args).unwrap()).await
}

#[tokio::test]
#[ignore = "requires TASTEBASE_TEST_DATABASE_URL pointing to a disposable PostgreSQL database"]
async fn recipe_variations_database_round_trip() {
    let url = std::env::var("TASTEBASE_TEST_DATABASE_URL").expect("disposable database required");
    let db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .unwrap();
    for migration in [
        include_str!("../../../db/migrations/001_create_users.sql"),
        include_str!("../../../db/migrations/002_create_tastings.sql"),
        include_str!("../../../db/migrations/003_create_recipes.sql"),
        include_str!("../../../db/migrations/004_ingredient_short_name.sql"),
        include_str!("../../../db/migrations/005_nullable_user_id.sql"),
        include_str!("../../../db/migrations/006_recipe_reviews_and_images.sql"),
        include_str!("../../../db/migrations/007_drop_cover_image_url.sql"),
        include_str!("../../../db/migrations/008_drop_image_sort_order.sql"),
        include_str!("../../../db/migrations/009_linked_recipe_ingredients.sql"),
        include_str!("../../../db/migrations/010_recipe_versioning.sql"),
    ] {
        sqlx::raw_sql(migration).execute(&db).await.unwrap();
    }
    // An existing recipe must acquire unconditional defaults during migration.
    let legacy_id = Uuid::new_v4();
    sqlx::query("INSERT INTO recipes (id, title, version_group_id) VALUES ($1, 'Legacy', $1)")
        .bind(legacy_id)
        .execute(&db)
        .await
        .unwrap();
    sqlx::query("INSERT INTO recipe_ingredients (recipe_id, widget_id, name, amount) VALUES ($1, 'milk', 'Milk', 1)")
        .bind(legacy_id).execute(&db).await.unwrap();
    sqlx::query("INSERT INTO recipe_steps (recipe_id, widget_id, title, content) VALUES ($1, 'heat', 'Heat', 'Heat milk')")
        .bind(legacy_id).execute(&db).await.unwrap();
    sqlx::raw_sql(include_str!(
        "../../../db/migrations/017_recipe_variations.sql"
    ))
    .execute(&db)
    .await
    .unwrap();
    let s3_config = aws_config_for_test();
    let state = AppState {
        db,
        s3: s3_config,
        media_bucket: "unused".into(),
        telemetry: shared::telemetry_config("test"),
    };
    let user = shared::types::UserContext {
        sub: "owner".into(),
        email: None,
        user_id: None,
    };
    let legacy = recipe_read::get(&state, legacy_id).await.unwrap();
    assert_eq!(legacy["variationGroups"], json!([]));
    assert_eq!(legacy["ingredients"][0]["when"], json!({}));
    assert_eq!(legacy["steps"][0]["when"], json!({}));
    let saved = save_recipe(&state, &user, serde_json::from_value(yogurt()).unwrap())
        .await
        .unwrap();
    let id: Uuid = serde_json::from_value(saved["recipe_id"].clone()).unwrap();
    let original = recipe_read::get(&state, id).await.unwrap();
    assert_eq!(original["variationGroups"], yogurt()["variation_groups"]);
    assert_eq!(
        original["ingredients"][1]["when"],
        json!({"flavor":"peach"})
    );
    update(&state, &user, json!({"recipe_id":id, "notes":"Updated"}))
        .await
        .unwrap();
    let changed = recipe_read::get(&state, id).await.unwrap();
    assert_eq!(changed["variationGroups"], original["variationGroups"]);
    assert_eq!(changed["steps"], original["steps"]);
    // Removing groups would orphan retained rows; the entire update must roll back.
    assert!(
        update(
            &state,
            &user,
            json!({"recipe_id":id, "variation_groups":[], "notes":"Must roll back"})
        )
        .await
        .is_err()
    );
    assert_eq!(
        recipe_read::get(&state, id).await.unwrap()["notes"],
        "Updated"
    );
    let next = update(
        &state,
        &user,
        json!({"recipe_id":id, "new_version":true, "title":"Yogurt v2"}),
    )
    .await
    .unwrap();
    let next_id = serde_json::from_value(next["recipe_id"].clone()).unwrap();
    let next_recipe = recipe_read::get(&state, next_id).await.unwrap();
    assert_eq!(next_recipe["version"], 2);
    assert_eq!(next_recipe["variationGroups"], original["variationGroups"]);
    assert_eq!(
        next_recipe["ingredients"][1]["when"],
        json!({"flavor":"peach"})
    );
    assert_eq!(next_recipe["steps"][2]["when"], json!({"style":"skyr"}));
    assert_eq!(
        recipe_read::get(&state, id).await.unwrap()["title"],
        "Yogurt"
    );
    assert!(
        update(
            &state,
            &user,
            json!({"recipe_id":id, "new_version":true, "variation_groups":[]})
        )
        .await
        .is_err()
    );
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM recipes WHERE version_group_id = $1")
        .bind(id)
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(count, 2);
    update(
        &state,
        &user,
        json!({"recipe_id":id, "variation_groups":[], "ingredients":[], "steps":[]}),
    )
    .await
    .unwrap();
    assert_eq!(
        recipe_read::get(&state, next_id).await.unwrap()["variationGroups"],
        json!([])
    );
}

fn aws_config_for_test() -> aws_sdk_s3::Client {
    aws_sdk_s3::Client::from_conf(
        aws_sdk_s3::config::Builder::new()
            .behavior_version_latest()
            .region(aws_sdk_s3::config::Region::new("us-east-1"))
            .build(),
    )
}
