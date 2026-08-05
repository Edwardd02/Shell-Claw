mod config;
mod diagnostics;
mod ipc;
mod memory;
mod model;
mod scheduler;

use config::DaemonConfig;
use ipc::server::IpcServer;
use tracing::info;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = DaemonConfig::load();

    diagnostics::init(&config.log_path);

    info!("Smart Shell Copilot daemon starting");
    info!("Socket path: {:?}", config.socket_path);

    let server = IpcServer::new(config.socket_path.clone());

    tokio::spawn(async move {
        scheduler::global().run().await;
    });

    if let Err(e) = server.run().await {
        tracing::error!("IPC server error: {}", e);
    }

    diagnostics::flush();
    Ok(())
}
