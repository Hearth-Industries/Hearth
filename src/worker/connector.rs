

use std::thread;






use kafka::producer::Producer;
use log::{error, info};


use tokio::runtime;
use tokio::runtime::Builder;


use crate::config::Config;
use crate::worker::queue_processor::{ErrorReport, Infrastructure};
use crate::utils::generic_connector::{ExternalQueueJobResponse, Message, MessageType, send_message_generic};
// Internal connector
use crate::utils::initialize_consume_generic;
use crate::worker::queue_processor::{process_job, ProcessorIncomingAction, ProcessorIPC, ProcessorIPCData};

pub fn initialize_api(config: &Config, ipc: &mut ProcessorIPC) {
    let broker = config.config.kafka_uri.to_owned();
    initialize_worker_consume(vec![broker],config,ipc);
}

fn report_error(error: ErrorReport) {
    error!("{}",error.error);
    //TODO: Report to client
    // send_message(&Message {
    //     message_type: MessageType::ErrorReport,
    //     analytics: None,
    //     queue_job_request: None,
    //     queue_job_internal: None,
    //     request_id: error.request_id.clone(),
    //     worker_id: None,
    //     direct_worker_communication: None,
    //     external_queue_job_response: None,
    //     job_event: None,
    //     error_report: Some(error),
    // },"communication",producer);
}

fn parse_message_callback(parsed_message: Message, mut producer: &mut Producer, config: &Config, ipc: &mut ProcessorIPC) {
    //TODO: Check if this message is for us
    //TODO: But worker ping pong/interface stuff first
    match parsed_message.message_type {
        // Parseable
        MessageType::DirectWorkerCommunication => {
            let dwc = parsed_message.direct_worker_communication.unwrap();
            let job_id = dwc.job_id.clone();
            let result = ipc.sender.send(ProcessorIPCData {
                action_type: ProcessorIncomingAction::Actions(dwc.action_type.clone()),
                songbird: None,
                job_id: job_id.clone(),
                dwc: Some(dwc),
                error_report: None
            });
            match result {
                Ok(_) => {},
                Err(e) => error!("Failed to send DWC to job: {}",&job_id),
            }
        },
        MessageType::InternalWorkerQueueJob => {
            let proc_config = config.clone();
            info!("{:?}",parsed_message);
            // This is a bit of a hack try and replace with tokio. Issue: Tokio task not executing when spawned inside another tokio task
            // rt.block_on(process_job(parsed_message, &proc_config, ipc.sender));
            // let sender = ipc.sender;
            let sender = ipc.sender.clone();
            let job_id = parsed_message.queue_job_internal.clone().unwrap().job_id;
            let request_id = parsed_message.request_id.clone();
            thread::spawn(move || {
                let rt = Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(process_job(parsed_message, &proc_config, &mut producer, sender,report_error));
            });
            send_message(&Message {
                message_type: MessageType::ExternalQueueJobResponse,
                analytics: None,
                queue_job_request: None,
                queue_job_internal: None,
                request_id: request_id,
                worker_id: None,
                direct_worker_communication: None,
                external_queue_job_response: Some(ExternalQueueJobResponse {
                    job_id: Some(job_id)
                }),
                job_event: None,
                error_report: None,
            }, "communication", producer);
        }
        _ => {}
    }
}


pub fn initialize_worker_consume(brokers: Vec<String>, config: &Config, ipc: &mut ProcessorIPC) {
    initialize_consume_generic(brokers, config, parse_message_callback, "WORKER", ipc);
}

pub fn send_message(message: &Message, topic: &str, producer: &mut Producer) {
    send_message_generic(message,topic,producer);
}