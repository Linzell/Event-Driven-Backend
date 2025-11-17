/// Dispense aggregate
pub mod aggregate;

/// Commands
pub mod commands;

/// Events
pub mod events;

/// Input DTOs
pub mod inputs;

/// View (read model)
pub mod view;

/// CQRS setup
pub mod cqrs;

pub use aggregate::{Dispense, DispenseStatus, Services, AGGREGATE_TYPE};
pub use commands::Command;
pub use events::Event;
pub use view::{Query, View};
