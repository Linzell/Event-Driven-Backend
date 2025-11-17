use std::sync::Arc;
use async_trait::async_trait;
use cqrs_es::{
    persist::{PersistenceError, ViewContext, ViewRepository},
    Aggregate, EventEnvelope, View as CqrsView,
};
use serde::{Deserialize, Serialize};
use super::{Dispense, AGGREGATE_TYPE};

#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct View {
    pub aggregate_type: String,
    pub command_id: String,
    pub id: String,
    pub dispense: Dispense,
}

impl CqrsView<Dispense> for View {
    fn update(&mut self, event: &EventEnvelope<Dispense>) {
        self.id.clone_from(&event.aggregate_id);
        self.aggregate_type = AGGREGATE_TYPE.to_string();
        self.command_id = event
            .metadata
            .get("command_id")
            .unwrap_or(&"".to_string())
            .to_string();
        self.dispense.apply(event.payload.clone());
    }
}

pub struct Query {
    repo: Arc<Box<dyn ViewRepository<View, Dispense>>>,
}

impl Query {
    pub fn new(repo: Arc<Box<dyn ViewRepository<View, Dispense>>>) -> Self {
        Self { repo }
    }

    async fn update(
        &self,
        dispense_id: &str,
        events: &[EventEnvelope<Dispense>],
    ) -> Result<(), PersistenceError> {
        let (mut view, view_context) = match self.repo.load_with_context(dispense_id).await? {
            None => {
                let view_context = ViewContext::new(dispense_id.to_string(), 0);
                (Default::default(), view_context)
            }
            Some((view, context)) => (view, context),
        };

        for event in events {
            view.update(event);
        }

        self.repo.update_view(view, view_context).await
    }
}

#[async_trait]
impl cqrs_es::Query<Dispense> for Query {
    async fn dispatch(&self, dispense_id: &str, events: &[EventEnvelope<Dispense>]) {
        if let Err(err) = self.update(dispense_id, events).await {
            eprintln!("DispenseQuery error for {}: {}", dispense_id, err);
        }
    }
}
