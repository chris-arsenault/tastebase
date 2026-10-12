# Tastebase

Personal tasting, recipe, and reading platform with Claude.ai and ChatGPT MCP integration.

## Architecture

- **Frontend**: Vite + React SPA — tasting tracker, recipe browser, and public bookshelf
- **Backend**: Rust Lambda functions behind shared ALB
  - `tastings-api` — tasting CRUD, media upload, async processing trigger
  - `recipes-api` — recipe CRUD with ingredients and steps
  - `books-api` — public books and publications with authenticated owner feedback
  - `mcp-server` — MCP tools for recipes, book/publication recommendations, and owner reviews
  - `processing` — async enrichment pipeline (Bedrock image analysis, Transcribe voice, nutrition/ingredient extraction)
- **Database**: PostgreSQL (shared platform RDS)
- **Media**: S3 for images and voice recordings
- **Auth**: Cognito (shared platform pool)

## Book recommendations

Connect Claude.ai to the Tastebase MCP server, then ask for book recommendations. Claude can:

- save title, author, summary, recommendation reason, page count, reusable key/value tags, and a purchase link;
- read prior recommendations, metadata, reading status, ratings, and writeups before suggesting more books;
- fetch the existing tag corpus before classifying books, keeping categories stable while allowing useful new values;
- save your book reviews from a conversation using `save_book_review`;
- correct recommendation metadata when explicitly requested with `update=true`.

Tastebase has one authenticated owner. All books and publications, including recommendation reasons, status, issue metadata, ratings and writeups, are publicly readable. The owner can update status or save reviews through the website or an authenticated MCP agent acting on their behalf. “Reviewed only” means a saved 1–5 rating and non-empty writeup, either on the item or one of its issues; it is independent of reading or subscription status.

For agents: the initial recommendation records why an item was suggested. Updating that recommendation is normally incorrect when the owner discusses their reading experience. Save a review instead, using the owner's stated opinions and a rating they supplied or agreed to. Do not invent their feedback. Read the existing record before saving: book reviews and reviews of a particular issue replace the previous review for that same book or issue. Saving a book review marks it `read`, except `did_not_finish` remains unchanged.

Both `save_*_recommendations` tools and both `patch_*_recommendation` tools accept the top-level optional boolean `update`, default `false`. Saving an existing identity without `update=true` returns a tool error and rolls back the whole batch, including any new items in it. Patching also requires `update=true`. Use this opt-in only for an explicitly requested correction to recommendation metadata, never as a routine response to a duplicate or a review request. Reading/subscription status and feedback survive recommendation corrections.

The bookshelf shows all items by default. Visitors can combine type, status, reviewed-only and tag filters, and sort by recommendation date, title, author/publisher or book page count. Items without page counts sort last in both directions.

## Publications

Publications share book field names for title, summary, recommendation reason, tags, rating, writeup and timestamps. They add publisher, homepage, publication type, cadence, audience level, editorial home, subscription options and outlook. The publication status is `recommended`, `subscribed`, `cancelled` or `not_interested`; saving a review does not change subscription status.

`publication_recommendations` holds identity and feedback in relational columns, with validated Rust structures for publication-specific JSONB metadata. Subscription prices use decimal strings, for example `"49.95"`, with a three-letter currency code and month/year/issue period. Region, access model, print/digital formats, introductory pricing, link, verification date and notes stay attached to each option. Unknown prices and verification dates remain absent. There is no currency conversion or automatic claim that a recorded price is current.

MCP recommendation tools mirror books: `list_publication_recommendations`, `save_publication_recommendations`, `patch_publication_recommendation` and `get_publication_tag_corpus`. Inputs accept snake_case names, while responses use the same camelCase convention as books. With `update=true`, re-saving the same case-insensitive title and publisher refreshes metadata without erasing status, feedback or issues. An omitted publisher is a distinct identity; use a patch by ID when correcting title or publisher. Patches preserve omitted fields, clear optional fields with null, and replace supplied nested objects and arrays as complete values. Use `[]` to clear tags or subscription options.

Magazine reviews are per issue. Other publications can also use issues, or use `save_publication_review` for publication-wide feedback. New magazine-wide reviews are rejected; existing publication-wide feedback is preserved because its issue is unknown. Issue reviews never change subscription status or the original recommendation.

An agent saving a magazine review should:

1. Call `list_publication_recommendations` to find the publication ID, then `list_publication_issues` with `publication_id` to find existing issues and reviews.
2. Reuse the existing issue ID, or call `save_publication_issue` with `publication_id`, a stable `label` such as `October 2026`, and optional `metadata`. The publication and case-insensitive label identify an issue. Reuse the same label to set metadata later; omitted fields survive and explicit null clears them. Metadata can be saved before any review exists.
3. Call `save_publication_issue_review` with `issue_id`, a 1–5 `rating` and a non-empty `writeup` grounded in the owner's conversation. This replaces only that issue's feedback.

Issue metadata supports `title`, `volume`, `issue_number` (a string, including combined issues), `publication_date` (an exact `YYYY-MM-DD` date), `page_count`, `url` and `summary`. Leave unknown values absent. Use the label for a month or season if the exact publication date is unknown. For example, creating an issue:

```json
{
  "publication_id": "<publication UUID>",
  "label": "October 2026",
  "metadata": { "volume": "12", "issue_number": "10", "page_count": 96 }
}
```

The public bookshelf includes issue metadata and reviews under each publication. `list_publication_recommendations` includes the same issues and feedback so future recommendations can use them.

Both tag-corpus tools return the combined vocabulary with `bookCount`, `publicationCount` and `itemCount`. Reuse `category`, `topic`, `style`, `fit` and `outlook` keys and existing values whenever accurate. Multiple outlook values are allowed. `fit=high` can coexist with `outlook.relationship=contrast`: usefulness and intellectual alignment are different dimensions.

Public HTTP reads use `/books/public` and `/books/publications`. Owner updates use `/books/{id}/status`, `/books/{id}/review`, `/books/publications/{id}/status` and `/books/publications/{id}/review`. MCP writes remain authenticated.

Migration `015` makes existing and future books public and adds publication storage. The legacy `is_public` book column is retained for rolling deployment compatibility; new code does not use it to restrict listing. Apply the migration before deploying the new Lambdas and frontend. Reverting this publication policy requires an explicit visibility decision; the migration does not retain a private/public history.

Migration `016` adds `publication_issues` with separate issue metadata and feedback. Apply it before deploying these Lambdas. It preserves existing recommendations and publication-level reviews.

## Validation

Run `make ci` for lint, formatting, TypeScript, Rust unit tests, frontend tests and Terraform formatting. `cd backend && cargo test -p mcp-server --bin mcp-server` checks MCP tool schemas and input defaults. Frontend tests cover reviewed filtering, issue feedback and public/owner rendering. `db/tests/public_shelf.sql` checks migration behavior in a disposable PostgreSQL database. The ignored Rust test `publication_database_round_trip` exercises recommendation update guards, batch rollback, issue metadata merging and book/issue review persistence:

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
