package com.rve.systemmonitor.utils

import android.util.Log

object BenchmarkUtils {
    private const val TAG = "BenchmarkUtils"

    fun run(iterations: Int = 500, warmup: Int = 50): String = runCatching {
        val data = benchRustNative(iterations, warmup)
        formatReport(data, iterations, warmup)
    }.getOrElse {
        Log.e(TAG, "bench failed", it)
        "bench failed: ${it.message}"
    }

    private external fun benchRustNative(iterations: Int, warmup: Int): DoubleArray

    private fun formatReport(d: DoubleArray, iters: Int, warmup: Int): String {
        // Layout matches Rust: [freq(6), gov(6), temp(6), all_temp(6), load(6), mem(6), full(6), native_mean(1), rss(1), ctx_vol(1), ctx_invol(1), cores(1), iters(1), warmup(1)]
        val cores = d[46].toInt()
        val rss = d[43].toLong()
        val ctxVol = d[44].toLong()
        val ctxInvol = d[45].toLong()
        val nativeMean = d[42]

        fun stats(prefix: String, off: Int): String {
            return " $prefix       ${fmt(d[off])}   ${fmt(d[off+1])}   ${fmt(d[off+2])}   ${fmt(d[off+3])}   ${fmt(d[off+4])}   ±${fmt(d[off+5])}"
        }

        val fullMean = d[36]
        val jniOverhead = fullMean - nativeMean

        return buildString {
            append("=== JNI Bridge Benchmark Summary ===\n")
            append("Config: $iters iters | $warmup warmup | Cores: $cores\n\n")
            append("Latency (μs):\n")
            append(" Operation         p50      p90      p95      p99      Max      StdDev\n")
            append(" ──────────────   ──────   ──────   ──────   ──────   ──────   ──────\n")
            append(stats("Freq (all)  ", 0))
            append("\n")
            append(stats("Governor    ", 6))
            append("\n")
            append(stats("CPU Temp    ", 12))
            append("\n")
            append(stats("All Core T  ", 18))
            append("\n")
            append(stats("/proc/stat  ", 24))
            append("\n")
            append(stats("Memory      ", 30))
            append("\n")
            append(stats("Full Cycle  ", 36))
            append("\n\n")
            append("JNI Breakdown (Full Cycle):\n")
            append(" Native Exec (Rust) : ${fmt(nativeMean)} μs\n")
            append(" JNI Overhead       : ${fmt(jniOverhead)} μs\n")
            append(" Total              : ${fmt(fullMean)} μs\n\n")
            append("Memory & System:\n")
            append(" RSS Delta          : ${rss} KB\n")
            append(" Context Switches   : $ctxVol vol + $ctxInvol invol\n")
            append(" Warmup Skipped     : $warmup iterations")
        }
    }

    private fun fmt(v: Double): String = String.format("%.1f", v)

    init {
        NativeLoader.load()
    }
}
