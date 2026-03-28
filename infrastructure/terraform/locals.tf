locals {
  prefix      = "tastebase"
  domain_name = "ahara.io"

  api_hostname      = "api.tastebase.${local.domain_name}"
  frontend_hostname = "tastebase.${local.domain_name}"

  media_bucket = "${local.prefix}-media"

  cognito_pool_id = nonsensitive(data.aws_ssm_parameter.cognito_user_pool_id.value)
  cognito_issuer  = "https://cognito-idp.us-east-1.amazonaws.com/${local.cognito_pool_id}"
  cognito_jwks    = "${local.cognito_issuer}/.well-known/jwks.json"
  cognito_domain  = nonsensitive(data.aws_ssm_parameter.cognito_domain.value)

  db_env = {
    DB_HOST     = nonsensitive(data.aws_ssm_parameter.rds_address.value)
    DB_PORT     = nonsensitive(data.aws_ssm_parameter.rds_port.value)
    DB_NAME     = nonsensitive(data.aws_ssm_parameter.db_database.value)
    DB_USERNAME = nonsensitive(data.aws_ssm_parameter.db_username.value)
    DB_PASSWORD = nonsensitive(data.aws_ssm_parameter.db_password.value)
  }

  common_env = merge(local.db_env, {
    COGNITO_USER_POOL_ID = local.cognito_pool_id
    COGNITO_DOMAIN       = local.cognito_domain
    COGNITO_ISSUER       = local.cognito_issuer
    MEDIA_BUCKET         = local.media_bucket
    API_BASE_URL         = "https://${local.api_hostname}"
    APP_BASE_URL         = "https://${local.frontend_hostname}"
  })

  lambda_subnet_ids = split(",", nonsensitive(data.aws_ssm_parameter.private_subnet_ids.value))
}
