mod communicator;
mod platform;
use clap::Parser;
use communicator::ZenohCommunicator;
use log::info;
use platform::machine::{LinuxMachine, Machine};

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

#[derive(Parser, Default, Debug)]
struct Arguments {
    #[clap(default_value = "pw_config.json")]
    config_file: String,
    #[clap(default_value = "1")]
    group: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Arguments::parse();

    info!("Starting {}", version_info());

    let machine = LinuxMachine::new().expect("Failed to load system information");

    let mut communicator =
        ZenohCommunicator::new(&args.config_file, &args.group, machine.mac()).await;

    communicator.run(&machine).await;
}
