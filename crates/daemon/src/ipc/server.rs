use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::net::{UnixListener, UnixStream};
use tracing::{error, info};

const MAX_FRAME_BYTES: usize = 64 * 1024;
static ARRIVAL_SEQUENCE: AtomicU64 = AtomicU64::new(1);

pub struct IpcServer {
    socket_path: PathBuf,
}

impl IpcServer {
    pub fn new(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    pub async fn run(self) -> std::io::Result<()> {
        if let Some(parent) = self.socket_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if self.socket_path.exists() {
            let metadata = std::fs::symlink_metadata(&self.socket_path)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::FileTypeExt;
                if !metadata.file_type().is_socket() {
                    return Err(std::io::Error::other("refusing to replace a non-socket IPC path"));
                }
            }
            if std::os::unix::net::UnixStream::connect(&self.socket_path).is_ok() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::AddrInUse,
                    "ShellClaw daemon is already listening",
                ));
            }
            std::fs::remove_file(&self.socket_path)?;
        }

        let listener = UnixListener::bind(&self.socket_path)?;
        let _socket_guard = SocketFileGuard(self.socket_path.clone());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&self.socket_path, std::fs::Permissions::from_mode(0o600))?;
        }
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

struct SocketFileGuard(PathBuf);

impl Drop for SocketFileGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

async fn handle_connection(stream: UnixStream) {
    let (reader, writer) = stream.into_split();
    let mut framed = tokio::io::BufReader::new(reader);
    let (response_tx, mut response_rx) = tokio::sync::mpsc::channel::<String>(32);

    let writer_task = tokio::spawn(async move {
        use tokio::io::AsyncWriteExt;
        let mut write_half = tokio::io::BufWriter::new(writer);
        while let Some(mut response) = response_rx.recv().await {
            response.push('\n');
            if write_half.write_all(response.as_bytes()).await.is_err()
                || write_half.flush().await.is_err()
            {
                break;
            }
        }
    });

    use tokio::io::{AsyncBufReadExt, AsyncReadExt};

    let mut line = String::new();
    loop {
        line.clear();
        let read_result = {
            // Limit allocation while reading, rather than checking only after
            // an arbitrarily large newline-delimited frame is already buffered.
            let mut limited = (&mut framed).take((MAX_FRAME_BYTES + 1) as u64);
            limited.read_line(&mut line).await
        };
        match read_result {
            Ok(0) => break,
            Ok(_) => {
                if line.len() > MAX_FRAME_BYTES {
                    break;
                }
                let trimmed = line.trim_end_matches(['\r', '\n']).to_string();
                if trimmed.is_empty() {
                    continue;
                }
                let tx = response_tx.clone();
                let arrival_sequence = ARRIVAL_SEQUENCE.fetch_add(1, Ordering::Relaxed);
                tokio::spawn(async move {
                    if let Some(response) =
                        crate::ipc::handler::dispatch(&trimmed, arrival_sequence).await
                    {
                        let _ = tx.send(response).await;
                    }
                });
            }
            Err(e) => {
                error!("Connection read error: {}", e);
                break;
            }
        }
    }

    drop(response_tx);
    let _ = writer_task.await;
}
