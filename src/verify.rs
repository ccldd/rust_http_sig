use std::{collections::HashMap, marker::PhantomData};

use either::Either;
use thiserror::Error;

use crate::WithoutKey;

pub trait VerifyRequest {
    fn verify<T>(&self, request: &mut http::Request<T>) -> Result<(), VerifyError>;
}

pub struct Verifier<Key> {
    verifying_key: Either<Key, HashMap<String, Key>>,
}

impl<Key> Verifier<Key> {
    pub fn builder() -> VerifierBuilder<Key, WithoutKey> {
        VerifierBuilder {
            inner: None,
            key_marker: WithoutKey,
        }
    }
}

pub struct VerifierBuilder<Key, K> {
    inner: Option<Verifier<Key>>,
    key_marker: K,
}

impl<Key, WithKey> VerifierBuilder<Key, WithKey> {
    pub fn build(self) -> Verifier<Key> {
        self.inner.unwrap()
    }
}

#[cfg(feature = "p256")]
mod p256 {
    use std::collections::HashMap;

    use either::Either;
    use http::Request;
    use p256::ecdsa::VerifyingKey;

    use crate::{VerifyError, VerifyRequest, WithKey, WithoutKey};

    use super::{Verifier, VerifierBuilder};

    impl VerifierBuilder<VerifyingKey, WithoutKey> {
        pub fn with_key(self, key: VerifyingKey) -> VerifierBuilder<VerifyingKey, WithKey> {
            VerifierBuilder {
                inner: Some(Verifier {
                    verifying_key: Either::Left(key),
                }),
                key_marker: WithKey,
            }
        }

        pub fn with_keys(
            self,
            keys: impl IntoIterator<Item = (String, VerifyingKey)>,
        ) -> VerifierBuilder<VerifyingKey, WithKey> {
            VerifierBuilder {
                inner: Some(Verifier {
                    verifying_key: Either::Right(HashMap::from_iter(keys)),
                }),
                key_marker: WithKey,
            }
        }
    }

    impl VerifyRequest for Verifier<VerifyingKey> {
        fn verify<T>(&self, request: &mut http::Request<T>) -> Result<(), VerifyError> {
            todo!()
        }
    }
}

#[derive(Error, Debug)]
pub enum VerifyError {
    #[error("signature header is missing")]
    MissingSignatureHeader,
    #[error("the signature is invalid")]
    InvalidSignature,
    #[error("actual signature \"{actual}\" does not match expected signature \"{expected}\"")]
    SignatureVerifyFailed { actual: String, expected: String },
}
