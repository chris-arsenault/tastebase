# Per-project DB credentials (not in platform-context)

data "aws_ssm_parameter" "db_username" {
  name = "/ahara/db/tastebase/username"
}

data "aws_ssm_parameter" "db_password" {
  name = "/ahara/db/tastebase/password"
}

data "aws_ssm_parameter" "db_database" {
  name = "/ahara/db/tastebase/database"
}
