#[derive(Debug)]
#[non_exhaustive]
pub enum DiscoveryError {
    MdnsSetupFailed(String),
    BrowseTimeout,
    RegisterFailed(String),
    UnregisterFailed(String),
    IoError(std::io::Error),
}

impl std::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiscoveryError::MdnsSetupFailed(s) => write!(f, "Mdns setup failed: {s}"),
            DiscoveryError::BrowseTimeout => write!(f, "Browse timeout"),
            DiscoveryError::RegisterFailed(s) => write!(f, "Register failed: {s}"),
            DiscoveryError::UnregisterFailed(s) => write!(f, "Unregister failed: {s}"),
            DiscoveryError::IoError(e) => write!(f, "IO error: {e}"),
        }
    }
}

impl std::error::Error for DiscoveryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DiscoveryError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, DiscoveryError>;