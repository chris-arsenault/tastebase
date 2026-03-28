output "mcp_client_id" {
  description = "MCP Cognito client ID — paste into Claude.ai connector settings"
  value       = aws_cognito_user_pool_client.mcp.id
}

output "mcp_client_secret" {
  description = "MCP Cognito client secret — paste into Claude.ai connector settings"
  value       = aws_cognito_user_pool_client.mcp.client_secret
  sensitive   = true
}

output "mcp_server_url" {
  description = "MCP server URL — paste into Claude.ai connector settings"
  value       = "https://${local.api_hostname}/mcp"
}

output "frontend_url" {
  value = "https://${local.frontend_hostname}"
}

output "api_url" {
  value = "https://${local.api_hostname}"
}
