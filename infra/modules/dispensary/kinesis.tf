resource "aws_kinesis_stream" "event_stream" {
  name             = "${local.prefix}-events"
  shard_count      = var.environment == "prod" ? null : 1
  retention_period = var.environment == "prod" ? 48 : 24

  stream_mode_details {
    stream_mode = var.environment == "prod" ? "ON_DEMAND" : "PROVISIONED"
  }

  shard_level_metrics = [
    "IncomingBytes",
    "OutgoingBytes",
  ]

  tags = local.common_tags
}
