use anyhow::Result;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};
use gtk4_layer_shell::{Edge, Layer, LayerShell as _};

use crate::config::Config;

pub fn create_layer_shell_window(app: &Application, config: &Config) -> Result<ApplicationWindow> {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Vesktop Voice Overlay")
        .decorated(false)
        .resizable(false)
        .build();

    // Initialize layer shell
    window.init_layer_shell();

    // Set layer to overlay (above normal windows, below fullscreen)
    window.set_layer(Layer::Overlay);

    // Make it click-through (no keyboard interactivity)
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);

    // Anchor based on config position
    set_anchors(
        &window,
        &config.overlay.position,
        config.overlay.custom_x,
        config.overlay.custom_y,
    );

    // Exclusive zone: 0 means no reserved space
    window.set_exclusive_zone(0);

    // Set default size (will be constrained by content)
    window.set_default_size(300, 200);

    // CSS class for styling
    window.add_css_class("vesktop-voice-overlay");

    Ok(window)
}

fn set_anchors(window: &ApplicationWindow, position: &str, custom_x: i32, custom_y: i32) {
    use gtk4_layer_shell::{Edge, LayerShell as _};

    // Reset all anchors first
    for edge in [Edge::Left, Edge::Right, Edge::Top, Edge::Bottom] {
        window.set_anchor(edge, false);
    }

    // Remove any margin
    window.set_margin(Edge::Left, 0);
    window.set_margin(Edge::Right, 0);
    window.set_margin(Edge::Top, 0);
    window.set_margin(Edge::Bottom, 0);

    match position {
        "top-left" => {
            window.set_anchor(Edge::Top, true);
            window.set_anchor(Edge::Left, true);
            window.set_margin(Edge::Top, 20);
            window.set_margin(Edge::Left, 20);
        }
        "top-right" => {
            window.set_anchor(Edge::Top, true);
            window.set_anchor(Edge::Right, true);
            window.set_margin(Edge::Top, 20);
            window.set_margin(Edge::Right, 20);
        }
        "bottom-left" => {
            window.set_anchor(Edge::Bottom, true);
            window.set_anchor(Edge::Left, true);
            window.set_margin(Edge::Bottom, 20);
            window.set_margin(Edge::Left, 20);
        }
        "bottom-right" => {
            window.set_anchor(Edge::Bottom, true);
            window.set_anchor(Edge::Right, true);
            window.set_margin(Edge::Bottom, 20);
            window.set_margin(Edge::Right, 20);
        }
        "center" => {
            window.set_anchor(Edge::Top, true);
            window.set_anchor(Edge::Bottom, true);
            window.set_anchor(Edge::Left, true);
            window.set_anchor(Edge::Right, true);
        }
        "custom" => {
            window.set_anchor(Edge::Top, true);
            window.set_anchor(Edge::Left, true);
            window.set_margin(Edge::Top, custom_y);
            window.set_margin(Edge::Left, custom_x);
        }
        _ => {
            // Default to top-right
            window.set_anchor(Edge::Top, true);
            window.set_anchor(Edge::Right, true);
            window.set_margin(Edge::Top, 20);
            window.set_margin(Edge::Right, 20);
        }
    }
}
