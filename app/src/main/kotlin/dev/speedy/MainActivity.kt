package dev.speedy

import android.Manifest
import android.app.Activity
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Bundle
import android.provider.Settings
import android.view.Gravity
import android.view.ViewGroup
import android.widget.Button
import android.widget.LinearLayout
import android.widget.TextView

/**
 * Minimal control panel: request the notification permission, start/stop the
 * indicator, and offer a shortcut to exempt the app from battery optimization
 * (GrapheneOS is aggressive about killing background work).
 */
class MainActivity : Activity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        requestNotificationPermission()

        val pad = (16 * resources.displayMetrics.density).toInt()
        val root = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            gravity = Gravity.CENTER_HORIZONTAL
            setPadding(pad, pad * 2, pad, pad)
        }

        root.addView(TextView(this).apply {
            text = getString(R.string.app_name)
            textSize = 24f
        })
        root.addView(TextView(this).apply {
            text = getString(R.string.intro)
            setPadding(0, pad, 0, pad)
        })

        root.addView(button(R.string.start) {
            setEnabled(true)
            startForegroundService(serviceIntent())
        })
        root.addView(button(R.string.stop) {
            setEnabled(false)
            startService(serviceIntent().setAction(SpeedService.ACTION_STOP))
        })
        root.addView(button(R.string.battery) {
            startActivity(Intent(Settings.ACTION_IGNORE_BATTERY_OPTIMIZATION_SETTINGS))
        })

        setContentView(root)
    }

    private fun button(textRes: Int, onClick: () -> Unit): Button =
        Button(this).apply {
            setText(textRes)
            layoutParams = LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                ViewGroup.LayoutParams.WRAP_CONTENT,
            )
            setOnClickListener { onClick() }
        }

    /** Persist whether the user wants the indicator, so [BootReceiver] can re-arm it. */
    private fun setEnabled(enabled: Boolean) {
        getSharedPreferences(PREFS, MODE_PRIVATE)
            .edit()
            .putBoolean(KEY_ENABLED, enabled)
            .apply()
    }

    private fun serviceIntent() = Intent(this, SpeedService::class.java)

    private fun requestNotificationPermission() {
        if (checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS)
            != PackageManager.PERMISSION_GRANTED
        ) {
            requestPermissions(arrayOf(Manifest.permission.POST_NOTIFICATIONS), 1)
        }
    }

    companion object {
        const val PREFS = "speedy"
        const val KEY_ENABLED = "enabled"
    }
}
