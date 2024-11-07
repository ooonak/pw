
#[tokio::main]
async fn main() {
    zenoh::init_log_from_env_or("error");

    let config = zenoh::Config::default();
    
    let session = zenoh::open(config).await.unwrap();

    
}
