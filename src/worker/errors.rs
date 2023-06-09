use hearth_interconnect::errors::ErrorReport;
use hearth_interconnect::messages::Message;
use log::error;

use crate::config::Config;
use crate::worker::connector::{send_message, WORKER_PRODUCER};

pub fn report_error(error: ErrorReport, config: &Config) {
    error!("{}", error.error);

    let t_config = config.clone();

    tokio::task::spawn(async move {
        let mut px = WORKER_PRODUCER.get().unwrap().lock().await;
        let p = px.as_mut();

        send_message(
            &Message::ErrorReport(error),
            t_config.kafka.kafka_topic.as_str(),
            p.unwrap(),
        )
        .await;
    });
}
