# Tastebase

Personal tasting, recipe, and reading platform with Claude.ai MCP integration.

## Architecture

- **Frontend**: Vite + React SPA — tasting tracker, recipe browser, and private reading list
- **Backend**: Rust Lambda functions behind shared ALB
  - `tastings-api` — tasting CRUD, media upload, async processing trigger
  - `recipes-api` — recipe CRUD with ingredients and steps
  - `books-api` — private book recommendations, owner reviews, and opt-in public sharing
  - `mcp-server` — MCP tools for recipes and personalized book recommendation rounds
  - `processing` — async enrichment pipeline (Bedrock image analysis, Transcribe voice, nutrition/ingredient extraction)
- **Database**: PostgreSQL (shared platform RDS)
- **Media**: S3 for images and voice recordings
- **Auth**: Cognito (shared platform pool)

## Book recommendations

Connect Claude.ai to the Tastebase MCP server, then ask for book recommendations. Claude can:

- save title, author, summary, and a personalized recommendation reason;
- read prior recommendations, reading status, ratings, and writeups before suggesting more books;
- refresh a repeated title without erasing existing feedback.

Tastebase has one authenticated owner. Recommendations, reading state, ratings, and writeups are private by default. After the owner saves a 1–5 rating and a non-empty writeup, they can explicitly publish that review for anonymous visitors to read.

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
