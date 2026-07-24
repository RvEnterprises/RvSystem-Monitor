package com.rve.systemmonitor.utils

import android.os.Build
import android.util.Log

object DeviceUtils {
    const val TAG = "DeviceUtils"

    fun getBrand(): String = runCatching {
        Build.BRAND
    }.getOrElse {
        Log.e(TAG, "getBrand: ${it.message}", it)
        "unknown"
    }

    fun getModel(): String = runCatching {
        Build.MODEL
    }.getOrElse {
        Log.e(TAG, "getModel: ${it.message}", it)
        "unknown"
    }

    fun getDevice(): String = runCatching {
        Build.DEVICE
    }.getOrElse {
        Log.e(TAG, "getDevice: ${it.message}", it)
        "unknown"
    }

    fun getMarketName(): String = runCatching {
        val clazz = Class.forName("android.os.SystemProperties")
        val getMethod = clazz.getMethod("get", String::class.java, String::class.java)
        val marketName = getMethod.invoke(clazz, "ro.product.marketname", "") as String

        marketName.ifEmpty { "unknown" }
    }.getOrElse {
        Log.e(TAG, "getMarketName: ${it.message}", it)
        "unknown"
    }

    fun getRustLibraryVersion(): String = runCatching {
        getRustLibraryVersionNative()
    }.getOrElse {
        Log.e(TAG, "getRustLibraryVersion: ${it.message}", it)
        "unknown"
    }

    private external fun getRustLibraryVersionNative(): String

    init {
        NativeLoader.load()
    }
}
