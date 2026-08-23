use gdk_pixbuf::Pixbuf;
use gtk4::prelude::*;
use gtk4::{Align, Picture};
use once_cell::sync::Lazy;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::sync::Mutex;

/// One shared worker pool for all avatar downloads: a single runtime per
/// process instead of one thread + one Tokio runtime per request.
static IO_RUNTIME: Lazy<tokio::runtime::Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .expect("create shared avatar IO runtime")
});

const AVATAR_CACHE_CAPACITY: usize = 128;

#[derive(Clone)]
struct CachedImage {
    width: i32,
    height: i32,
    rgba: Vec<u8>,
}

impl CachedImage {
    fn to_pixbuf(&self) -> anyhow::Result<Pixbuf> {
        Ok(Pixbuf::from_mut_slice(
            self.rgba.clone(),
            gdk_pixbuf::Colorspace::Rgb,
            true,
            8,
            self.width,
            self.height,
            self.width * 4,
        ))
    }
}

fn decode_rgba(bytes: &[u8]) -> anyhow::Result<CachedImage> {
    let img = image::load_from_memory(bytes)?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok(CachedImage {
        width: width as i32,
        height: height as i32,
        rgba: rgba.into_raw(),
    })
}

/// Bounded FIFO-evicted cache keyed by avatar URL. Generic over the stored
/// value so the eviction policy can be unit-tested without graphical types.
struct AvatarCache<V> {
    capacity: usize,
    entries: HashMap<String, V>,
    order: VecDeque<String>,
}

impl<V> AvatarCache<V> {
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    fn get(&self, key: &str) -> Option<&V> {
        self.entries.get(key)
    }

    fn put(&mut self, key: String, value: V) {
        if let Some(existing) = self.entries.get_mut(&key) {
            *existing = value;
            return;
        }
        while self.entries.len() >= self.capacity {
            if let Some(oldest) = self.order.pop_front() {
                self.entries.remove(&oldest);
            } else {
                break;
            }
        }
        self.order.push_back(key.clone());
        self.entries.insert(key, value);
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries.len()
    }
}

static IMAGE_CACHE: Lazy<Mutex<AvatarCache<CachedImage>>> =
    Lazy::new(|| Mutex::new(AvatarCache::new(AVATAR_CACHE_CAPACITY)));

fn cached_image(url: &str) -> Option<Pixbuf> {
    let cache = IMAGE_CACHE.lock().unwrap();
    cache.get(url).and_then(|image| image.to_pixbuf().ok())
}

async fn fetch_and_decode(url: String) -> Option<CachedImage> {
    let response = reqwest::get(&url).await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    let bytes = response.bytes().await.ok()?;
    decode_rgba(&bytes).ok()
}

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
        if let Some(pixbuf) = cached_image(url) {
            let texture = gdk4::Texture::for_pixbuf(&pixbuf);
            self.picture.set_paintable(Some(&texture));
            return;
        }

        let url_owned = url.to_string();
        let (tx, rx) = tokio::sync::oneshot::channel();
        IO_RUNTIME.handle().spawn(async move {
            let decoded = fetch_and_decode(url_owned).await;
            let _ = tx.send(decoded);
        });

        if let Some(image) = rx.await.ok().flatten() {
            if let Ok(pixbuf) = image.to_pixbuf() {
                IMAGE_CACHE
                    .lock()
                    .unwrap()
                    .put(url.to_string(), image.clone());
                let texture = gdk4::Texture::for_pixbuf(&pixbuf);
                self.picture.set_paintable(Some(&texture));
                return;
            }
        }

        self.set_placeholder();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_evicts_oldest_beyond_capacity() {
        let mut cache: AvatarCache<u32> = AvatarCache::new(2);
        cache.put("a".into(), 1);
        cache.put("b".into(), 2);
        assert_eq!(cache.len(), 2);

        cache.put("c".into(), 3);
        assert_eq!(cache.len(), 2);
        assert!(cache.get("a").is_none(), "oldest entry must be evicted");
        assert_eq!(cache.get("b"), Some(&2));
        assert_eq!(cache.get("c"), Some(&3));

        cache.put("c".into(), 30);
        cache.put("d".into(), 4);
        assert_eq!(cache.get("c"), Some(&30), "refreshed key stays");
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn decode_rejects_invalid_bytes() {
        assert!(decode_rgba(b"not an image").is_err());
    }

    #[test]
    fn decode_round_trips_dimensions_and_pixels() {
        let img = image::RgbaImage::from_pixel(2, 3, image::Rgba([10u8, 20, 30, 255]));
        let mut png_bytes = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(
                &mut std::io::Cursor::new(&mut png_bytes),
                image::ImageFormat::Png,
            )
            .expect("png encodes");

        let decoded = decode_rgba(&png_bytes).expect("decodes");
        assert_eq!((decoded.width, decoded.height), (2, 3));
        assert_eq!(decoded.rgba.len(), 2 * 3 * 4);
    }
}
