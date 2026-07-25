//! # RvSystem Monitor Rust Backend
//!
//! This crate provides the native implementation for system monitoring tasks in the RvSystem Monitor application.
//! It interfaces with the Android application via JNI (Java Native Interface).

#![allow(non_snake_case)]

use jni::objects::JString;
use jni::strings::JNIString;
use jni::sys::{jdouble, jdoubleArray, jint, jlong, jlongArray, jobjectArray, jstring};

pub mod bench;
pub mod drivers;
pub mod kernel;
pub mod macros;
pub mod mm;

fn map_ram_data(ram: &mm::memory::RamData) -> [f64; 9] {
    [
        ram.total,
        ram.available,
        ram.used,
        ram.used_percentage,
        ram.cached,
        ram.buffers,
        ram.active,
        ram.inactive,
        ram.slab,
    ]
}

fn map_zram_data(zram: &mm::memory::ZramData) -> [f64; 5] {
    [
        if zram.is_active { 1.0 } else { 0.0 },
        zram.total,
        zram.available,
        zram.used,
        zram.used_percentage,
    ]
}

jni_fn! {
    fn Java_com_rve_systemmonitor_utils_DeviceUtils_getRustLibraryVersionNative(env) -> jstring {
        let version = env!("CARGO_PKG_VERSION");
        Ok(env.new_string(version)?.into_raw())
    }
}

jni_fn! {
    fn Java_com_rve_systemmonitor_utils_GpuUtils_getVulkanVersionNative(env) -> jstring {
        let version = drivers::gpu::vulkan::get_vulkan_version();
        Ok(env.new_string(version)?.into_raw())
    }
}

jni_fn! {
    fn Java_com_rve_systemmonitor_utils_GpuUtils_getGpuTemperatureNative(env) -> jdouble {
        let _ = env;
        Ok(kernel::thermal::get_gpu_temperature())
    }
}

jni_fn! {
    fn Java_com_rve_systemmonitor_utils_MemoryUtils_getMemoryDataNative(env) -> jdoubleArray {
        let (ram, zram) = mm::memory::get_memory_data();

        let ram_data = map_ram_data(&ram);
        let zram_data = map_zram_data(&zram);

        let mut data = [0.0; 14];
        data[..9].copy_from_slice(&ram_data);
        data[9..].copy_from_slice(&zram_data);

        jni_double_array!(env, data)
    }
}

jni_fn! {
    fn Java_com_rve_systemmonitor_utils_MemoryUtils_getRamDataNative(env) -> jdoubleArray {
        let (ram, _) = mm::memory::get_memory_data();
        jni_double_array!(env, map_ram_data(&ram))
    }
}

jni_fn! {
    fn Java_com_rve_systemmonitor_utils_MemoryUtils_getZramDataNative(env) -> jdoubleArray {
        let (_, zram) = mm::memory::get_memory_data();
        jni_double_array!(env, map_zram_data(&zram))
    }
}

jni_fn! {
    fn Java_com_rve_systemmonitor_utils_CpuUtils_getStaticCoreInfoNative(env) -> jlongArray {
        let cores = kernel::cpu::get_core_count();
        let mut data = Vec::with_capacity(cores as usize * 2);

        for i in 0..cores {
            data.push(kernel::cpu::get_core_frequency(i, "min_info"));
            data.push(kernel::cpu::get_core_frequency(i, "max_info"));
        }

        jni_long_array!(env, data)
    }
}

jni_fn! {
    fn Java_com_rve_systemmonitor_utils_CpuUtils_getAllCoreGovernorsNative(env) -> jobjectArray {
        let cores = kernel::cpu::get_core_count();
        let first_gov = kernel::cpu::get_core_governor(0);
        let initial_element = env.new_string(first_gov)?;

        let class = env.find_class(JNIString::from("java/lang/String"))?;
        let array = env.new_object_array(cores, &class, initial_element)?;

        for i in 1..cores {
            let governor = kernel::cpu::get_core_governor(i);
            let s = env.new_string(governor)?;
            array.set_element(env, i as usize, s)?;
        }

        Ok(array.into_raw())
    }
}

jni_fn! {
    fn Java_com_rve_systemmonitor_utils_CpuUtils_getAllCoreFrequenciesNative(env) -> jlongArray {
        let cores = kernel::cpu::get_core_count();
        let mut frequencies = Vec::with_capacity(cores as usize);

        for i in 0..cores {
            frequencies.push(kernel::cpu::get_core_frequency(i, "cur"));
        }

        jni_long_array!(env, frequencies)
    }
}

jni_fn! {
    fn Java_com_rve_systemmonitor_utils_CpuUtils_getCoreCountNative(env) -> jint {
        let _ = env;
        Ok(kernel::cpu::get_core_count())
    }
}

jni_fn! {
    fn Java_com_rve_systemmonitor_utils_CpuUtils_getCoreFrequencyNative(env, core_id: jint, freq_type: JString<'local>) -> jlong {
        let freq_type_jstr = freq_type.mutf8_chars(env).unwrap();
        let freq_type_cow = freq_type_jstr.to_str();
        Ok(kernel::cpu::get_core_frequency(core_id, freq_type_cow.as_ref()))
    }
}

jni_fn! {
    fn Java_com_rve_systemmonitor_utils_CpuUtils_getCoreGovernorNative(env, core_id: jint) -> jstring {
        let governor = kernel::cpu::get_core_governor(core_id);
        Ok(env.new_string(governor)?.into_raw())
    }
}

jni_fn! {
    fn Java_com_rve_systemmonitor_utils_CpuUtils_getCpuTemperatureNative(env) -> jdouble {
        let _ = env;
        Ok(kernel::thermal::get_cpu_temperature())
    }
}

jni_fn! {
    fn Java_com_rve_systemmonitor_utils_CpuUtils_getAllCoreTemperaturesNative(env) -> jdoubleArray {
        let cores = kernel::cpu::get_core_count();
        let mut temps = Vec::with_capacity(cores as usize);

        for i in 0..cores {
            temps.push(kernel::thermal::get_core_temperature(i));
        }

        jni_double_array!(env, temps)
    }
}

jni_fn! {
    fn Java_com_rve_systemmonitor_utils_CpuUtils_getCpuDynamicDataNative(env) -> jdoubleArray {
        let cores = kernel::cpu::get_core_count() as usize;
        let mut data = Vec::with_capacity(1 + 2 * cores);

        data.push(kernel::thermal::get_cpu_temperature());

        for i in 0..cores {
            data.push(kernel::cpu::get_core_frequency(i as i32, "cur") as f64);
            data.push(kernel::thermal::get_core_temperature(i as i32));
        }

        jni_double_array!(env, data)
    }
}

jni_fn! {
    fn Java_com_rve_systemmonitor_utils_CpuUtils_calculateCpuLoadNative(env, proc_stat: JString<'local>) -> jdoubleArray {
        let proc_stat_jstr = proc_stat.mutf8_chars(env).unwrap();
        let proc_stat_cow = proc_stat_jstr.to_str();
        let results = kernel::cpu::calculate_cpu_load(proc_stat_cow.as_ref());
        jni_double_array!(env, results)
    }
}

jni_fn! {
    fn Java_com_rve_systemmonitor_utils_BenchmarkUtils_benchRustNative(env, iters: jint, warmup: jint) -> jstring {
        let n = iters as usize;
        let w = warmup as usize;
        let cores = kernel::cpu::get_core_count();
        let proc_stat = std::fs::read_to_string("/proc/stat").unwrap_or_default();
        let _ = env;

        // System snapshot before
        let snap_before = bench::system::SystemSnapshot::capture();

        // --- WARMUP ---
        for _ in 0..w {
            for i in 0..cores { let _ = kernel::cpu::get_core_frequency(i, "cur"); }
            for i in 0..cores { let _ = kernel::cpu::get_core_governor(i); }
            let _ = kernel::thermal::get_cpu_temperature();
            for i in 0..cores { let _ = kernel::thermal::get_core_temperature(i); }
            let _ = kernel::cpu::calculate_cpu_load(&proc_stat);
            let _ = mm::memory::get_memory_data();
        }

        // --- MEASUREMENT: individual samples per operation ---
        let mut freq_samples = Vec::with_capacity(n);
        let mut gov_samples = Vec::with_capacity(n);
        let mut temp_samples = Vec::with_capacity(n);
        let mut all_temp_samples = Vec::with_capacity(n);
        let mut load_samples = Vec::with_capacity(n);
        let mut mem_samples = Vec::with_capacity(n);
        let mut full_samples = Vec::with_capacity(n);

        // Track native exec time vs total for JNI overhead calculation
        let mut native_exec_samples = Vec::with_capacity(n);

        for _ in 0..n {
            // 1. Freq
            let s = std::time::Instant::now();
            for i in 0..cores { let _ = kernel::cpu::get_core_frequency(i, "cur"); }
            freq_samples.push(s.elapsed());

            // 2. Governor
            let s = std::time::Instant::now();
            for i in 0..cores { let _ = kernel::cpu::get_core_governor(i); }
            gov_samples.push(s.elapsed());

            // 3. CPU temp
            let s = std::time::Instant::now();
            let _ = kernel::thermal::get_cpu_temperature();
            temp_samples.push(s.elapsed());

            // 4. All core temps
            let s = std::time::Instant::now();
            for i in 0..cores { let _ = kernel::thermal::get_core_temperature(i); }
            all_temp_samples.push(s.elapsed());

            // 5. /proc/stat
            let s = std::time::Instant::now();
            let _ = kernel::cpu::calculate_cpu_load(&proc_stat);
            load_samples.push(s.elapsed());

            // 6. Memory
            let s = std::time::Instant::now();
            let _ = mm::memory::get_memory_data();
            mem_samples.push(s.elapsed());

            // 7. Full cycle — measure native exec vs total for JNI overhead
            let s_total = std::time::Instant::now();
            let s_native = std::time::Instant::now();
            let _ = kernel::thermal::get_cpu_temperature();
            for i in 0..cores {
                let _ = kernel::cpu::get_core_frequency(i, "cur");
                let _ = kernel::thermal::get_core_temperature(i);
            }
            let _ = kernel::cpu::calculate_cpu_load(&proc_stat);
            let _ = mm::memory::get_memory_data();
            let native_elapsed = s_native.elapsed();

            native_exec_samples.push(native_elapsed);
            full_samples.push(s_total.elapsed());
        }

        // System snapshot after
        let snap_after = bench::system::SystemSnapshot::capture();
        let sys_delta = snap_after.delta(&snap_before);

        // Calculate statistics
        let freq_stats = bench::stats::LatencyStats::from_durations(&freq_samples);
        let gov_stats = bench::stats::LatencyStats::from_durations(&gov_samples);
        let temp_stats = bench::stats::LatencyStats::from_durations(&temp_samples);
        let all_temp_stats = bench::stats::LatencyStats::from_durations(&all_temp_samples);
        let load_stats = bench::stats::LatencyStats::from_durations(&load_samples);
        let mem_stats = bench::stats::LatencyStats::from_durations(&mem_samples);
        let full_stats = bench::stats::LatencyStats::from_durations(&full_samples);
        let native_stats = bench::stats::LatencyStats::from_durations(&native_exec_samples);

        // Format output
        let out = format!(
            "=== JNI Bridge Benchmark Summary ===\n\
             Config: {n} iters | {w} warmup | Cores: {cores} | Affinity: {affinity}\n\n\
             Latency (μs):\n\
             \x20 Operation         p50      p90      p95      p99      Max      StdDev\n\
             \x20 ──────────────   ──────   ──────   ──────   ──────   ──────   ──────\n\
             \x20 Freq (all)       {f50:<7.1} {f90:<7.1} {f95:<7.1} {f99:<7.1} {fmax:<7.1} ±{fstd:<5.1}\n\
             \x20 Governor (all)   {g50:<7.1} {g90:<7.1} {g95:<7.1} {g99:<7.1} {gmax:<7.1} ±{gstd:<5.1}\n\
             \x20 CPU Temp         {t50:<7.1} {t90:<7.1} {t95:<7.1} {t99:<7.1} {tmax:<7.1} ±{tstd:<5.1}\n\
             \x20 All Core Temps   {at50:<7.1} {at90:<7.1} {at95:<7.1} {at99:<7.1} {atmax:<7.1} ±{atstd:<5.1}\n\
             \x20 /proc/stat       {l50:<7.1} {l90:<7.1} {l95:<7.1} {l99:<7.1} {lmax:<7.1} ±{lstd:<5.1}\n\
             \x20 Memory           {m50:<7.1} {m90:<7.1} {m95:<7.1} {m99:<7.1} {mmax:<7.1} ±{mstd:<5.1}\n\
             \x20 Full Cycle       {fl50:<7.1} {fl90:<7.1} {fl95:<7.1} {fl99:<7.1} {flmax:<7.1} ±{flstd:<5.1}\n\n\
             JNI Breakdown (Full Cycle):\n\
             \x20 Native Exec (Rust) : {native_avg:.1} μs\n\
             \x20 JNI Overhead       : {jni_overhead:.1} μs\n\
             \x20 Total              : {total_avg:.1} μs\n\n\
             Memory & System:\n\
             \x20 RSS Delta          : {rss_delta:+} KB\n\
             \x20 Context Switches   : {ctx_vol} vol + {ctx_invol} invol\n\
             \x20 Warmup Skipped     : {w} iterations",
            affinity = snap_before.core_affinity,
            f50 = freq_stats.p50, f90 = freq_stats.p90, f95 = freq_stats.p95, f99 = freq_stats.p99, fmax = freq_stats.max, fstd = freq_stats.stddev,
            g50 = gov_stats.p50, g90 = gov_stats.p90, g95 = gov_stats.p95, g99 = gov_stats.p99, gmax = gov_stats.max, gstd = gov_stats.stddev,
            t50 = temp_stats.p50, t90 = temp_stats.p90, t95 = temp_stats.p95, t99 = temp_stats.p99, tmax = temp_stats.max, tstd = temp_stats.stddev,
            at50 = all_temp_stats.p50, at90 = all_temp_stats.p90, at95 = all_temp_stats.p95, at99 = all_temp_stats.p99, atmax = all_temp_stats.max, atstd = all_temp_stats.stddev,
            l50 = load_stats.p50, l90 = load_stats.p90, l95 = load_stats.p95, l99 = load_stats.p99, lmax = load_stats.max, lstd = load_stats.stddev,
            m50 = mem_stats.p50, m90 = mem_stats.p90, m95 = mem_stats.p95, m99 = mem_stats.p99, mmax = mem_stats.max, mstd = mem_stats.stddev,
            fl50 = full_stats.p50, fl90 = full_stats.p90, fl95 = full_stats.p95, fl99 = full_stats.p99, flmax = full_stats.max, flstd = full_stats.stddev,
            native_avg = native_stats.mean,
            jni_overhead = full_stats.mean - native_stats.mean,
            total_avg = full_stats.mean,
            rss_delta = sys_delta.rss_delta_kb,
            ctx_vol = sys_delta.ctx_voluntary_delta,
            ctx_invol = sys_delta.ctx_involuntary_delta,
        );

        let jstr = env.new_string(&out)?;
        Ok(jstr.into_raw())
    }
}
