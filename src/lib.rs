pub mod error;
pub mod handler;
mod log;
pub mod macros;
pub mod messaging;
pub mod supervision;

pub use handler::AskHandlerTrait;
pub use handler::TellHandlerTrait;
pub use messaging::bounded_channel;
pub use messaging::unbounded_channel;
pub use supervision::ActorTrait;
pub use supervision::CommandMessage;
pub use supervision::spawn;

#[cfg(feature = "macros")]
pub use ascolt_macros::*;
