use common::{deserialize_machine, BASE_KEY_EXPR, LIVELINESS_KEY_EXPR, MACHINE_KEY_EXPR};
use zenoh::{config::ZenohId, sample::SampleKind};

pub const GROUP_KEY_EXPR: &str = "grp1";

async fn zenoh_info(session: &zenoh::Session) {
    let info = session.info();
    println!("zid: {}", info.zid().await);
    println!(
        "routers zid: {:?}",
        info.routers_zid().await.collect::<Vec<ZenohId>>()
    );
    println!(
        "peers zid: {:?}",
        info.peers_zid().await.collect::<Vec<ZenohId>>()
    );
}

#[tokio::main]
async fn main() {
    zenoh::init_log_from_env_or("error");

    let config = zenoh::Config::default();

    let session = zenoh::open(config).await.unwrap();
    zenoh_info(&session).await;

    let key_expr_machine = format!(
        "{}/{}/{}/*",
        BASE_KEY_EXPR,
        GROUP_KEY_EXPR,
        MACHINE_KEY_EXPR,
    );

    let replies = session.get(key_expr_machine).await.unwrap();
    while let Ok(reply) = replies.recv_async().await {
        match reply.result() {
            Ok(sample) => {
                let payload = &*(sample.payload().to_bytes());
                let machine = deserialize_machine(payload);

                println!(
                    ">> Received ('{}': '{:?}')",
                    sample.key_expr().as_str(),
                    machine
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

    let key_expr_liveliness = format!(
        "{}/{}/{}/**",
        BASE_KEY_EXPR,
        GROUP_KEY_EXPR,
        LIVELINESS_KEY_EXPR
    );

    println!("Declaring Liveliness Subscriber on '{key_expr_liveliness}'...");
    
    let subscriber = session
    .liveliness()
    .declare_subscriber(&key_expr_liveliness)
    .history(true)
    .await
    .unwrap();

    println!("Press CTRL-C to quit...");
    while let Ok(sample) = subscriber.recv_async().await {
        match sample.kind() {
            SampleKind::Put => println!(
                ">> [LivelinessSubscriber] New alive token ('{}')",
                sample.key_expr().as_str()
            ),
            SampleKind::Delete => println!(
                ">> [LivelinessSubscriber] Dropped token ('{}')",
                sample.key_expr().as_str()
            ),
        }
    }

}
