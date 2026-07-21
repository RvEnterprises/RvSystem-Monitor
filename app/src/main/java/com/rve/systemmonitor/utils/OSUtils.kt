package com.rve.systemmonitor.utils

import android.os.Build
import android.util.Log
import com.rve.systemmonitor.R

object OSUtils {
    private const val TAG = "OSUtils"

    fun getAndroidVersion(): String = runCatching {
        Build.VERSION.RELEASE
    }.getOrElse {
        Log.e(TAG, "getAndroidVersion: ${it.message}", it)
        "unknown"
    }

    fun getSdkInt(): Int = runCatching {
        Build.VERSION.SDK_INT
    }.getOrElse {
        Log.e(TAG, "getSdkInt: ${it.message}", it)
        0
    }

    fun getDessertNameRes(sdkInt: Int): Int {
        return when (sdkInt) {
            37 -> R.string.dessert_name_cinnamon_bun
            36 -> R.string.dessert_name_baklava
            35 -> R.string.dessert_name_vanilla_ice_cream
            34 -> R.string.dessert_name_upside_down_cake
            else -> R.string.value_unknown
        }
    }

    fun getSecurityPatch(): String = runCatching {
        Build.VERSION.SECURITY_PATCH
    }.getOrElse {
        Log.e(TAG, "getSecurityPatch: ${it.message}", it)
        "unknown"
    }
}
