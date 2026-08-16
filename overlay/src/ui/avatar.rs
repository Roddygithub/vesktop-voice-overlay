use anyhow::Result;
use gtk4::prelude::*;
use gtk4::{Picture, Widget};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::Config;

pub struct AvatarWidget {
    picture: Picture,
    current_url: Arc<Mutex<Option<String>>>,
    size: i32,
}

impl AvatarWidget {
    pub fn new(url: &str, size: i32) -> Arc<Self> {
        let picture = Picture::new();
        picture.set_size_request(size, size);
        picture.set_can_shrink(false);
        picture.add_css_class("participant-avatar");
        
        let this = Arc::new(Self {
            picture,
            current_url: Arc::new(Mutex::new(None)),
            size,
        });

        if !url.is_empty() {
            let this_clone = this.clone();
            let _url = url.to_string();
            glib::spawn_future_local(async move {
                this_clone.load_avatar().await;
            });
        } else {
            this.set_placeholder();
        }

        this
    }

    pub fn widget(&self) -> &Picture {
        &self.picture
    }

    pub async fn update_url(&self, url: &str) {
        let mut current = self.current_url.lock().await;
        if *current == Some(url.to_string()) {
            return;
        }
        *current = Some(url.to_string());
        drop(current);
        self.load_avatar().await;
    }

    async fn load_avatar(&self) {
        // TODO: Implement proper avatar loading with image crate + cairo
        // For now, use placeholder
        self.set_placeholder();
    }

    fn set_placeholder(&self) {
        let texture = self.create_placeholder_texture();
        self.picture.set_paintable(Some(&texture));
    }

    fn create_placeholder_texture(&self) -> gdk4::Texture {
        use cairo::ImageSurface;
        use gdk_pixbuf::Pixbuf;
        
        let mut surface = ImageSurface::create(cairo::Format::ARgb32, self.size, self.size)
            .expect("Failed to create surface");
        let cr = cairo::Context::new(&surface).expect("Failed to create context");
        
        // Draw circle background
        let radius = (self.size as f64) / 2.0;
        cr.arc(radius, radius, radius - 1.0, 0.0, 2.0 * std::f64::consts::PI);
        cr.set_source_rgba(0.3, 0.3, 0.3, 1.0);
        cr.fill().expect("Failed to fill");
        
        // Draw initial letter
        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
        cr.set_font_size(radius * 0.7);
        let extents = cr.text_extents("?").expect("Failed to get text extents");
        cr.move_to(
            radius - extents.width() / 2.0 - extents.x_bearing(),
            radius - extents.height() / 2.0 - extents.y_bearing(),
        );
        cr.show_text("?").expect("Failed to show text");
        
        surface.flush();
        
        let stride = surface.stride();
        let data = surface.data().expect("Failed to get surface data");
        let pixbuf = Pixbuf::from_mut_slice(
            data.to_vec(),
            gdk_pixbuf::Colorspace::Rgb,
            true,
            8,
            self.size,
            self.size,
            stride as i32,
        );
        
        gdk4::Texture::for_pixbuf(&pixbuf)
    }
}
