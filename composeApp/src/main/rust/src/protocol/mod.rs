mod capabilities;
mod error;
mod frame;
mod handshake;

pub use capabilities::Capabilities;
pub use error::{ProtocolError, Result};
pub use frame::{Frame, FrameType, PROTOCOL_VERSION, MAGIC_BYTES};
pub use handshake::HandshakePayload;