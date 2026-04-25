//! Domain events - business events that happen in the system
//!
//! Domain events represent facts that have occurred in the business domain.
//! They are immutable and should be named in past tense (e.g., UserRegistered).

use std::fmt::Debug;

/// Base trait for all domain events
///
/// All domain events must implement this trait. Events should be:
/// - Immutable
/// - Named in past tense
/// - Contain all necessary data to represent what happened
/// - Be serializable (for event sourcing/messaging)
pub trait DomainEvent: Debug + Send + Sync {
    /// Returns the event type identifier (e.g., "user.registered")
    fn event_type(&self) -> &'static str;

    /// Returns the ID of the aggregate that emitted this event
    fn aggregate_id(&self) -> &str;
}
