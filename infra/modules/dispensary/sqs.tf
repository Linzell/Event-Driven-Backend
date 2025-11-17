# Dead Letter Queues for Lambda failures

resource "aws_sqs_queue" "publisher_dlq" {
  name = "${local.prefix}-publisher-dlq"
  tags = local.common_tags
}

resource "aws_sqs_queue" "projector_views_dlq" {
  name = "${local.prefix}-projector-views-dlq"
  tags = local.common_tags
}

resource "aws_sqs_queue" "projector_analyzer_dlq" {
  name = "${local.prefix}-projector-analyzer-dlq"
  tags = local.common_tags
}
