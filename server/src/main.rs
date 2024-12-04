mod config;
mod queue;
mod server;

use anyhow::Result;
use config::Config;
use server::{
    dsqs::{self, queue_server::QueueServer},
    QueueService,
};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::load_from_env()?;

    let serve_address = format!("{}:{}", config.server.host, config.server.port).parse()?;
    tracing::info!("Starting server on {}", serve_address);

    let queue_service = QueueService::new(config.queue_behavior);

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(dsqs::FILE_DESCRIPTOR_SET)
        .build_v1alpha()?;

    Server::builder()
        .add_service(reflection_service)
        .add_service(QueueServer::new(queue_service))
        .serve(serve_address)
        .await?;

    Ok(())
}
