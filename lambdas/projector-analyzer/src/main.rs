use aws_config::BehaviorVersion;
use aws_lambda_events::{
    event::s3::S3Event,
    kinesis::{KinesisEvent, KinesisEventRecord},
    streams::{KinesisBatchItemFailure, KinesisEventResponse},
};
use domain::{
    dispenses::{self, Dispense},
    DomainEvent,
};
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::Value;
use std::collections::HashMap;
use ulid::Ulid;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    let config = aws_config::defaults(BehaviorVersion::latest()).load().await;
    let dynamodb_client = aws_sdk_dynamodb::Client::new(&config);
    let s3_client = aws_sdk_s3::Client::new(&config);

    let dispenses_repo = dispenses::cqrs::init_repo(dynamodb_client.clone());
    let dispenses_cqrs = dispenses::cqrs::init(dynamodb_client, dispenses_repo);

    lambda_runtime::run(service_fn(|event: LambdaEvent<Value>| async {
        handle_event(event, &dispenses_cqrs, &s3_client).await
    }))
    .await
}

async fn handle_event(
    event: LambdaEvent<Value>,
    cqrs: &cqrs_es::CqrsFramework<
        Dispense,
        cqrs_es::persist::PersistedEventStore<dynamo_es::DynamoEventRepository, Dispense>,
    >,
    s3_client: &aws_sdk_s3::Client,
) -> Result<Value, Error> {
    // Detect event type
    if event.payload.get("Records").is_some() {
        if let Some(records) = event.payload.get("Records").and_then(|r| r.as_array()) {
            if let Some(first_record) = records.first() {
                // Check if it's an S3 event
                if first_record.get("s3").is_some() {
                    tracing::info!("Detected S3 event");
                    let s3_event: S3Event = serde_json::from_value(event.payload)?;
                    handle_s3_event(s3_event, cqrs, s3_client).await?;
                    return Ok(serde_json::json!({"statusCode": 200}));
                }
                // Check if it's a Kinesis event
                else if first_record.get("kinesis").is_some() {
                    tracing::info!("Detected Kinesis event");
                    let kinesis_event: KinesisEvent = serde_json::from_value(event.payload)?;
                    let response = handle_kinesis_event(kinesis_event, cqrs, s3_client).await?;
                    return Ok(serde_json::to_value(response)?);
                }
            }
        }
    }

    tracing::warn!("Unknown event type");
    Ok(serde_json::json!({"statusCode": 200}))
}

async fn handle_s3_event(
    event: S3Event,
    cqrs: &cqrs_es::CqrsFramework<
        Dispense,
        cqrs_es::persist::PersistedEventStore<dynamo_es::DynamoEventRepository, Dispense>,
    >,
    s3_client: &aws_sdk_s3::Client,
) -> Result<(), Error> {
    tracing::info!("Processing {} S3 records", event.records.len());

    for record in event.records {
        let bucket = record.s3.bucket.name.ok_or("Missing bucket name")?;
        let key = record.s3.object.key.ok_or("Missing object key")?;

        tracing::info!("New file uploaded: s3://{}/{}", bucket, key);

        // Extract dispense ID from key pattern: prescriptions/{dispense_id}/prescription.jpg
        let parts: Vec<&str> = key.split('/').collect();
        if parts.len() >= 2 && parts[0] == "prescriptions" {
            let dispense_id = parts[1];

            tracing::info!("Analyzing prescription for dispense {}", dispense_id);

            tracing::info!("Processing prescription for dispense {}", dispense_id);

            // Step 1: Set prescription URL in the aggregate
            let prescription_id = Ulid::new().to_string();
            let prescription_url = format!("s3://{}/{}", bucket, key);
            let mut metadata = HashMap::new();
            metadata.insert("command_id".to_string(), Ulid::new().to_string());

            let upload_command = dispenses::Command::UploadPrescription {
                prescription_id,
                url: prescription_url.clone(),
            };

            cqrs.execute_with_metadata(dispense_id, upload_command, metadata.clone())
                .await?;

            tracing::info!("Prescription URL set for {}", dispense_id);

            // Step 2: Download and analyze file
            let file_data = download_from_s3(s3_client, &bucket, &key).await?;

            // TODO: Actual AI analysis
            // 1. Call Textract for OCR
            // 2. Call Claude for structured extraction
            // 3. Validate extracted data

            // Mock analysis result
            let analysis_data = serde_json::json!({
                "file_key": key,
                "file_size": file_data.len(),
                "patient_name": "John Doe",
                "medications": [
                    {"name": "Aspirin", "dosage": "500mg", "quantity": 30},
                    {"name": "Ibuprofen", "dosage": "200mg", "quantity": 20}
                ],
                "analyzed_at": chrono::Utc::now().to_rfc3339()
            });

            // Step 3: Store analysis results
            metadata.insert("command_id".to_string(), Ulid::new().to_string());

            let analyze_command = dispenses::Command::AnalyzePrescription {
                analysis_data: serde_json::to_string(&analysis_data)?,
            };

            cqrs.execute_with_metadata(dispense_id, analyze_command, metadata)
                .await?;

            tracing::info!("Prescription analyzed for {}", dispense_id);
        } else {
            tracing::warn!("Invalid S3 key format: {}", key);
        }
    }

    Ok(())
}

async fn handle_kinesis_event(
    event: KinesisEvent,
    cqrs: &cqrs_es::CqrsFramework<
        Dispense,
        cqrs_es::persist::PersistedEventStore<dynamo_es::DynamoEventRepository, Dispense>,
    >,
    _s3_client: &aws_sdk_s3::Client,
) -> Result<KinesisEventResponse, Error> {
    tracing::info!("Processing {} Kinesis records", event.records.len());

    let mut batch_item_failures = Vec::new();

    for record in event.records.iter() {
        let sequence = record.kinesis.sequence_number.clone();

        if let Err(e) = handle_kinesis_record(record, cqrs).await {
            tracing::error!("Failed to process: {}", e);
            batch_item_failures.push(KinesisBatchItemFailure {
                item_identifier: sequence,
            });
        }
    }

    Ok(KinesisEventResponse {
        batch_item_failures,
    })
}

async fn handle_kinesis_record(
    record: &KinesisEventRecord,
    cqrs: &cqrs_es::CqrsFramework<
        Dispense,
        cqrs_es::persist::PersistedEventStore<dynamo_es::DynamoEventRepository, Dispense>,
    >,
) -> Result<(), Error> {
    let data = std::str::from_utf8(&record.kinesis.data)?;
    let event: DomainEvent = serde_json::from_str(data)?;

    // Only process PrescriptionUploaded events
    if event.event_type == "Dispense:PrescriptionUploaded" {
        tracing::info!(
            "Processing PrescriptionUploaded event for dispense {}",
            event.id
        );
        // Additional processing if needed when prescription URL is set via API
    }

    Ok(())
}

async fn download_from_s3(
    s3_client: &aws_sdk_s3::Client,
    bucket: &str,
    key: &str,
) -> Result<Vec<u8>, Error> {
    let response = s3_client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await?;

    let data = response.body.collect().await?;
    Ok(data.to_vec())
}
