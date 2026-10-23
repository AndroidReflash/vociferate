use uniffi;

#[derive(Debug, Clone, uniffi::Record)]
pub struct JitterBufferConfig {
    pub capacity_frames: u32,
    pub target_delay_frames: u32,
    pub min_delay_frames: u32,
    pub max_delay_frames: u32,
    pub adaptive: bool,
}

impl Default for JitterBufferConfig {
    fn default() -> Self {
        Self {
            capacity_frames: 200,
            target_delay_frames: 3,
            min_delay_frames: 2,
            max_delay_frames: 8,
            adaptive: true,
        }
    }
}

impl JitterBufferConfig {
    pub fn new(capacity_frames: u32, target_delay_frames: u32) -> Self {
        Self {
            capacity_frames,
            target_delay_frames,
            min_delay_frames: target_delay_frames.saturating_sub(1),
            max_delay_frames: target_delay_frames.saturating_mul(2).max(4),
            adaptive: true,
        }
    }

    pub fn with_adaptive(mut self, adaptive: bool) -> Self {
        self.adaptive = adaptive;
        self
    }

    pub fn with_bounds(mut self, min_delay_frames: u32, max_delay_frames: u32) -> Self {
        self.min_delay_frames = min_delay_frames;
        self.max_delay_frames = max_delay_frames;
        self
    }
}