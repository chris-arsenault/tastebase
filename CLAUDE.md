# Tastebase

General culinary platform — tasting tracker + recipe storage with Claude.ai MCP integration.
Evolved from scorchbook (hot sauce tracker). Scorchbook remains untouched at
`~/src/websites/apps/scorchbook/` as a read-only reference.

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
| `mcp-server` | ALB HTTP | `/mcp`, `/.well-known/*` |
| `processing` | Lambda.Invoke (async) | N/A — event-driven |

Rust workspace in `backend/`. Shared code in `backend/shared/` (types, auth, db, media, errors).

The `processing` crate has internal modules:
- `llm` — Bedrock Claude invocation helpers (text + vision prompts)
- `extraction` — Image, ingredients, nutrition, and voice metric extraction
- `voice` — AWS Transcribe integration, tasting notes formatting

## Frontend

Vite + React SPA with two sections:
- **Tastings** — product tasting tracker with photo/voice capture and AI enrichment (ported from scorchbook)
- **Recipes** — recipe browser for Claude-saved recipes (RecipeList grid + RecipeDetail modal)

Section toggle in the header switches between the two. Product type toggle (sauce/drink/all) only shows in tastings section.

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
`recipes` + `recipe_ingredients` + `recipe_steps` + `collections` (recipe system).

## Platform Integration

Follows `~/src/platform/INTEGRATION.md`. Registration files in `docs/`:
- `docs/platform-control-registration.tf` — managed-project module for platform-control repo
- `docs/platform-services-registration.md` — migration_projects entry for platform-services repo

Cognito resource server defines `recipe/read` and `recipe/write` scopes for MCP OAuth.

## Data Migration

`scripts/migrate-dynamo.sh` exports scorchbook DynamoDB data to SQL seed file
(`db/migrations/seed/001_migrate_dynamo.sql`). One-time operation for cutover.

## Key Decisions

- ALB routes by path prefix to separate Lambdas (no API Gateway)
- App-level JWT validation (not ALB jwt-validation) because endpoints mix public reads + authenticated writes
- MCP server needs selective auth per MCP protocol (initialize is unauthenticated)
- Processing Lambda is invoked asynchronously by tastings-api for media enrichment
- S3 for media blobs, PostgreSQL for structured data only
- Tastings are public-read, write requires auth (same as scorchbook, not the spec's private-by-default)
