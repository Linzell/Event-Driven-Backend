use std::{env, sync::Arc};
use cqrs_es::{
    persist::{PersistedEventStore, ViewRepository},
    CqrsFramework,
};
use dynamo_es::{DynamoEventRepository, DynamoViewRepository};
use super::{Dispense, Query, Services, View};

pub fn init(
    client: aws_sdk_dynamodb::Client,
    repo: Arc<Box<dyn ViewRepository<View, Dispense>>>,
) -> Arc<CqrsFramework<Dispense, PersistedEventStore<DynamoEventRepository, Dispense>>> {
    let event_log_table = env::var("DYNAMODB_EVENT_LOG_TABLE")
        .unwrap_or("dispensary-event-log".to_string());

    let event_snapshots_table = env::var("DYNAMODB_EVENT_SNAPSHOTS_TABLE")
        .unwrap_or("dispensary-event-snapshots".to_string());

    let store: PersistedEventStore<DynamoEventRepository, Dispense> =
        PersistedEventStore::new_snapshot_store(
            DynamoEventRepository::new(client)
                .with_tables(&event_log_table, &event_snapshots_table),
            5,
        );

    let query = Box::new(Query::new(repo));

    Arc::new(CqrsFramework::new(store, vec![query], Services::default()))
}

pub fn init_repo(client: aws_sdk_dynamodb::Client) -> Arc<Box<dyn ViewRepository<View, Dispense>>> {
    let view_table = env::var("DYNAMODB_DISPENSES_VIEW_TABLE")
        .unwrap_or("dispensary-dispenses-view".to_string());

    Arc::new(Box::new(DynamoViewRepository::new(&view_table, client)))
}
