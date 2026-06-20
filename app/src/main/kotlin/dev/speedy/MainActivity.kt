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
import android.widget.Toast

/**
 * Minimal control panel: request the notification permission, start/stop the
 * indicator, and offer a shortcut to exempt the app from battery optimization
 * (GrapheneOS is aggressive about killing background work).
 */
class MainActivity : Activity() {

    private var pendingStart = false

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

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

        root.addView(button(R.string.start) { startIndicator() })
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

    /**
     * Start the indicator, but only once POST_NOTIFICATIONS is granted — without
     * it the foreground-service notification is suppressed and nothing shows in
     * the status bar (the service would run invisibly).
     */
    private fun startIndicator() {
        if (hasNotificationPermission()) {
            setEnabled(true)
            startForegroundService(serviceIntent())
        } else {
            pendingStart = true
            requestPermissions(arrayOf(Manifest.permission.POST_NOTIFICATIONS), REQ_NOTIF)
        }
    }

    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray,
    ) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        if (requestCode != REQ_NOTIF || !pendingStart) return
        pendingStart = false
        if (grantResults.firstOrNull() == PackageManager.PERMISSION_GRANTED) {
            setEnabled(true)
            startForegroundService(serviceIntent())
        } else {
            Toast.makeText(this, R.string.need_notif_permission, Toast.LENGTH_LONG).show()
        }
    }

    private fun hasNotificationPermission(): Boolean =
        checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) ==
            PackageManager.PERMISSION_GRANTED

    companion object {
        const val PREFS = "speedy"
        const val KEY_ENABLED = "enabled"
        private const val REQ_NOTIF = 1
    }
}
