# Speedy

A tiny Android app that shows **live download (↓) and upload (↑) network speed in the status bar**, built for a Pixel 10 running GrapheneOS. As much logic as possible lives in **Rust**; Kotlin is a thin shell.

## How it works

Android has no public API to write text into the status bar. Instead, Speedy runs a persistent **foreground-service notification whose small icon is a bitmap with the speed drawn into it** — the status bar renders that icon. Everything interesting happens in Rust:

```
Kotlin shell (app/)                         Rust core (rust/)  →  libspeedy_core.so
  SpeedService  ── TrafficStats rx/tx ──▶   speed.rs   delta → bytes/sec + EMA smoothing
  (1 Hz ticker)    + elapsedRealtimeNanos   format.rs  auto-scale → "1.2M" / "85K" / "0"
                ◀── ARGB icon pixels ─────   icon.rs    rasterize ↓/↑ via a 5×7 bitmap font
                ◀── label string ────────   lib.rs     JNI bridge (nativeInit/Tick/Label)
  → Bitmap → Notification small icon
```

The ticker stays in Kotlin because both the data source (`android.net.TrafficStats`) and the sink (`Notification`) are JVM-side; Rust owns 100% of the per-tick math, smoothing, formatting, and pixel drawing — and is fully host-testable with zero Android dependency.

Sample icons (`cargo run --example preview` in `rust/`):

| ↓1.2M / ↑340K | ↓85K / ↑2.0M | ↓950B / ↑0 | ↓12M / ↑8.5M |
|---|---|---|---|

## Layout

- `rust/` — `speedy_core` crate (`cdylib` + `rlib`). The hard logic + all unit/integration tests.
- `app/` — Android module: `SpeedService` (foreground service), `MainActivity` (controls + permission), `BootReceiver`, `RustBridge` (JNI).
- `scripts/build-rust.sh` — builds the arm64 `.so` into `app/src/main/jniLibs/` via `cargo-ndk`.
- `.github/workflows/ci.yml` — **builds and tests everything in CI** and uploads the debug APK.

## Build & test

Everything is built and tested in **GitHub Actions** — no local Android toolchain required. Push the branch and download the `speedy-debug-apk` artifact from the run.

Locally:

```bash
# Rust core (no Android needed)
cd rust && cargo test && cargo clippy --all-targets -- -D warnings

# Full APK (needs Rust android target, cargo-ndk, and an NDK on $ANDROID_NDK_HOME)
rustup target add aarch64-linux-android
cargo install cargo-ndk
./scripts/build-rust.sh
gradle assembleDebug          # or ./gradlew once a wrapper is generated
```

## Install on GrapheneOS

1. `adb install -r app/build/outputs/apk/debug/app-debug.apk` (or transfer the APK and tap it).
2. Open Speedy → grant the notification permission → **Start indicator**.
3. Optionally tap **Disable battery optimization** so GrapheneOS doesn't kill the service.

Run a speedtest or large download and watch the status-bar ↓/↑ track it; airplane mode drops it to `0`.

## Tech notes

- **Single ABI** (`arm64-v8a`) — the Pixel 10 is arm64-only.
- **`specialUse` foreground-service type** + `POST_NOTIFICATIONS` (Android 13+). The `specialUse` justification is declared but never Play-reviewed since this is sideloaded.
- Counters via `TrafficStats` (permissionless); `/proc/net/dev` is SELinux-restricted for apps.
- The status-bar font is a hand-rolled 5×7 bitmap font (`rust/src/font.rs`) — zero dependencies, deterministic, and trivially testable, which suits a tiny monochrome icon better than a full TTF rasterizer.
