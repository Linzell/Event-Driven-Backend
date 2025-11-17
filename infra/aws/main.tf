provider "aws" {
  region = var.region
}

terraform {
  backend "s3" {
    # Configure with -backend-config
  }

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

  environment        = var.environment
  region             = var.region
  enable_api_gateway = true
}

output "api_endpoint" {
  value = module.dispensary.api_endpoint
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
