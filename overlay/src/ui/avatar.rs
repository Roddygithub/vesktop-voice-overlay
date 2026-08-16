use cairo::ImageSurface;
use gdk_pixbuf::Pixbuf;
use gtk4::prelude::*;
use gtk4::Picture;
use std::rc::Rc;
use std::sync::Mutex;

pub struct AvatarWidget {
    picture: Picture,
    current_url: Mutex<Option<String>>,
    size: i32,
}

impl AvatarWidget {
    pub fn new(url: &str, size: i32) -> Rc<Self> {
        let picture = Picture::new();
        picture.set_size_request(size, size);
        picture.set_can_shrink(false);
        picture.add_css_class("participant-avatar");

        let this = Rc::new(Self {
            picture,
            current_url: Mutex::new(None),
            size,
        });

        if !url.is_empty() {
            let this_clone = this.clone();
            let url = url.to_string();
            glib::spawn_future_local(async move {
                this_clone.load_avatar(&url).await;
            });
        } else {
            this.set_placeholder();
        }

        this
    }

    pub fn widget(&self) -> &Picture {
        &self.picture
    }

    #[expect(dead_code)]
    pub async fn update_url(&self, url: &str) {
        let should_load = {
            let mut current = self.current_url.lock().unwrap();
            if *current == Some(url.to_string()) {
                return;
            }
            *current = Some(url.to_string());
            true
        };
        if should_load {
            self.load_avatar(url).await;
        }
    }

    async fn load_avatar(&self, url: &str) {
        let bytes = match reqwest::get(url).await {
            Ok(response) if response.status().is_success() => response.bytes().await.ok(),
            _ => None,
        };

        if let Some(bytes) = bytes {
            if let Ok(texture) = self.bytes_to_texture(&bytes).await {
                self.picture.set_paintable(Some(&texture));
                return;
            }
        }

        self.set_placeholder();
    }

    async fn bytes_to_texture(&self, bytes: &[u8]) -> anyhow::Result<gdk4::Texture> {
        let img = image::load_from_memory(bytes)?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        let stride = cairo::Format::ARgb32.stride_for_width(width).unwrap() as u32;
        let mut surface = ImageSurface::create(cairo::Format::ARgb32, width as i32, height as i32)?;

        {
            let mut data = surface.data().unwrap();
            for y in 0..height {
                let src_row =
                    &rgba.as_raw()[(y * width * 4) as usize..((y + 1) * width * 4) as usize];
                let dst_row = &mut data[(y * stride) as usize..((y + 1) * stride) as usize];
                for x in 0..width {
                    let src_idx = (x * 4) as usize;
                    let dst_idx = (x * 4) as usize;
                    if dst_idx + 3 < dst_row.len() && src_idx + 3 < src_row.len() {
                        dst_row[dst_idx] = src_row[src_idx + 2];
                        dst_row[dst_idx + 1] = src_row[src_idx + 1];
                        dst_row[dst_idx + 2] = src_row[src_idx];
                        dst_row[dst_idx + 3] = src_row[src_idx + 3];
                    }
                }
            }
        }
        surface.flush();

        let data = surface.data().unwrap();
        let pixbuf = Pixbuf::from_mut_slice(
            data.to_vec(),
            gdk_pixbuf::Colorspace::Rgb,
            true,
            8,
            width as i32,
            height as i32,
            stride.try_into().unwrap(),
        );

        Ok(gdk4::Texture::for_pixbuf(&pixbuf))
    }

    fn set_placeholder(&self) {
        let texture = self.create_placeholder_texture();
        self.picture.set_paintable(Some(&texture));
    }

    fn create_placeholder_texture(&self) -> gdk4::Texture {
        let mut surface = ImageSurface::create(cairo::Format::ARgb32, self.size, self.size)
            .expect("Failed to create surface");
        let cr = cairo::Context::new(&surface).expect("Failed to create context");

        let radius = (self.size as f64) / 2.0;
        cr.arc(
            radius,
            radius,
            radius - 1.0,
            0.0,
            2.0 * std::f64::consts::PI,
        );
        cr.set_source_rgba(0.3, 0.3, 0.3, 1.0);
        cr.fill().expect("Failed to fill");

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

        let stride = surface.stride() as u32;
        let data = surface.data().expect("Failed to get surface data");
        let pixbuf = Pixbuf::from_mut_slice(
            data.to_vec(),
            gdk_pixbuf::Colorspace::Rgb,
            true,
            8,
            self.size,
            self.size,
            stride.try_into().unwrap(),
        );

        gdk4::Texture::for_pixbuf(&pixbuf)
    }
}
