use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use fly_dist_sys::{Error, Node, NodeMetadata};

#[derive(Debug, Clone, Default)]
struct State {
    ctr: Arc<AtomicU32>,
}

#[tokio::main]
async fn main() {
    Node::<State>::default()
        .serve(|node, req| async move {
            if req.ty() == "generate" {
                let NodeMetadata { node_id, node_ids } = node.node_metadata().await.clone();

                let node_idx = node_ids
                    .iter()
                    .position(|n| n == &node_id)
                    .unwrap_or_default() as u32;
                let ctr = node.state().ctr.fetch_add(1, Ordering::SeqCst);

                let id = (node_idx.to_le() as u64) << 32 | ctr.to_le() as u64;

                Ok(("generate_ok", [("id", id)]))
            } else {
                Err(Error::not_supported())
            }
        })
        .await;
}
