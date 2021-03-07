use base64::DecodeError as Base64DecodeError;
use hyper::Error as HyperError;
use prost::DecodeError as ProstDecodeError;
use thiserror::Error;
use tonic::transport::Error as TransportError;
use tonic::Status;

#[derive(Error, Debug)]
pub enum Error {
    #[error("No services found")]
    NoServices,
    #[error("No response")]
    NoResponse,
    #[error("Invalid request")]
    InvalidRequest,

    #[error("HyperError: {0}")]
    HyperError(#[from] HyperError),
    #[error("TransportError: {0}")]
    TransportError(#[from] TransportError),
    #[error("Base64DecodeError: {0}")]
    Base64DecodeError(#[from] Base64DecodeError),
    #[error("ProstDecodeError: {0}")]
    ProstDecodeError(#[from] ProstDecodeError),
    #[error("Status: {0}")]
    Status(#[from] Status),
}
