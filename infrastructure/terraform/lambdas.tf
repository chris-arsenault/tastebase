# Security group for Lambda functions (RDS access)
resource "aws_security_group" "lambda" {
  name_prefix = "${local.prefix}-lambda-"
  description = "Tastebase Lambda functions"
  vpc_id      = data.aws_ssm_parameter.vpc_id.value

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

data "aws_ssm_parameter" "vpc_id" {
  name = "/platform/network/vpc-id"
}

# IAM role shared by all Lambdas
resource "aws_iam_role" "lambda" {
  name = "${local.prefix}-lambda"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action    = "sts:AssumeRole"
      Effect    = "Allow"
      Principal = { Service = "lambda.amazonaws.com" }
    }]
  })
}

resource "aws_iam_role_policy_attachment" "lambda_vpc" {
  role       = aws_iam_role.lambda.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaVPCAccessExecutionRole"
}

resource "aws_iam_role_policy" "lambda" {
  name = "${local.prefix}-lambda"
  role = aws_iam_role.lambda.id
  policy = jsonencode({
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
        Resource = aws_lambda_function.processing.arn
      }
    ]
  })
}

# --- Archive: zip each cargo-lambda binary for deployment ---

data "archive_file" "tastings_api" {
  type        = "zip"
  source_file = "${path.module}/../../backend/target/lambda/tastings-api/bootstrap"
  output_path = "${path.module}/../../backend/target/lambda/tastings-api/bootstrap.zip"
}

data "archive_file" "recipes_api" {
  type        = "zip"
  source_file = "${path.module}/../../backend/target/lambda/recipes-api/bootstrap"
  output_path = "${path.module}/../../backend/target/lambda/recipes-api/bootstrap.zip"
}

data "archive_file" "mcp_server" {
  type        = "zip"
  source_file = "${path.module}/../../backend/target/lambda/mcp-server/bootstrap"
  output_path = "${path.module}/../../backend/target/lambda/mcp-server/bootstrap.zip"
}

data "archive_file" "processing" {
  type        = "zip"
  source_file = "${path.module}/../../backend/target/lambda/processing/bootstrap"
  output_path = "${path.module}/../../backend/target/lambda/processing/bootstrap.zip"
}

# --- Tastings API ---

resource "aws_lambda_function" "tastings_api" {
  function_name = "${local.prefix}-tastings-api"
  role          = aws_iam_role.lambda.arn
  handler       = "bootstrap"
  runtime       = "provided.al2023"
  architectures = ["x86_64"]
  timeout       = 30
  memory_size   = 256

  filename         = data.archive_file.tastings_api.output_path
  source_code_hash = data.archive_file.tastings_api.output_base64sha256

  vpc_config {
    subnet_ids         = local.lambda_subnet_ids
    security_group_ids = [aws_security_group.lambda.id]
  }

  environment {
    variables = merge(local.common_env, {
      PROCESSING_FUNCTION_NAME = aws_lambda_function.processing.function_name
    })
  }
}

# --- Recipes API ---

resource "aws_lambda_function" "recipes_api" {
  function_name = "${local.prefix}-recipes-api"
  role          = aws_iam_role.lambda.arn
  handler       = "bootstrap"
  runtime       = "provided.al2023"
  architectures = ["x86_64"]
  timeout       = 30
  memory_size   = 256

  filename         = data.archive_file.recipes_api.output_path
  source_code_hash = data.archive_file.recipes_api.output_base64sha256

  vpc_config {
    subnet_ids         = local.lambda_subnet_ids
    security_group_ids = [aws_security_group.lambda.id]
  }

  environment {
    variables = merge(local.common_env, {
      PROCESSING_FUNCTION_NAME = aws_lambda_function.processing.function_name
    })
  }
}

# --- MCP Server ---

resource "aws_lambda_function" "mcp_server" {
  function_name = "${local.prefix}-mcp-server"
  role          = aws_iam_role.lambda.arn
  handler       = "bootstrap"
  runtime       = "provided.al2023"
  architectures = ["x86_64"]
  timeout       = 30
  memory_size   = 256

  filename         = data.archive_file.mcp_server.output_path
  source_code_hash = data.archive_file.mcp_server.output_base64sha256

  vpc_config {
    subnet_ids         = local.lambda_subnet_ids
    security_group_ids = [aws_security_group.lambda.id]
  }

  environment {
    variables = merge(local.common_env, {
      COGNITO_CLIENT_ID = aws_cognito_user_pool_client.app.id
    })
  }
}

# --- Processing (async, not HTTP-triggered) ---

resource "aws_lambda_function" "processing" {
  function_name = "${local.prefix}-processing"
  role          = aws_iam_role.lambda.arn
  handler       = "bootstrap"
  runtime       = "provided.al2023"
  architectures = ["x86_64"]
  timeout       = 300
  memory_size   = 512

  filename         = data.archive_file.processing.output_path
  source_code_hash = data.archive_file.processing.output_base64sha256

  vpc_config {
    subnet_ids         = local.lambda_subnet_ids
    security_group_ids = [aws_security_group.lambda.id]
  }

  environment {
    variables = merge(local.common_env, {
      BEDROCK_MODEL_ID = "us.anthropic.claude-haiku-4-5-20251001-v1:0"
    })
  }
}
