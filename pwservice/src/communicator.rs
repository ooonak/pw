use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::platform::machine::Machine;
use common::{
    BASE_KEY_EXPR, COMMAND_KEY_EXPR, LIVELINESS_KEY_EXPR, MACHINE_KEY_EXPR, METRICS_KEY_EXPR,
    PROCESS_LISTING_KEY_EXPR,
};
use log::{debug, info, warn};
use zenoh::bytes::ZBytes;

pub struct ZenohCommunicator<'a, M> {
    session: zenoh::Session,
    machine: &'a M,
    metrics_enabled: bool,
    process_listing_enabled: bool,
    key_expr_machine: String,
    key_expr_liveliness: String,
    key_expr_command: String,
    key_expr_metrics: String,
    key_expr_processes: String,
}

impl<'a, M> ZenohCommunicator<'a, M>
where
    M: Machine,
{
    pub async fn new(config_file: &str, grp: &str, machine: &'a M) -> Self {
        zenoh::init_log_from_env_or("error");
        let config = zenoh::Config::from_file(config_file).unwrap();

        Self {
            session: zenoh::open(config).await.unwrap(),
            machine,

            metrics_enabled: false,
            process_listing_enabled: false,

            // pw/<grp>/<machine_info>/<id> : Sent from service on start.
            key_expr_machine: format!(
                "{}/{}/{}/{}",
                BASE_KEY_EXPR,
                grp,
                MACHINE_KEY_EXPR,
                machine.mac()
            ),

            // pw/<grp>/<liveliness>/<id> : Sent from service on start.
            key_expr_liveliness: format!(
                "{}/{}/{}/{}",
                BASE_KEY_EXPR,
                grp,
                LIVELINESS_KEY_EXPR,
                machine.mac()
            ),

            // pw/<grp>/<id>/<boot_id>/<command> : Sent from client to service.
            key_expr_command: format!(
                "{}/{}/{}/{}/{}/*",
                BASE_KEY_EXPR,
                grp,
                machine.mac(),
                machine.bootid(),
                COMMAND_KEY_EXPR
            ),

            // pw/<grp>/<id>/<boot_id>/<metrics> : Sent from service to client.
            key_expr_metrics: format!(
                "{}/{}/{}/{}/{}",
                BASE_KEY_EXPR,
                grp,
                machine.mac(),
                machine.bootid(),
                METRICS_KEY_EXPR
            ),

            // pw/<grp>/<id>/<boot_id>/<processes> : Sent from service to client.
            key_expr_processes: format!(
                "{}/{}/{}/{}/{}",
                BASE_KEY_EXPR,
                grp,
                machine.mac(),
                machine.bootid(),
                PROCESS_LISTING_KEY_EXPR
            ),
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

        let subscriber_commands = self
            .session
            .declare_subscriber(&self.key_expr_command)
            .await
            .unwrap();

        let publisher_metrics = self
            .session
            .declare_publisher(&self.key_expr_metrics)
            .await
            .unwrap();

        let publisher_processes = self
            .session
            .declare_publisher(&self.key_expr_processes)
            .await
            .unwrap();

        while let Ok(sample) = subscriber_commands.recv_async().await {
            // Refer to z_bytes.rs to see how to deserialize different types of message
            let payload = sample
                .payload()
                .try_to_string()
                .unwrap_or_else(|e| e.to_string().into());

            /*
            info!(
                "[Subscriber] Received command from client: {} ('{}': '{}')",
                sample.kind(),
                sample.key_expr().as_str(),
                payload
            );
            */

            if sample.key_expr().len() > self.key_expr_command.len() {
                let command: &str = &sample.key_expr()[self.key_expr_command.len() - 1..];
                match command {
                    "metrics_on" => {
                        info!("Enabling metrics.");
                        self.metrics_enabled = true;
                    }
                    "metrics_off" => {
                        info!("Disabling metrics.");
                        self.metrics_enabled = false;
                    }
                    "processes_on" => {
                        info!("Enabling process listing.");
                        self.process_listing_enabled = true;
                    }
                    "processes_off" => {
                        info!("Disabling process listing.");
                        self.process_listing_enabled = false;
                    }
                    _ => {
                        info!("Unknown command '{}'.", command)
                    }
                }
            }

            publisher_metrics
                .put("TODO test a metric...")
                .await
                .unwrap();
            publisher_processes
                .put("TODO test a process list...")
                .await
                .unwrap();

            if let Some(att) = sample.attachment() {
                let att = att.try_to_string().unwrap_or_else(|e| e.to_string().into());
                warn!("{}", att);
            }
        }

        liveliness.undeclare().await.unwrap();
    }
}
