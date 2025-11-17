//! Dispensary Domain Models

/// Dispense aggregate
pub mod dispenses;

/// Domain errors
pub mod errors;

/// Domain events wrapper
pub mod event;

pub use errors::Error;
pub use event::DomainEvent;
