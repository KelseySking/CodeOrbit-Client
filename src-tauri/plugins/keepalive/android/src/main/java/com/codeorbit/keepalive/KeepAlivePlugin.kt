package com.codeorbit.keepalive

import android.Manifest
import android.app.Activity
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin

@InvokeArg
class KeepAliveArgs {
  var targetName: String = "Runtime"
  var pendingCount: Int = 0
}

@InvokeArg
class PendingAlertArgs {
  var title: String = "CodeOrbit · 待处理"
  var body: String = ""
  var count: Int = 1
}

@TauriPlugin
class KeepAlivePlugin(private val activity: Activity) : Plugin(activity) {
  init {
    // Best-effort permission; never crash.
    try {
      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
        val granted =
          ContextCompat.checkSelfPermission(activity, Manifest.permission.POST_NOTIFICATIONS) ==
            PackageManager.PERMISSION_GRANTED
        if (!granted) {
          ActivityCompat.requestPermissions(
            activity,
            arrayOf(Manifest.permission.POST_NOTIFICATIONS),
            42146,
          )
        }
      }
    } catch (_: Throwable) {
    }
  }

  @Command
  fun keepAliveStart(invoke: Invoke) {
    try {
      val args = invoke.parseArgs(KeepAliveArgs::class.java)
      val ctx = activity.applicationContext
      KeepAliveService.ensureChannel(ctx)
      val intent =
        Intent(ctx, KeepAliveService::class.java).apply {
          action = KeepAliveService.ACTION_START
          putExtra(KeepAliveService.EXTRA_TARGET_NAME, args.targetName)
          putExtra(KeepAliveService.EXTRA_PENDING_COUNT, args.pendingCount)
        }
      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
        ctx.startForegroundService(intent)
      } else {
        ctx.startService(intent)
      }
      invoke.resolve()
    } catch (ex: Exception) {
      invoke.reject(ex.message)
    }
  }

  @Command
  fun keepAliveUpdate(invoke: Invoke) {
    try {
      val args = invoke.parseArgs(KeepAliveArgs::class.java)
      val ctx = activity.applicationContext
      KeepAliveService.updateNotification(ctx, args.targetName, args.pendingCount)
      val intent =
        Intent(ctx, KeepAliveService::class.java).apply {
          action = KeepAliveService.ACTION_UPDATE
          putExtra(KeepAliveService.EXTRA_TARGET_NAME, args.targetName)
          putExtra(KeepAliveService.EXTRA_PENDING_COUNT, args.pendingCount)
        }
      try {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
          ctx.startForegroundService(intent)
        } else {
          ctx.startService(intent)
        }
      } catch (_: Exception) {
      }
      invoke.resolve()
    } catch (ex: Exception) {
      invoke.reject(ex.message)
    }
  }

  @Command
  fun keepAliveStop(invoke: Invoke) {
    try {
      val ctx = activity.applicationContext
      val intent =
        Intent(ctx, KeepAliveService::class.java).apply {
          action = KeepAliveService.ACTION_STOP
        }
      try {
        ctx.startService(intent)
      } catch (_: Exception) {
      }
      try {
        ctx.getSystemService(android.app.NotificationManager::class.java)
          ?.cancel(KeepAliveService.NOTIFICATION_ID)
      } catch (_: Exception) {
      }
      PendingAlert.clear(ctx)
      invoke.resolve()
    } catch (ex: Exception) {
      invoke.reject(ex.message)
    }
  }

  @Command
  fun pendingAlertNotify(invoke: Invoke) {
    try {
      val args = invoke.parseArgs(PendingAlertArgs::class.java)
      PendingAlert.notify(
        activity.applicationContext,
        args.title,
        args.body,
        args.count,
      )
      invoke.resolve()
    } catch (ex: Exception) {
      invoke.reject(ex.message)
    }
  }

  @Command
  fun pendingAlertClear(invoke: Invoke) {
    try {
      PendingAlert.clear(activity.applicationContext)
      invoke.resolve()
    } catch (ex: Exception) {
      invoke.reject(ex.message)
    }
  }
}
