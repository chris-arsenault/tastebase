# Tastebase

Personal tasting, recipe, and reading platform with Claude.ai MCP integration.
Evolved from scorchbook (hot sauce tracker). Data fully migrated, scorchbook decommissioned.

## Architecture

- **Frontend**: Vite + React SPA on `tastebase.ahara.io` (CloudFront + S3)
- **API**: Multiple Rust Lambdas behind shared ALB on `api.tastebase.ahara.io`
- **Database**: PostgreSQL on shared platform RDS
- **Media**: S3 bucket for images and voice recordings
- **Auth**: Cognito (shared platform pool), app-level JWT validation
- **AI Pipeline**: Bedrock Claude Haiku + Transcribe for async enrichment

## Backend Lambdas

| Crate | Trigger | Routes |
|-------|---------|--------|
| `tastings-api` | ALB HTTP | `/tastings*` |
| `recipes-api` | ALB HTTP | `/recipes*` |
| `books-api` | ALB HTTP | `/books*` |
| `mcp-server` | ALB HTTP | `/mcp`, `/.well-known/*` |
| `processing` | Lambda.Invoke (async) | N/A — event-driven |

Rust workspace in `backend/`. Shared code in `backend/shared/` (types, auth, db, media, errors).

The `processing` crate has internal modules:
- `llm` — Bedrock Claude invocation helpers (text + vision prompts)
- `extraction` — Image, ingredients, nutrition, and voice metric extraction
- `voice` — AWS Transcribe integration, tasting notes formatting

## Frontend

Vite + React SPA with three sections:
- **Tastings** — product tasting tracker with photo/voice capture and AI enrichment (ported from scorchbook)
- **Recipes** — recipe browser for Claude-saved recipes (RecipeList grid + RecipeDetail modal)
- **Books** — private Claude recommendations with reading state, 1–5 ratings, writeups, and opt-in public reviews

Section toggle in the header switches between them. Product type toggle (sauce/drink/all) only shows in tastings section.

## Build & Deploy

```bash
# Build all Lambdas
cd backend && cargo lambda build --release

# Run migrations
db-migrate

# Full deploy (build + migrate + terraform apply)
bash scripts/deploy.sh
```

## Database

PostgreSQL via shared RDS. Migrations in `db/migrations/`. Uses `sqlx` with
runtime query strings (not compile-time checked).

Schema: `users` + `cognito_users` (shared identity), `tastings` (tasting records),
`recipes` + `recipe_ingredients` + `recipe_steps` + `collections` (recipe system),
`recipe_reviews` + `recipe_images` (recipe media and reviews), and
`book_recommendations` (private reading state and reader-controlled public reviews).

## Platform Integration

Follows `~/src/ahara/INTEGRATION.md`. Registered in ahara-control and ahara-services.

## Key Decisions

- ALB routes by path prefix to separate Lambdas (no API Gateway)
- ALB jwt-validation for tastings/recipes write routes and all private book routes; MCP uses app-level auth for WWW-Authenticate header
- Tastings/recipes and explicitly published book reviews support public reads; private book recommendations remain owner-scoped
- Processing Lambda is invoked asynchronously for media enrichment (tastings + recipe reviews)
- S3 for media blobs (presigned upload URLs), PostgreSQL for structured data
- OG Lambda generates HTML with per-recipe OpenGraph tags; CloudFront caches at edge
- Path-based routing (not hash) for crawler compatibility

## Pre-commit CI check

**Run `make ci` before committing any change.** This runs the same lint, format, typecheck, and test steps as GitHub Actions. Do not commit if it fails.
