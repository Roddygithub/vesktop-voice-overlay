use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, info};

use crate::config::OverlaySettings;
use crate::protocol::Snapshot;

#[derive(Debug)]
pub enum OverlayCommand {
    UpdateSnapshot(Snapshot),
    UpdateSettings(OverlaySettings),
    Clear,
    Hide,
    ClientConnected,
    ClientDisconnected,
}

pub struct OverlayLifecycle {
    connected_clients: Cell<usize>,
    cmd_tx: mpsc::Sender<OverlayCommand>,
    hide_timeout: RefCell<Option<glib::SourceId>>,
}

impl OverlayLifecycle {
    pub fn new(cmd_tx: mpsc::Sender<OverlayCommand>) -> Rc<Self> {
        Rc::new(Self {
            connected_clients: Cell::new(0),
            cmd_tx,
            hide_timeout: RefCell::new(None),
        })
    }

    pub fn on_client_connected(&self) {
        self.connected_clients
            .set(self.connected_clients.get().saturating_add(1));
        info!("Plugin connected ({} active)", self.connected_clients.get());
        self.cancel_hide_timeout();
    }

    pub fn on_client_disconnected(self: &Rc<Self>) {
        let connected_clients = self.connected_clients.get();
        if connected_clients == 0 {
            debug!("Ignoring client disconnect with no active client");
            return;
        }
        self.connected_clients.set(connected_clients - 1);
        info!(
            "Plugin disconnected ({} active)",
            self.connected_clients.get()
        );
        if self.connected_clients.get() == 0 {
            self.schedule_hide(Duration::from_secs(5));
        }
    }

    fn schedule_hide(self: &Rc<Self>, delay: Duration) {
        self.cancel_hide_timeout();
        let lifecycle = self.clone();
        let source_id = glib::timeout_add_local_once(delay, move || {
            lifecycle.hide_timeout.borrow_mut().take();
            if lifecycle.connected_clients.get() == 0 {
                glib::spawn_future_local(async move {
                    if lifecycle.cmd_tx.send(OverlayCommand::Hide).await.is_ok() {
                        debug!("Queued overlay hide (no connected client)");
                    }
                });
            }
        });
        *self.hide_timeout.borrow_mut() = Some(source_id);
    }

    fn cancel_hide_timeout(&self) {
        if let Some(id) = self.hide_timeout.borrow_mut().take() {
            id.remove();
        }
    }

    pub fn should_hide(&self) -> bool {
        self.connected_clients.get() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_lifecycle() -> (Rc<OverlayLifecycle>, mpsc::Receiver<OverlayCommand>) {
        let (tx, rx) = mpsc::channel(8);
        (OverlayLifecycle::new(tx), rx)
    }

    #[test]
    fn on_client_connected_sets_flag_without_showing_empty_overlay() {
        let (lifecycle, mut rx) = make_lifecycle();
        assert_eq!(lifecycle.connected_clients.get(), 0);

        lifecycle.on_client_connected();

        assert_eq!(lifecycle.connected_clients.get(), 1);
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn disconnecting_one_of_multiple_clients_does_not_schedule_hide() {
        let (lifecycle, mut rx) = make_lifecycle();

        lifecycle.on_client_connected();
        lifecycle.on_client_connected();
        lifecycle.on_client_disconnected();

        assert_eq!(lifecycle.connected_clients.get(), 1);
        assert!(lifecycle.hide_timeout.borrow().is_none());
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn extra_disconnect_never_underflows_connection_count() {
        let (lifecycle, _rx) = make_lifecycle();
        lifecycle.on_client_disconnected();
        assert_eq!(lifecycle.connected_clients.get(), 0);
        assert!(lifecycle.hide_timeout.borrow().is_none());
    }

    #[test]
    fn reconnect_cancels_delayed_hide_and_rejects_a_stale_hide() {
        let (lifecycle, _rx) = make_lifecycle();
        lifecycle.on_client_connected();
        lifecycle.on_client_disconnected();
        assert!(lifecycle.hide_timeout.borrow().is_some());

        lifecycle.on_client_connected();

        assert!(lifecycle.hide_timeout.borrow().is_none());
        assert!(!lifecycle.should_hide());
    }
}
