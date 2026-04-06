# Per-project DB credentials (not in platform-context)

data "aws_ssm_parameter" "db_username" {
  name = "/platform/db/tastebase/username"
}

data "aws_ssm_parameter" "db_password" {
  name = "/platform/db/tastebase/password"
}

data "aws_ssm_parameter" "db_database" {
  name = "/platform/db/tastebase/database"
}
