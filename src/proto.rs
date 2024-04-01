use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use crate::Error;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: MessageBody,
}

impl Message {
    pub fn ty(&self) -> &str {
        &self.body.ty
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageBody {
    #[serde(rename = "type", default, skip_serializing_if = "String::is_empty")]
    pub ty: String,
    #[serde(default, skip_serializing_if = "u32_is_zero")]
    pub msg_id: u32,
    #[serde(default, skip_serializing_if = "u32_is_zero")]
    pub in_reply_to: u32,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

impl MessageBody {
    pub fn new(ty: impl Into<String>) -> Self {
        Self {
            ty: ty.into(),
            ..Default::default()
        }
    }

    pub fn with_field(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        self.extra.insert(
            key.into(),
            serde_json::to_value(value).expect("Failed to serialize value"),
        );
        self
    }

    pub fn to_message<T: DeserializeOwned>(&self) -> T {
        serde_json::from_value(serde_json::to_value(self).expect("Failed to serialize value"))
            .expect("Failed to serialize value")
    }
}

pub trait IntoBody {
    fn into_body(self) -> Option<MessageBody>;
}

impl IntoBody for MessageBody {
    fn into_body(self) -> Option<MessageBody> {
        Some(self)
    }
}

impl<T> IntoBody for Option<T>
where
    T: IntoBody,
{
    fn into_body(self) -> Option<MessageBody> {
        self.map(IntoBody::into_body).flatten()
    }
}

impl IntoBody for () {
    fn into_body(self) -> Option<MessageBody> {
        None
    }
}

impl<T, E> IntoBody for Result<T, E>
where
    T: IntoBody,
    E: IntoBody,
{
    fn into_body(self) -> Option<MessageBody> {
        match self {
            Ok(body) => body.into_body(),
            Err(err) => err.into_body(),
        }
    }
}

impl IntoBody for Error {
    fn into_body(self) -> Option<MessageBody> {
        Some(self.into())
    }
}

impl IntoBody for String {
    fn into_body(self) -> Option<MessageBody> {
        Some(MessageBody::new(self))
    }
}

impl IntoBody for &str {
    fn into_body(self) -> Option<MessageBody> {
        Some(MessageBody::new(self))
    }
}

impl<S, I, IStr, IVal> IntoBody for (S, I)
where
    S: Into<String>,
    I: IntoIterator<Item = (IStr, IVal)>,
    IStr: Into<String>,
    IVal: Serialize,
{
    fn into_body(self) -> Option<MessageBody> {
        let (ty, iter) = self;
        let mut body = MessageBody::new(ty);
        for (key, value) in iter {
            body.extra
                .insert(key.into(), serde_json::to_value(value).ok()?);
        }
        Some(body)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitMessage {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

fn u32_is_zero(v: &u32) -> bool {
    *v == 0
}
