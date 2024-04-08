use chrono::{DateTime, TimeDelta, Utc};
use sfv::BareItem;
use uuid::Uuid;

#[derive(Debug)]
pub enum Parameter {
    Created(Created),
    Expires(Expires),
    Nonce(Nonce),
}

#[derive(Debug)]
pub enum Created {
    Now,
    Value(DateTime<Utc>),
}

#[derive(Debug)]
pub enum Expires {
    FromNow(TimeDelta),
    Value(DateTime<Utc>),
}

#[derive(Debug)]
pub enum Nonce {
    Random,
    Value(String),
}

impl Parameter {
    pub fn name(&self) -> &str {
        match self {
            Parameter::Created(_) => "created",
            Parameter::Expires(_) => "expires",
            Parameter::Nonce(_) => "nonce",
        }
    }
}

impl From<&Parameter> for BareItem {
    fn from(value: &Parameter) -> Self {
        match value {
            Parameter::Created(c) => c.into(),
            Parameter::Expires(e) => e.into(),
            Parameter::Nonce(n) => n.into(),
        }
    }
}

impl From<&Created> for BareItem {
    fn from(value: &Created) -> Self {
        match value {
            Created::Now => BareItem::Integer(Utc::now().timestamp()),
            Created::Value(dt) => BareItem::Integer(dt.timestamp()),
        }
    }
}

impl From<&Expires> for BareItem {
    fn from(value: &Expires) -> Self {
        match value {
            Expires::FromNow(delta) => BareItem::Integer((Utc::now() + *delta).timestamp()),
            Expires::Value(dt) => BareItem::Integer(dt.timestamp()),
        }
    }
}

impl From<&Nonce> for BareItem {
    fn from(value: &Nonce) -> Self {
        match value {
            Nonce::Random => BareItem::String(Uuid::new_v4().to_string()),
            Nonce::Value(v) => BareItem::String(v.clone()),
        }
    }
}
