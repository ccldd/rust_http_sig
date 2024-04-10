use std::fmt::{Debug, Display};

use http::{HeaderName, Request};
use indexmap::IndexMap;
use sfv::{BareItem, Dictionary, InnerList, Item, ListEntry, Parameters, SerializeValue};
use thiserror::Error;

use crate::parameters::Parameter;

#[derive(Default)]
pub(crate) struct SignatureBase {
    components: IndexMap<String, String>,
    parameters: SignatureParams,
}

impl SignatureBase {
    pub(crate) fn from_parts<T>(
        req: &Request<T>,
        components: &[Component],
        parameters: &[Parameter],
    ) -> Result<Self, SignatureBaseError> {
        let mut sb = SignatureBase::default();

        for component in components {
            let (name, value) = match component {
                Component::Derived(derived) => {
                    let value = match derived {
                        DerivedComponent::Method => req.method().to_string(),
                    };
                    (derived.to_string(), value)
                }
                Component::Header(header_name) => match req.headers().get(header_name) {
                    Some(header_value) => (
                        header_name.to_string(),
                        header_value.to_str().unwrap_or_default().to_owned(),
                    ),
                    None => return Err(SignatureBaseError::MissingHeader(header_name.clone())),
                },
            };
            sb.components.insert(name.clone(), value);
            sb.parameters
                .0
                .items
                .push(Item::new(BareItem::String(name)));
        }

        for p in parameters {
            sb.parameters
                .0
                .params
                .insert(p.name().to_string(), p.into());
        }

        Ok(sb)
    }

    pub(crate) fn serialize(&self) -> Result<String, SignatureBaseError> {
        use std::fmt::Write;
        let mut s = String::new();

        // components
        for c in &self.components {
            writeln!(s, "\"{}\": {}", c.0, c.1)
                .map_err(|_| SignatureBaseError::CannotSerializeSignatureParams)?;
        }

        // then @signature-params
        write!(
            s,
            "\"{}\": {}",
            SignatureParams::component_name(),
            self.parameters.serialize()?
        )
        .map_err(|_| SignatureBaseError::CannotSerializeSignatureParams)?;

        Ok(s)
    }
}

struct SignatureParams(InnerList);

impl SignatureParams {
    fn component_name() -> &'static str {
        "@signature-params"
    }

    fn serialize(&self) -> Result<String, SignatureBaseError> {
        let d = Dictionary::from([(
            "placeholder".to_string(),
            ListEntry::InnerList(self.0.clone()),
        )]);
        let s = d
            .serialize_value()
            .map_err(|_| SignatureBaseError::CannotSerializeSignatureParams)?;
        Ok(s["placeholder=".len()..].to_owned())
    }
}

impl Default for SignatureParams {
    fn default() -> Self {
        Self(InnerList {
            items: Vec::default(),
            params: Parameters::default(),
        })
    }
}

pub enum DerivedComponent {
    Method,
}

impl DerivedComponent {
    fn from_request<T>(&self, r: &Request<T>) -> String {
        match self {
            DerivedComponent::Method => r.method().to_string(),
        }
    }
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

    #[error("error forming @signature-params")]
    CannotSerializeSignatureParams,
}

pub enum Component {
    Derived(DerivedComponent),
    Header(HeaderName),
}

#[cfg(test)]
mod tests {
    use http::Request;

    use crate::parameters::{Created, Parameter};

    use super::*;

    #[test]
    fn test() {
        let req = Request::get("uri").body(()).unwrap();
        let sb = SignatureBase::from_parts(
            &req,
            &[Component::Derived(super::DerivedComponent::Method)],
            &[Parameter::Created(Created::Now)],
        )
        .unwrap();

        assert_eq!("", sb.serialize().unwrap())
    }
}
