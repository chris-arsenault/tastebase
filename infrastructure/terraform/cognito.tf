# Cognito client for the Tastebase frontend (shared platform pool)
module "cognito_app" {
  source  = "git::https://github.com/chris-arsenault/ahara-tf-patterns.git//modules/cognito-app"
  name    = "${local.prefix}-app"
  cognito = module.ctx.cognito
}

# Cognito client for MCP/Claude.ai connector (confidential, with secret)
module "cognito_mcp" {
  source        = "git::https://github.com/chris-arsenault/ahara-tf-patterns.git//modules/cognito-app"
  name          = "${local.prefix}-mcp"
  callback_urls = ["https://claude.ai/api/mcp/auth_callback"]
  cognito       = module.ctx.cognito
}
