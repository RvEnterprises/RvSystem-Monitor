use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::time::{Duration, Instant};

fn read_fd_parsed<T: std::str::FromStr>(file: &mut std::fs::File, buf: &mut String) -> Option<T> {
    buf.clear();
    file.seek(SeekFrom::Start(0)).ok()?;
    file.read_to_string(buf).ok()?;
    buf.trim().parse::<T>().ok()
}

fn get_mem_kb() -> Option<u64> {
    fs::read_to_string("/proc/self/status")
        .ok()?
        .lines()
        .find(|l| l.starts_with("VmRSS:"))
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u64>().ok())
}

fn get_core_count() -> i32 {
    if let Ok(content) = fs::read_to_string("/sys/devices/system/cpu/present") {
        let content = content.trim();
        if let Some((start_str, end_str)) = content.split_once('-') {
            let start: i32 = start_str.parse().unwrap_or(0);
            let end: i32 = end_str.parse().unwrap_or(0);
            return end - start + 1;
        }
        return content.split(',').count() as i32;
    }
    0
}

fn bench_freq_read(cores: i32, iterations: usize) -> Duration {
    let mut total = Duration::ZERO;
    let mut buf = String::with_capacity(32);

    for _ in 0..iterations {
        for i in 0..cores {
            let start = Instant::now();
            let paths = [
                format!("/sys/devices/system/cpu/cpu{i}/cpufreq/scaling_cur_freq"),
                format!("/sys/devices/system/cpu/cpufreq/policy{i}/scaling_cur_freq"),
            ];
            for p in &paths {
                if let Ok(mut f) = std::fs::File::open(p) {
                    let _: Option<i64> = read_fd_parsed(&mut f, &mut buf);
                    break;
                }
            }
            total += start.elapsed();
        }
    }
    total
}

fn bench_governor_read(cores: i32, iterations: usize) -> Duration {
    let mut total = Duration::ZERO;

    for _ in 0..iterations {
        for i in 0..cores {
            let start = Instant::now();
            let _ = fs::read_to_string(format!(
                "/sys/devices/system/cpu/cpu{i}/cpufreq/scaling_governor"
            ));
            total += start.elapsed();
        }
    }
    total
}

fn bench_proc_stat(iterations: usize) -> Duration {
    let mut total = Duration::ZERO;
    for _ in 0..iterations {
        let start = Instant::now();
        let _ = fs::read_to_string("/proc/stat");
        total += start.elapsed();
    }
    total
}

fn bench_thermal(iterations: usize) -> Duration {
    let mut total = Duration::ZERO;
    for _ in 0..iterations {
        let start = Instant::now();
        if let Ok(entries) = fs::read_dir("/sys/class/thermal") {
            for entry in entries.flatten() {
                let temp_path = entry.path().join("temp");
                let _ = fs::read_to_string(&temp_path);
            }
        }
        total += start.elapsed();
    }
    total
}

fn bench_meminfo(iterations: usize) -> Duration {
    let mut total = Duration::ZERO;
    for _ in 0..iterations {
        let start = Instant::now();
        let _ = fs::read_to_string("/proc/meminfo");
        total += start.elapsed();
    }
    total
}

fn main() {
    let iters = 100;
    let cores = get_core_count();

    println!("=== RvSystem Monitor Benchmark ===");
    println!("Device cores: {cores}");
    println!("Iterations: {iters}\n");

    // Baseline memory
    let mem_before = get_mem_kb().unwrap_or(0);

    // 1. CPU freq read
    let t = bench_freq_read(cores, iters);
    let avg_us = t.as_micros() as f64 / iters as f64;
    println!("[cpu_freq]     avg {avg_us:.1}µs/call ({cores} cores)");

    // 2. Governor read
    let t = bench_governor_read(cores, iters);
    let avg_us = t.as_micros() as f64 / iters as f64;
    println!("[governor]     avg {avg_us:.1}µs/call ({cores} cores)");

    // 3. /proc/stat
    let t = bench_proc_stat(iters);
    let avg_us = t.as_micros() as f64 / iters as f64;
    println!("[proc_stat]    avg {avg_us:.1}µs/call");

    // 4. Thermal zones
    let t = bench_thermal(iters);
    let avg_us = t.as_micros() as f64 / iters as f64;
    println!("[thermal]      avg {avg_us:.1}µs/call");

    // 5. /proc/meminfo
    let t = bench_meminfo(iters);
    let avg_us = t.as_micros() as f64 / iters as f64;
    println!("[meminfo]      avg {avg_us:.1}µs/call");

    // Memory after
    let mem_after = get_mem_kb().unwrap_or(0);
    println!("\n--- Memory ---");
    println!("RSS before: {mem_before} kB");
    println!("RSS after:  {mem_after} kB");
    println!("Delta:      {} kB", mem_after as i64 - mem_before as i64);

    // Full cycle simulation (what the app does on each refresh)
    println!("\n--- Full cycle (all data) ---");
    let start = Instant::now();
    for _ in 0..iters {
        // CPU freq
        for i in 0..cores {
            let _ = fs::read_to_string(format!(
                "/sys/devices/system/cpu/cpu{i}/cpufreq/scaling_cur_freq"
            ));
        }
        // Governor
        for i in 0..cores {
            let _ = fs::read_to_string(format!(
                "/sys/devices/system/cpu/cpu{i}/cpufreq/scaling_governor"
            ));
        }
        // Temp
        if let Ok(entries) = fs::read_dir("/sys/class/thermal") {
            for entry in entries.flatten() {
                let _ = fs::read_to_string(entry.path().join("temp"));
            }
        }
        // /proc/stat
        let _ = fs::read_to_string("/proc/stat");
        // /proc/meminfo
        let _ = fs::read_to_string("/proc/meminfo");
    }
    let total = start.elapsed();
    let avg_ms = total.as_secs_f64() * 1000.0 / iters as f64;
    println!("avg {avg_ms:.2}ms/call ({iters} iterations)");
    println!("throughput: {:.0} calls/sec", 1000.0 / avg_ms);
}
