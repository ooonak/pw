use common::{
    deserialize_machine, stringify_message, BASE_KEY_EXPR, GROUP_KEY_EXPR, LIVELINESS_KEY_EXPR,
    MACHINE_KEY_EXPR,
};
use log::{info, warn};
use zenoh::{config::ZenohId, sample::SampleKind};

fn version_info() -> String {
    let mut build_type = "release";
    if cfg!(debug_assertions) {
        build_type = "debug";
    }

    format!(
        "{} v{} {} (Git SHA: {}, dirty: {}, build time: {})",
        option_env!("CARGO_PKG_NAME").unwrap_or_default(),
        option_env!("CARGO_PKG_VERSION").unwrap_or_default(),
        build_type,
        option_env!("VERGEN_GIT_SHA").unwrap_or_default(),
        option_env!("VERGEN_GIT_DIRTY").unwrap_or_default(),
        option_env!("VERGEN_BUILD_TIMESTAMP").unwrap_or_default()
    )
}

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
    env_logger::init();
    info!("Starting {}", version_info());

    zenoh::init_log_from_env_or("error");

    let config = zenoh::Config::default();

    let session = zenoh::open(config).await.unwrap();
    zenoh_info(&session).await;

    let key_expr_machine = format!(
        "{}/{}/{}/**",
        BASE_KEY_EXPR, GROUP_KEY_EXPR, MACHINE_KEY_EXPR,
    );

    println!("Declaring Machine getter on '{key_expr_machine}'...");

    let machine_getter = session.get(key_expr_machine).await.unwrap();
    while let Ok(reply) = machine_getter.recv_async().await {
        match reply.result() {
            Ok(sample) => {
                let payload = &*(sample.payload().to_bytes());
                match deserialize_machine(payload) {
                    Ok(machine) => {
                        info!(
                            "Received ('{}': '{:?}')",
                            sample.key_expr().as_str(),
                            stringify_message(&machine)
                        );
                    }
                    Err(err) => {
                        warn!("Could not parse message (ERROR: '{}')", err);
                    }
                }
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
        BASE_KEY_EXPR, GROUP_KEY_EXPR, LIVELINESS_KEY_EXPR
    );

    println!("Declaring Liveliness Subscriber on '{key_expr_liveliness}'...");

    let liveliness_subscriber = session
        .liveliness()
        .declare_subscriber(&key_expr_liveliness)
        .history(true)
        .await
        .unwrap();

    println!("Press CTRL-C to quit...");
    while let Ok(sample) = liveliness_subscriber.recv_async().await {
        match sample.kind() {
            SampleKind::Put => println!("machine online ('{}')", sample.key_expr().as_str()),
            SampleKind::Delete => println!("machine offline ('{}')", sample.key_expr().as_str()),
        }
    }
}
