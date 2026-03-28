# ALB target groups and listener rules for each Lambda
#
# Public reads (GET) have no jwt-validation.
# Writes (POST/DELETE) require jwt-validation.
# MCP has selective auth (initialize is unauthenticated) so the Lambda handles it.

# CORS preflight (OPTIONS) is handled platform-wide by platform-network.
# App Lambdas still need tower-http CorsLayer for CORS headers on actual responses.

# --- Tastings API ---

resource "aws_lb_target_group" "tastings_api" {
  name        = "${local.prefix}-tastings-tg"
  target_type = "lambda"
}

resource "aws_lb_target_group_attachment" "tastings_api" {
  target_group_arn = aws_lb_target_group.tastings_api.arn
  target_id        = aws_lambda_function.tastings_api.arn
  depends_on       = [aws_lambda_permission.tastings_api]
}

resource "aws_lambda_permission" "tastings_api" {
  statement_id  = "AllowALBInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.tastings_api.function_name
  principal     = "elasticloadbalancing.amazonaws.com"
  source_arn    = aws_lb_target_group.tastings_api.arn
}

# Public reads
resource "aws_lb_listener_rule" "tastings_api_read" {
  listener_arn = nonsensitive(data.aws_ssm_parameter.alb_listener_arn.value)
  priority     = 210

  condition {
    host_header {
      values = [local.api_hostname]
    }
  }

  condition {
    path_pattern {
      values = ["/tastings", "/tastings/*"]
    }
  }

  condition {
    http_request_method {
      values = ["GET", "HEAD"]
    }
  }

  action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.tastings_api.arn
  }
}

# Authenticated writes
resource "aws_lb_listener_rule" "tastings_api_write" {
  listener_arn = nonsensitive(data.aws_ssm_parameter.alb_listener_arn.value)
  priority     = 211

  condition {
    host_header {
      values = [local.api_hostname]
    }
  }

  condition {
    path_pattern {
      values = ["/tastings", "/tastings/*"]
    }
  }

  action {
    type = "jwt-validation"

    jwt_validation {
      issuer        = local.cognito_issuer
      jwks_endpoint = local.cognito_jwks
    }
  }

  action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.tastings_api.arn
  }
}

# --- Recipes API ---

resource "aws_lb_target_group" "recipes_api" {
  name        = "${local.prefix}-recipes-tg"
  target_type = "lambda"
}

resource "aws_lb_target_group_attachment" "recipes_api" {
  target_group_arn = aws_lb_target_group.recipes_api.arn
  target_id        = aws_lambda_function.recipes_api.arn
  depends_on       = [aws_lambda_permission.recipes_api]
}

resource "aws_lambda_permission" "recipes_api" {
  statement_id  = "AllowALBInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.recipes_api.function_name
  principal     = "elasticloadbalancing.amazonaws.com"
  source_arn    = aws_lb_target_group.recipes_api.arn
}

# Public reads
resource "aws_lb_listener_rule" "recipes_api_read" {
  listener_arn = nonsensitive(data.aws_ssm_parameter.alb_listener_arn.value)
  priority     = 212

  condition {
    host_header {
      values = [local.api_hostname]
    }
  }

  condition {
    path_pattern {
      values = ["/recipes", "/recipes/*"]
    }
  }

  condition {
    http_request_method {
      values = ["GET", "HEAD"]
    }
  }

  action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.recipes_api.arn
  }
}

# Authenticated writes
resource "aws_lb_listener_rule" "recipes_api_write" {
  listener_arn = nonsensitive(data.aws_ssm_parameter.alb_listener_arn.value)
  priority     = 213

  condition {
    host_header {
      values = [local.api_hostname]
    }
  }

  condition {
    path_pattern {
      values = ["/recipes", "/recipes/*"]
    }
  }

  action {
    type = "jwt-validation"

    jwt_validation {
      issuer        = local.cognito_issuer
      jwks_endpoint = local.cognito_jwks
    }
  }

  action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.recipes_api.arn
  }
}

# --- MCP Server ---
# MCP protocol requires selective auth: initialize is unauthenticated,
# tools/list and tools/call require a valid token. ALB can't distinguish
# JSON-RPC methods, so the Lambda handles auth internally.
# .well-known endpoints are also public (OAuth metadata discovery).

resource "aws_lb_target_group" "mcp_server" {
  name        = "${local.prefix}-mcp-tg"
  target_type = "lambda"
}

resource "aws_lb_target_group_attachment" "mcp_server" {
  target_group_arn = aws_lb_target_group.mcp_server.arn
  target_id        = aws_lambda_function.mcp_server.arn
  depends_on       = [aws_lambda_permission.mcp_server]
}

resource "aws_lambda_permission" "mcp_server" {
  statement_id  = "AllowALBInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.mcp_server.function_name
  principal     = "elasticloadbalancing.amazonaws.com"
  source_arn    = aws_lb_target_group.mcp_server.arn
}

# .well-known endpoints are public (OAuth metadata discovery)
resource "aws_lb_listener_rule" "mcp_wellknown" {
  listener_arn = nonsensitive(data.aws_ssm_parameter.alb_listener_arn.value)
  priority     = 214

  condition {
    host_header {
      values = [local.api_hostname]
    }
  }

  condition {
    path_pattern {
      values = ["/.well-known/*"]
    }
  }

  action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.mcp_server.arn
  }
}

# MCP protocol — all requests require auth (including initialize).
# First unauthenticated request gets 401 with WWW-Authenticate from ALB,
# triggering the OAuth flow. After OAuth, all requests include the token.
resource "aws_lb_listener_rule" "mcp_server" {
  listener_arn = nonsensitive(data.aws_ssm_parameter.alb_listener_arn.value)
  priority     = 215

  condition {
    host_header {
      values = [local.api_hostname]
    }
  }

  condition {
    path_pattern {
      values = ["/mcp"]
    }
  }

  action {
    type = "jwt-validation"

    jwt_validation {
      issuer        = local.cognito_issuer
      jwks_endpoint = local.cognito_jwks
    }
  }

  action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.mcp_server.arn
  }
}
