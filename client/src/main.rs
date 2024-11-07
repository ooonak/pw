use common::{deserialize_machine, MACHINE_KEY_EXPR};

#[tokio::main]
async fn main() {
    zenoh::init_log_from_env_or("error");

    let config = zenoh::Config::default();
    
    let session = zenoh::open(config).await.unwrap();
    
    let key = format!("{}/**", MACHINE_KEY_EXPR);

    let replies = session
    .get(key)
    .await
    .unwrap();

    while let Ok(reply) = replies.recv_async().await {
        match reply.result() {
            Ok(sample) => {
                let payload = &*(sample
                    .payload()
                    .to_bytes());
                let machine = deserialize_machine(payload);
                

                println!(">> Received ('{}': '{:?}')", sample.key_expr().as_str(), machine);
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
}
