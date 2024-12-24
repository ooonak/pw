use crate::platform::machine::Machine;
use common::{
    BASE_KEY_EXPR, BOOTID_KEY_EXPR, COMMAND_KEY_EXPR, LIVELINESS_KEY_EXPR, MACHINE_KEY_EXPR,
};
use log::{info, warn};
use zenoh::bytes::ZBytes;

pub struct ZenohCommunicator<'a, M> {
    session: zenoh::Session,
    machine: &'a M,
    key_expr_machine: String,
    key_expr_liveliness: String,
    key_expr_command: String,
    key_expr_metrics: String,
}

impl<'a, M> ZenohCommunicator<'a, M> where M: Machine {
    pub async fn new(config_file: &str, grp: &str, machine: &'a M) -> Self {
        zenoh::init_log_from_env_or("error");
        let config = zenoh::Config::from_file(config_file).unwrap();

        Self {
            session: zenoh::open(config).await.unwrap(),
            machine,
            key_expr_machine: format!("{}/{}/{}/{}", BASE_KEY_EXPR, grp, MACHINE_KEY_EXPR, machine.mac()),
            key_expr_liveliness: format!(
                "{}/{}/{}/{}",
                BASE_KEY_EXPR, grp, LIVELINESS_KEY_EXPR, machine.mac()
            ),
            key_expr_command: format!("{}/{}/{}/{}/*", BASE_KEY_EXPR, grp, COMMAND_KEY_EXPR, machine.mac()),
            key_expr_metrics: format!("{}/{}/{}/{}", BASE_KEY_EXPR, grp, BOOTID_KEY_EXPR, machine.mac()),
        }
    }

    pub async fn run(&mut self) {
        let payload = ZBytes::from(self.machine.serialize());
        self.session
            .put(&self.key_expr_machine, payload)
            .await
            .unwrap();

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
