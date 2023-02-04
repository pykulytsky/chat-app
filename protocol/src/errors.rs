use thiserror::Error;
use tracing_subscriber::filter::ParseError;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("frame encode error")]
    Encode(#[from] std::io::Error),
    #[error("frame decode error")]
    Decode,
    #[error("unable to serialize value")]
    Serialize(#[from] bincode::Error),
}

pub type Result<T> = std::result::Result<T, ProtocolError>;

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("unable listen to adderss")]
    AlreadyInUse(#[from] std::io::Error),
    #[error("frame parsing error")]
    Parse(#[from] ProtocolError),
    #[error("peer is not authorized")]
    Unauthorized,
    #[error("mesage parsing error")]
    MessageParse,
    #[error("failed to establish tracing")]
    TracingError(#[from] ParseError),
}
