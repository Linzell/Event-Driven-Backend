use chrono::{DateTime, Utc};
use cqrs_es::DomainEvent;
use serde::{Deserialize, Serialize};
use super::aggregate::{DispenseStatus, DrugItem};

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "type")]
pub enum Event {
    DispenseStarted {
        id: String,
        created_at: DateTime<Utc>,
        status: DispenseStatus,
    },

    PrescriptionUploaded {
        id: String,
        prescription_id: String,
        url: String,
        updated_at: DateTime<Utc>,
    },

    PrescriptionAnalyzed {
        id: String,
        analysis_data: String, // JSON with extracted info
        updated_at: DateTime<Utc>,
    },

    PatientAdded {
        id: String,
        patient_id: String,
        patient_name: String,
        updated_at: DateTime<Utc>,
    },

    DrugsAdded {
        id: String,
        drugs: Vec<DrugItem>,
        updated_at: DateTime<Utc>,
    },

    DispenseCompleted {
        id: String,
        updated_at: DateTime<Utc>,
    },

    DispenseCancelled {
        id: String,
        updated_at: DateTime<Utc>,
    },
}

impl DomainEvent for Event {
    fn event_type(&self) -> String {
        match self {
            Event::DispenseStarted { .. } => "Dispense:Started".to_string(),
            Event::PrescriptionUploaded { .. } => "Dispense:PrescriptionUploaded".to_string(),
            Event::PrescriptionAnalyzed { .. } => "Dispense:PrescriptionAnalyzed".to_string(),
            Event::PatientAdded { .. } => "Dispense:PatientAdded".to_string(),
            Event::DrugsAdded { .. } => "Dispense:DrugsAdded".to_string(),
            Event::DispenseCompleted { .. } => "Dispense:Completed".to_string(),
            Event::DispenseCancelled { .. } => "Dispense:Cancelled".to_string(),
        }
    }

    fn event_version(&self) -> String {
        "1.0".to_string()
    }
}
