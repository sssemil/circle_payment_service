use crate::api::ApiError;
use crate::models::RequestId;
use std::error::Error;
use std::fmt::{Display, Formatter};

pub type Result<T> = std::result::Result<T, CircleError>;

#[derive(Debug)]
pub enum CircleError {
    ApiError(RequestId, ApiError),
    ValueError,
    MissingRequestId,
    RequestIdIsNotAValidString(reqwest::header::ToStrError),
    RequestIdIsNotAValidUuid(uuid::Error),
    UnknownRequestError(reqwest::Error),
    FromHexError(hex::FromHexError),
    RsaError(rsa::errors::Error),
}

impl Display for CircleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "CircleError({:?})", self)
    }
}

impl Error for CircleError {}

impl From<reqwest::header::ToStrError> for CircleError {
    fn from(err: reqwest::header::ToStrError) -> Self {
        CircleError::RequestIdIsNotAValidString(err)
    }
}

impl From<uuid::Error> for CircleError {
    fn from(err: uuid::Error) -> Self {
        CircleError::RequestIdIsNotAValidUuid(err)
    }
}

impl From<reqwest::Error> for CircleError {
    fn from(err: reqwest::Error) -> Self {
        CircleError::UnknownRequestError(err)
    }
}

impl From<hex::FromHexError> for CircleError {
    fn from(err: hex::FromHexError) -> Self {
        CircleError::FromHexError(err)
    }
}

impl From<rsa::errors::Error> for CircleError {
    fn from(err: rsa::errors::Error) -> Self {
        CircleError::RsaError(err)
    }
}
