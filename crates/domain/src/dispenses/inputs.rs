use super::aggregate::DrugItem;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StartDispenseInput {
    // Empty for now, can add metadata later
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UploadPrescriptionInput {
    pub file_name: String,
    pub content_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddPatientInput {
    pub patient_id: String,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddDrugsInput {
    pub drugs: Vec<DrugItem>,
}
