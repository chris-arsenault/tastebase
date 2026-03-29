use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use sqlx::PgPool;

#[derive(Clone)]
struct AppState {
    db: PgPool,
    entry_js: String,
    entry_css: String,
    site_url: String,
}

#[derive(sqlx::FromRow)]
struct RecipeMeta {
    title: String,
    description: Option<String>,
    image_url: Option<String>,
}

fn slugify(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

async fn lookup_recipe(db: &PgPool, slug: &str) -> Option<RecipeMeta> {
    let recipes: Vec<RecipeMeta> = sqlx::query_as(
        "SELECT r.title, r.description, ri.image_url
         FROM recipes r
         LEFT JOIN LATERAL (
           SELECT image_url FROM recipe_images WHERE recipe_id = r.id
           ORDER BY sort_order, created_at LIMIT 1
         ) ri ON true",
    )
    .fetch_all(db)
    .await
    .ok()?;

    recipes.into_iter().find(|r| slugify(&r.title) == slug)
}

fn render_html(state: &AppState, og: OgTags) -> String {
    let title = html_escape(&og.title);
    let desc = html_escape(&og.description);
    let image = html_escape(&og.image);
    let url = html_escape(&og.url);

    [
        "<!DOCTYPE html><html lang=\"en\"><head>",
        "<meta charset=\"utf-8\" />",
        "<link rel=\"icon\" type=\"image/svg+xml\" href=\"/favicon.svg\" />",
        "<link rel=\"icon\" type=\"image/png\" sizes=\"32x32\" href=\"/favicon-32x32.png\" />",
        "<link rel=\"icon\" type=\"image/png\" sizes=\"16x16\" href=\"/favicon-16x16.png\" />",
        "<link rel=\"apple-touch-icon\" href=\"/apple-touch-icon.png\" />",
        "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />",
        "<meta name=\"theme-color\" content=\"#d63831\" />",
        &format!("<title>{title} | Tastebase</title>"),
        &format!("<meta name=\"description\" content=\"{desc}\" />"),
        &format!("<meta property=\"og:title\" content=\"{title}\" />"),
        &format!("<meta property=\"og:description\" content=\"{desc}\" />"),
        &format!("<meta property=\"og:image\" content=\"{image}\" />"),
        &format!("<meta property=\"og:url\" content=\"{url}\" />"),
        &format!("<meta property=\"og:type\" content=\"{}\" />", og.og_type),
        "<meta property=\"og:site_name\" content=\"Tastebase\" />",
        "<meta name=\"twitter:card\" content=\"summary_large_image\" />",
        &format!("<meta name=\"twitter:title\" content=\"{title}\" />"),
        &format!("<meta name=\"twitter:description\" content=\"{desc}\" />"),
        &format!("<meta name=\"twitter:image\" content=\"{image}\" />"),
        &format!(
            "<link rel=\"stylesheet\" crossorigin href=\"{}\" />",
            state.entry_css
        ),
        "</head><body><div id=\"root\"></div>",
        "<script src=\"/config.js\"></script>",
        &format!(
            "<script type=\"module\" crossorigin src=\"{}\"></script>",
            state.entry_js
        ),
        "</body></html>",
    ]
    .join("\n")
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

struct OgTags {
    title: String,
    description: String,
    image: String,
    url: String,
    og_type: &'static str,
}

fn default_og(site_url: &str, path: &str) -> OgTags {
    OgTags {
        title: "Tastebase".into(),
        description: "Culinary platform — track tastings, save recipes, review dishes.".into(),
        image: format!("{site_url}/tastebase-social.png"),
        url: format!("{site_url}{path}"),
        og_type: "website",
    }
}

async fn handle_request(
    State(state): State<AppState>,
    req: axum::extract::Request,
) -> axum::response::Response {
    let path = req.uri().path().to_string();

    tracing::info!(path = %path, "og-server request");

    // Parse /recipes/:slug
    let og = if let Some(slug) = path.strip_prefix("/recipes/") {
        if let Some(meta) = lookup_recipe(&state.db, slug).await {
            let image = meta
                .image_url
                .unwrap_or_else(|| format!("{}/tastebase-social.png", state.site_url));
            OgTags {
                title: meta.title,
                description: meta.description.unwrap_or_default(),
                image,
                url: format!("{}{}", state.site_url, path),
                og_type: "article",
            }
        } else {
            default_og(&state.site_url, &path)
        }
    } else {
        default_og(&state.site_url, &path)
    };

    let cache_control = if path.starts_with("/recipes/") {
        "public, s-maxage=86400, max-age=0"
    } else {
        "public, s-maxage=3600, max-age=0"
    };

    (
        StatusCode::OK,
        [
            ("content-type", "text/html; charset=utf-8"),
            ("cache-control", cache_control),
        ],
        render_html(&state, og),
    )
        .into_response()
}

fn router(state: AppState) -> Router {
    Router::new()
        .fallback(get(handle_request))
        .with_state(state)
}

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    shared::init_tracing();

    let db = shared::db::connect().await;
    let entry_js = std::env::var("ENTRY_JS").unwrap_or_else(|_| "/assets/index.js".into());
    let entry_css = std::env::var("ENTRY_CSS").unwrap_or_else(|_| "/assets/index.css".into());
    let site_url =
        std::env::var("SITE_URL").unwrap_or_else(|_| "https://tastebase.ahara.io".into());
    tracing::info!(entry_js = %entry_js, entry_css = %entry_css, "og-server starting");

    let state = AppState {
        db,
        entry_js,
        entry_css,
        site_url,
    };

    let app = router(state);
    lambda_http::run(app).await
}
