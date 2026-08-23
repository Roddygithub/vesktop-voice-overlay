use anyhow::Result;
use gtk4::prelude::*;
use gtk4::Application;
use gtk4_layer_shell::{Layer, LayerShell as _};

use crate::config::Config;

/// Pointer pass-through: an empty input region makes the Wayland compositor
/// deliver all pointer input to surfaces below the overlay while it stays
/// fully rendered. Keyboard interactivity stays disabled independently.
pub fn empty_input_region() -> gtk4::cairo::Region {
    gtk4::cairo::Region::create()
}

pub fn apply_click_through(window: &gtk4::ApplicationWindow) {
    let Some(surface) = window.surface() else {
        return;
    };
    surface.set_input_region(Some(&empty_input_region()));
    tracing::debug!("Applied empty input region (pointer click-through)");
}

pub fn create_layer_shell_window(
    app: &Application,
    config: &Config,
) -> Result<gtk4::ApplicationWindow> {
    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("Vesktop Voice Overlay")
        .decorated(false)
        .resizable(false)
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);
    set_anchors(
        &window,
        &config.overlay.position,
        config.overlay.custom_x,
        config.overlay.custom_y,
    );
    window.set_exclusive_zone(0);
    window.set_default_size(240, 60);
    window.add_css_class("vesktop-voice-overlay");

    // The GdkSurface only exists once the window is mapped; re-apply on every
    // map so remounts keep pointer pass-through.
    window.connect_map(apply_click_through);

    Ok(window)
}

pub fn update_position(
    window: &gtk4::ApplicationWindow,
    position: &str,
    custom_x: i32,
    custom_y: i32,
) {
    tracing::debug!(
        "update_position called: position={}, custom_x={}, custom_y={}",
        position,
        custom_x,
        custom_y
    );
    set_anchors(window, position, custom_x, custom_y);
}

fn set_anchors(window: &gtk4::ApplicationWindow, position: &str, custom_x: i32, custom_y: i32) {
    use gtk4_layer_shell::{Edge, LayerShell as _};

    for edge in [Edge::Left, Edge::Right, Edge::Top, Edge::Bottom] {
        window.set_anchor(edge, false);
    }

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
            window.set_anchor(Edge::Top, true);
            window.set_anchor(Edge::Right, true);
            window.set_margin(Edge::Top, 20);
            window.set_margin(Edge::Right, 20);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_region_is_actually_empty() {
        let region = empty_input_region();
        assert!(region.is_empty());
        assert_eq!(region.num_rectangles(), 0);
    }
}
