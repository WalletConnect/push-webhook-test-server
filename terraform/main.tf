locals {
  app_name = "push-webhook-test-server"
  domain = var.fqdn_subdomain != null ? "${var.fqdn_subdomain}.${var.fqdn}" : var.fqdn
}

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

  zone_domain    = var.fqdn
  cert_subdomain = var.fqdn_subdomain
}

module "dynamodb_table" {
  source   = "terraform-aws-modules/dynamodb-table/aws"

  name     = "${terraform.workspace}-push-webhook-topic"
  hash_key = "topic"

  ttl_attribute_name = "expiry"
  ttl_enabled = true

  attributes = [
    {
      name = "topic"
      type = "S"
    }
  ]
}

module "lambda_function_existing_package_local" {
  source = "terraform-aws-modules/lambda/aws"

  function_name = "${terraform.workspace}-push-sns-broadcast"
  description   = "Function to broadcast messages on SNS"
  handler       = "bootstrap"
  runtime       = "provided.al2"

  environment_variables = {
    RUST_BACKTRACE = 1
    DDB_TABLE_NAME = "${terraform.workspace}-push-webhook-topic"
  }

  attach_policy_statements = true
  policy_statements = {
    dynamodb = {
      effect    = "Allow",
      actions   = ["dynamodb:PutItem", "dynamodb:GetItem"],
      resources = [module.dynamodb_table.dynamodb_table_arn]
    }
  }

  architectures = ["arm64"]

  tracing_mode = "Active"

  create_package         = false
  publish       = true
  local_existing_package = "../target/lambda/push-webhook-test-server/bootstrap.zip"

  allowed_triggers = {
    AllowExecutionFromAPIGatewayDefault = {
      service    = "apigateway"
      source_arn = "${module.api_gateway.apigatewayv2_api_execution_arn}/*/$default"
    }
    AllowExecutionFromAPIGatewayRoot = {
      service    = "apigateway"
      source_arn = "${module.api_gateway.apigatewayv2_api_execution_arn}/*/*/"
    }
    AllowExecutionFromAPIGatewayPostTopic = {
      service    = "apigateway"
      source_arn = "${module.api_gateway.apigatewayv2_api_execution_arn}/*/"
    }
    AllowExecutionFromAPIGatewayGetTopic = {
      service    = "apigateway"
      source_arn = "${module.api_gateway.apigatewayv2_api_execution_arn}/*/*/{topic}"
    }
  }
}

module "api_gateway" {
  depends_on = [
    module.domain,
  ]

  source = "terraform-aws-modules/apigateway-v2/aws"

  name          = "${terraform.workspace}-push-sns-broadcast-http"
  description   = "API to test the webhook functionality"
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
      lambda_arn = module.lambda_function_existing_package_local.lambda_function_arn
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
    example_function_arn = module.lambda_function_existing_package_local.lambda_function_arn
  })
}

resource "aws_route53_record" "sub_domain" {
  name    = "${local.domain}"
  type    = "A"
  zone_id = "${module.domain.zone_id}"

  alias {
    name                   = "${module.api_gateway.apigatewayv2_domain_name_target_domain_name}"
    zone_id                = "${module.api_gateway.apigatewayv2_domain_name_hosted_zone_id}"
    evaluate_target_health = false
  }
}
