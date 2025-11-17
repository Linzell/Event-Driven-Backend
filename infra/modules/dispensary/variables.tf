variable "environment" {
  type        = string
  description = "Environment name"
}

variable "region" {
  type        = string
  description = "AWS region"
  default     = "us-east-1"
}

variable "enable_api_gateway" {
  type        = bool
  description = "Enable API Gateway"
  default     = true
}

variable "lambda_architecture" {
  type        = string
  description = "Lambda architecture (arm64 for local, x86_64 for AWS)"
  default     = "x86_64"
}

locals {
  prefix = "dispensary-${var.environment}"

  common_tags = {
    Environment   = var.environment
    Project       = "dispensary"
    ProvisionedBy = "terraform"
  }
}
