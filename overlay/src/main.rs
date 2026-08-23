mod config;
mod layer_shell;
mod lifecycle;
mod protocol;
mod socket_server;
mod ui;

use anyhow::Result;
use clap::Parser;
use gtk4::gio;
use gtk4::prelude::*;
use gtk4::Application;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info};

use crate::config::Config;
use crate::layer_shell::{create_layer_shell_window, update_position};
use crate::lifecycle::{OverlayCommand, OverlayLifecycle};
use crate::socket_server::SocketServer;
use crate::ui::OverlayUI;

#[derive(Parser, Debug)]
#[command(
    name = "vesktop-voice-overlay",
    version,
    about,
    disable_version_flag = true
)]
struct Args {
    #[arg(short, long)]
    debug: bool,

    #[arg(short, long)]
    version: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.version {
        println!("vesktop-voice-overlay {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    init_logging(args.debug);

    info!(
        "Starting Vesktop Voice Overlay v{}",
        env!("CARGO_PKG_VERSION")
    );

    let application = Application::builder()
        .application_id("com.github.roddygithub.vesktop-voice-overlay")
        // Independent processes: duplicate-instance refusal is owned by the
        // Unix socket bind, not by GTK's DBus remote activation (which would
        // otherwise forward launches to the primary and confuse lifecycle).
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();

    application.connect_activate(move |app| {
        if let Err(e) = run_application(app) {
            error!("Application error: {}", e);
            // Surface duplicate-instance / bind failures to systemd and shells
            // instead of exiting successfully with a dead UI.
            std::process::exit(1);
        }
    });

    application.run_with_args(&[] as &[&str]);
    Ok(())
}

fn run_application(app: &Application) -> Result<()> {
    let config = Config::load().unwrap_or_default();
    let config = Arc::new(config);

    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel();

    // Bind synchronously before touching GTK: a second overlay instance
    // (e.g. manual launch while the systemd service runs) must fail here
    // instead of running a second UI against a dead socket.
    let server = SocketServer::new(config.socket_path().to_string(), cmd_tx.clone());
    let listener = server.bind()?;

    let window = create_layer_shell_window(app, &config)?;

    let lifecycle = OverlayLifecycle::new(cmd_tx.clone());
    let ui = OverlayUI::new(&window, &config)?;

    // Present the (fully transparent, row-less) window up front so the
    // process stays alive while waiting for Vesktop to connect; otherwise a
    // GTK application with no mapped window would exit immediately. The
    // compositor renders nothing visible until participants appear.
    window.present();

    let window_clone = window.clone();
    glib::spawn_future_local(async move {
        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                OverlayCommand::UpdateSnapshot(snapshot) => {
                    info!("Applying snapshot to overlay UI");
                    let visible = ui.update_from_snapshot(&snapshot);
                    info!(
                        "Snapshot speaking state: self={}, participants={}, visible={}",
                        snapshot.self_.speaking,
                        snapshot
                            .participants
                            .iter()
                            .filter(|participant| participant.speaking)
                            .count(),
                        visible
                    );
                    if visible {
                        if !window_clone.is_visible() {
                            window_clone.present();
                        }
                    } else {
                        window_clone.hide();
                    }
                }
                OverlayCommand::UpdateSettings(settings) => {
                    update_position(
                        &window_clone,
                        &settings.position,
                        settings.custom_x,
                        settings.custom_y,
                    );
                    if ui.update_settings(settings) {
                        if !window_clone.is_visible() {
                            window_clone.present();
                        }
                    } else {
                        window_clone.hide();
                    }
                }
                OverlayCommand::Show => {
                    info!("Showing overlay window");
                    if !window_clone.is_visible() {
                        window_clone.present();
                    }
                }
                OverlayCommand::Hide => {
                    window_clone.hide();
                }
                OverlayCommand::ClientConnected => {
                    lifecycle.on_client_connected();
                }
                OverlayCommand::ClientDisconnected => {
                    lifecycle.on_client_disconnected();
                }
                OverlayCommand::SocketReady => {
                    lifecycle.set_socket_ready(true);
                }
                OverlayCommand::SocketNotReady => {
                    lifecycle.set_socket_ready(false);
                }
            }
        }
    });

    std::thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().expect("create socket server runtime");
        let mut server = server;
        if let Err(e) = runtime.block_on(async move { server.run(listener) }) {
            error!("Socket server error: {}", e);
        }
    });

    let window_clone = window.clone();
    app.connect_shutdown(move |_| {
        window_clone.close();
    });

    Ok(())
}

fn init_logging(debug: bool) {
    let filter = if debug {
        "debug,vesktop_voice_overlay=trace"
    } else {
        "info,vesktop_voice_overlay=debug"
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();
}
