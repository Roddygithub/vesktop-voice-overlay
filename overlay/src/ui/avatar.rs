use futures_util::StreamExt;
use gdk_pixbuf::Pixbuf;
use gtk4::prelude::*;
use gtk4::{Align, Image};
use once_cell::sync::Lazy;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::io::Cursor;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// One shared worker pool for all avatar downloads: a single runtime per
/// process instead of one thread + one Tokio runtime per request.
static IO_RUNTIME: Lazy<Option<tokio::runtime::Runtime>> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .ok()
});

static HTTP_CLIENT: Lazy<Option<reqwest::Client>> = Lazy::new(|| {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(10))
        .https_only(true)
        .redirect(reqwest::redirect::Policy::none())
        .user_agent(concat!("vesktop-voice-overlay/", env!("CARGO_PKG_VERSION")))
        .build()
        .ok()
});

static FETCH_SLOTS: Lazy<Arc<tokio::sync::Semaphore>> =
    Lazy::new(|| Arc::new(tokio::sync::Semaphore::new(FETCH_CONCURRENCY)));

const FETCH_CONCURRENCY: usize = 8;
const AVATAR_CACHE_CAPACITY: usize = 128;
const MAX_AVATAR_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
const MAX_AVATAR_DIMENSION: u32 = 256;
const MAX_AVATAR_DECODE_BYTES: u64 = 8 * 1024 * 1024;

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
    let mut reader = image::ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_AVATAR_DIMENSION);
    limits.max_image_height = Some(MAX_AVATAR_DIMENSION);
    limits.max_alloc = Some(MAX_AVATAR_DECODE_BYTES);
    reader.limits(limits);
    let img = reader.decode()?;
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
    if !is_allowed_avatar_url(&url) {
        return None;
    }

    let _permit = wait_for_fetch_slot(FETCH_SLOTS.clone()).await?;
    let response = HTTP_CLIENT.as_ref()?.get(&url).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }

    if response
        .content_length()
        .is_some_and(|length| length > MAX_AVATAR_RESPONSE_BYTES as u64)
    {
        return None;
    }

    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.ok()?;
        if bytes.len().saturating_add(chunk.len()) > MAX_AVATAR_RESPONSE_BYTES {
            return None;
        }
        bytes.extend_from_slice(&chunk);
    }
    decode_rgba(&bytes).ok()
}

async fn wait_for_fetch_slot(
    slots: Arc<tokio::sync::Semaphore>,
) -> Option<tokio::sync::OwnedSemaphorePermit> {
    slots.acquire_owned().await.ok()
}

fn is_allowed_avatar_url(url: &str) -> bool {
    reqwest::Url::parse(url).is_ok_and(|url| {
        url.scheme() == "https"
            && url.username().is_empty()
            && url.password().is_none()
            && url.host_str() == Some("cdn.discordapp.com")
    })
}

pub struct AvatarWidget {
    image: Image,
    state: RefCell<AvatarState>,
    size: Cell<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LoadStatus {
    Placeholder,
    Loading,
    Loaded,
    Failed,
}

#[derive(Debug)]
struct AvatarState {
    url: String,
    generation: u64,
    status: LoadStatus,
}

impl AvatarState {
    fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            generation: u64::from(!url.is_empty()),
            status: if url.is_empty() {
                LoadStatus::Placeholder
            } else {
                LoadStatus::Loading
            },
        }
    }

    fn loading_generation(&self) -> Option<u64> {
        (self.status == LoadStatus::Loading).then_some(self.generation)
    }

    fn request(&mut self, url: &str) -> Option<u64> {
        if self.url == url {
            if url.is_empty() || matches!(self.status, LoadStatus::Loading | LoadStatus::Loaded) {
                return None;
            }
        } else {
            self.url.clear();
            self.url.push_str(url);
        }

        self.generation = self.generation.wrapping_add(1);
        self.status = if url.is_empty() {
            LoadStatus::Placeholder
        } else {
            LoadStatus::Loading
        };
        self.loading_generation()
    }

    fn finish(&mut self, generation: u64, loaded: bool) -> bool {
        if self.generation != generation || self.status != LoadStatus::Loading {
            return false;
        }
        self.status = if loaded {
            LoadStatus::Loaded
        } else {
            LoadStatus::Failed
        };
        true
    }
}

impl AvatarWidget {
    pub fn new(url: &str, size: i32) -> Rc<Self> {
        let image = Image::new();
        // GtkImage measures exactly pixel_size regardless of the paintable's
        // intrinsic size, so the configured avatar size is authoritative.
        // GtkPicture would size itself from the downloaded texture instead.
        image.set_pixel_size(size);
        image.set_halign(Align::Start);
        image.set_valign(Align::Center);
        // GtkImage does not clip its paintable to the CSS border-radius on
        // its own; force the circular avatar mask.
        image.set_overflow(gtk4::Overflow::Hidden);
        image.add_css_class("participant-avatar");

        let state = AvatarState::new(url);
        let generation = state.loading_generation();
        let this = Rc::new(Self {
            image,
            state: RefCell::new(state),
            size: Cell::new(size),
        });
        this.set_placeholder();

        if let Some(generation) = generation {
            let this_clone = this.clone();
            let url = url.to_string();
            glib::spawn_future_local(async move {
                this_clone.load_avatar(&url, generation).await;
            });
        }

        this
    }

    pub fn widget(&self) -> &Image {
        &self.image
    }

    pub fn update_url(self: &Rc<Self>, url: &str) {
        let url_changed = self.state.borrow().url != url;
        let generation = self.state.borrow_mut().request(url);

        if url_changed {
            self.set_placeholder();
        }
        if let Some(generation) = generation {
            let this = self.clone();
            let url = url.to_string();
            glib::spawn_future_local(async move {
                this.load_avatar(&url, generation).await;
            });
        }
    }

    async fn load_avatar(&self, url: &str, generation: u64) {
        if let Some(pixbuf) = cached_image(url) {
            if self.state.borrow_mut().finish(generation, true) {
                let texture = gdk4::Texture::for_pixbuf(&pixbuf);
                self.image.set_paintable(Some(&texture));
            }
            return;
        }

        let Some(runtime) = IO_RUNTIME.as_ref() else {
            if self.state.borrow_mut().finish(generation, false) {
                self.set_placeholder();
            }
            return;
        };
        let url_owned = url.to_string();
        let (tx, rx) = tokio::sync::oneshot::channel();
        runtime.handle().spawn(async move {
            let decoded = fetch_and_decode(url_owned).await;
            let _ = tx.send(decoded);
        });

        if let Some(image) = rx.await.ok().flatten() {
            if let Ok(pixbuf) = image.to_pixbuf() {
                IMAGE_CACHE
                    .lock()
                    .unwrap()
                    .put(url.to_string(), image.clone());
                if self.state.borrow_mut().finish(generation, true) {
                    let texture = gdk4::Texture::for_pixbuf(&pixbuf);
                    self.image.set_paintable(Some(&texture));
                }
                return;
            }
        }

        if self.state.borrow_mut().finish(generation, false) {
            self.set_placeholder();
        }
    }

    fn set_placeholder(&self) {
        if let Some(texture) = self.create_placeholder_texture() {
            self.image.set_paintable(Some(&texture));
        } else {
            self.image.clear();
        }
    }

    fn create_placeholder_texture(&self) -> Option<gdk4::Texture> {
        let size = self.size.get();
        let pixbuf = Pixbuf::new(gdk_pixbuf::Colorspace::Rgb, true, 8, size, size)?;
        pixbuf.fill(0x4d4d4dff);

        Some(gdk4::Texture::for_pixbuf(&pixbuf))
    }

    pub fn set_size(&self, size: i32) {
        self.size.set(size);
        self.image.set_pixel_size(size);
        if self.image.paintable().is_none() {
            self.set_placeholder();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_avatar_stays_placeholder_without_a_request() {
        let mut state = AvatarState::new("");
        assert_eq!(state.status, LoadStatus::Placeholder);
        assert_eq!(state.loading_generation(), None);
        assert_eq!(state.request(""), None);
    }

    #[test]
    fn same_url_is_suppressed_while_loading_and_after_success() {
        let mut state = AvatarState::new("https://cdn.discordapp.com/avatars/1/a.png");
        let generation = state.loading_generation().expect("initial request");
        assert_eq!(state.request(&state.url.clone()), None);

        assert!(state.finish(generation, true));
        assert_eq!(state.status, LoadStatus::Loaded);
        assert_eq!(state.request(&state.url.clone()), None);
    }

    #[test]
    fn url_replacement_invalidates_every_older_completion() {
        let mut state = AvatarState::new("A");
        let first_a = state.loading_generation().expect("first A request");
        let b = state.request("B").expect("B request");
        let second_a = state.request("A").expect("second A request");

        assert!(!state.finish(first_a, true), "stale A must not win");
        assert!(!state.finish(b, true), "stale B must not win");
        assert_eq!(state.status, LoadStatus::Loading);
        assert!(state.finish(second_a, true));
        assert_eq!(state.status, LoadStatus::Loaded);
    }

    #[test]
    fn clearing_url_invalidates_an_in_flight_load() {
        let mut state = AvatarState::new("A");
        let a = state.loading_generation().expect("A request");

        assert_eq!(state.request(""), None);
        assert_eq!(state.status, LoadStatus::Placeholder);
        assert!(!state.finish(a, true));
        assert_eq!(state.status, LoadStatus::Placeholder);
    }

    #[test]
    fn failed_current_load_can_retry_without_duplicate_requests() {
        let mut state = AvatarState::new("A");
        let first = state.loading_generation().expect("first request");
        assert_eq!(state.request("A"), None, "in-flight load is deduplicated");

        assert!(state.finish(first, false));
        assert_eq!(state.status, LoadStatus::Failed);
        let retry = state.request("A").expect("failed load is retryable");
        assert_ne!(retry, first);
        assert_eq!(state.request("A"), None, "retry is also deduplicated");
        assert!(state.finish(retry, true));
        assert_eq!(state.status, LoadStatus::Loaded);
    }

    #[test]
    fn fetches_wait_for_a_slot_instead_of_being_dropped() {
        tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .expect("test runtime builds")
            .block_on(async {
                let slots = Arc::new(tokio::sync::Semaphore::new(FETCH_CONCURRENCY));
                let mut permits: Vec<_> = (0..FETCH_CONCURRENCY)
                    .map(|_| slots.clone().try_acquire_owned().expect("slot available"))
                    .collect();
                let mut waiting = Box::pin(wait_for_fetch_slot(slots));

                assert!(
                    tokio::time::timeout(Duration::from_millis(1), waiting.as_mut())
                        .await
                        .is_err(),
                    "the ninth fetch must wait"
                );
                drop(permits.pop());
                assert!(tokio::time::timeout(Duration::from_secs(1), waiting)
                    .await
                    .expect("waiting fetch resumes")
                    .is_some());
            });
    }

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

    #[test]
    fn decode_rejects_images_over_dimension_limit() {
        let img = image::RgbaImage::from_pixel(
            MAX_AVATAR_DIMENSION + 1,
            1,
            image::Rgba([10u8, 20, 30, 255]),
        );
        let mut png_bytes = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(
                &mut std::io::Cursor::new(&mut png_bytes),
                image::ImageFormat::Png,
            )
            .expect("png encodes");

        assert!(decode_rgba(&png_bytes).is_err());
    }

    #[test]
    fn avatar_urls_are_restricted_to_discord_https_cdn() {
        assert!(is_allowed_avatar_url(
            "https://cdn.discordapp.com/avatars/1/hash.png?size=128"
        ));
        assert!(!is_allowed_avatar_url(
            "http://cdn.discordapp.com/avatars/1/hash.png"
        ));
        assert!(!is_allowed_avatar_url("https://example.com/avatar.png"));
        assert!(!is_allowed_avatar_url("file:///etc/passwd"));
        assert!(!is_allowed_avatar_url("not a URL"));
    }
}
