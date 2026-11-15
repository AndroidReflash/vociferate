#[derive(Debug)]
pub enum DiscoveryError {}

impl std::fmt::Display for DiscoveryError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {}
    }
}

impl std::error::Error for DiscoveryError {}
