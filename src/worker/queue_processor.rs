// Processing streaming jobs from kafka queue

// Parallelize with tokio

use log::info;
use crate::config::Config;
use crate::utils::generic_connector::Message;
use crate::worker::bot_handler::initialize_bot_instance;

pub async fn process_job(message: Message,config: &Config) {
    // Expect and Unwrap are fine here because if we panic it will only panic the thread so it should be fine in most cases
    let queue_job = message.queue_job_internal.expect("Internal job empty!");
    initialize_bot_instance(queue_job.voice_channel_id,queue_job.guild_id,config).await;
    info!("Job Processed!");
}