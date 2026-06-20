package dev.speedy

import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Test

/** JVM unit tests for the native-bridge contract (no Android runtime needed). */
class JniContractTest {

    @Test
    fun expectedPixelCountIsSquare() {
        assertEquals(72 * 72, JniContract.expectedPixelCount(72))
        assertEquals(0, JniContract.expectedPixelCount(0))
    }

    @Test
    fun acceptsCorrectlySizedIcon() {
        JniContract.requireValidIcon(IntArray(48 * 48), 48) // must not throw
    }

    @Test
    fun rejectsWrongSizedIcon() {
        assertThrows(IllegalArgumentException::class.java) {
            JniContract.requireValidIcon(IntArray(10), 48)
        }
    }

    @Test
    fun rejectsNonPositiveSize() {
        assertThrows(IllegalArgumentException::class.java) {
            JniContract.requireValidIcon(IntArray(0), 0)
        }
    }
}
