use crate::{
    parameters::Parameter,
    signature_base::{Component, SignatureBaseError},
    WithoutKey,
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SigningError {
    #[error("error forming signature base")]
    SignatureBaseError(#[from] SignatureBaseError),
    #[error("failed to sign signature base")]
    SigningFailed,
}

pub trait SignRequest {
    fn sign<T>(&self, request: &mut http::Request<T>) -> Result<(), SigningError>;
}

pub struct Signer<Key> {
    signing_key: Key,
    components: Vec<Component>,
    parameters: Vec<Parameter>,
}

impl<Key> Signer<Key> {
    pub fn builder() -> SignerBuilder<Key, WithoutKey> {
        SignerBuilder {
            key: None,
            key_marker: WithoutKey,
        }
    }
}

pub struct SignerBuilder<Key, K> {
    key: Option<Key>,

    key_marker: K,
}

impl<Key, WithKey> SignerBuilder<Key, WithKey> {
    pub fn build(self) -> Signer<Key> {
        Signer {
            signing_key: self.key.unwrap(),
            components: Vec::new(),
            parameters: Vec::new(),
        }
    }
}

#[cfg(feature = "p256")]
pub mod p256 {
    use ::p256::ecdsa::signature::{RandomizedSigner, Signer as _};
    use ::p256::ecdsa::{Signature, SigningKey};
    use ::p256::elliptic_curve::rand_core::OsRng;
    use http::HeaderValue;

    use crate::signature_base::{SignatureBase, SignatureInput};
    use crate::WithKey;

    use super::*;

    impl SignerBuilder<SigningKey, WithoutKey> {
        pub fn with_key(self, key: SigningKey) -> SignerBuilder<SigningKey, WithKey> {
            SignerBuilder {
                key: Some(key),
                key_marker: WithKey,
            }
        }
    }

    impl SignRequest for Signer<SigningKey> {
        fn sign<T>(&self, request: &mut http::Request<T>) -> Result<(), SigningError> {
            let sb = SignatureBase::from_parts(&request, &self.components, &self.parameters)?;
            let sb_str = sb.serialize()?;
            let signature: Signature = RandomizedSigner::try_sign_with_rng(
                &self.signing_key,
                &mut OsRng,
                sb_str.as_bytes(),
            )
            .map_err(|_| SigningError::SigningFailed)?;
            request.headers_mut().append(
                "Signature",
                HeaderValue::from_str(signature.to_string().as_str()).unwrap(),
            );

            // TODO: signature input header
            let label = "";
            let sig_input = SignatureInput(label.into(), sb.parameters);
            request.headers_mut().append(
                SignatureInput::HEADER_NAME,
                HeaderValue::from_str(sig_input.serialize(label.into())?.as_str()).unwrap(),
            );

            Ok(())
        }
    }
}
