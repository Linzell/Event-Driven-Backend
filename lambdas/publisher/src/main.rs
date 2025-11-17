use aws_config::BehaviorVersion;
use aws_lambda_events::{
    dynamodb::{Event, EventRecord},
    streams::{DynamoDbBatchItemFailure, DynamoDbEventResponse},
};
use aws_sdk_kinesis::primitives::Blob;
use domain::DomainEvent;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct EventLogRecord {
    aggregate_type_and_id: String,
    event_type: String,
    aggregate_id: String,
    aggregate_type: String,
    #[serde(with = "serde_bytes")]
    metadata: Vec<u8>,
    #[serde(with = "serde_bytes")]
    payload: Vec<u8>,
    event_version: String,
    aggregate_id_sequence: usize,
}

impl TryFrom<EventLogRecord> for DomainEvent {
    type Error = String;

    fn try_from(record: EventLogRecord) -> Result<Self, Self::Error> {
        let payload = String::from_utf8(record.payload)
            .map_err(|e| format!("Invalid payload UTF-8: {}", e))?;
        let metadata = String::from_utf8(record.metadata)
            .map_err(|e| format!("Invalid metadata UTF-8: {}", e))?;

        Ok(DomainEvent::new(
            record.aggregate_id,
            record.aggregate_type,
            record.aggregate_id_sequence,
            record.event_type,
            record.event_version,
            payload,
            metadata,
        ))
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    let config = aws_config::defaults(BehaviorVersion::latest()).load().await;
    let kinesis_client = aws_sdk_kinesis::Client::new(&config);

    lambda_runtime::run(service_fn(|event: LambdaEvent<Event>| async {
        handle(event, &kinesis_client).await
    }))
    .await
}

async fn handle(
    event: LambdaEvent<Event>,
    kinesis_client: &aws_sdk_kinesis::Client,
) -> Result<DynamoDbEventResponse, Error> {
    tracing::info!("Processing {} DynamoDB records", event.payload.records.len());

    let stream_name = std::env::var("EVENT_STREAM_NAME")?;
    let mut batch_item_failures = Vec::new();

    for record in event.payload.records.iter() {
        if record.event_name == "INSERT" {
            let event_id = record.event_id.clone();
            
            if let Err(e) = handle_record(record, kinesis_client, &stream_name).await {
                tracing::error!("Failed to process {}: {}", event_id, e);
                batch_item_failures.push(DynamoDbBatchItemFailure {
                    item_identifier: Some(event_id),
                });
            }
        }
    }

    Ok(DynamoDbEventResponse { batch_item_failures })
}

async fn handle_record(
    record: &EventRecord,
    kinesis_client: &aws_sdk_kinesis::Client,
    stream_name: &str,
) -> Result<(), Error> {
    let item = &record.change.new_image;
    let event_log: EventLogRecord = serde_dynamo::from_item(item.clone())?;
    let domain_event: DomainEvent = event_log.clone().try_into()?;

    tracing::info!(
        "Publishing {} for {}",
        domain_event.event_type,
        domain_event.id
    );

    let data = serde_json::to_string(&domain_event)?;

    kinesis_client
        .put_record()
        .stream_name(stream_name)
        .partition_key(event_log.aggregate_type)
        .data(Blob::new(data))
        .send()
        .await?;

    Ok(())
}
