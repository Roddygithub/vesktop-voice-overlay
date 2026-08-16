use anyhow::Result;
use std::io::{BufRead, Write};
use std::os::unix::net::UnixListener;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::lifecycle::OverlayCommand;
use crate::protocol::{Snapshot, MAX_PAYLOAD_SIZE, PROTOCOL_HEADER};

pub struct SocketServer {
    socket_path: String,
    cmd_tx: mpsc::UnboundedSender<OverlayCommand>,
}

impl SocketServer {
    pub fn new(socket_path: String, cmd_tx: mpsc::UnboundedSender<OverlayCommand>) -> Self {
        Self {
            socket_path,
            cmd_tx,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        // Remove existing socket file
        let _ = std::fs::remove_file(&self.socket_path);

        let listener = UnixListener::bind(&self.socket_path)?;

        // Set socket permissions to user-only (0700)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&self.socket_path)?.permissions();
            perms.set_mode(0o700);
            std::fs::set_permissions(&self.socket_path, perms)?;
        }

        // Set non-blocking for accept with timeout
        listener.set_nonblocking(true)?;

        info!("Socket server listening on {}", self.socket_path);

        // Notify lifecycle that socket is ready
        let _ = self.cmd_tx.send(OverlayCommand::SocketReady);

        loop {
            match listener.accept() {
                Ok((stream, _addr)) => {
                    let cmd_tx = self.cmd_tx.clone();
                    thread::spawn(move || {
                        if let Err(e) = handle_connection(stream, cmd_tx) {
                            error!("Connection error: {}", e);
                        }
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No connection ready, sleep briefly
                    thread::sleep(Duration::from_millis(50));
                    continue;
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }

        #[allow(unreachable_code)]
        {
            let _ = self.cmd_tx.send(OverlayCommand::SocketNotReady);
            let _ = std::fs::remove_file(&self.socket_path);
            Ok(())
        }
    }
}

fn handle_connection(
    mut stream: std::os::unix::net::UnixStream,
    cmd_tx: mpsc::UnboundedSender<OverlayCommand>,
) -> Result<()> {
    // Validate peer credentials (UID match)
    #[cfg(unix)]
    {
        use libc::{getsockopt, ucred, SOL_SOCKET, SO_PEERCRED};
        use std::os::unix::io::AsRawFd;

        let fd = stream.as_raw_fd();
        let mut cred: ucred = unsafe { std::mem::zeroed() };
        let mut cred_len = std::mem::size_of::<ucred>() as u32;

        let result = unsafe {
            getsockopt(
                fd,
                SOL_SOCKET,
                SO_PEERCRED,
                &mut cred as *mut _ as *mut _,
                &mut cred_len,
            )
        };

        if result == 0 {
            let current_uid = unsafe { libc::getuid() };
            if cred.uid != current_uid {
                warn!(
                    "Rejected connection from different UID: {} (expected {})",
                    cred.uid, current_uid
                );
                return Ok(());
            }
            debug!("Accepted connection from UID: {}", cred.uid);
        }
    }

    // Send protocol header
    stream.write_all(PROTOCOL_HEADER.as_bytes())?;
    stream.flush()?;

    let _ = cmd_tx.send(OverlayCommand::ClientConnected);

    let mut reader = std::io::BufReader::new(stream);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line)?;

        if bytes_read == 0 {
            // Client disconnected
            break;
        }

        if line.len() > MAX_PAYLOAD_SIZE {
            warn!("Payload too large, ignoring");
            continue;
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(snapshot) = Snapshot::deserialize(line) {
            let _ = cmd_tx.send(OverlayCommand::UpdateSnapshot(snapshot));
        } else {
            warn!("Failed to parse snapshot: {}", line);
        }
    }

    let _ = cmd_tx.send(OverlayCommand::ClientDisconnected);
    Ok(())
}
