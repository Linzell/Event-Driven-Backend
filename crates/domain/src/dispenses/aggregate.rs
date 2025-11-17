use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cqrs_es::Aggregate;
use serde::{Deserialize, Serialize};

use crate::errors::Error;

use super::{Command, Event};

/// Dispense workflow status
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DispenseStatus {
    /// Initial state - user started dispense
    Pending,
    /// Prescription uploaded, waiting for analysis
    Analyzing,
    /// Analysis complete, ready to add patient/drugs
    Ready,
    /// Patient and drugs added, ready to dispense
    Complete,
    /// Dispense cancelled
    Cancelled,
}

/// Dispense aggregate
#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct Dispense {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: DispenseStatus,
    
    // Prescription data
    pub prescription_id: Option<String>,
    pub prescription_url: Option<String>,
    pub prescription_analyzed: bool,
    
    // Patient data
    pub patient_id: Option<String>,
    pub patient_name: Option<String>,
    
    // Drugs data
    pub drugs: Vec<DrugItem>,
    
    pub deleted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct DrugItem {
    pub drug_id: String,
    pub name: String,
    pub quantity: u32,
}

impl Default for DispenseStatus {
    fn default() -> Self {
        Self::Pending
    }
}

pub const AGGREGATE_TYPE: &str = "Dispense";

#[derive(Clone, Default)]
pub struct Services {}

#[async_trait]
impl Aggregate for Dispense {
    type Command = Command;
    type Event = Event;
    type Error = Error;
    type Services = Services;

    fn aggregate_type() -> String {
        AGGREGATE_TYPE.to_string()
    }

    async fn handle(
        &self,
        command: Self::Command,
        _services: &Self::Services,
    ) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            Command::StartDispense { id } => {
                self.validate_new()?;
                let now = Utc::now();
                
                Ok(vec![Event::DispenseStarted {
                    id,
                    created_at: now,
                    status: DispenseStatus::Pending,
                }])
            }

            Command::UploadPrescription { prescription_id, url } => {
                self.validate_existing()?;
                
                Ok(vec![Event::PrescriptionUploaded {
                    id: self.id.clone(),
                    prescription_id,
                    url,
                    updated_at: Utc::now(),
                }])
            }

            Command::AnalyzePrescription { analysis_data } => {
                self.validate_existing()?;
                
                Ok(vec![Event::PrescriptionAnalyzed {
                    id: self.id.clone(),
                    analysis_data,
                    updated_at: Utc::now(),
                }])
            }

            Command::AddPatient { patient_id, name } => {
                self.validate_existing()?;
                
                Ok(vec![Event::PatientAdded {
                    id: self.id.clone(),
                    patient_id,
                    patient_name: name,
                    updated_at: Utc::now(),
                }])
            }

            Command::AddDrugs { drugs } => {
                self.validate_existing()?;
                
                Ok(vec![Event::DrugsAdded {
                    id: self.id.clone(),
                    drugs,
                    updated_at: Utc::now(),
                }])
            }

            Command::CompleteDispense => {
                self.validate_existing()?;
                self.validate_can_complete()?;
                
                Ok(vec![Event::DispenseCompleted {
                    id: self.id.clone(),
                    updated_at: Utc::now(),
                }])
            }

            Command::CancelDispense => {
                self.validate_existing()?;
                
                Ok(vec![Event::DispenseCancelled {
                    id: self.id.clone(),
                    updated_at: Utc::now(),
                }])
            }
        }
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            Event::DispenseStarted { id, created_at, status } => {
                self.id = id;
                self.created_at = created_at;
                self.updated_at = created_at;
                self.status = status;
            }

            Event::PrescriptionUploaded { prescription_id, url, updated_at, .. } => {
                self.prescription_id = Some(prescription_id);
                self.prescription_url = Some(url);
                self.status = DispenseStatus::Analyzing;
                self.updated_at = updated_at;
            }

            Event::PrescriptionAnalyzed { updated_at, .. } => {
                self.prescription_analyzed = true;
                self.status = DispenseStatus::Ready;
                self.updated_at = updated_at;
            }

            Event::PatientAdded { patient_id, patient_name, updated_at, .. } => {
                self.patient_id = Some(patient_id);
                self.patient_name = Some(patient_name);
                self.updated_at = updated_at;
            }

            Event::DrugsAdded { drugs, updated_at, .. } => {
                self.drugs = drugs;
                self.updated_at = updated_at;
            }

            Event::DispenseCompleted { updated_at, .. } => {
                self.status = DispenseStatus::Complete;
                self.updated_at = updated_at;
            }

            Event::DispenseCancelled { updated_at, .. } => {
                self.status = DispenseStatus::Cancelled;
                self.updated_at = updated_at;
            }
        }
    }
}

impl Dispense {
    fn validate_new(&self) -> Result<(), Error> {
        if !self.id.is_empty() {
            return Err(Error::Uniqueness { field: "id".to_string() });
        }
        Ok(())
    }

    fn validate_existing(&self) -> Result<(), Error> {
        if self.id.is_empty() {
            return Err(Error::NotFound { entity: AGGREGATE_TYPE.to_string() });
        }
        if self.deleted {
            return Err(Error::Forbidden);
        }
        Ok(())
    }

    fn validate_can_complete(&self) -> Result<(), Error> {
        if self.patient_id.is_none() {
            return Err(Error::Validation {
                message: "Cannot complete dispense without patient".to_string(),
            });
        }
        if self.drugs.is_empty() {
            return Err(Error::Validation {
                message: "Cannot complete dispense without drugs".to_string(),
            });
        }
        Ok(())
    }
}
