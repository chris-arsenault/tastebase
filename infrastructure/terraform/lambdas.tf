# ── Platform context ────────────────────────────────────────

module "ctx" {
  source = "git::https://github.com/chris-arsenault/ahara-tf-patterns.git//modules/platform-context"
}

# ── ALB APIs ────────────────────────────────────────────────

module "api" {
  source   = "git::https://github.com/chris-arsenault/ahara-tf-patterns.git//modules/alb-api"
  hostname = local.api_hostname

  environment = local.common_env

  iam_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect   = "Allow"
        Action   = ["s3:GetObject", "s3:PutObject"]
        Resource = "${aws_s3_bucket.media.arn}/*"
      },
      {
        Effect   = "Allow"
        Action   = ["bedrock:InvokeModel"]
        Resource = "*"
      },
      {
        Effect = "Allow"
        Action = [
          "transcribe:StartTranscriptionJob",
          "transcribe:GetTranscriptionJob"
        ]
        Resource = "*"
      },
      {
        Effect   = "Allow"
        Action   = ["lambda:InvokeFunction"]
        Resource = module.processing.function_arn
      },
      {
        Effect   = "Allow"
        Action   = ["cloudfront:CreateInvalidation"]
        Resource = module.frontend.distribution_arn
      }
    ]
  })

  lambdas = {
    tastings-api = {
      binary = "${path.module}/../../backend/target/lambda/tastings-api/bootstrap"
      routes = [
        { priority = 210, paths = ["/tastings", "/tastings/*"], methods = ["GET", "HEAD"], authenticated = false },
        { priority = 211, paths = ["/tastings", "/tastings/*"], authenticated = true },
      ]
      environment = { PROCESSING_FUNCTION_NAME = module.processing.function_name }
    }
    recipes-api = {
      binary = "${path.module}/../../backend/target/lambda/recipes-api/bootstrap"
      routes = [
        { priority = 212, paths = ["/recipes", "/recipes/*"], methods = ["GET", "HEAD"], authenticated = false },
        { priority = 213, paths = ["/recipes", "/recipes/*"], authenticated = true },
      ]
      environment = {
        PROCESSING_FUNCTION_NAME   = module.processing.function_name
        CLOUDFRONT_DISTRIBUTION_ID = module.frontend.distribution_id
      }
    }
    mcp-server = {
      binary = "${path.module}/../../backend/target/lambda/mcp-server/bootstrap"
      routes = [
        { priority = 214, paths = ["/mcp", "/.well-known/*"], authenticated = false },
      ]
      environment = { COGNITO_CLIENT_ID = module.cognito_app.client_id }
    }
  }
}

# ── Processing (async, not HTTP-triggered) ──────────────────

module "processing" {
  source             = "git::https://github.com/chris-arsenault/ahara-tf-patterns.git//modules/lambda"
  name               = "${local.prefix}-processing"
  binary             = "${path.module}/../../backend/target/lambda/processing/bootstrap"
  role_arn           = module.api.role_arn
  subnet_ids         = module.ctx.private_subnet_ids
  security_group_ids = [module.api.security_group_id]

  environment = merge(local.common_env, {
    BEDROCK_MODEL_ID = "us.anthropic.claude-haiku-4-5-20251001-v1:0"
  })
}
