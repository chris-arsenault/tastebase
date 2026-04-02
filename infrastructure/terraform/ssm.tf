# Platform SSM parameters

data "aws_ssm_parameter" "alb_listener_arn" {
  name = "/platform/network/alb-listener-arn"
}

data "aws_ssm_parameter" "alb_dns_name" {
  name = "/platform/network/alb-dns-name"
}

data "aws_ssm_parameter" "alb_zone_id" {
  name = "/platform/network/alb-zone-id"
}

data "aws_ssm_parameter" "route53_zone_id" {
  name = "/platform/network/route53-zone-id"
}

data "aws_ssm_parameter" "cognito_user_pool_id" {
  name = "/platform/cognito/user-pool-id"
}

data "aws_ssm_parameter" "cognito_domain" {
  name = "/platform/cognito/domain"
}

data "aws_subnets" "private" {
  filter {
    name   = "tag:subnet:access"
    values = ["private"]
  }
}

data "aws_ssm_parameter" "rds_security_group_id" {
  name = "/platform/rds/security-group-id"
}

data "aws_ssm_parameter" "db_username" {
  name = "/platform/db/tastebase/username"
}

data "aws_ssm_parameter" "db_password" {
  name = "/platform/db/tastebase/password"
}

data "aws_ssm_parameter" "db_database" {
  name = "/platform/db/tastebase/database"
}

data "aws_ssm_parameter" "rds_address" {
  name = "/platform/rds/address"
}

data "aws_ssm_parameter" "rds_port" {
  name = "/platform/rds/port"
}
