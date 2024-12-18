use clap::Parser;
use common::{
    pw, BASE_KEY_EXPR, COMMAND_KEY_EXPR, GROUP_KEY_EXPR, LIVELINESS_KEY_EXPR, MACHINE_KEY_EXPR,
};
mod platform;
use log::{error, info};
use zenoh::bytes::ZBytes;

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

async fn send_machine_info(session: &zenoh::Session, key: &str, machine: &pw::messages::Machine) {
    let payload = ZBytes::from(common::serialize_machine(machine));

    info!("Joining, telling about me on '{key}'");
    session.put(key, payload).await.unwrap();
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

    let machine = platform::machine::load();
    if machine.network_interface.is_none() || machine.network_interface.as_ref().unwrap().mac == 0 {
        error!("Failed to collect information about default network interface, giving up.");
        std::process::abort();
    }

    zenoh::init_log_from_env_or("error");
    let config = zenoh::Config::from_file(args.config_file).unwrap();
    let session = zenoh::open(config).await.unwrap();

    let key_expr_machine = format!(
        "{}/{}/{}/{}",
        BASE_KEY_EXPR,
        GROUP_KEY_EXPR,
        MACHINE_KEY_EXPR,
        machine.network_interface.as_ref().unwrap().mac
    );
    send_machine_info(&session, &key_expr_machine, &machine).await;

    let key_expr_liveliness = format!(
        "{}/{}/{}/{}",
        BASE_KEY_EXPR,
        GROUP_KEY_EXPR,
        LIVELINESS_KEY_EXPR,
        machine.network_interface.as_ref().unwrap().mac
    );

    let token = session
        .liveliness()
        .declare_token(&key_expr_liveliness)
        .await
        .unwrap();

    let key_expr_command = format!(
        "{}/{}/{}/{}",
        BASE_KEY_EXPR,
        GROUP_KEY_EXPR,
        COMMAND_KEY_EXPR,
        machine.network_interface.as_ref().unwrap().mac
    );

    let subscriber = session.declare_subscriber(&key_expr_command).await.unwrap();

    println!("Press CTRL-C to quit...");

    while let Ok(sample) = subscriber.recv_async().await {
        // Refer to z_bytes.rs to see how to deserialize different types of message
        let payload = sample
            .payload()
            .try_to_string()
            .unwrap_or_else(|e| e.to_string().into());

        print!(
            ">> [Subscriber] Received {} ('{}': '{}')",
            sample.kind(),
            sample.key_expr().as_str(),
            payload
        );

        if let Some(att) = sample.attachment() {
            let att = att.try_to_string().unwrap_or_else(|e| e.to_string().into());
            print!(" ({})", att);
        }
    }

    token.undeclare().await.unwrap();
}
