use anyhow::Result;
use std::io::{self, BufRead, Write};
use std::os::unix::net::UnixListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::lifecycle::OverlayCommand;
use crate::protocol::{
    deserialize_client_message, ClientMessage, Snapshot, MAX_PAYLOAD_SIZE, PROTOCOL_HEADER,
};

pub struct SocketServer {
    socket_path: String,
    cmd_tx: mpsc::Sender<OverlayCommand>,
}

const MAX_CONNECTIONS: usize = 1;

impl SocketServer {
    pub fn new(socket_path: String, cmd_tx: mpsc::Sender<OverlayCommand>) -> Self {
        Self {
            socket_path,
            cmd_tx,
        }
    }

    /// Bind the listening socket. Fails loudly when another live overlay
    /// instance already owns the socket path so callers can refuse duplicate
    /// instances; a stale socket left behind by a crashed instance is replaced.
    pub fn bind(&self) -> Result<UnixListener> {
        use std::os::unix::net::UnixStream;

        let listener = match UnixListener::bind(&self.socket_path) {
            Ok(listener) => listener,
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                if UnixStream::connect(&self.socket_path).is_ok() {
                    anyhow::bail!(
                        "another vesktop-voice-overlay instance owns {}",
                        self.socket_path
                    );
                }
                debug!("Removing stale socket at {}", self.socket_path);
                std::fs::remove_file(&self.socket_path)?;
                UnixListener::bind(&self.socket_path)?
            }
            Err(e) => return Err(e.into()),
        };

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&self.socket_path)?.permissions();
            perms.set_mode(0o700);
            std::fs::set_permissions(&self.socket_path, perms)?;
        }

        listener.set_nonblocking(true)?;

        info!("Socket server listening on {}", self.socket_path);

        Ok(listener)
    }

    pub fn run(&mut self, listener: UnixListener) -> Result<()> {
        let active_connections = Arc::new(AtomicUsize::new(0));

        loop {
            match listener.accept() {
                Ok((stream, _addr)) => {
                    if active_connections
                        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |active| {
                            (active < MAX_CONNECTIONS).then_some(active + 1)
                        })
                        .is_err()
                    {
                        warn!(
                            max_connections = MAX_CONNECTIONS,
                            "Rejecting connection: connection limit reached"
                        );
                        continue;
                    }

                    let cmd_tx = self.cmd_tx.clone();
                    let thread_connections = active_connections.clone();
                    let spawn_result = thread::Builder::new()
                        .name("overlay-ipc-client".to_string())
                        .spawn(move || {
                            if let Err(e) = handle_connection(stream, cmd_tx) {
                                error!("Connection error: {}", e);
                            }
                            thread_connections.fetch_sub(1, Ordering::Relaxed);
                        });
                    if let Err(error) = spawn_result {
                        active_connections.fetch_sub(1, Ordering::Relaxed);
                        error!("Failed to start connection thread: {error}");
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
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
            let _ = std::fs::remove_file(&self.socket_path);
            Ok(())
        }
    }
}

fn handle_connection(
    mut stream: std::os::unix::net::UnixStream,
    cmd_tx: mpsc::Sender<OverlayCommand>,
) -> Result<()> {
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

        if result != 0 || cred_len as usize != std::mem::size_of::<ucred>() {
            anyhow::bail!(
                "failed to read peer credentials: {}",
                io::Error::last_os_error()
            );
        }

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

    stream.write_all(PROTOCOL_HEADER.as_bytes())?;
    stream.flush()?;

    info!("Socket client connected; dispatching ClientConnected");
    cmd_tx
        .blocking_send(OverlayCommand::ClientConnected)
        .map_err(|_| anyhow::anyhow!("GTK command receiver closed"))?;

    let mut reader = std::io::BufReader::new(stream);
    let result = read_messages(&mut reader, &cmd_tx);
    let _ = cmd_tx.blocking_send(OverlayCommand::ClientDisconnected);
    result
}

fn read_messages<R: BufRead>(reader: &mut R, cmd_tx: &mpsc::Sender<OverlayCommand>) -> Result<()> {
    loop {
        let line = match read_bounded_line(reader)? {
            BoundedLine::Eof => break,
            BoundedLine::TooLarge => {
                warn!(max_bytes = MAX_PAYLOAD_SIZE, "Payload too large, ignoring");
                continue;
            }
            BoundedLine::InvalidUtf8 => {
                warn!("Payload is not valid UTF-8, ignoring");
                continue;
            }
            BoundedLine::Line(line) => line,
        };

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(snapshot) = Snapshot::deserialize(line) {
            info!(
                "Received snapshot with {} participants",
                snapshot.participants.len()
            );
            cmd_tx
                .blocking_send(OverlayCommand::UpdateSnapshot(snapshot))
                .map_err(|_| anyhow::anyhow!("GTK command receiver closed"))?;
        } else if let Some(ClientMessage::Settings { settings }) = deserialize_client_message(line)
        {
            info!("Received overlay settings update");
            cmd_tx
                .blocking_send(OverlayCommand::UpdateSettings(settings))
                .map_err(|_| anyhow::anyhow!("GTK command receiver closed"))?;
        } else if matches!(deserialize_client_message(line), Some(ClientMessage::Clear)) {
            info!("Received voice state clear");
            cmd_tx
                .blocking_send(OverlayCommand::Clear)
                .map_err(|_| anyhow::anyhow!("GTK command receiver closed"))?;
        } else {
            warn!(
                payload_bytes = line.len(),
                context = %safe_parse_context(line),
                "Failed to parse client message; payload not logged"
            );
        }
    }

    Ok(())
}

enum BoundedLine {
    Eof,
    Line(String),
    TooLarge,
    InvalidUtf8,
}

/// Reads and consumes one line while retaining at most MAX_PAYLOAD_SIZE bytes.
/// Oversized input is discarded through the newline without allocating in
/// proportion to attacker-controlled input.
fn read_bounded_line<R: BufRead>(reader: &mut R) -> io::Result<BoundedLine> {
    let mut bytes = Vec::with_capacity(1024);
    let mut payload_len = 0usize;
    let mut too_large = false;

    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            if payload_len == 0 {
                return Ok(BoundedLine::Eof);
            }
            break;
        }

        let newline = available.iter().position(|byte| *byte == b'\n');
        let payload_bytes = newline.unwrap_or(available.len());
        payload_len = payload_len.saturating_add(payload_bytes);

        if !too_large && payload_len <= MAX_PAYLOAD_SIZE {
            bytes.extend_from_slice(&available[..payload_bytes]);
        } else {
            too_large = true;
        }

        let consumed = newline.map_or(available.len(), |index| index + 1);
        reader.consume(consumed);
        if newline.is_some() {
            break;
        }
    }

    if too_large {
        return Ok(BoundedLine::TooLarge);
    }

    match String::from_utf8(bytes) {
        Ok(line) => Ok(BoundedLine::Line(line)),
        Err(_) => Ok(BoundedLine::InvalidUtf8),
    }
}

/// Parse failures may contain names or user IDs, so diagnostics only expose
/// the byte count and never any portion of the payload.
fn safe_parse_context(line: &str) -> String {
    format!("len={}", line.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_parse_context_never_includes_payload_content() {
        let long = format!("{{\"userId\":\"{}\"}}", "A".repeat(500));
        let context = safe_parse_context(&long);
        assert!(context.contains("len=513"), "context was: {context}");
        assert!(!context.contains("userId"));
        assert!(!context.contains('A'));
    }

    #[test]
    fn safe_parse_context_does_not_log_control_characters() {
        let context = safe_parse_context("bad\r\njson\x00here");
        assert!(!context.contains('\r'));
        assert!(!context.contains('\n'));
        assert!(!context.contains('\0'));
        assert_eq!(context, "len=14");
    }

    #[test]
    fn bounded_reader_accepts_payload_at_limit() {
        let input = format!("{}\nnext\n", "a".repeat(MAX_PAYLOAD_SIZE));
        let mut reader = std::io::BufReader::with_capacity(17, input.as_bytes());

        match read_bounded_line(&mut reader).expect("line reads") {
            BoundedLine::Line(line) => assert_eq!(line.len(), MAX_PAYLOAD_SIZE),
            _ => panic!("payload at the limit must be accepted"),
        }
        match read_bounded_line(&mut reader).expect("next line reads") {
            BoundedLine::Line(line) => assert_eq!(line, "next"),
            _ => panic!("reader must remain synchronized after a valid line"),
        }
    }

    #[test]
    fn bounded_reader_discards_oversized_line_and_resynchronizes() {
        let input = format!("{}\nvalid\n", "a".repeat(MAX_PAYLOAD_SIZE + 10_000));
        let mut reader = std::io::BufReader::with_capacity(31, input.as_bytes());

        assert!(matches!(
            read_bounded_line(&mut reader).expect("oversized line reads"),
            BoundedLine::TooLarge
        ));
        match read_bounded_line(&mut reader).expect("next line reads") {
            BoundedLine::Line(line) => assert_eq!(line, "valid"),
            _ => panic!("reader must resynchronize at the next newline"),
        }
    }

    #[test]
    fn bounded_reader_rejects_invalid_utf8_without_losing_next_line() {
        let bytes = [b"bad\xff\n".as_slice(), b"valid\n".as_slice()].concat();
        let mut reader = std::io::BufReader::new(bytes.as_slice());

        assert!(matches!(
            read_bounded_line(&mut reader).expect("invalid line reads"),
            BoundedLine::InvalidUtf8
        ));
        assert!(matches!(
            read_bounded_line(&mut reader).expect("valid line reads"),
            BoundedLine::Line(line) if line == "valid"
        ));
    }
}
