use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, info};

use crate::protocol::Snapshot;

#[derive(Debug)]
pub enum OverlayCommand {
    UpdateSnapshot(Snapshot),
    Show,
    Hide,
    ClientConnected,
    ClientDisconnected,
    SocketReady,
    SocketNotReady,
}

#[derive(Clone)]
pub struct OverlayLifecycle {
    snapshot: Arc<Mutex<Option<Snapshot>>>,
    client_connected: Arc<Mutex<bool>>,
    socket_ready: Arc<Mutex<bool>>,
    cmd_tx: mpsc::UnboundedSender<OverlayCommand>,
    hide_timeout: Arc<Mutex<Option<glib::SourceId>>>,
}

impl OverlayLifecycle {
    pub fn new(cmd_tx: mpsc::UnboundedSender<OverlayCommand>) -> Arc<Self> {
        Arc::new(Self {
            snapshot: Arc::new(Mutex::new(None)),
            client_connected: Arc::new(Mutex::new(false)),
            socket_ready: Arc::new(Mutex::new(false)),
            cmd_tx,
            hide_timeout: Arc::new(Mutex::new(None)),
        })
    }

    #[expect(dead_code)]
    pub fn update_snapshot(&self, snapshot: Snapshot) {
        {
            let mut current = self.snapshot.lock().unwrap();
            *current = Some(snapshot.clone());
        }
        let _ = self.cmd_tx.send(OverlayCommand::UpdateSnapshot(snapshot));
        self.show_overlay();
    }

    pub fn on_client_connected(&self) {
        *self.client_connected.lock().unwrap() = true;
        info!("Plugin connected");
        self.cancel_hide_timeout();
        let _ = self.cmd_tx.send(OverlayCommand::Show);
    }

    pub fn on_client_disconnected(&self) {
        *self.client_connected.lock().unwrap() = false;
        info!("Plugin disconnected");
        self.schedule_hide(Duration::from_secs(5));
    }

    pub fn set_socket_ready(&self, ready: bool) {
        *self.socket_ready.lock().unwrap() = ready;
        if !ready {
            self.schedule_hide(Duration::from_secs(2));
        } else {
            self.cancel_hide_timeout();
        }
    }

    fn show_overlay(&self) {
        self.cancel_hide_timeout();
        let _ = self.cmd_tx.send(OverlayCommand::Show);
    }

    fn schedule_hide(&self, delay: Duration) {
        self.cancel_hide_timeout();
        let lifecycle = self.clone();
        let source_id = glib::timeout_add_local_once(delay, move || {
            let connected = *lifecycle.client_connected.lock().unwrap();
            let socket_ready = *lifecycle.socket_ready.lock().unwrap();

            if !connected || !socket_ready {
                let _ = lifecycle.cmd_tx.send(OverlayCommand::Hide);
                debug!("Overlay hidden (no client or socket not ready)");
            }
        });
        *self.hide_timeout.lock().unwrap() = Some(source_id);
    }

    fn cancel_hide_timeout(&self) {
        if let Some(id) = self.hide_timeout.lock().unwrap().take() {
            id.remove();
        }
    }

    #[allow(dead_code)]
    pub fn current_snapshot(&self) -> Option<Snapshot> {
        self.snapshot.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_lifecycle() -> (
        Arc<OverlayLifecycle>,
        mpsc::UnboundedReceiver<OverlayCommand>,
    ) {
        let (tx, rx) = mpsc::unbounded_channel();
        (OverlayLifecycle::new(tx), rx)
    }

    #[test]
    fn on_client_connected_sets_flag_and_sends_show() {
        let (lifecycle, mut rx) = make_lifecycle();
        assert!(!*lifecycle.client_connected.lock().unwrap());

        lifecycle.on_client_connected();

        assert!(*lifecycle.client_connected.lock().unwrap());
        assert!(matches!(rx.try_recv(), Ok(OverlayCommand::Show)));
    }

    #[test]
    fn on_client_connected_can_be_called_repeatedly() {
        let (lifecycle, mut rx) = make_lifecycle();

        lifecycle.on_client_connected();
        let _ = rx.try_recv(); // drain Show

        lifecycle.on_client_connected();
        assert!(*lifecycle.client_connected.lock().unwrap());
        assert!(matches!(rx.try_recv(), Ok(OverlayCommand::Show)));
    }

    #[test]
    fn set_socket_ready_true_sets_flag() {
        let (lifecycle, mut rx) = make_lifecycle();
        assert!(!*lifecycle.socket_ready.lock().unwrap());

        lifecycle.set_socket_ready(true);

        assert!(*lifecycle.socket_ready.lock().unwrap());
        // No commands sent when becoming ready (just cancels timeout)
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn set_socket_ready_true_cancels_hide_timeout() {
        let (lifecycle, _rx) = make_lifecycle();

        // Set ready=true — no timeout should be pending
        lifecycle.set_socket_ready(true);
        assert!(lifecycle.hide_timeout.lock().unwrap().is_none());
    }

    #[test]
    fn initial_state_all_false() {
        let (lifecycle, _rx) = make_lifecycle();
        assert!(!*lifecycle.client_connected.lock().unwrap());
        assert!(!*lifecycle.socket_ready.lock().unwrap());
        assert!(lifecycle.current_snapshot().is_none());
    }

    #[test]
    fn lifecycle_sends_only_expected_commands() {
        let (lifecycle, mut rx) = make_lifecycle();

        lifecycle.on_client_connected();
        assert!(matches!(rx.try_recv(), Ok(OverlayCommand::Show)));

        // No other commands should be queued
        assert!(rx.try_recv().is_err());
    }
}
