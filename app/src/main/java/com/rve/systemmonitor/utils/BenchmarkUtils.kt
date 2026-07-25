package com.rve.systemmonitor.utils

import android.util.Log

object BenchmarkUtils {
    private const val TAG = "BenchmarkUtils"

    fun run(iterations: Int = 500, warmup: Int = 50): String = runCatching {
        benchRustNative(iterations, warmup)
    }.getOrElse {
        Log.e(TAG, "bench failed", it)
        "bench failed: ${it.message}"
    }

    private external fun benchRustNative(iterations: Int, warmup: Int): String

    init {
        NativeLoader.load()
    }
}
