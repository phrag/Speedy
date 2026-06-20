package dev.speedy

/**
 * JNI bridge to `libspeedy_core.so`. All throughput math, smoothing, formatting,
 * and the status-bar icon pixels are computed in Rust; this object is the only
 * seam between the Kotlin shell and that native library.
 */
object RustBridge {
    init {
        System.loadLibrary("speedy_core")
    }

    /** Configure the icon dimensions (square, ARGB_8888) and screen density. */
    external fun nativeInit(width: Int, height: Int, density: Float)

    /**
     * Feed the latest cumulative counters and a monotonic timestamp.
     * @return `width * height` ARGB_8888 pixels for the status-bar icon.
     */
    external fun nativeTick(rx: Long, tx: Long, elapsedNanos: Long): IntArray

    /** Human-readable text (e.g. "↓ 1.2M/s   ↑ 340K/s") for the notification body. */
    external fun nativeLabel(): String
}
