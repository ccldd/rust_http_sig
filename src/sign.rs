use crate::signature_base::SignatureBaseError;
use ecdsa::{PrimeCurve, Signature};

use thiserror::Error;


#[derive(Error, Debug)]
pub enum SigningError {
    #[error("error forming signature base")]
    SignatureBaseError(#[from] SignatureBaseError),
}

pub struct Signer<C: PrimeCurve> {
    ecdsa: dyn ecdsa::signature::Signer<Signature<C>>,
}
    

trait SignRequest {
    fn sign<T>(&self, request: http::Request<T>) -> http::Request<T>;
}

impl<C: PrimeCurve> SignRequest for Signer<C> {
    fn sign<T>(&self, request: http::Request<T>) -> http::Request<T> {
        request
    }
}