package com.codeorbit.keepalive

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat

class KeepAliveService : Service() {
  override fun onBind(intent: Intent?): IBinder? = null

  override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
    try {
      when (intent?.action) {
        ACTION_STOP -> {
          if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
            stopForeground(STOP_FOREGROUND_REMOVE)
          } else {
            @Suppress("DEPRECATION")
            stopForeground(true)
          }
          stopSelf()
          return START_NOT_STICKY
        }
        else -> {
          val targetName = intent?.getStringExtra(EXTRA_TARGET_NAME) ?: lastTargetName
          val pendingCount =
            intent?.getIntExtra(EXTRA_PENDING_COUNT, lastPendingCount) ?: lastPendingCount
          lastTargetName = targetName
          lastPendingCount = pendingCount
          ensureChannel(this)
          val notification = buildNotification(this, targetName, pendingCount)
          // Prefer platform API (works without newer ServiceCompat dependency).
          if (Build.VERSION.SDK_INT >= 34) {
            startForeground(
              NOTIFICATION_ID,
              notification,
              ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC,
            )
          } else {
            startForeground(NOTIFICATION_ID, notification)
          }
        }
      }
    } catch (_: Throwable) {
      stopSelf()
      return START_NOT_STICKY
    }
    return START_STICKY
  }

  companion object {
    const val CHANNEL_ID = "keep_alive"
    const val NOTIFICATION_ID = 42145
    const val ACTION_START = "com.codeorbit.keepalive.START"
    const val ACTION_UPDATE = "com.codeorbit.keepalive.UPDATE"
    const val ACTION_STOP = "com.codeorbit.keepalive.STOP"
    const val EXTRA_TARGET_NAME = "targetName"
    const val EXTRA_PENDING_COUNT = "pendingCount"

    @Volatile private var lastTargetName: String = "Runtime"
    @Volatile private var lastPendingCount: Int = 0

    fun ensureChannel(context: Context) {
      if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) return
      val nm = context.getSystemService(NotificationManager::class.java) ?: return
      if (nm.getNotificationChannel(CHANNEL_ID) != null) return
      nm.createNotificationChannel(
        NotificationChannel(CHANNEL_ID, "连接状态", NotificationManager.IMPORTANCE_LOW).apply {
          description = "已连接 Runtime 时保持前台服务"
          setShowBadge(false)
        },
      )
    }

    fun buildNotification(context: Context, targetName: String, pendingCount: Int): Notification {
      val launch =
        context.packageManager.getLaunchIntentForPackage(context.packageName)
          ?: Intent().apply { setPackage(context.packageName) }
      launch.addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP)
      val flags =
        PendingIntent.FLAG_UPDATE_CURRENT or
          if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) PendingIntent.FLAG_IMMUTABLE else 0
      val contentIntent = PendingIntent.getActivity(context, 0, launch, flags)
      val text =
        if (pendingCount > 0) "$targetName · 待处理 $pendingCount 条" else targetName
      return NotificationCompat.Builder(context, CHANNEL_ID)
        .setContentTitle("CodeOrbit · 已连接")
        .setContentText(text)
        .setSmallIcon(R.drawable.ic_stat_notify)
        .setOngoing(true)
        .setOnlyAlertOnce(true)
        .setContentIntent(contentIntent)
        .setCategory(NotificationCompat.CATEGORY_SERVICE)
        .setPriority(NotificationCompat.PRIORITY_LOW)
        .build()
    }

    fun updateNotification(context: Context, targetName: String, pendingCount: Int) {
      lastTargetName = targetName
      lastPendingCount = pendingCount
      ensureChannel(context)
      val nm = context.getSystemService(NotificationManager::class.java) ?: return
      nm.notify(NOTIFICATION_ID, buildNotification(context, targetName, pendingCount))
    }
  }
}

object PendingAlert {
  const val CHANNEL_ID = "pending_alert"
  const val NOTIFICATION_ID = 42147

  private fun ensureChannel(context: Context) {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) return
    val nm = context.getSystemService(NotificationManager::class.java) ?: return
    if (nm.getNotificationChannel(CHANNEL_ID) != null) return
    nm.createNotificationChannel(
      NotificationChannel(CHANNEL_ID, "待处理提醒", NotificationManager.IMPORTANCE_HIGH).apply {
        description = "新的权限审批或 AI 提问"
        setShowBadge(true)
      },
    )
  }

  fun notify(context: Context, title: String, body: String, count: Int) {
    try {
      ensureChannel(context)
      val launch =
        context.packageManager.getLaunchIntentForPackage(context.packageName)
          ?: Intent().apply { setPackage(context.packageName) }
      launch.addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP)
      launch.putExtra("openTab", "pending")
      val flags =
        PendingIntent.FLAG_UPDATE_CURRENT or
          if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) PendingIntent.FLAG_IMMUTABLE else 0
      val contentIntent = PendingIntent.getActivity(context, 1, launch, flags)
      val notification =
        NotificationCompat.Builder(context, CHANNEL_ID)
          .setContentTitle(title.ifBlank { "CodeOrbit · 待处理" })
          .setContentText(body.ifBlank { "有 $count 条待处理" })
          .setSmallIcon(R.drawable.ic_stat_notify)
          .setOnlyAlertOnce(true)
          .setAutoCancel(true)
          .setContentIntent(contentIntent)
          .setCategory(NotificationCompat.CATEGORY_MESSAGE)
          .setPriority(NotificationCompat.PRIORITY_HIGH)
          .setNumber(count.coerceAtLeast(1))
          .build()
      context.getSystemService(NotificationManager::class.java)
        ?.notify(NOTIFICATION_ID, notification)
    } catch (_: Throwable) {
    }
  }

  fun clear(context: Context) {
    try {
      context.getSystemService(NotificationManager::class.java)?.cancel(NOTIFICATION_ID)
    } catch (_: Throwable) {
    }
  }
}
