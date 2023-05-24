use tokio::sync::Mutex;
use hearth_interconnect::messages::{JobRequest, Message};
use hearth_interconnect::worker_communication::Job;


use once_cell::sync::Lazy;

use nanoid::nanoid;
use rdkafka::producer::FutureProducer;
use crate::config::Config;
use crate::scheduler::connector::{send_message};
use anyhow::{bail, Result};
// Handles distribution across worker nodes via round robin or maybe another method?


pub static ROUND_ROBIN_INDEX: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));
pub static WORKERS: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(vec![]));

pub async fn distribute_job(job: JobRequest,producer: &mut FutureProducer,config: &Config) -> Result<()> {

    let mut index_guard = ROUND_ROBIN_INDEX.lock().await;
    let workers_guard = WORKERS.lock().await;

    if workers_guard.len() == 0 {
        bail!("No Workers Registered! Can't distribute Job!")
    }

    let job_id = nanoid!();
    let internal_message = &Message::InternalWorkerQueueJob(Job {
        job_id,
        worker_id: workers_guard[*index_guard].clone(),
        request_id: job.request_id,
        guild_id: job.guild_id
    });
    send_message(internal_message,config.kafka.kafka_topic.as_str(),producer).await;
    *index_guard += 1;
    if *index_guard == workers_guard.len() {
        *index_guard = 0;
    }
    Ok(())
}