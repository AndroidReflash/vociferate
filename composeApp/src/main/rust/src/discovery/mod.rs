mod config;
mod peer;
mod error;
mod service;
mod heartbeat;

pub use error::DiscoveryError;
pub use heartbeat::{ GlobalDiscoverySender, GlobalDiscoveryReceiver };