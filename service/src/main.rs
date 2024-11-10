use common::{pw, MACHINE_KEY_EXPR};
mod platform;
use zenoh::bytes::ZBytes;

async fn send_machine_info(session: &zenoh::Session, machine: &pw::messages::Machine) {
    let payload = ZBytes::from(common::serialize_machine(machine));

    let key = format!("{}/{}", MACHINE_KEY_EXPR, machine.mac);

    println!("Putting Data ('{key}': {} bytes)...", payload.len());

    session.put(&key, payload).await.unwrap();
}

#[tokio::main]
async fn main() {
    zenoh::init_log_from_env_or("error");

    let config = zenoh::Config::default();

    println!("Opening session...");
    let session = zenoh::open(config).await.unwrap();

    let machine = platform::machine::load();

    send_machine_info(&session, &machine).await;

    let key = format!("pw/command/{}", machine.mac);
    println!("Declaring Subscriber on '{}'...", &key);

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
