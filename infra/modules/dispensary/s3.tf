resource "aws_s3_bucket" "prescriptions" {
  bucket = "${local.prefix}-prescriptions"

  tags = local.common_tags
}

resource "aws_s3_bucket_versioning" "prescriptions" {
  bucket = aws_s3_bucket.prescriptions.id

  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "prescriptions" {
  bucket = aws_s3_bucket.prescriptions.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

# S3 event notification to trigger analyzer Lambda on file upload
resource "aws_s3_bucket_notification" "prescription_uploads" {
  bucket = aws_s3_bucket.prescriptions.id

  lambda_function {
    lambda_function_arn = aws_lambda_function.projector_analyzer.arn
    events              = ["s3:ObjectCreated:*"]
    filter_prefix       = "prescriptions/"
    filter_suffix       = ".jpg"
  }

  lambda_function {
    lambda_function_arn = aws_lambda_function.projector_analyzer.arn
    events              = ["s3:ObjectCreated:*"]
    filter_prefix       = "prescriptions/"
    filter_suffix       = ".jpeg"
  }

  lambda_function {
    lambda_function_arn = aws_lambda_function.projector_analyzer.arn
    events              = ["s3:ObjectCreated:*"]
    filter_prefix       = "prescriptions/"
    filter_suffix       = ".png"
  }

  lambda_function {
    lambda_function_arn = aws_lambda_function.projector_analyzer.arn
    events              = ["s3:ObjectCreated:*"]
    filter_prefix       = "prescriptions/"
    filter_suffix       = ".pdf"
  }

  depends_on = [aws_lambda_permission.s3_invoke_analyzer]
}

# Lambda permission for S3 to invoke analyzer
resource "aws_lambda_permission" "s3_invoke_analyzer" {
  statement_id  = "AllowS3Invoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.projector_analyzer.function_name
  principal     = "s3.amazonaws.com"
  source_arn    = aws_s3_bucket.prescriptions.arn
}
