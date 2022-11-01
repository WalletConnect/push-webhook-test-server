locals {
  app_name   = "data-lake-api"
  fqdn       = "data.walletconnect.com"
  domain     = terraform.workspace != "prod" ? "${terraform.workspace}.${local.fqdn}" : local.fqdn
  account_id = data.aws_caller_identity.current.account_id
}

data "aws_caller_identity" "current" {}

data "assert_test" "workspace" {
  test  = terraform.workspace != "default"
  throw = "default workspace is not valid in this project"
}

module "tags" {
  source = "github.com/WalletConnect/terraform-modules/modules/tags"

  application = local.app_name
  env         = terraform.workspace
}

resource "random_pet" "this" {
  length = 2
}

resource "aws_cloudwatch_log_group" "logs" {
  name = random_pet.this.id
}

module "domain" {
  source = "./dns"

  zone_domain    = local.fqdn
  cert_subdomain = terraform.workspace == "prod" ? null : terraform.workspace
}

module "lambda" {
  source = "terraform-aws-modules/lambda/aws"

  function_name = "${terraform.workspace}-${local.app_name}"
  description   = "Function to expose data lake API"
  handler       = "bootstrap"
  runtime       = "provided.al2"
  timeout       = 25

  environment_variables = {
    RUST_BACKTRACE = 1,
    RDS_SECRET_ARN = "arn:aws:secretsmanager:eu-central-1:898587786287:secret:rds-db-credentials/cluster-S3WKERTXO6C5T6H7DEO3UEG5VY/root/1667092233967-Mg8sNk",
    RDS_CLUSTER_ARN = "arn:aws:rds:eu-central-1:898587786287:cluster:prod-relay-customer-metrics",
  }

  architectures = ["arm64"]

  tracing_mode = "Active"

  attach_policies = true
  number_of_policies = 1
  policies = ["arn:aws:iam::898587786287:policy/prod-relay-customer-metrics-data-api-access"]

  create_package         = false
  publish                = true
  local_existing_package = "../target/lambda/${local.app_name}/bootstrap.zip"

  allowed_triggers = {
    AllowExecutionFromAPIGatewayDefault = {
      service    = "apigateway"
      source_arn = "${module.api_gateway.apigatewayv2_api_execution_arn}/*/$default"
    }
    AllowExecutionFromAPIGatewayRoot = {
      service    = "apigateway"
      source_arn = "${module.api_gateway.apigatewayv2_api_execution_arn}/*/*/"
    }
    AllowExecutionFromAPIGatewayGetProjectId = {
      principal  = "apigateway.amazonaws.com"
      source_arn = "arn:aws:execute-api:${var.region}:${local.account_id}:${module.api_gateway.apigatewayv2_api_id}/*/*/{projectId}"
    }
  }
}

module "api_gateway" {
  depends_on = [
    module.domain,
  ]

  source = "terraform-aws-modules/apigateway-v2/aws"

  name          = "${terraform.workspace}-${local.app_name}-http"
  description   = "API to query the data lake API"
  protocol_type = "HTTP"

  cors_configuration = {
    allow_headers = ["content-type", "x-amz-date", "authorization", "x-api-key", "x-amz-security-token", "x-amz-user-agent"]
    allow_methods = ["*"]
    allow_origins = ["*"]
  }

  default_route_settings = {
    detailed_metrics_enabled = true
    throttling_burst_limit   = 100
    throttling_rate_limit    = 100
  }

  default_stage_access_log_destination_arn = aws_cloudwatch_log_group.logs.arn
  default_stage_access_log_format          = "$context.identity.sourceIp - - [$context.requestTime] \"$context.httpMethod $context.routeKey $context.protocol\" $context.status $context.responseLength $context.requestId $context.integrationErrorMessage"

  # Custom domain
  create_api_domain_name      = true
  domain_name                 = local.domain
  domain_name_certificate_arn = module.domain.certificate_arn

  # Routes and integrations
  integrations = {
    "$default" = {
      timeout_milliseconds   = 28000
      lambda_arn = module.lambda.lambda_function_arn
      tls_config = jsonencode({
        server_name_to_verify = local.domain
      })

      response_parameters = jsonencode([
        {
          status_code = 500
          mappings = {
            "append:header.header1" = "$context.requestId"
            "overwrite:statuscode"  = "403"
          }
        },
        {
          status_code = 404
          mappings = {
            "append:header.error" = "$stageVariables.environmentId"
          }
        }
      ])
    }
  }

  body = templatefile("api.yml", {
    example_function_arn = module.lambda.lambda_function_arn
  })
}

resource "aws_route53_record" "sub_domain" {
  name    = local.domain
  type    = "A"
  zone_id = module.domain.zone_id

  alias {
    name                   = module.api_gateway.apigatewayv2_domain_name_target_domain_name
    zone_id                = module.api_gateway.apigatewayv2_domain_name_hosted_zone_id
    evaluate_target_health = false
  }
}
