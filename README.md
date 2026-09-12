# Tastebase

Personal tasting, recipe, and reading platform with Claude.ai and ChatGPT MCP integration.

## Architecture

- **Frontend**: Vite + React SPA — tasting tracker, recipe browser, and public bookshelf
- **Backend**: Rust Lambda functions behind shared ALB
  - `tastings-api` — tasting CRUD, media upload, async processing trigger
  - `recipes-api` — recipe CRUD with ingredients and steps
  - `books-api` — public books and publications with authenticated owner feedback
  - `mcp-server` — MCP tools for recipes and personalized book and publication recommendations
  - `processing` — async enrichment pipeline (Bedrock image analysis, Transcribe voice, nutrition/ingredient extraction)
- **Database**: PostgreSQL (shared platform RDS)
- **Media**: S3 for images and voice recordings
- **Auth**: Cognito (shared platform pool)

## Book recommendations

Connect Claude.ai to the Tastebase MCP server, then ask for book recommendations. Claude can:

- save title, author, summary, recommendation reason, page count, reusable key/value tags, and a purchase link;
- read prior recommendations, metadata, reading status, ratings, and writeups before suggesting more books;
- fetch the existing tag corpus before classifying books, keeping categories stable while allowing useful new values;
- patch recommendation metadata or refresh a repeated title without erasing existing feedback.

Tastebase has one authenticated owner. All books and publications, including recommendation reasons, status, ratings and writeups, are publicly readable. Only the owner can update status or save reviews. “Reviewed only” means a saved 1–5 rating and non-empty writeup; it is independent of reading or subscription status.

The bookshelf shows all items by default. Visitors can combine type, status, reviewed-only and tag filters, and sort by recommendation date, title, author/publisher or book page count. Items without page counts sort last in both directions.

## Publications

Publications share book field names for title, summary, recommendation reason, tags, rating, writeup and timestamps. They add publisher, homepage, publication type, cadence, audience level, editorial home, subscription options and outlook. The publication status is `recommended`, `subscribed`, `cancelled` or `not_interested`; saving a review does not change subscription status.

`publication_recommendations` holds identity and feedback in relational columns, with validated Rust structures for publication-specific JSONB metadata. Subscription prices use decimal strings, for example `"49.95"`, with a three-letter currency code and month/year/issue period. Region, access model, print/digital formats, introductory pricing, link, verification date and notes stay attached to each option. Unknown prices and verification dates remain absent. There is no currency conversion or automatic claim that a recorded price is current.

MCP tools mirror books: `list_publication_recommendations`, `save_publication_recommendations`, `patch_publication_recommendation` and `get_publication_tag_corpus`. Inputs accept snake_case names, while responses use the same camelCase convention as books. Re-saving the same case-insensitive title and publisher refreshes metadata without erasing status or feedback. An omitted publisher is a distinct identity; use a patch by ID when correcting title or publisher. Patches preserve omitted fields, clear optional fields with null, and replace supplied nested objects and arrays as complete values. Use `[]` to clear tags or subscription options.

Both tag-corpus tools return the combined vocabulary with `bookCount`, `publicationCount` and `itemCount`. Reuse `category`, `topic`, `style`, `fit` and `outlook` keys and existing values whenever accurate. Multiple outlook values are allowed. `fit=high` can coexist with `outlook.relationship=contrast`: usefulness and intellectual alignment are different dimensions.

Public HTTP reads use `/books/public` and `/books/publications`. Owner updates use `/books/{id}/status`, `/books/{id}/review`, `/books/publications/{id}/status` and `/books/publications/{id}/review`. MCP writes remain authenticated.

Migration `015` makes existing and future books public and adds publication storage. The legacy `is_public` book column is retained for rolling deployment compatibility; new code does not use it to restrict listing. Apply the migration before deploying the new Lambdas and frontend. Reverting this publication policy requires an explicit visibility decision; the migration does not retain a private/public history.

## Validation

Run `make ci` for lint, formatting, TypeScript, Rust unit tests, frontend tests and Terraform formatting. Frontend tests cover reviewed filtering and public/owner rendering. `db/tests/public_shelf.sql` checks migration behavior in a disposable PostgreSQL database. The ignored Rust test `publication_database_round_trip` exercises the actual SQL queries and metadata updates:

```bash
cd backend
TASTEBASE_TEST_DATABASE_URL=postgres://postgres@localhost/tastebase_test cargo test -p shared --lib publication_database_round_trip -- --ignored
```

Use a disposable local database. If it requires a secret, supply the URL through environment variables; in the Sulion environment, run the command through `with-cred --`.

## URLs

- App: https://tastebase.ahara.io
- API: https://api.tastebase.ahara.io

## Local Development

```bash
# Frontend
cd frontend
pnpm install
cp .env.example .env   # configure API URL and Cognito
pnpm dev

# Backend
cd backend
cargo lambda build --release
```

## Deploy

```bash
bash scripts/deploy.sh
```

Builds frontend and backend, runs database migrations, and applies Terraform.

## License

MIT
