package io.github.lumpsoid.coffeetimer

import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import android.os.PowerManager
import androidx.core.app.NotificationChannelCompat
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.app.ServiceCompat
import androidx.core.content.ContextCompat

/**
 * Keeps a running brew alive while the app is in the background: a
 * foreground service showing the brew notification, holding a partial wake
 * lock so the Rust timer keeps ticking with the screen off.
 *
 * Rust decides what it shows and when; this only performs the
 * `ShowBrewNotification` / `HideBrewNotification` effects.
 */
class BrewService : Service() {
    private var wakeLock: PowerManager.WakeLock? = null

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onCreate() {
        super.onCreate()
        wakeLock = getSystemService(PowerManager::class.java)
            .newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "coffee_timer:brew")
            .apply { acquire() }
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val title = intent?.getStringExtra(EXTRA_TITLE)
        if (title == null) {
            stopSelf()
            return START_NOT_STICKY
        }
        ServiceCompat.startForeground(
            this,
            NOTIFICATION_ID,
            notification(this, title, intent.getStringExtra(EXTRA_NEXT)),
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
                ServiceInfo.FOREGROUND_SERVICE_TYPE_SPECIAL_USE
            } else {
                0
            },
        )
        return START_NOT_STICKY
    }

    /** The user swiped the app away: the brew goes with it. */
    override fun onTaskRemoved(rootIntent: Intent?) {
        running = false
        stopSelf()
    }

    override fun onDestroy() {
        wakeLock?.takeIf { it.isHeld }?.release()
        ServiceCompat.stopForeground(this, ServiceCompat.STOP_FOREGROUND_REMOVE)
        super.onDestroy()
    }

    companion object {
        private const val CHANNEL_ID = "brew_timer"
        private const val NOTIFICATION_ID = 1
        private const val EXTRA_TITLE = "title"
        private const val EXTRA_NEXT = "next"

        /** Set by [show], cleared by [hide]: whether updates go to the running service. */
        @Volatile
        private var running = false

        /**
         * Starts the service with the notification, or refreshes the
         * notification of the one already running. The first call must come
         * while the app is in the foreground (the user tapped Start).
         */
        fun show(context: Context, title: String, next: String?) {
            createChannel(context)
            if (running) {
                val notifications = NotificationManagerCompat.from(context)
                // Denied: the service still runs; the notification is just not shown.
                if (notifications.areNotificationsEnabled()) {
                    notifications.notify(NOTIFICATION_ID, notification(context, title, next))
                }
                return
            }
            running = true
            val intent = Intent(context, BrewService::class.java)
                .putExtra(EXTRA_TITLE, title)
                .putExtra(EXTRA_NEXT, next)
            ContextCompat.startForegroundService(context, intent)
        }

        fun hide(context: Context) {
            running = false
            context.stopService(Intent(context, BrewService::class.java))
        }

        private fun createChannel(context: Context) {
            val channel = NotificationChannelCompat
                .Builder(CHANNEL_ID, NotificationManagerCompat.IMPORTANCE_LOW)
                .setName("Brew timer")
                .setShowBadge(false)
                .build()
            NotificationManagerCompat.from(context).createNotificationChannel(channel)
        }

        private fun notification(context: Context, title: String, next: String?) =
            NotificationCompat.Builder(context, CHANNEL_ID)
                .setSmallIcon(R.mipmap.ic_launcher)
                .setContentTitle(title)
                .setContentText(next)
                .setContentIntent(openApp(context))
                .setCategory(NotificationCompat.CATEGORY_STOPWATCH)
                .setOngoing(true)
                .setSilent(true)
                .setOnlyAlertOnce(true)
                .setForegroundServiceBehavior(NotificationCompat.FOREGROUND_SERVICE_IMMEDIATE)
                .build()

        /** Brings the existing task to the front, still on the brew page. */
        private fun openApp(context: Context): PendingIntent {
            val intent = Intent(context, MainActivity::class.java)
                .setAction(Intent.ACTION_MAIN)
                .addCategory(Intent.CATEGORY_LAUNCHER)
            return PendingIntent.getActivity(context, 0, intent, PendingIntent.FLAG_IMMUTABLE)
        }
    }
}
