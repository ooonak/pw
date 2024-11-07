use common::MACHINE_KEY_EXPR;

#[tokio::main]
async fn main() {
    zenoh::init_log_from_env_or("error");

    let config = zenoh::Config::default();
    
    let session = zenoh::open(config).await.unwrap();

    // TODO: Subscribe to liveliness.
    
    let key = format!("{}/**", MACHINE_KEY_EXPR);

    /*
    let replies = session
    .get(key)
    .payload(payload.unwrap_or_default())
    .target(target)
    .timeout(timeout)
    .await
    .unwrap();

    while let Ok(reply) = replies.recv_async().await {
        match reply.result() {
            Ok(sample) => {
                // Refer to z_bytes.rs to see how to deserialize different types of message
                let payload = sample
                    .payload()
                    .try_to_string()
                    .unwrap_or_else(|e| e.to_string().into());
                println!(
                    ">> Received ('{}': '{}')",
                    sample.key_expr().as_str(),
                    payload,
                );
            }
            Err(err) => {
                let payload = err
                    .payload()
                    .try_to_string()
                    .unwrap_or_else(|e| e.to_string().into());
                println!(">> Received (ERROR: '{}')", payload);
            }
        }
    }
    */
}
