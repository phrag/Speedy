package dev.speedy

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import android.content.pm.ServiceInfo
import android.graphics.Bitmap
import android.graphics.drawable.Icon
import android.net.TrafficStats
import android.os.IBinder
import android.os.SystemClock
import android.util.Log
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch

/**
 * Foreground service that drives the indicator: once a second it reads the
 * device's cumulative byte counters, hands them to Rust, and renders the
 * returned pixels into the ongoing notification's small icon — which is what
 * the status bar displays.
 */
class SpeedService : Service() {

    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
    private var loop: Job? = null

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onCreate() {
        super.onCreate()
        Log.i(TAG, "onCreate")
        createChannel()
        RustBridge.nativeInit(ICON_PX, ICON_PX, resources.displayMetrics.density)
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent?.action == ACTION_STOP) {
            Log.i(TAG, "stop requested")
            stopSelf()
            return START_NOT_STICKY
        }
        return try {
            // Must call startForeground promptly; seed with a priming tick.
            val (icon, label) = tickOnce()
            startForeground(
                NOTIF_ID,
                buildNotification(icon, label),
                ServiceInfo.FOREGROUND_SERVICE_TYPE_SPECIAL_USE,
            )
            Log.i(
                TAG,
                "startForeground OK; notificationsEnabled=" +
                    "${notificationManager().areNotificationsEnabled()}",
            )
            startLoop()
            START_STICKY
        } catch (t: Throwable) {
            Log.e(TAG, "failed to start foreground service", t)
            stopSelf()
            START_NOT_STICKY
        }
    }

    override fun onDestroy() {
        scope.cancel()
        super.onDestroy()
    }

    private fun startLoop() {
        if (loop?.isActive == true) return
        loop = scope.launch {
            while (isActive) {
                delay(INTERVAL_MS)
                val (icon, label) = tickOnce()
                notificationManager().notify(NOTIF_ID, buildNotification(icon, label))
            }
        }
    }

    /** Sample counters, run them through Rust, and turn the result into a Bitmap. */
    private fun tickOnce(): Pair<Bitmap, String> {
        val rx = TrafficStats.getTotalRxBytes()
        val tx = TrafficStats.getTotalTxBytes()
        val ns = SystemClock.elapsedRealtimeNanos()

        val pixels = RustBridge.nativeTick(rx, tx, ns)
        JniContract.requireValidIcon(pixels, ICON_PX)

        val bmp = Bitmap.createBitmap(ICON_PX, ICON_PX, Bitmap.Config.ARGB_8888)
        bmp.setPixels(pixels, 0, ICON_PX, 0, 0, ICON_PX, ICON_PX)
        return bmp to RustBridge.nativeLabel()
    }

    private fun buildNotification(icon: Bitmap, label: String): Notification {
        val open = PendingIntent.getActivity(
            this,
            0,
            Intent(this, MainActivity::class.java),
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
        )
        return Notification.Builder(this, CHANNEL_ID)
            .setSmallIcon(Icon.createWithBitmap(icon))
            .setContentTitle(getString(R.string.notif_title))
            .setContentText(label)
            .setOngoing(true)
            .setOnlyAlertOnce(true)
            .setCategory(Notification.CATEGORY_SERVICE)
            .setVisibility(Notification.VISIBILITY_PUBLIC)
            // Show the status-bar icon immediately rather than letting the
            // system defer the foreground-service notification by up to 10s.
            .setForegroundServiceBehavior(Notification.FOREGROUND_SERVICE_IMMEDIATE)
            .setContentIntent(open)
            .build()
    }

    private fun createChannel() {
        val channel = NotificationChannel(
            CHANNEL_ID,
            getString(R.string.channel_name),
            NotificationManager.IMPORTANCE_LOW, // silent, no heads-up
        ).apply {
            description = getString(R.string.channel_desc)
            setShowBadge(false)
        }
        notificationManager().createNotificationChannel(channel)
    }

    private fun notificationManager() =
        getSystemService(NotificationManager::class.java)

    companion object {
        private const val TAG = "Speedy"
        const val ACTION_STOP = "dev.speedy.action.STOP"
        private const val CHANNEL_ID = "throughput"
        private const val NOTIF_ID = 1
        /** Square ARGB icon edge; Android downscales it for the status bar. */
        private const val ICON_PX = 72
        private const val INTERVAL_MS = 1000L
    }
}
