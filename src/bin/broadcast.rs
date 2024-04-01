use std::{collections::HashSet, sync::Arc};

use fly_dist_sys::{
    proto::{IntoBody, MessageBody},
    Error, Node, NodeMetadata,
};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Default)]
struct State {
    messages: Arc<Mutex<HashSet<i64>>>,
}

#[tokio::main]
async fn main() {
    Node::<State>::default()
        .serve(|node, req| async move {
            match req.ty() {
                "broadcast" => {
                    let i = req.body.extra.get("message").unwrap().as_i64().unwrap();
                    node.state().messages.lock().await.insert(i);

                    // only broadcast if received from a client
                    if req.src.starts_with('c') {
                        let NodeMetadata { node_id, node_ids } = node.node_metadata().await.clone();
                        for n in &*node_ids {
                            if n != &node_id {
                                node.send(
                                    n.into(),
                                    MessageBody::new("set").with_field(
                                        "set",
                                        node.state().messages.lock().await.clone(),
                                    ),
                                )
                                .await;
                            }
                        }
                    }

                    Ok("broadcast_ok".into_body())
                }
                "set" => {
                    let i = req.body.extra.get("set").unwrap().as_array().unwrap();
                    let mut state = node.state().messages.lock().await;
                    for j in i {
                        state.insert(j.as_i64().unwrap());
                    }
                    Ok(None)
                }
                "read" => {
                    let messages = node.state().messages.lock().await.clone();
                    Ok(("read_ok", [("messages", messages)]).into_body())
                }
                "topology" => Ok("topology_ok".into_body()),
                _ => Err(Error::not_supported()),
            }
        })
        .await;
}
