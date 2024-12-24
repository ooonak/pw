use crate::platform::machine::Machine;
use common::{
    BASE_KEY_EXPR, COMMAND_KEY_EXPR, LIVELINESS_KEY_EXPR, MACHINE_KEY_EXPR, METRICS_KEY_EXPR,
    PROCESS_LISTING_KEY_EXPR,
};
use log::{info, warn};
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

            (self.metrics_enabled, self.process_listing_enabled) = parse_command(&self.key_expr_command, self.metrics_enabled, self.process_listing_enabled, &sample);

            if self.metrics_enabled {
            publisher_metrics
                .put("TODO test a metric...")
                .await
                .unwrap();
            }

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

fn parse_command(key_expr_command: &str, metrics_enabled: bool, process_listing_enabled: bool, sample: &zenoh::sample::Sample) -> (bool, bool) {
    let mut metrics = metrics_enabled;
    let mut processes = process_listing_enabled;
    
    if sample.key_expr().len() > key_expr_command.len() {
        let command: &str = &sample.key_expr()[key_expr_command.len() - 1..];
        match command {
            "metrics_on" => {
                if !metrics_enabled {
                    info!("Enabling metrics.");
                    metrics = true;
                }
            }
            "metrics_off" => {
                if metrics_enabled {
                    info!("Disabling metrics.");
                    metrics = false;
                }
            }
            "processes_on" => {
                if !process_listing_enabled {
                    info!("Enabling process listing.");
                    processes = true;
                }
            }
            "processes_off" => {
                if !process_listing_enabled {
                    info!("Disabling process listing.");
                    processes = false;
                }
            }
            _ => {
                info!("Unknown command '{}'.", command)
            }
        }
    }

    (metrics, processes)
}
