# API Lambda
resource "aws_lambda_function" "api" {
  filename         = "../../target/lambda/api/bootstrap.zip"
  function_name    = "${local.prefix}-api"
  role             = aws_iam_role.lambda_exec.arn
  handler          = "bootstrap"
  runtime          = "provided.al2023"
  architectures    = [var.lambda_architecture]
  timeout          = 30
  source_code_hash = filebase64sha256("../../target/lambda/api/bootstrap.zip")

  environment {
    variables = {
      DYNAMODB_EVENT_LOG_TABLE       = aws_dynamodb_table.event_log.name
      DYNAMODB_EVENT_SNAPSHOTS_TABLE = aws_dynamodb_table.event_snapshots.name
      DYNAMODB_DISPENSES_VIEW_TABLE  = aws_dynamodb_table.dispenses_view.name
      PRESCRIPTIONS_BUCKET           = aws_s3_bucket.prescriptions.id
      RUST_LOG                       = "info"
    }
  }

  tags = local.common_tags
}

# Publisher Lambda
resource "aws_lambda_function" "publisher" {
  filename         = "../../target/lambda/publisher/bootstrap.zip"
  function_name    = "${local.prefix}-publisher"
  role             = aws_iam_role.lambda_exec.arn
  handler          = "bootstrap"
  runtime          = "provided.al2023"
  architectures    = [var.lambda_architecture]
  timeout          = 60
  source_code_hash = filebase64sha256("../../target/lambda/publisher/bootstrap.zip")

  environment {
    variables = {
      EVENT_STREAM_NAME = aws_kinesis_stream.event_stream.name
      RUST_LOG          = "info"
    }
  }

  tags = local.common_tags
}

# Event source mapping: DynamoDB Stream -> Publisher Lambda
resource "aws_lambda_event_source_mapping" "dynamodb_to_publisher" {
  event_source_arn                   = aws_dynamodb_table.event_log.stream_arn
  function_name                      = aws_lambda_function.publisher.arn
  starting_position                  = "LATEST"
  maximum_retry_attempts             = 3
  bisect_batch_on_function_error     = true
  maximum_record_age_in_seconds      = 604800

  destination_config {
    on_failure {
      destination_arn = aws_sqs_queue.publisher_dlq.arn
    }
  }
}

# Projector Views Lambda
resource "aws_lambda_function" "projector_views" {
  filename         = "../../target/lambda/projector-views/bootstrap.zip"
  function_name    = "${local.prefix}-projector-views"
  role             = aws_iam_role.lambda_exec.arn
  handler          = "bootstrap"
  runtime          = "provided.al2023"
  architectures    = [var.lambda_architecture]
  timeout          = 60
  source_code_hash = filebase64sha256("../../target/lambda/projector-views/bootstrap.zip")

  environment {
    variables = {
      RUST_LOG = "info"
    }
  }

  tags = local.common_tags
}

# Event source mapping: Kinesis -> Projector Views Lambda
resource "aws_lambda_event_source_mapping" "kinesis_to_views" {
  event_source_arn        = aws_kinesis_stream.event_stream.arn
  function_name           = aws_lambda_function.projector_views.arn
  starting_position       = "LATEST"
  batch_size              = 10
  maximum_retry_attempts  = 3
  function_response_types = ["ReportBatchItemFailures"]

  destination_config {
    on_failure {
      destination_arn = aws_sqs_queue.projector_views_dlq.arn
    }
  }
}

# Projector Analyzer Lambda
resource "aws_lambda_function" "projector_analyzer" {
  filename         = "../../target/lambda/projector-analyzer/bootstrap.zip"
  function_name    = "${local.prefix}-projector-analyzer"
  role             = aws_iam_role.lambda_exec.arn
  handler          = "bootstrap"
  runtime          = "provided.al2023"
  architectures    = [var.lambda_architecture]
  timeout          = 300
  source_code_hash = filebase64sha256("../../target/lambda/projector-analyzer/bootstrap.zip")

  environment {
    variables = {
      DYNAMODB_EVENT_LOG_TABLE       = aws_dynamodb_table.event_log.name
      DYNAMODB_EVENT_SNAPSHOTS_TABLE = aws_dynamodb_table.event_snapshots.name
      DYNAMODB_DISPENSES_VIEW_TABLE  = aws_dynamodb_table.dispenses_view.name
      PRESCRIPTIONS_BUCKET           = aws_s3_bucket.prescriptions.id
      RUST_LOG                       = "info"
    }
  }

  tags = local.common_tags
}

# Event source mapping: Kinesis -> Projector Analyzer Lambda
resource "aws_lambda_event_source_mapping" "kinesis_to_analyzer" {
  event_source_arn        = aws_kinesis_stream.event_stream.arn
  function_name           = aws_lambda_function.projector_analyzer.arn
  starting_position       = "LATEST"
  batch_size              = 1
  maximum_retry_attempts  = 3
  function_response_types = ["ReportBatchItemFailures"]

  destination_config {
    on_failure {
      destination_arn = aws_sqs_queue.projector_analyzer_dlq.arn
    }
  }
}
