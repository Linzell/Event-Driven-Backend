use aws_config::BehaviorVersion;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use domain::dispenses::{self, Dispense};
use std::{collections::HashMap, sync::Arc};
use ulid::Ulid;

#[derive(Clone)]
struct AppState {
    dispenses_repo: Arc<Box<dyn cqrs_es::persist::ViewRepository<dispenses::View, Dispense>>>,
    dispenses_cqrs: Arc<
        cqrs_es::CqrsFramework<
            Dispense,
            cqrs_es::persist::PersistedEventStore<dynamo_es::DynamoEventRepository, Dispense>,
        >,
    >,
    s3_client: aws_sdk_s3::Client,
}

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
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
    let dispenses_cqrs = dispenses::cqrs::init(dynamodb_client, dispenses_repo.clone());

    let state = AppState {
        dispenses_repo,
        dispenses_cqrs,
        s3_client,
    };

    let app = Router::new()
        .route("/dispenses", post(create_dispense).get(list_dispenses))
        .route("/dispenses/:id", get(get_dispense).delete(cancel_dispense))
        .route(
            "/dispenses/:id/prescription/upload-url",
            post(get_upload_url),
        )
        .route("/dispenses/:id/patient", post(add_patient))
        .route("/dispenses/:id/drugs", post(add_drugs))
        .route("/dispenses/:id/complete", post(complete_dispense))
        .with_state(state);

    let app = tower::ServiceBuilder::new()
        .layer(axum_aws_lambda::LambdaLayer::default())
        .service(app);

    lambda_http::run(app).await?;
    Ok(())
}

// Create dispense
async fn create_dispense(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let command_id = Ulid::new().to_string();
    let aggregate_id = Ulid::new().to_string();

    let mut metadata = HashMap::new();
    metadata.insert("command_id".to_string(), command_id);

    let command = dispenses::Command::StartDispense {
        id: aggregate_id.clone(),
    };

    state
        .dispenses_cqrs
        .execute_with_metadata(&aggregate_id, command, metadata)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let view = state
        .dispenses_repo
        .load(&aggregate_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Not found".to_string()))?;

    Ok((StatusCode::CREATED, Json(view)))
}

// Get dispense
async fn get_dispense(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let view = state
        .dispenses_repo
        .load(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Not found".to_string()))?;

    Ok(Json(view))
}

// List dispenses (simplified - in production use pagination)
async fn list_dispenses(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: Implement proper listing with DynamoDB scan/query
    Ok(Json(
        serde_json::json!({ "message": "List not implemented yet" }),
    ))
}

// Get S3 presigned URL for upload
async fn get_upload_url(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(input): Json<dispenses::inputs::UploadPrescriptionInput>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let bucket =
        std::env::var("PRESCRIPTIONS_BUCKET").unwrap_or("dispensary-prescriptions".to_string());

    let prescription_id = Ulid::new().to_string();
    let key = format!("prescriptions/{}/{}", id, prescription_id);

    let presigned = state
        .s3_client
        .put_object()
        .bucket(&bucket)
        .key(&key)
        .content_type(&input.content_type)
        .presigned(
            aws_sdk_s3::presigning::PresigningConfig::expires_in(std::time::Duration::from_secs(
                3600,
            ))
            .unwrap(),
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "upload_url": presigned.uri(),
        "prescription_id": prescription_id,
        "key": key,
    })))
}

// Add patient
async fn add_patient(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(input): Json<dispenses::inputs::AddPatientInput>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut metadata = HashMap::new();
    metadata.insert("command_id".to_string(), Ulid::new().to_string());

    let command = dispenses::Command::AddPatient {
        patient_id: input.patient_id,
        name: input.name,
    };

    state
        .dispenses_cqrs
        .execute_with_metadata(&id, command, metadata)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::OK, "Patient added"))
}

// Add drugs
async fn add_drugs(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(input): Json<dispenses::inputs::AddDrugsInput>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut metadata = HashMap::new();
    metadata.insert("command_id".to_string(), Ulid::new().to_string());

    let command = dispenses::Command::AddDrugs { drugs: input.drugs };

    state
        .dispenses_cqrs
        .execute_with_metadata(&id, command, metadata)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::OK, "Drugs added"))
}

// Complete dispense
async fn complete_dispense(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut metadata = HashMap::new();
    metadata.insert("command_id".to_string(), Ulid::new().to_string());

    let command = dispenses::Command::CompleteDispense;

    state
        .dispenses_cqrs
        .execute_with_metadata(&id, command, metadata)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::OK, "Dispense completed"))
}

// Cancel dispense
async fn cancel_dispense(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut metadata = HashMap::new();
    metadata.insert("command_id".to_string(), Ulid::new().to_string());

    let command = dispenses::Command::CancelDispense;

    state
        .dispenses_cqrs
        .execute_with_metadata(&id, command, metadata)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::OK, "Dispense cancelled"))
}
