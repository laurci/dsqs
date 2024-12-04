use tonic::{Request, Response, Status};

use dsqs::queue_server::Queue;
use dsqs::{DequeueReply, DequeueRequest, EnqueueRequest};

use crate::queue::{MessageQueue, QueueBehavior};

pub mod dsqs {
    tonic::include_proto!("dsqs");

    pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("dsqs_descriptor");
}

pub struct QueueService {
    queue: MessageQueue<Vec<u8>>,
}

impl QueueService {
    pub fn new(queue_behavior: QueueBehavior) -> Self {
        QueueService {
            queue: MessageQueue::new(queue_behavior),
        }
    }
}

#[tonic::async_trait]
impl Queue for QueueService {
    async fn enqueue(&self, request: Request<EnqueueRequest>) -> Result<Response<()>, Status> {
        let data = request.into_inner();
        self.queue.send(data.message);
        Ok(Response::new(()))
    }

    async fn dequeue(
        &self,
        request: Request<DequeueRequest>,
    ) -> Result<Response<DequeueReply>, Status> {
        let data = request.into_inner();

        let receive_message = async {
            loop {
                let sub = self.queue.subscribe();
                let msg = sub.receive().await;
                if let Some(msg) = msg {
                    return DequeueReply { message: msg };
                }
            }
        };

        if let Some(timeout) = data.max_wait_delay_ms {
            match tokio::select! {
                msg = receive_message => Some(msg),
                _ = async {
                    tokio::time::sleep(tokio::time::Duration::from_millis(timeout)).await;
                } => None,
            } {
                Some(msg) => Ok(Response::new(msg)),
                None => Err(Status::deadline_exceeded("Timeout")),
            }
        } else {
            Ok(Response::new(receive_message.await))
        }
    }
}
