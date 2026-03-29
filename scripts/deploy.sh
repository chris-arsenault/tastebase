#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TF_DIR="${ROOT_DIR}/infrastructure/terraform"

STATE_BUCKET="${STATE_BUCKET:-tfstate-559098897826}"
STATE_REGION="${STATE_REGION:-us-east-1}"

# Build Rust Lambdas
echo "Building Lambdas..."
cd "${ROOT_DIR}/backend"
cargo lambda build --release
cd "${ROOT_DIR}"

# Build frontend
echo "Building frontend..."
cd "${ROOT_DIR}/frontend"
if [ -f "package.json" ]; then
  pnpm install --frozen-lockfile
  pnpm run build
fi
cd "${ROOT_DIR}"

# Run migrations
echo "Running migrations..."
db-migrate

# Deploy infrastructure
echo "Deploying infrastructure..."
terraform -chdir="${TF_DIR}" init -reconfigure \
  -backend-config="bucket=${STATE_BUCKET}" \
  -backend-config="region=${STATE_REGION}" \
  -backend-config="use_lockfile=true"

terraform -chdir="${TF_DIR}" apply -auto-approve

echo ""
echo "=== Deploy complete ==="
echo "Frontend: $(terraform -chdir="${TF_DIR}" output -raw frontend_url)"
echo "API:      $(terraform -chdir="${TF_DIR}" output -raw api_url)"
echo ""
echo "=== MCP Connector (Claude.ai Settings > Connectors) ==="
echo "Server URL:    $(terraform -chdir="${TF_DIR}" output -raw mcp_server_url)"
echo "Client ID:     $(terraform -chdir="${TF_DIR}" output -raw mcp_client_id)"
echo "Client Secret: $(terraform -chdir="${TF_DIR}" output -raw mcp_client_secret)"
