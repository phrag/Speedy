package dev.speedy

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent

/** Re-arms the indicator after reboot, but only if the user had it enabled. */
class BootReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action != Intent.ACTION_BOOT_COMPLETED) return
        val enabled = context
            .getSharedPreferences(MainActivity.PREFS, Context.MODE_PRIVATE)
            .getBoolean(MainActivity.KEY_ENABLED, false)
        if (enabled) {
            context.startForegroundService(Intent(context, SpeedService::class.java))
        }
    }
}
