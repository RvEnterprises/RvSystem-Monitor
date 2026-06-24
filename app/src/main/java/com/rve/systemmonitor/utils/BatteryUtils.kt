package com.rve.systemmonitor.utils

import android.annotation.SuppressLint
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.os.BatteryManager
import android.os.Build
import android.util.Log
import com.rve.systemmonitor.R
import kotlin.math.abs
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.channels.onFailure
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow

object BatteryUtils {
    fun getBatteryIntent(context: Context): Intent? {
        return context.registerReceiver(null, IntentFilter(Intent.ACTION_BATTERY_CHANGED))
    }

    fun getBatteryFlow(context: Context): Flow<Intent> = callbackFlow {
        val receiver = object : BroadcastReceiver() {
            override fun onReceive(context: Context, intent: Intent) {
                trySend(intent).onFailure { error ->
                    Log.w("BatteryUtils", "Failed to send battery intent: ${error?.message}")
                }
            }
        }
        val sticky = context.registerReceiver(receiver, IntentFilter(Intent.ACTION_BATTERY_CHANGED))
        if (sticky != null) {
            trySend(sticky).onFailure { error ->
                Log.w("BatteryUtils", "Failed to send initial battery intent: ${error?.message}")
            }
        }
        awaitClose { context.unregisterReceiver(receiver) }
    }

    fun getLevel(intent: Intent): Int {
        val level = intent.getIntExtra(BatteryManager.EXTRA_LEVEL, -1)
        val scale = intent.getIntExtra(BatteryManager.EXTRA_SCALE, -1)
        return if (level != -1 && scale != -1) (level * 100 / scale.toFloat()).toInt() else -1
    }

    fun getHealthRes(intent: Intent): Int {
        return when (intent.getIntExtra(BatteryManager.EXTRA_HEALTH, BatteryManager.BATTERY_HEALTH_UNKNOWN)) {
            BatteryManager.BATTERY_HEALTH_GOOD -> R.string.battery_health_good
            BatteryManager.BATTERY_HEALTH_OVERHEAT -> R.string.battery_health_overheat
            BatteryManager.BATTERY_HEALTH_DEAD -> R.string.battery_health_dead
            BatteryManager.BATTERY_HEALTH_OVER_VOLTAGE -> R.string.battery_health_over_voltage
            BatteryManager.BATTERY_HEALTH_UNSPECIFIED_FAILURE -> R.string.battery_health_unspecified_failure
            BatteryManager.BATTERY_HEALTH_COLD -> R.string.battery_health_cold
            else -> R.string.value_unknown
        }
    }

    fun getStatusRes(intent: Intent): Int {
        return when (intent.getIntExtra(BatteryManager.EXTRA_STATUS, BatteryManager.BATTERY_STATUS_UNKNOWN)) {
            BatteryManager.BATTERY_STATUS_CHARGING -> R.string.battery_status_charging
            BatteryManager.BATTERY_STATUS_DISCHARGING -> R.string.battery_status_discharging
            BatteryManager.BATTERY_STATUS_FULL -> R.string.battery_status_full
            BatteryManager.BATTERY_STATUS_NOT_CHARGING -> R.string.battery_status_not_charging
            else -> R.string.value_unknown
        }
    }

    fun getStatus(intent: Intent): String {
        return when (intent.getIntExtra(BatteryManager.EXTRA_STATUS, BatteryManager.BATTERY_STATUS_UNKNOWN)) {
            BatteryManager.BATTERY_STATUS_CHARGING -> "Charging"
            BatteryManager.BATTERY_STATUS_DISCHARGING -> "Discharging"
            BatteryManager.BATTERY_STATUS_FULL -> "Full"
            BatteryManager.BATTERY_STATUS_NOT_CHARGING -> "Not Charging"
            else -> "Unknown"
        }
    }

    fun getPowerSourceRes(intent: Intent): Int {
        return when (intent.getIntExtra(BatteryManager.EXTRA_PLUGGED, -1)) {
            BatteryManager.BATTERY_PLUGGED_AC -> R.string.battery_power_source_ac
            BatteryManager.BATTERY_PLUGGED_USB -> R.string.battery_power_source_usb
            BatteryManager.BATTERY_PLUGGED_WIRELESS -> R.string.battery_power_source_wireless
            0 -> R.string.battery_power_source_battery
            else -> R.string.value_unknown
        }
    }

    fun getTechnology(intent: Intent): String {
        return intent.getStringExtra(BatteryManager.EXTRA_TECHNOLOGY) ?: "Unknown"
    }

    fun getVoltage(intent: Intent): Int {
        return intent.getIntExtra(BatteryManager.EXTRA_VOLTAGE, -1)
    }

    fun getTemperature(intent: Intent): Float {
        return intent.getIntExtra(BatteryManager.EXTRA_TEMPERATURE, -1) / 10f
    }

    fun getCycleCount(intent: Intent): Int {
        return intent.getIntExtra(BatteryManager.EXTRA_CYCLE_COUNT, -1)
    }

    @SuppressLint("PrivateApi")
    fun getCapacity(context: Context): Double {
        val powerProfileClass = "com.android.internal.os.PowerProfile"
        return try {
            val mPowerProfile = Class.forName(powerProfileClass)
                .getConstructor(Context::class.java)
                .newInstance(context)
            Class.forName(powerProfileClass)
                .getMethod("getBatteryCapacity")
                .invoke(mPowerProfile) as Double
        } catch (e: Exception) {
            -1.0
        }
    }

    fun getCurrent(context: Context): Int {
        val batteryManager = context.getSystemService(Context.BATTERY_SERVICE) as BatteryManager

        var current = batteryManager.getLongProperty(BatteryManager.BATTERY_PROPERTY_CURRENT_NOW)

        if (current == 0L || current == Long.MIN_VALUE) {
            val oldCurrent = current
            current = batteryManager.getLongProperty(BatteryManager.BATTERY_PROPERTY_CURRENT_AVERAGE)

            if (com.rve.systemmonitor.BuildConfig.DEBUG && current != oldCurrent) {
                Log.d("BatteryUtils", "Fallback to CURRENT_AVERAGE: $current (NOW was $oldCurrent)")
            }
        }

        if (current == Long.MIN_VALUE) return 0

        val manufacturer = Build.MANUFACTURER.lowercase()
        val isBBK = manufacturer.contains("oneplus") ||
            manufacturer.contains("oppo") ||
            manufacturer.contains("realme")

        val result = if (isBBK && abs(current) > 0 && abs(current) < 20000) {
            current.toInt()
        } else {
            (current / 1000).toInt()
        }

        if (com.rve.systemmonitor.BuildConfig.DEBUG && result == 0 && current != 0L) {
            Log.d("BatteryUtils", "Current truncated to 0 mA from $current uA")
        }

        return result
    }
}
