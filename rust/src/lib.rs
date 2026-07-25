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
    fn Java_com_rve_systemmonitor_utils_BenchmarkUtils_benchRustNative(env, iters: jint, warmup: jint) -> jdoubleArray {
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

            // 4. All core temps — single lock, read all at once
            let s = std::time::Instant::now();
            let _ = kernel::thermal::get_all_core_temperatures();
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

        // Return raw stats as jdoubleArray — zero-copy, format in Kotlin
        // Layout: [freq(6), gov(6), temp(6), all_temp(6), load(6), mem(6), full(6), native_mean(1), rss(1), ctx_vol(1), ctx_invol(1), cores(1), iters(1), warmup(1)] = 49
        let mut out = Vec::with_capacity(49);
        out.extend_from_slice(&[freq_stats.p50, freq_stats.p90, freq_stats.p95, freq_stats.p99, freq_stats.max, freq_stats.stddev]);
        out.extend_from_slice(&[gov_stats.p50, gov_stats.p90, gov_stats.p95, gov_stats.p99, gov_stats.max, gov_stats.stddev]);
        out.extend_from_slice(&[temp_stats.p50, temp_stats.p90, temp_stats.p95, temp_stats.p99, temp_stats.max, temp_stats.stddev]);
        out.extend_from_slice(&[all_temp_stats.p50, all_temp_stats.p90, all_temp_stats.p95, all_temp_stats.p99, all_temp_stats.max, all_temp_stats.stddev]);
        out.extend_from_slice(&[load_stats.p50, load_stats.p90, load_stats.p95, load_stats.p99, load_stats.max, load_stats.stddev]);
        out.extend_from_slice(&[mem_stats.p50, mem_stats.p90, mem_stats.p95, mem_stats.p99, mem_stats.max, mem_stats.stddev]);
        out.extend_from_slice(&[full_stats.p50, full_stats.p90, full_stats.p95, full_stats.p99, full_stats.max, full_stats.stddev]);
        out.push(native_stats.mean);
        out.push(sys_delta.rss_delta_kb as f64);
        out.push(sys_delta.ctx_voluntary_delta as f64);
        out.push(sys_delta.ctx_involuntary_delta as f64);
        out.push(cores as f64);
        out.push(iters as f64);
        out.push(warmup as f64);

        jni_double_array!(env, out)
    }
}
