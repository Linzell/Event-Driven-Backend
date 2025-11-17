# Event Log Table
resource "aws_dynamodb_table" "event_log" {
  name           = "${local.prefix}-event-log"
  billing_mode   = "PAY_PER_REQUEST"
  hash_key       = "AggregateTypeAndId"
  range_key      = "AggregateIdSequence"
  stream_enabled = true
  stream_view_type = "NEW_IMAGE"

  attribute {
    name = "AggregateTypeAndId"
    type = "S"
  }

  attribute {
    name = "AggregateIdSequence"
    type = "N"
  }

  tags = local.common_tags
}

# Event Snapshots Table
resource "aws_dynamodb_table" "event_snapshots" {
  name         = "${local.prefix}-event-snapshots"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "AggregateTypeAndId"

  attribute {
    name = "AggregateTypeAndId"
    type = "S"
  }

  tags = local.common_tags
}

# Dispenses View Table
resource "aws_dynamodb_table" "dispenses_view" {
  name         = "${local.prefix}-dispenses-view"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "ViewId"

  attribute {
    name = "ViewId"
    type = "S"
  }

  tags = local.common_tags
}
