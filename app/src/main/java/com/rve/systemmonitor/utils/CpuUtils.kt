package com.rve.systemmonitor.utils

import android.os.Build
import android.util.Log
import java.util.Locale

object CpuUtils {
    private const val TAG = "CpuUtils"

    init {
        NativeLoader.load()
    }

    @JvmStatic
    private external fun getAllCoreFrequenciesNative(): LongArray

    fun getAllCoreFrequenciesKhz(): LongArray = runCatching {
        getAllCoreFrequenciesNative()
    }.getOrElse {
        Log.e(TAG, "getAllCoreFrequenciesKhz: ${it.message}", it)
        LongArray(0)
    }

    fun getAllCoreFrequencies(): Array<String> = runCatching {
        val frequencies = getAllCoreFrequenciesKhz()
        frequencies.map { formatFrequency(it) }.toTypedArray()
    }.getOrElse {
        Log.e(TAG, "getAllCoreFrequencies: ${it.message}", it)
        emptyArray()
    }

    @JvmStatic
    private external fun getStaticCoreInfoNative(): LongArray

    @JvmStatic
    private external fun getAllCoreGovernorsNative(): Array<String>

    @JvmStatic
    private external fun getCoreCountNative(): Int

    @JvmStatic
    private external fun getCoreFrequencyNative(coreId: Int, type: String): Long

    @JvmStatic
    private external fun getCoreGovernorNative(core_id: Int): String

    @JvmStatic
    private external fun getCpuTemperatureNative(): Double

    @JvmStatic
    private external fun getAllCoreTemperaturesNative(): DoubleArray

    @JvmStatic
    private external fun getCpuDynamicDataNative(): DoubleArray

    @JvmStatic
    private external fun calculateCpuLoadNative(procStat: String): DoubleArray

    fun getCpuTemperature(): Double = runCatching {
        getCpuTemperatureNative()
    }.getOrElse { 0.0 }

    fun getAllCoreTemperatures(): DoubleArray = runCatching {
        getAllCoreTemperaturesNative()
    }.getOrElse { DoubleArray(0) }

    fun getCpuDynamicData(): DoubleArray = runCatching {
        getCpuDynamicDataNative()
    }.getOrElse { DoubleArray(0) }

    fun calculateCpuLoad(procStat: String): DoubleArray = runCatching {
        calculateCpuLoadNative(procStat)
    }.getOrElse { DoubleArray(0) }

    fun formatFrequency(freqKhz: Long): String {
        return String.format(Locale.US, "%.2f GHz", freqKhz / 1_000_000.0)
    }

    fun getSocManufacturer(): String = runCatching {
        val manufacturer = Build.SOC_MANUFACTURER
        if (manufacturer != Build.UNKNOWN) {
            manufacturer.replaceFirstChar { it.uppercase() }
        } else {
            "Unknown"
        }
    }.getOrElse {
        Log.e(TAG, "getSocManufacturer: ${it.message}", it)
        "Unknown"
    }

    fun getSocModel(): String = runCatching {
        val model = Build.SOC_MODEL
        if (model != Build.UNKNOWN) {
            model.uppercase()
        } else {
            "Unknown"
        }
    }.getOrElse {
        Log.e(TAG, "getSocModel: ${it.message}", it)
        "Unknown"
    }

    fun getHardware(): String = runCatching { Build.HARDWARE }.getOrElse { "Unknown" }

    fun getBoard(): String = runCatching { Build.BOARD }.getOrElse { "Unknown" }

    fun getArchitecture(): String = runCatching {
        Build.SUPPORTED_ABIS.firstOrNull() ?: "Unknown"
    }.getOrElse { "Unknown" }

    fun getCoreCount(): Int = runCatching {
        getCoreCountNative()
    }.getOrElse {
        Log.e(TAG, "getCoreCount: ${it.message}", it)
        0
    }

    fun getStaticCoreInfo(): LongArray = runCatching {
        getStaticCoreInfoNative()
    }.getOrElse { LongArray(0) }

    fun getAllCoreGovernors(): Array<String> = runCatching {
        getAllCoreGovernorsNative()
    }.getOrElse { emptyArray() }

    fun getCoreFrequencyKhz(coreId: Int, type: String): Long = runCatching {
        getCoreFrequencyNative(coreId, type)
    }.getOrElse { 0L }

    fun getCoreFrequency(coreId: Int, type: String): String = runCatching {
        formatFrequency(getCoreFrequencyKhz(coreId, type))
    }.getOrElse { "N/A" }

    fun getCoreGovernor(coreId: Int): String = runCatching {
        getCoreGovernorNative(coreId)
    }.getOrElse { "N/A" }
}
