use serde::Serialize;
use serde_json::Value;

use crate::{proto::MessageBody, Error, Node};

/// Linearizable key-value store.
const LIN_KV: &str = "lin-kv";
/// Sequentially consistent key-value store.
const SEQ_KV: &str = "seq-kv";
/// Last-writer-wins key-value store.
const LWW_KV: &str = "lww-kv";

/// Key-value store
pub struct Kv<'a, S> {
    node: &'a Node<S>,
    ty: &'static str,
}

impl<'a, S> Kv<'a, S> {
    const fn new(node: &'a Node<S>, ty: &'static str) -> Self {
        Self { node, ty }
    }

    /// Create a new linearizable key-value store
    pub const fn new_lin_kv(node: &'a Node<S>) -> Self {
        Self::new(node, LIN_KV)
    }

    /// Create a new sequentially consistent key-value store
    pub const fn new_seq_kv(node: &'a Node<S>) -> Self {
        Self::new(node, SEQ_KV)
    }

    /// Create a new last-writer-wins key-value store
    pub const fn new_lww_kv(node: &'a Node<S>) -> Self {
        Self::new(node, LWW_KV)
    }

    /// Read a value from the key-value store
    pub async fn read(&self, key: &str) -> Result<Option<Value>, Error> {
        let message = self
            .node
            .rpc(
                self.ty.into(),
                MessageBody::new("read").with_field("key", key),
            )
            .await;

        match message {
            Ok(mut message) => Ok(Some(message.body.extra.remove("value").unwrap())),
            Err(err) if err.is_key_does_not_exist() => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// Write a value to the key-value store
    pub async fn write(&self, key: &str, value: impl Serialize) -> Result<(), Error> {
        self.node
            .rpc(
                self.ty.into(),
                MessageBody::new("write")
                    .with_field("key", key)
                    .with_field("value", value),
            )
            .await?;

        Ok(())
    }

    /// Compare and swap a value in the key-value store
    pub async fn compare_and_swap(
        &self,
        key: &str,
        from: &Value,
        to: &Value,
        create_if_not_exists: bool,
    ) -> Result<(), Error> {
        self.node
            .rpc(
                self.ty.into(),
                MessageBody::new("compare_and_swap")
                    .with_field("key", key)
                    .with_field("from", from)
                    .with_field("to", to)
                    .with_field("create_if_not_exists", create_if_not_exists),
            )
            .await?;

        Ok(())
    }
}
