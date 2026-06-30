locals {
  prefix      = "tastebase"
  domain_name = "ahara.io"

  api_hostname      = "api.tastebase.${local.domain_name}"
  frontend_hostname = "tastebase.${local.domain_name}"

  media_bucket = "${local.prefix}-media"

  db_env = {
    DB_HOST     = module.ctx.rds_address
    DB_PORT     = module.ctx.rds_port
    DB_NAME     = nonsensitive(data.aws_ssm_parameter.db_database.value)
    DB_USERNAME = nonsensitive(data.aws_ssm_parameter.db_username.value)
    DB_PASSWORD = nonsensitive(data.aws_ssm_parameter.db_password.value)
  }

  otel_env = {
    DEPLOYMENT_ENVIRONMENT      = "production"
    OTEL_EXPORTER_OTLP_ENDPOINT = nonsensitive(data.aws_ssm_parameter.observability_otlp_http_endpoint.value)
    OTEL_LOGS_EXPORTER          = "otlp"
    OTEL_METRICS_EXPORTER       = "otlp"
    OTEL_TRACES_EXPORTER        = "otlp"
  }

  common_env = merge(local.db_env, local.otel_env, {
    COGNITO_USER_POOL_ID = module.ctx.cognito_user_pool_id
    COGNITO_DOMAIN       = module.ctx.cognito_domain
    COGNITO_ISSUER       = module.ctx.cognito_issuer
    MEDIA_BUCKET         = local.media_bucket
    API_BASE_URL         = "https://${local.api_hostname}"
    APP_BASE_URL         = "https://${local.frontend_hostname}"
  })
}
