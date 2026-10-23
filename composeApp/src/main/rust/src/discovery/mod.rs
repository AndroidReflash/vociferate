mod config;
mod peer;
mod error;
mod service;
mod heartbeat;

pub use config::SERVICE_TYPE;
pub use peer::Capabilities;
pub use error::DiscoveryError;
pub use service::MdnsService;
pub use heartbeat::{ GlobalDiscoverySender, GlobalDiscoveryReceiver };