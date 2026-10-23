#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Capabilities {
    pub opus: bool,
    pub sample_rates: Vec<u32>,
    pub channels: u16,
    pub max_bitrate: u32,
    pub supports_fec: bool,
    pub supports_dtx: bool,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            opus: true,
            sample_rates: vec![48000, 44100, 16000],
            channels: 1,
            max_bitrate: 64000,
            supports_fec: true,
            supports_dtx: true,
        }
    }
}