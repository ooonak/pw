use clap::Parser;
use common::{
    deserialize_machine, stringify_duration, stringify_message, BASE_KEY_EXPR, GROUP_KEY_EXPR, LIVELINESS_KEY_EXPR, MACHINE_KEY_EXPR
};
use log::{debug, info, warn};
use zenoh::sample::SampleKind;

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

#[derive(Parser,Default,Debug)]
struct Arguments {
    #[clap(default_value = "pw_config.json")]
    config_file: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Arguments::parse();

    info!("Starting {}", version_info());

    zenoh::init_log_from_env_or("error");
    let config = zenoh::Config::from_file(args.config_file).unwrap();
    let session = zenoh::open(config).await.unwrap();
    
    let key_expr_machine = format!(
        "{}/{}/{}/**",
        BASE_KEY_EXPR, GROUP_KEY_EXPR, MACHINE_KEY_EXPR,
    );

    debug!("Declaring Machine getter on '{key_expr_machine}'...");

    let machine_getter = session.get(key_expr_machine).await.unwrap();
    while let Ok(reply) = machine_getter.recv_async().await {
        match reply.result() {
            Ok(sample) => {
                let payload = &*(sample.payload().to_bytes());
                match deserialize_machine(payload) {
                    Ok(machine) => {
                        //debug!("{:?}", sample);
                        info!(
                            "Received [from {}, to {}, when {}] : '{:?}')",
                            "<zid>",
                            sample.key_expr().as_str(),
                            stringify_duration(sample.timestamp().unwrap().get_time().as_secs().into()),
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
                warn!(">> Received (ERROR: '{}')", payload);
            }
        }
    }

    let key_expr_liveliness = format!(
        "{}/{}/{}/**",
        BASE_KEY_EXPR, GROUP_KEY_EXPR, LIVELINESS_KEY_EXPR
    );

    debug!("Declaring Liveliness Subscriber on '{key_expr_liveliness}'...");

    let liveliness_subscriber = session
        .liveliness()
        .declare_subscriber(&key_expr_liveliness)
        .history(true)
        .await
        .unwrap();

    info!("Press CTRL-C to quit...");
    while let Ok(sample) = liveliness_subscriber.recv_async().await {
        match sample.kind() {
            SampleKind::Put => info!("machine online ('{}')", sample.key_expr().as_str()),
            SampleKind::Delete => info!("machine offline ('{}')", sample.key_expr().as_str()),
        }
    }
}
