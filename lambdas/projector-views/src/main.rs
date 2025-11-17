use aws_lambda_events::{
    kinesis::{KinesisEvent, KinesisEventRecord},
    streams::{KinesisBatchItemFailure, KinesisEventResponse},
};
use domain::DomainEvent;
use lambda_runtime::{service_fn, Error, LambdaEvent};

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    lambda_runtime::run(service_fn(|event: LambdaEvent<KinesisEvent>| async {
        handle(event).await
    }))
    .await
}

async fn handle(event: LambdaEvent<KinesisEvent>) -> Result<KinesisEventResponse, Error> {
    tracing::info!("Processing {} Kinesis records", event.payload.records.len());

    let mut batch_item_failures = Vec::new();

    for record in event.payload.records.iter() {
        let sequence = record.kinesis.sequence_number.clone();
        
        if let Err(e) = handle_record(record).await {
            tracing::error!("Failed to process: {}", e);
            batch_item_failures.push(KinesisBatchItemFailure {
                item_identifier: sequence,
            });
        }
    }

    Ok(KinesisEventResponse { batch_item_failures })
}

async fn handle_record(record: &KinesisEventRecord) -> Result<(), Error> {
    let data = std::str::from_utf8(&record.kinesis.data)?;
    let event: DomainEvent = serde_json::from_str(data)?;

    tracing::info!("Received event: {} for {}", event.event_type, event.id);

    // Views are updated via CQRS Query automatically
    // This projector could be used for other side effects:
    // - Notifications
    // - External system integration
    // - Analytics

    Ok(())
}
