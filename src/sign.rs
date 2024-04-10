use crate::{
    parameters::Parameter,
    signature_base::{Component, SignatureBaseError},
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

pub struct Signer<S> {
    signing_key: S,
    components: Vec<Component>,
    parameters: Vec<Parameter>,
}

impl<S> Signer<S> {
    pub fn builder() -> SignerBuilder<S, WithoutKey> {
        SignerBuilder {
            key: None,
            _key_marker: WithoutKey,
        }
    }
}

pub struct WithKey;
pub struct WithoutKey;

pub struct SignerBuilder<S, K> {
    key: Option<S>,

    _key_marker: K,
}

impl<S, WithKey> SignerBuilder<S, WithKey> {
    pub fn build(self) -> Signer<S> {
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
    use ::p256::ecdsa::{Error, Signature, SigningKey};
    use ::p256::elliptic_curve::rand_core::OsRng;
    use http::HeaderValue;

    use crate::signature_base::SignatureBase;

    use super::*;

    impl SignerBuilder<SigningKey, WithoutKey> {
        pub fn with_key(self, key: SigningKey) -> SignerBuilder<SigningKey, WithKey> {
            SignerBuilder {
                key: Some(key),
                _key_marker: WithKey,
            }
        }
    }

    impl SignRequest for Signer<SigningKey> {
        fn sign<T>(&self, request: &mut http::Request<T>) -> Result<(), SigningError> {
            let sb = SignatureBase::from_parts(&request, &self.components, &self.parameters)?
                .serialize()?;
            let signature: Signature =
                RandomizedSigner::try_sign_with_rng(&self.signing_key, &mut OsRng, sb.as_bytes())
                    .map_err(|_| SigningError::SigningFailed)?;
            request.headers_mut().append(
                "Signature",
                HeaderValue::from_str(signature.to_string().as_str()).unwrap(),
            );

            // TODO: signature input header

            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::p256::*;
        use super::*;
        use ::p256::ecdsa::SigningKey;
        use ::p256::SecretKey;
        use indoc::indoc;

        const ECC_P256_TEST_KEY: &str = indoc! {"
            -----BEGIN EC PRIVATE KEY-----
            MHcCAQEEIFKbhfNZfpDsW43+0+JjUr9K+bTeuxopu653+hBaXGA7oAoGCCqGSM49
            AwEHoUQDQgAEqIVYZVLCrPZHGHjP17CTW0/+D9Lfw0EkjqF7xB4FivAxzic30tMM
            4GF+hR6Dxh71Z50VGGdldkkDXZCnTNnoXQ==
            -----END EC PRIVATE KEY-----
        "};

        #[test]
        fn test_sign() {
            let sk = SecretKey::from_sec1_pem(ECC_P256_TEST_KEY).unwrap();
            let key: SigningKey = SigningKey::from(sk);
            let signer = Signer::builder().with_key(key).build();
            let mut request = http::Request::get("https://example.com").body(()).unwrap();

            signer.sign(&mut request).unwrap();
        }
    }
}
