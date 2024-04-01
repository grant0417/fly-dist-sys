use std::{
    collections::HashMap,
    future::Future,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};

pub use error::Error;
use proto::{IntoBody, Message};
use serde_json::Value;
use tokio::{
    io::{AsyncBufReadExt as _, AsyncWriteExt, BufReader, BufWriter},
    sync::{oneshot, MappedMutexGuard, Mutex, MutexGuard},
};

use crate::proto::{InitMessage, MessageBody};

pub mod error;
pub mod kv;
pub mod proto;

#[derive(Clone, Debug)]
pub struct NodeMetadata {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

// #[derive(Debug)]
pub struct NodeInner<S> {
    state: S,
    node_data: Mutex<Option<NodeMetadata>>,
    channel_map: Mutex<HashMap<u32, oneshot::Sender<Result<Message, Error>>>>,
    msg_ctr: AtomicU32,
}

#[derive(Clone)]
pub struct Node<S> {
    inner: Arc<NodeInner<S>>,
}

impl Node<()> {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(NodeInner {
                state: (),
                node_data: Mutex::new(None),
                channel_map: Mutex::new(HashMap::new()),
                msg_ctr: AtomicU32::new(0),
            }),
        }
    }
}

impl<S> Default for Node<S>
where
    S: Default,
{
    fn default() -> Self {
        Self::with_state(S::default())
    }
}

impl<S> Node<S> {
    pub fn with_state(state: S) -> Self {
        Self {
            inner: Arc::new(NodeInner {
                state,
                node_data: Mutex::new(None),
                channel_map: Mutex::new(HashMap::new()),
                msg_ctr: AtomicU32::new(0),
            }),
        }
    }

    pub async fn id(&self) -> MappedMutexGuard<'_, String> {
        MutexGuard::map(self.inner.node_data.lock().await, |node_data| {
            &mut node_data.as_mut().unwrap().node_id
        })
    }

    pub async fn node_ids<'a>(&'a self) -> MappedMutexGuard<'a, Vec<String>> {
        MutexGuard::map(self.inner.node_data.lock().await, |node_data| {
            &mut node_data.as_mut().unwrap().node_ids
        })
    }

    pub async fn node_metadata<'a>(&'a self) -> MappedMutexGuard<'a, NodeMetadata> {
        MutexGuard::map(self.inner.node_data.lock().await, |node_data| {
            node_data.as_mut().unwrap()
        })
    }

    pub fn state(&self) -> &S {
        &self.inner.state
    }

    /// Send a message to a destination node with no expectation of a reply.
    pub async fn send(&self, dst: String, body: MessageBody) {
        tracing::info!(%dst, ?body, "Sending message");
        let msg_id = self.inner.msg_ctr.fetch_add(1, Ordering::SeqCst);
        let msg = Message {
            src: self.id().await.clone(),
            dst,
            body: MessageBody { msg_id, ..body },
        };
        write_message(msg).await;
    }

    /// Send a message to a destination node and wait for a reply.
    pub async fn rpc(&self, dst: String, body: MessageBody) -> Result<Message, Error> {
        let msg_id = self.inner.msg_ctr.fetch_add(1, Ordering::SeqCst);

        let (tx, rx) = oneshot::channel();
        self.inner.channel_map.lock().await.insert(msg_id, tx);

        let msg = Message {
            src: self.id().await.clone(),
            dst,
            body: MessageBody { msg_id, ..body },
        };
        write_message(msg).await;

        let res = rx.await.unwrap()?;

        if res.ty() == "error" {
            Err(Error::from(res.body))
        } else {
            Ok(res)
        }
    }
}

impl<S> Node<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub async fn serve<'a, F, Fut, B>(&self, f: F)
    where
        F: Fn(Node<S>, Message) -> Fut + Copy + Send + Sync + 'static,
        Fut: Future<Output = B> + Send + 'static,
        B: IntoBody,
    {
        tracing_subscriber::fmt()
            .with_ansi(false)
            .with_writer(std::io::stderr)
            .init();

        let buf = BufReader::new(tokio::io::stdin());
        let mut lines = buf.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            let self_ = self.clone();

            tokio::spawn(async move {
                let req: Message = match serde_json::from_str(&line) {
                    Ok(req) => req,
                    Err(err) => {
                        tracing::error!(?err, "Failed to parse message");
                        write_message(Message {
                            src: self_.id().await.clone(),
                            dst: "error".to_string(),
                            body: Error::malformed_request().into(),
                        })
                        .await;
                        return;
                    }
                };

                let req_id = req.body.msg_id;
                tracing::info!(%req_id, ?req, "Received request");

                let dst = req.src.clone();

                if req.ty() == "init" {
                    let InitMessage { node_id, node_ids }: InitMessage =
                        serde_json::from_value(Value::Object(req.body.extra)).unwrap();

                    self_.inner.node_data.lock().await.replace(NodeMetadata {
                        node_id: node_id.clone(),
                        node_ids,
                    });

                    let msg = Message {
                        src: node_id,
                        dst,
                        body: MessageBody {
                            ty: "init_ok".to_string(),
                            in_reply_to: req_id,
                            ..Default::default()
                        },
                    };

                    write_message(msg).await;
                } else if req.body.in_reply_to != 0 {
                    if let Some(tx) = self_
                        .inner
                        .channel_map
                        .lock()
                        .await
                        .remove(&req.body.in_reply_to)
                    {
                        tx.send(Ok(req)).unwrap();
                    }
                } else {
                    let src = self_.id().await.clone();

                    let mut body = match f(self_, req).await.into_body() {
                        Some(body) => body,
                        None => return,
                    };

                    body.in_reply_to = req_id;

                    let msg = Message { src, dst, body };

                    tracing::info!(%req_id, ?msg, "Sending response");
                    write_message(msg).await;
                };
            });
        }
    }
}

async fn write_message(msg: Message) {
    let mut stdout = BufWriter::new(tokio::io::stdout());
    stdout
        .write_all(&serde_json::to_vec(&msg).unwrap())
        .await
        .unwrap();
    stdout.write_all(b"\n").await.unwrap();
    stdout.flush().await.unwrap();
}
