use std::fmt::Display;

use http::{HeaderName, Request};
use indexmap::IndexMap;
use thiserror::Error;

use crate::sign::SignConfig;

#[derive(Default)]
pub struct SignatureBase {
    components: IndexMap<String, String>,
}

impl SignatureBase {
    fn for_signing<T>(req: Request<T>, config: SignConfig) -> Result<Self, SignatureBaseError> {
        let mut sb = SignatureBase::default();

        for component in config.components {
            let (name, value) = match component {
                Component::Derived(derived) => {
                    let value = match derived {
                        DerivedComponent::Method => req.method().to_string(),
                    };
                    (derived.to_string(), value)
                }
                Component::Header(header_name) => match req.headers().get(&header_name) {
                    Some(header_value) => (
                        header_name.to_string(),
                        header_value.to_str().unwrap_or_default().to_owned(),
                    ),
                    None => return Err(SignatureBaseError::MissingHeader(header_name)),
                },
            };
            sb.components.insert(name, value);
        }

        Ok(sb)
    }
}

pub enum DerivedComponent {
    Method,
}

impl Display for DerivedComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Method => write!(f, "@method"),
        }
    }
}

#[derive(Error, Debug)]
pub enum SignatureBaseError {
    #[error("{0} header is missing from the message")]
    MissingHeader(HeaderName),
}

pub enum Component {
    Derived(DerivedComponent),
    Header(HeaderName),
}
