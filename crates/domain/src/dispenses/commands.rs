use serde::{Deserialize, Serialize};
use super::aggregate::DrugItem;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Command {
    /// Start a new dispense workflow
    StartDispense {
        id: String,
    },

    /// Upload prescription document
    UploadPrescription {
        prescription_id: String,
        url: String,
    },

    /// Analyze prescription (triggered by projector)
    AnalyzePrescription {
        analysis_data: String, // JSON
    },

    /// Add patient information
    AddPatient {
        patient_id: String,
        name: String,
    },

    /// Add drugs to dispense
    AddDrugs {
        drugs: Vec<DrugItem>,
    },

    /// Mark dispense as complete
    CompleteDispense,

    /// Cancel the dispense
    CancelDispense,
}
