mod data_types;

use data_types::pw;
use zenoh::{bytes::ZBytes, key_expr::{self, KeyExpr}, Config};

async fn send_machine_info(session: &zenoh::Session, machine: &pw::messages::Machine) {
    let payload = ZBytes::from(data_types::machine::serialize_machine(&machine));

    let key = format!("pw/machine/{}", machine.mac);

    println!("Putting Data ('{key}': {} bytes)...", payload.len());
    
    session.put(&key, payload).await.unwrap();
}

#[tokio::main]
async fn main() {
    // initiate logging
    zenoh::init_log_from_env_or("error");

    let config = zenoh::Config::default();
    //config.connect.endpoints.set(["tcp/127.0.0.1:7447"].iter().map(|s|s.parse().unwrap()).collect());
    
    println!("Opening session...");
    let session = zenoh::open(config).await.unwrap();

    let machine = data_types::machine::load();
    send_machine_info(&session, &machine).await;

    //println!("Serialized, buffer contains: {:?}", buffer);
    /*
    let deserialized = data_types::machine::deserialize_machine(&buffer);
    println!("Deserialized: {:?}", deserialized);
    */
}
