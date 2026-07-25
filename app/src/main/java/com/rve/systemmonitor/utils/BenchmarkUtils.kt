package com.rve.systemmonitor.utils

import android.os.Debug
import android.util.Log
import kotlin.system.measureNanoTime

object BenchmarkUtils {
    private const val TAG = "BenchmarkUtils"

    fun run(iterations: Int = 500): String = runCatching {
        val labels = listOf("freq", "governor", "cpu_temp", "all_temps", "proc_stat", "memory", "full_cycle")

        // First call to warm up
        benchRustNative(1)

        // Measure JNI round-trip wall clock
        val wallNanos = measureNanoTime {
            benchRustNative(iterations)
        }

        // Measure Rust-internal times (µs) via JNI
        val raw = benchRustNative(iterations)
        val cores = raw[7].toInt()
        val totalFileOps = raw[8].toLong()

        // JVM allocation tracking
        val rt = Runtime.getRuntime()
        val gcBefore = rt.freeMemory()
        rt.gc()
        val memBefore = rt.totalMemory() - rt.freeMemory()
        benchRustNative(iterations)
        rt.gc()
        val memAfter = rt.totalMemory() - rt.freeMemory()

        val rustTotalUs = (0..6).sumOf { raw[it] }
        val wallPerIterUs = wallNanos / iterations.toDouble() / 1000.0
        val jniOverheadUs = wallPerIterUs - rustTotalUs
        val jniPct = if (rustTotalUs > 0.0) jniOverheadUs / rustTotalUs * 100.0 else 0.0

        val sb = StringBuilder()
        sb.appendLine("=== Benchmark ($iterations iters, $cores cores) ===")
        sb.appendLine()
        sb.appendLine("Rust-internal (µs/call):")
        for ((i, label) in labels.withIndex()) {
            sb.appendLine("  %-12s %,10.1f".format(label, raw[i]))
        }
        sb.appendLine()
        sb.appendLine("JNI overhead:")
        sb.appendLine("  Rust total     %,10.1f µs".format(rustTotalUs))
        sb.appendLine("  Wall clock     %,10.1f µs".format(wallPerIterUs))
        sb.appendLine("  JNI overhead   %,10.1f µs (%.1f%%)".format(jniOverheadUs, jniPct))
        sb.appendLine()
        sb.appendLine("Allocations per full_cycle:")
        sb.appendLine("  File opens     %,d per call".format(totalFileOps / iterations.toLong()))
        sb.appendLine("  Total file ops %,d (%d iters)".format(totalFileOps, iterations))
        sb.appendLine()
        sb.appendLine("Heap delta: %,d bytes".format(memAfter - memBefore))
        sb.appendLine("Throughput: %,.0f full_cycle calls/sec".format(1_000_000.0 / wallPerIterUs))
        sb.toString()
    }.getOrElse {
        Log.e(TAG, "bench failed", it)
        "bench failed: ${it.message}"
    }

    private external fun benchRustNative(iterations: Int): DoubleArray

    init {
        NativeLoader.load()
    }
}
