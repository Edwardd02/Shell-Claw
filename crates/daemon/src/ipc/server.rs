use std::path::PathBuf;
use tokio::net::{UnixListener, UnixStream};
use tracing::{error, info};

pub struct IpcServer {
    socket_path: PathBuf,
}

impl IpcServer {
    pub fn new(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    pub async fn run(self) -> std::io::Result<()> {
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)?;
        }

        let listener = UnixListener::bind(&self.socket_path)?;
        info!("IPC server listening on {:?}", self.socket_path);

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("New connection from {:?}", addr);
                    tokio::spawn(handle_connection(stream));
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }
}

async fn handle_connection(stream: UnixStream) {
    let (reader, writer) = stream.into_split();
    let mut framed = tokio::io::BufReader::new(reader);
    let mut write_half = tokio::io::BufWriter::new(writer);

    use tokio::io::AsyncBufReadExt;

    let mut line = String::new();
    loop {
        line.clear();
        match framed.read_line(&mut line).await {
            Ok(0) => break,
            Ok(_) => {
                let trimmed = line.trim().to_string();
                if trimmed.is_empty() {
                    continue;
                }
                if let Some(response) =
                    crate::ipc::handler::dispatch(&trimmed).await
                {
                    use tokio::io::AsyncWriteExt;
                    let mut resp = response;
                    resp.push('\n');
                    if write_half.write_all(resp.as_bytes()).await.is_err() {
                        break;
                    }
                    if write_half.flush().await.is_err() {
                        break;
                    }
                }
            }
            Err(e) => {
                error!("Connection read error: {}", e);
                break;
            }
        }
    }
}
