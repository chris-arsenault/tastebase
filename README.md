# Tastebase

Culinary platform for tracking tastings and storing recipes, with Claude.ai MCP integration.

## Architecture

- **Frontend**: Vite + React SPA — tasting tracker with photo/voice capture + recipe browser
- **Backend**: Rust Lambda functions behind shared ALB
  - `tastings-api` — tasting CRUD, media upload, async processing trigger
  - `recipes-api` — recipe CRUD with ingredients and steps
  - `mcp-server` — MCP protocol for Claude.ai `save_recipe` tool
  - `processing` — async enrichment pipeline (Bedrock image analysis, Transcribe voice, nutrition/ingredient extraction)
- **Database**: PostgreSQL (shared platform RDS)
- **Media**: S3 for images and voice recordings
- **Auth**: Cognito (shared platform pool)

## URLs

- App: https://tastebase.ahara.io
- API: https://api.tastebase.ahara.io

## Local Development

```bash
# Frontend
cd frontend
npm install
cp .env.example .env   # configure API URL and Cognito
npm run dev

# Backend
cd backend
cargo lambda build --release --arm64
```

## Deploy

```bash
bash scripts/deploy.sh
```

Builds frontend and backend, runs database migrations, and applies Terraform.

## License

MIT
