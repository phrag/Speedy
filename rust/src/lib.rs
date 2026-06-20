//! `speedy_core` — all of Speedy's non-Android logic.
//!
//! The Kotlin shell only reads `TrafficStats` counters and renders a
//! notification; everything else (rate math, smoothing, formatting, and the
//! status-bar icon pixels) lives here and is exercised by host-side tests.

pub mod font;
pub mod format;
pub mod icon;
pub mod speed;

use speed::SpeedState;

/// Default icon edge in pixels if the host never calls `init`. Android downsizes
/// the bitmap for the status bar, so a generous square keeps glyphs crisp.
const DEFAULT_ICON: usize = 72;

/// The full application state shared across JNI calls.
pub struct App {
    speed: SpeedState,
    icon_w: usize,
    icon_h: usize,
    last_label: String,
    last_icon: Vec<u32>,
}

impl App {
    pub fn new(icon_w: usize, icon_h: usize) -> Self {
        App {
            speed: SpeedState::new(),
            icon_w,
            icon_h,
            last_label: "↓ 0  ↑ 0".to_string(),
            last_icon: vec![0u32; icon_w * icon_h],
        }
    }

    /// Process one counter sample. Returns the freshly rendered ARGB_8888 icon.
    pub fn tick(&mut self, rx: i64, tx: i64, ns: i64) -> &[u32] {
        let (down, up) = self.speed.tick(rx, tx, ns);
        let down_s = format::fmt_rate(down);
        let up_s = format::fmt_rate(up);
        self.last_label = format!("↓ {down_s}/s   ↑ {up_s}/s");
        self.last_icon = icon::render(self.icon_w, self.icon_h, &down_s, &up_s);
        &self.last_icon
    }

    /// Human-readable text for the expanded notification body.
    pub fn label(&self) -> &str {
        &self.last_label
    }
}

// ---------------------------------------------------------------------------
// JNI bridge. Compiled for the host too (the `jni` crate is host-portable), so
// `cargo test` still builds this module even though these functions only run on
// Android. The pure logic above is what the tests target.
// ---------------------------------------------------------------------------

mod jni_bridge {
    use super::{App, DEFAULT_ICON};
    use jni::objects::JClass;
    use jni::sys::{jfloat, jint, jintArray, jlong, jstring};
    use jni::JNIEnv;
    use std::sync::{Mutex, OnceLock};

    fn state() -> &'static Mutex<App> {
        static STATE: OnceLock<Mutex<App>> = OnceLock::new();
        STATE.get_or_init(|| Mutex::new(App::new(DEFAULT_ICON, DEFAULT_ICON)))
    }

    /// `RustBridge.nativeInit(width, height, density)` — set the icon size.
    #[no_mangle]
    pub extern "system" fn Java_dev_speedy_RustBridge_nativeInit(
        _env: JNIEnv,
        _class: JClass,
        width: jint,
        height: jint,
        _density: jfloat,
    ) {
        let w = width.max(1) as usize;
        let h = height.max(1) as usize;
        *state().lock().unwrap() = App::new(w, h);
    }

    /// `RustBridge.nativeTick(rx, tx, elapsedNanos)` — returns ARGB_8888 icon pixels.
    #[no_mangle]
    pub extern "system" fn Java_dev_speedy_RustBridge_nativeTick<'local>(
        env: JNIEnv<'local>,
        _class: JClass<'local>,
        rx: jlong,
        tx: jlong,
        elapsed_nanos: jlong,
    ) -> jintArray {
        let mut app = state().lock().unwrap();
        let pixels = app.tick(rx, tx, elapsed_nanos);
        // ARGB u32 -> jint (i32) is a lossless bit reinterpretation.
        let ints: Vec<jint> = pixels.iter().map(|&p| p as jint).collect();
        let arr = env
            .new_int_array(ints.len() as jint)
            .expect("alloc jintArray");
        env.set_int_array_region(&arr, 0, &ints)
            .expect("fill jintArray");
        arr.into_raw()
    }

    /// `RustBridge.nativeLabel()` — text for the expanded notification body.
    #[no_mangle]
    pub extern "system" fn Java_dev_speedy_RustBridge_nativeLabel<'local>(
        env: JNIEnv<'local>,
        _class: JClass<'local>,
    ) -> jstring {
        let app = state().lock().unwrap();
        env.new_string(app.label())
            .expect("alloc jstring")
            .into_raw()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_produces_icon_and_label() {
        let mut app = App::new(48, 48);
        app.tick(0, 0, 0); // seed
        let icon = app.tick(1_048_576, 0, 1_000_000_000); // ~1 MB/s down
        assert_eq!(icon.len(), 48 * 48);
        assert!(app.label().contains('↓'));
        assert!(app.label().contains('↑'));
        assert!(app.label().contains("/s"));
    }

    #[test]
    fn idle_label_reads_zero() {
        let mut app = App::new(32, 32);
        app.tick(0, 0, 0);
        let _ = app.tick(0, 0, 1_000_000_000);
        assert!(app.label().contains("0/s"));
    }
}
