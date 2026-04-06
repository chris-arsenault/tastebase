# Frontend: S3 static assets + OG Lambda for dynamic HTML via CloudFront

module "frontend" {
  source         = "git::https://github.com/chris-arsenault/ahara-tf-patterns.git//modules/website"
  prefix         = local.prefix
  hostname       = local.frontend_hostname
  site_directory = "${path.module}/../../frontend/dist"

  runtime_config = {
    apiBaseUrl        = "https://${local.api_hostname}"
    cognitoUserPoolId = module.ctx.cognito_user_pool_id
    cognitoClientId   = module.cognito_app.client_id
  }

  og_config = {
    site_name = "Tastebase"

    defaults = {
      title       = "Tastebase"
      description = "Culinary platform — track tastings, save recipes, review dishes."
      image       = "/tastebase-social.png"
    }

    routes = [
      {
        pattern     = "/recipes/:slug"
        query       = "SELECT r.title, r.description, ri.image_url FROM recipes r LEFT JOIN LATERAL (SELECT image_url FROM recipe_images WHERE recipe_id = r.id ORDER BY created_at DESC LIMIT 1) ri ON true"
        match_field = "title"
        title       = "{{title}}"
        description = "{{description}}"
        image       = "{{image_url}}"
        og_type     = "article"
      }
    ]

    environment = local.db_env
  }
}
