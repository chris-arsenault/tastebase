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
  allowed_oauth_scopes                 = ["openid", "profile", "email"]
  callback_urls                        = ["https://claude.ai/api/mcp/auth_callback"]
  supported_identity_providers         = ["COGNITO"]
}
