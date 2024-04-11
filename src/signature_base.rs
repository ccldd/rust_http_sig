use std::fmt::{Debug, Display};

use http::{HeaderName, Request};
use indexmap::IndexMap;
use sfv::{BareItem, Dictionary, InnerList, Item, ListEntry, Parameters, SerializeValue};
use thiserror::Error;

use crate::parameters::Parameter;

#[derive(Default)]
pub(crate) struct SignatureBase {
    components: IndexMap<String, String>,
    pub parameters: SignatureParams,
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
            SignatureParams::COMPONENT_NAME,
            self.parameters.serialize()?
        )
        .map_err(|_| SignatureBaseError::CannotSerializeSignatureParams)?;

        Ok(s)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SignatureParams(InnerList);

impl SignatureParams {
    const COMPONENT_NAME: &'static str = "@signature-params";

    fn serialize(&self) -> Result<String, SignatureBaseError> {
        let mut s = self.serialize_with_label("x")?;
        s.remove(0); // remove x
        s.remove(0); // remove =
        Ok(s)
    }

    fn serialize_with_label(&self, label: &str) -> Result<String, SignatureBaseError> {
        let d = Dictionary::from([(label.to_string(), ListEntry::InnerList(self.0.clone()))]);
        let s = d
            .serialize_value()
            .map_err(|_| SignatureBaseError::CannotSerializeSignatureParams)?;
        Ok(s)
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

pub(crate) struct SignatureInput(pub String, pub SignatureParams);

impl SignatureInput {
    pub const HEADER_NAME: &'static str = "Signature-Input";

    pub fn serialize(&self, label: &str) -> Result<String, SignatureBaseError> {
        let s = format!(
            "{}: {}",
            SignatureInput::HEADER_NAME,
            self.1
                .serialize_with_label(label)
                .map_err(|_| SignatureBaseError::CannotSerializeSignatureInput)?
        );
        Ok(s)
    }
}

#[derive(Error, Debug)]
pub enum SignatureBaseError {
    #[error("{0} header is missing from the message")]
    MissingHeader(HeaderName),

    #[error("error forming @signature-params")]
    CannotSerializeSignatureParams,

    #[error("error forming Signature-Input")]
    CannotSerializeSignatureInput,
}

pub enum Component {
    Derived(DerivedComponent),
    Header(HeaderName),
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use http::Request;
    use indoc::indoc;

    use crate::parameters::{Created, Parameter};

    use super::*;

    #[test]
    fn test() {
        let req = Request::get("https://subdomain.example.com/path/another_path?a=1,b=2#c")
            .body(())
            .unwrap();
        let sb = SignatureBase::from_parts(
            &req,
            &[Component::Derived(super::DerivedComponent::Method)],
            &[Parameter::Created(Created::Value(
                DateTime::from_timestamp(1712821981, 0).unwrap(),
            ))],
        )
        .unwrap();

        assert_eq!(
            sb.serialize().unwrap(),
            indoc! {r#"
                "@method": GET
                "@signature-params": ("@method");created=1712821981"#
            }
        )
    }
}
