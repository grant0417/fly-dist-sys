use fly_dist_sys::{kv::Kv, proto::MessageBody, Error, Node};

#[tokio::main]
async fn main() {
    Node::new()
        .serve(|node, req| async move {
            let kv = Kv::new_seq_kv(&node);

            match req.ty() {
                "add" => {
                    let delta = req.body.extra.get("delta").unwrap().as_u64().unwrap() as u64;

                    Ok(MessageBody::new("add_ok"))
                }
                "read" => Ok(MessageBody::new("read_ok").with_field("value", 0)),
                _ => Err(Error::not_supported()),
            }
        })
        .await;
}
