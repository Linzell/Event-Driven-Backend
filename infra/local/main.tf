provider "aws" {
  access_key                  = "test"
  secret_key                  = "test"
  region                      = "us-east-1"
  s3_use_path_style           = false
  skip_credentials_validation = true
  skip_metadata_api_check     = true
  skip_requesting_account_id  = true

  endpoints {
    apigateway     = "http://localhost:4566"
    apigatewayv2   = "http://localhost:4566"
    cloudwatch     = "http://localhost:4566"
    cloudwatchlogs = "http://localhost:4566"
    dynamodb       = "http://localhost:4566"
    iam            = "http://localhost:4566"
    kinesis        = "http://localhost:4566"
    lambda         = "http://localhost:4566"
    s3             = "http://s3.localhost.localstack.cloud:4566"
    sqs            = "http://localhost:4566"
    sts            = "http://localhost:4566"
  }
}

terraform {
  backend "local" {}

  required_version = "~> 1.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = ">= 5.0"
    }
  }
}

module "dispensary" {
  source = "../modules/dispensary"

  environment        = "local"
  region             = "us-east-1"
  enable_api_gateway = false # API Gateway V2 requires LocalStack Pro (not available in freemium)
  lambda_architecture = "arm64" # Use arm64 for local Mac development
}

output "api_endpoint" {
  value = "Direct Lambda invocation: http://localhost:4566/2015-03-31/functions/dispensary-local-api/invocations"
}

output "event_log_table" {
  value = module.dispensary.event_log_table
}

output "event_stream" {
  value = module.dispensary.event_stream
}

output "prescriptions_bucket" {
  value = module.dispensary.prescriptions_bucket
}

output "lambda_functions" {
  value = module.dispensary.lambda_functions
}
