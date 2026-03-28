# Cognito client for the Tastebase frontend (shared platform pool)
resource "aws_cognito_user_pool_client" "app" {
  name         = "${local.prefix}-app"
  user_pool_id = local.cognito_pool_id

  explicit_auth_flows = [
    "ALLOW_USER_PASSWORD_AUTH",
    "ALLOW_REFRESH_TOKEN_AUTH",
    "ALLOW_USER_SRP_AUTH"
  ]
}

# Cognito resource server for MCP API scopes
resource "aws_cognito_resource_server" "api" {
  identifier   = "https://api.tastebase.ahara.io"
  name         = "${local.prefix}-api"
  user_pool_id = local.cognito_pool_id

  scope {
    scope_name        = "recipe.write"
    scope_description = "Create and update recipes"
  }

  scope {
    scope_name        = "recipe.read"
    scope_description = "Read recipes"
  }
}

# Cognito client for MCP/Claude.ai connector (confidential, with secret)
resource "aws_cognito_user_pool_client" "mcp" {
  name         = "${local.prefix}-mcp"
  user_pool_id = local.cognito_pool_id

  generate_secret = true

  explicit_auth_flows = [
    "ALLOW_USER_PASSWORD_AUTH",
    "ALLOW_REFRESH_TOKEN_AUTH",
    "ALLOW_USER_SRP_AUTH"
  ]

  allowed_oauth_flows                  = ["code"]
  allowed_oauth_flows_user_pool_client = true
  allowed_oauth_scopes = [
    "openid",
    "profile",
    "email",
    "${aws_cognito_resource_server.api.identifier}/recipe.read",
    "${aws_cognito_resource_server.api.identifier}/recipe.write",
  ]
  callback_urls                = ["https://claude.ai/api/mcp/auth_callback"]
  supported_identity_providers = ["COGNITO"]

  depends_on = [aws_cognito_resource_server.api]
}
