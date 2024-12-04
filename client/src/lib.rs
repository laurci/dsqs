use proto::queue_client::QueueClient;
use thiserror::Error;
use tonic::transport::Error;

pub mod proto {
    tonic::include_proto!("dsqs");
}

#[derive(Error, Debug)]
pub enum DsqsError {
    #[error("enqueue error")]
    EnqueueError(tonic::Status),
    #[error("dequeue error")]
    DequeueError(tonic::Status),
    #[error("transport error")]
    Transport(#[from] Error),
}

pub struct Client {
    client: QueueClient<tonic::transport::Channel>,
}

pub struct ClientConfig {
    pub host: String,
    pub port: u16,
}

impl Client {
    pub async fn new(config: ClientConfig) -> Result<Self, DsqsError> {
        let addr = format!("http://{}:{}", config.host, config.port);
        let client = QueueClient::connect(addr).await?;

        Ok(Client { client })
    }

    pub async fn enqueue(&self, message: Vec<u8>) -> Result<(), DsqsError> {
        self.client
            .clone()
            .enqueue(proto::EnqueueRequest { message })
            .await
            .map_err(|e| DsqsError::EnqueueError(e))?;

        Ok(())
    }

    pub async fn dequeue(&self, max_wait_delay_ms: Option<u64>) -> Result<Vec<u8>, DsqsError> {
        let response = self
            .client
            .clone()
            .dequeue(proto::DequeueRequest { max_wait_delay_ms })
            .await
            .map_err(|e| DsqsError::DequeueError(e))?;

        Ok(response.into_inner().message)
    }
}
