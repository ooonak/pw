use common::{pw, BASE_KEY_EXPR, MACHINE_KEY_EXPR};
mod platform;
use log::{error, info};
use zenoh::bytes::ZBytes;

pub const GROUP_KEY_EXPR: &str = "1";

async fn send_machine_info(session: &zenoh::Session, key: &str, machine: &pw::messages::Machine) {
    let payload = ZBytes::from(common::serialize_machine(machine));

    info!("Joining, telling about me on '{key}'");
    session.put(key, payload).await.unwrap();
}

#[tokio::main]
async fn main() {
    env_logger::init();
    zenoh::init_log_from_env_or("error");

    let config = zenoh::Config::default();
    let session = zenoh::open(config).await.unwrap();

    let machine = platform::machine::load();
    if machine.network_interface.is_none() {
        error!("Failed to collect information about default network interface, giving up.");

    }

    let key = format!(
        "{}/{}/{}/{}",
        BASE_KEY_EXPR,
        GROUP_KEY_EXPR,
        MACHINE_KEY_EXPR,
        machine.network_interface.as_ref().unwrap().mac
    );

    send_machine_info(&session, &key, &machine).await;

    //let key = format!("pw/command/{}", machine.network_interface.unwrap().mac);
    //println!("Declaring Subscriber on '{}'...", &key);

    let subscriber = session.declare_subscriber(&key).await.unwrap();

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
        println!();
    }
}
