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

    fun getHyperOSVersion(): String? = runCatching {
        val manufacturer = Build.MANUFACTURER.lowercase()
        if (manufacturer !in listOf("xiaomi", "redmi", "poco")) {
            return@runCatching null
        }

        val process = Runtime.getRuntime().exec("getprop ro.build.version.incremental")
        val version = process.inputStream.bufferedReader().use { it.readText().trim() }

        if (version.isNotEmpty() && (version.startsWith("V816") || version.startsWith("OS"))) {
            return@runCatching formatHyperOSVersion(version)
        }
        null
    }.getOrNull()

    private fun formatHyperOSVersion(version: String): String {
        var cleanVersion = version.removePrefix("OS").removePrefix("V816").removePrefix(".")
        val codeRegex = "\\.([A-Z]{7})$".toRegex()
        val match = codeRegex.find(cleanVersion)
        
        if (match != null) {
            val code = match.groupValues[1]
            val regionCode = code.substring(3, 5)
            val regionName = when (regionCode) {
                "MI" -> "Global"
                "EU" -> "EEA"
                "IN" -> "India"
                "ID" -> "Indonesia"
                "RU" -> "Russia"
                "TW" -> "Taiwan"
                "TR" -> "Turkey"
                "JP" -> "Japan"
                "CN" -> "China"
                "KR" -> "Korea"
                else -> regionCode
            }
            cleanVersion = cleanVersion.replace(".$code", " $regionName")
        }
        return cleanVersion
    }
}
