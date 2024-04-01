use fly_dist_sys::{proto::MessageBody, Error, Node};

#[tokio::main]
async fn main() {
    Node::new()
        .serve(|_, req| async move {
            if req.ty() == "echo" {
                Ok(MessageBody {
                    ty: "echo_ok".to_string(),
                    ..req.body
                })
            } else {
                Err(Error::not_supported())
            }
        })
        .await;
}
