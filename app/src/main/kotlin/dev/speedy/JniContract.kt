package dev.speedy

/**
 * Pure-Kotlin guards for the contract between [RustBridge] and the notification
 * code, kept free of Android types so they run under plain JVM unit tests.
 */
object JniContract {
    /** Pixel count a square icon of `size` must produce. */
    fun expectedPixelCount(size: Int): Int = size * size

    /** Validate that native returned the exact number of pixels we will blit. */
    fun requireValidIcon(pixels: IntArray, size: Int) {
        require(size > 0) { "icon size must be positive, was $size" }
        require(pixels.size == expectedPixelCount(size)) {
            "native returned ${pixels.size} pixels, expected ${expectedPixelCount(size)} for ${size}x$size"
        }
    }
}
