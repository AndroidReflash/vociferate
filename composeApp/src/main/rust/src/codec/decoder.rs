use crate::codec::config::SAMPLE_RATE;
use crate::codec::error::DecoderError;
use opus::Decoder as OpusDecoderInner;
use opus::Channels;

pub struct OpusDecoder {
    decoder: OpusDecoderInner,
}

impl OpusDecoder {
    pub fn new() -> Result<Self, DecoderError> {
        let decoder = OpusDecoderInner::new(SAMPLE_RATE, Channels::Mono)
            .map_err(|e| DecoderError::Creation(e.to_string()))?;
        Ok(Self { decoder })
    }

    pub fn decode(&mut self, opus_data: &[u8]) -> Result<Vec<f32>, DecoderError> {
        let mut pcm_out = vec![0.0f32; 5760];

        let samples_decoded = self.decoder.decode_float(opus_data, &mut pcm_out, false)
            .map_err(|e| DecoderError::Decode(e.to_string()))?;

        pcm_out.truncate(samples_decoded);

        Ok(pcm_out)
    }


}

impl Default for OpusDecoder {
    fn default() -> Self {
        Self::new().expect("OpusDecoder creation failed")
    }
}