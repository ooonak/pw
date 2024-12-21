use crate::platform::machine::Machine;
use common::{
    BASE_KEY_EXPR, BOOTID_KEY_EXPR, COMMAND_KEY_EXPR, LIVELINESS_KEY_EXPR, MACHINE_KEY_EXPR,
};
use log::{info, warn};
use zenoh::bytes::ZBytes;

pub struct ZenohCommunicator {
    session: zenoh::Session,
    key_expr_machine: String,
    key_expr_liveliness: String,
    key_expr_command: String,
    key_expr_metrics: String,
}

impl ZenohCommunicator {
    pub async fn new(config_file: &str, grp: &str, id: u64) -> Self {
        zenoh::init_log_from_env_or("error");
        let config = zenoh::Config::from_file(config_file).unwrap();

        Self {
            session: zenoh::open(config).await.unwrap(),
            key_expr_machine: format!("{}/{}/{}/{}", BASE_KEY_EXPR, grp, MACHINE_KEY_EXPR, id),
            key_expr_liveliness: format!(
                "{}/{}/{}/{}",
                BASE_KEY_EXPR, grp, LIVELINESS_KEY_EXPR, id
            ),
            key_expr_command: format!("{}/{}/{}/{}/*", BASE_KEY_EXPR, grp, COMMAND_KEY_EXPR, id),
            key_expr_metrics: format!("{}/{}/{}/{}", BASE_KEY_EXPR, grp, BOOTID_KEY_EXPR, id),
        }
    }

    pub async fn init<M: Machine>(&mut self, machine: &M) {
        /*
        let payload = ZBytes::from(common::serialize_machine(machine));
        self.session
            .put(&self.key_expr_machine, payload)
            .await
            .unwrap();
        */
        todo!()
    }

    pub async fn run(&mut self) {
        let liveliness = self
            .session
            .liveliness()
            .declare_token(&self.key_expr_liveliness)
            .await
            .unwrap();

        let subscriber = self
            .session
            .declare_subscriber(&self.key_expr_command)
            .await
            .unwrap();
        let publisher = self
            .session
            .declare_publisher(&self.key_expr_metrics)
            .await
            .unwrap();

        while let Ok(sample) = subscriber.recv_async().await {
            // Refer to z_bytes.rs to see how to deserialize different types of message
            let payload = sample
                .payload()
                .try_to_string()
                .unwrap_or_else(|e| e.to_string().into());

            info!(
                "[Subscriber] Received command from client: {} ('{}': '{}')",
                sample.kind(),
                sample.key_expr().as_str(),
                payload
            );

            publisher.put("TODO test...").await.unwrap();

            if let Some(att) = sample.attachment() {
                let att = att.try_to_string().unwrap_or_else(|e| e.to_string().into());
                warn!("{}", att);
            }
        }

        liveliness.undeclare().await.unwrap();
    }
}
