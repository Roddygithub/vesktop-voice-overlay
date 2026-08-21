use gdk_pixbuf::Pixbuf;
use gtk4::prelude::*;
use gtk4::{Align, Picture};
use std::rc::Rc;
use std::sync::Mutex;

pub struct AvatarWidget {
    picture: Picture,
    current_url: Mutex<Option<String>>,
    size: Mutex<i32>,
}

impl AvatarWidget {
    pub fn new(url: &str, size: i32) -> Rc<Self> {
        let picture = Picture::new();
        picture.set_size_request(size, size);
        picture.set_can_shrink(true);
        picture.set_halign(Align::Start);
        picture.set_valign(Align::Center);
        picture.add_css_class("participant-avatar");

        let this = Rc::new(Self {
            picture,
            current_url: Mutex::new(None),
            size: Mutex::new(size),
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
        let url = url.to_string();
        let (tx, rx) = tokio::sync::oneshot::channel();
        std::thread::spawn(move || {
            let bytes = tokio::runtime::Runtime::new().ok().and_then(|runtime| {
                runtime.block_on(async {
                    match reqwest::get(&url).await {
                        Ok(response) if response.status().is_success() => {
                            response.bytes().await.ok().map(|bytes| bytes.to_vec())
                        }
                        _ => None,
                    }
                })
            });
            let _ = tx.send(bytes);
        });

        let bytes = rx.await.ok().flatten();

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
        let pixbuf = Pixbuf::from_mut_slice(
            rgba.into_raw(),
            gdk_pixbuf::Colorspace::Rgb,
            true,
            8,
            width as i32,
            height as i32,
            (width * 4) as i32,
        );

        Ok(gdk4::Texture::for_pixbuf(&pixbuf))
    }

    fn set_placeholder(&self) {
        let texture = self.create_placeholder_texture();
        self.picture.set_paintable(Some(&texture));
    }

    fn create_placeholder_texture(&self) -> gdk4::Texture {
        let size = *self.size.lock().unwrap();
        let pixbuf = Pixbuf::new(gdk_pixbuf::Colorspace::Rgb, true, 8, size, size)
            .expect("Failed to create placeholder pixbuf");
        pixbuf.fill(0x4d4d4dff);

        gdk4::Texture::for_pixbuf(&pixbuf)
    }

    pub fn set_size(&self, size: i32) {
        *self.size.lock().unwrap() = size;
        self.picture.set_size_request(size, size);
        self.picture.queue_resize();
        if self.picture.paintable().is_none() {
            self.set_placeholder();
        }
    }
}
