use crate::signature_base::{Component, DerivedComponent, SignatureBaseError};
use thiserror::Error;

pub struct SignConfig {
    pub components: Vec<Component>,
}

#[derive(Error, Debug)]
pub enum SigningError {
    #[error("error forming signature base")]
    SignatureBaseError(#[from] SignatureBaseError),
}
