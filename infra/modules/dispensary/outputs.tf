output "api_endpoint" {
  value       = var.enable_api_gateway ? "${aws_apigatewayv2_api.main[0].api_endpoint}" : "API Gateway disabled"
  description = "API Gateway endpoint URL"
}

output "event_log_table" {
  value       = aws_dynamodb_table.event_log.name
  description = "Event log table name"
}

output "event_stream" {
  value       = aws_kinesis_stream.event_stream.name
  description = "Kinesis event stream name"
}

output "prescriptions_bucket" {
  value       = aws_s3_bucket.prescriptions.id
  description = "Prescriptions S3 bucket name"
}

output "lambda_functions" {
  value = {
    api                = aws_lambda_function.api.function_name
    publisher          = aws_lambda_function.publisher.function_name
    projector_views    = aws_lambda_function.projector_views.function_name
    projector_analyzer = aws_lambda_function.projector_analyzer.function_name
  }
  description = "Lambda function names"
}
