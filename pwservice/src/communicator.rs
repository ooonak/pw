use crate::platform::{machine::Machine, metrics::Metrics, processes::Processes};
use common::{
    BASE_KEY_EXPR, COMMAND_KEY_EXPR, LIVELINESS_KEY_EXPR, MACHINE_KEY_EXPR, METRICS_KEY_EXPR,
    PROCESS_LISTING_KEY_EXPR,
};
use log::{debug, info};
use tokio::select;
use zenoh::bytes::ZBytes;

static AUTO_STOP_PUBLISH_COUNT: u32 = 900;

pub struct ZenohCommunicator<'a, M> {
    session: zenoh::Session,
    machine: &'a M,
    metrics_enabled_counter: u32,
    process_listing_enabled_counter: u32,
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

            metrics_enabled_counter: 0,
            process_listing_enabled_counter: 0,

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

        let _liveliness = self
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

        loop {
            select! {
                sample = subscriber_commands.recv_async() => {
                    if sample.is_ok() {
                        (self.metrics_enabled_counter, self.process_listing_enabled_counter) = parse_command(
                            &self.key_expr_command,
                            self.metrics_enabled_counter,
                            self.process_listing_enabled_counter,
                            &sample.unwrap(),
                        );
                    }
                }

                _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {
                    if self.metrics_enabled_counter > 0 {
                        self.metrics_enabled_counter -= 1;

                        if let Ok(metrics) = Metrics::new() {
                            let payload = ZBytes::from(metrics.serialize());

                            publisher_metrics
                                .put(payload)
                                .await
                                .unwrap();
                        }
                    }

                    if self.process_listing_enabled_counter > 0 {
                        self.process_listing_enabled_counter -= 1;

                        if let Ok(process_list) = Processes::new() {
                            let payload = ZBytes::from(process_list.serialize());

                            publisher_processes
                                .put(payload)
                                .await
                                .unwrap();
                        }
                    }
                }
            }
        }

        // _liveliness.undeclare().await.unwrap();
    }
}

fn parse_command(
    key_expr_command: &str,
    metrics_enabled: u32,
    process_listing_enabled: u32,
    sample: &zenoh::sample::Sample,
) -> (u32, u32) {
    let mut metrics = metrics_enabled;
    let mut processes = process_listing_enabled;

    if sample.key_expr().len() > key_expr_command.len() {
        let command: &str = &sample.key_expr()[key_expr_command.len() - 1..];
        match command {
            "metrics_on" => {
                if metrics_enabled == 0 {
                    debug!("Enabling metrics.");
                } else {
                    debug!("Re-enabling metrics.");
                }
                metrics = AUTO_STOP_PUBLISH_COUNT;
            }
            "metrics_off" => {
                if metrics_enabled > 0 {
                    debug!("Disabling metrics.");
                    metrics = 0;
                }
            }
            "processes_on" => {
                if process_listing_enabled == 0 {
                    debug!("Enabling process listing.");
                } else {
                    debug!("Re-enabling process listing.");
                }
                processes = AUTO_STOP_PUBLISH_COUNT;
            }
            "processes_off" => {
                if !process_listing_enabled > 0 {
                    debug!("Disabling process listing.");
                    processes = 0;
                }
            }
            _ => {
                info!("Unknown command '{}'.", command)
            }
        }
    }

    (metrics, processes)
}
