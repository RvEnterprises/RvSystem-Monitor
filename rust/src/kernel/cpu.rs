//! # CPU Data Provider
//!
//! Provides functions to read and parse CPU information from the system.

use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::Mutex;

fn read_fd_parsed<T: std::str::FromStr>(file: &mut File, buf: &mut String) -> Option<T> {
    buf.clear();
    file.seek(SeekFrom::Start(0)).ok()?;
    file.read_to_string(buf).ok()?;
    buf.trim().parse::<T>().ok()
}

fn read_path_parsed<T: std::str::FromStr>(path: &str, buf: &mut String) -> Option<T> {
    buf.clear();
    if let Ok(mut f) = File::open(path)
        && f.read_to_string(buf).is_ok()
    {
        return buf.trim().parse::<T>().ok();
    }
    None
}

struct CpuFds {
    cur_freq: Vec<Option<File>>,
    max_freq: Vec<Option<File>>,
    min_freq: Vec<Option<File>>,
    governor: Vec<Option<File>>,
}

static CPU_FDS: OnceCell<Mutex<CpuFds>> = OnceCell::new();

fn get_cpu_fds() -> &'static Mutex<CpuFds> {
    CPU_FDS.get_or_init(|| {
        let cores = get_core_count() as usize;


        // Try per-core path first, fall back to cpufreq policy directory
        let open_with_fallback = |core: usize, file: &str| -> Option<File> {
            let per_core = format!("/sys/devices/system/cpu/cpu{}/cpufreq/{}", core, file);
            if let Some(f) = File::open(&per_core).ok() {
                return Some(f);
            }
            let policy = format!("/sys/devices/system/cpu/cpufreq/policy{}/{}", core, file);
            File::open(&policy).ok()
        };

        let mut cur_freq = Vec::with_capacity(cores);
        let mut max_freq = Vec::with_capacity(cores);
        let mut min_freq = Vec::with_capacity(cores);
        let mut governor = Vec::with_capacity(cores);

        for i in 0..cores {
            cur_freq.push(open_with_fallback(i, "scaling_cur_freq"));
            max_freq.push(open_with_fallback(i, "cpuinfo_max_freq"));
            min_freq.push(open_with_fallback(i, "cpuinfo_min_freq"));
            governor.push(open_with_fallback(i, "scaling_governor"));
        }

        Mutex::new(CpuFds {
            cur_freq,
            max_freq,
            min_freq,
            governor,
        })
    })
}

static THERMAL_MAP: OnceCell<HashMap<String, PathBuf>> = OnceCell::new();

fn get_thermal_map() -> &'static HashMap<String, PathBuf> {
    THERMAL_MAP.get_or_init(|| {
        let mut map = HashMap::new();
        if let Ok(entries) = fs::read_dir("/sys/class/thermal") {
            for entry in entries.flatten() {
                let base = entry.file_name();
                let name = base.to_string_lossy();
                if !name.starts_with("thermal_zone") {
                    continue;
                }
                let type_path = entry.path().join("type");
                let temp_path = entry.path().join("temp");
                if let Ok(tz_type) = fs::read_to_string(&type_path) {
                    let key = tz_type.trim().to_lowercase();
                    log::debug!("thermal zone: {} -> {}", key, temp_path.display());
                    map.insert(key, temp_path);
                }
            }
        }
        if map.is_empty() {
            log::warn!("No thermal zones found in /sys/class/thermal");
        }
        map
    })
}

static CPU_THERMAL_FD: OnceCell<Mutex<Option<File>>> = OnceCell::new();
static GPU_THERMAL_FD: OnceCell<Mutex<Option<File>>> = OnceCell::new();

fn get_thermal_fd_from_priority(
    map: &HashMap<String, PathBuf>,
    priority: &[&str],
) -> Mutex<Option<File>> {
    let mut best_path = None;
    for zone in priority {
        if let Some(path) = map.get(*zone) {
            best_path = Some(path.clone());
            break;
        }
    }
    if best_path.is_none() {
        for (tz_type, temp_path) in map {
            if priority.iter().any(|p| tz_type.contains(p)) {
                best_path = Some(temp_path.clone());
                break;
            }
        }
    }
    let file = best_path.and_then(|p| File::open(p).ok());
    Mutex::new(file)
}

fn get_cpu_thermal_fd() -> &'static Mutex<Option<File>> {
    CPU_THERMAL_FD.get_or_init(|| {
        let map = get_thermal_map();
        let fd = get_thermal_fd_from_priority(
            map,
            &[
                // Generic
                "cpu-thermal",
                "soc-thermal",
                "cpu",
                "soc",
                "thermal-cpufreq",
                // MediaTek
                "mtktscpu",
                "mtktsap",
                "mtk_thermal",
                "cpu_big_thermal",
                "cpu_little_thermal",
                // Qualcomm
                "cpuss-0-usr",
                "cpuss-1-usr",
                "cpu-0-0-usr",
                "aoss0-usr",
                // Samsung Exynos
                "big_thermal",
                "little_thermal",
                "mid_thermal",
                // UNISOC
                "cluster0-thermal",
                "cluster1-thermal",
            ],
        );
        // Last-resort fallback: if no thermal zone was found, try thermal_zone0
        // which is typically the main CPU/SoC sensor on most Android devices
        if fd.lock().unwrap().is_none() {
            let fallback_path = PathBuf::from("/sys/class/thermal/thermal_zone0/temp");
            if fallback_path.exists() {
                return Mutex::new(File::open(fallback_path).ok());
            }
        }
        fd
    })
}

fn get_gpu_thermal_fd() -> &'static Mutex<Option<File>> {
    GPU_THERMAL_FD.get_or_init(|| {
        get_thermal_fd_from_priority(
            get_thermal_map(),
            &[
                // Generic
                "gpu-thermal",
                "gpu0-thermal",
                "gpu",
                // Qualcomm
                "gpuss-0-usr",
                "tsens_tz_sensor9",
                // MediaTek
                "mtkts_gpu",
                "gpu_thermal",
                // Samsung Exynos
                "g3d-thermal",
            ],
        )
    })
}

static CORE_THERMAL_FDS: OnceCell<Mutex<Vec<Option<File>>>> = OnceCell::new();

fn get_core_thermal_fds() -> &'static Mutex<Vec<Option<File>>> {
    CORE_THERMAL_FDS.get_or_init(|| {
        let cores = get_core_count() as usize;
        let map = get_thermal_map();
        let mut fds = Vec::with_capacity(cores);
        for i in 0..cores {
            let key = format!("cpu{}-thermal", i);
            let file = map.get(&key).and_then(|p| File::open(p).ok());
            fds.push(file);
        }
        Mutex::new(fds)
    })
}

pub fn get_core_count() -> i32 {
    if let Ok(content) = fs::read_to_string("/sys/devices/system/cpu/present") {
        let content = content.trim();
        if let Some((start_str, end_str)) = content.split_once('-') {
            let start: i32 = start_str.parse().unwrap_or(0);
            let end: i32 = end_str.parse().unwrap_or(0);
            return end - start + 1;
        }
        return content.split(',').count() as i32;
    }
    std::thread::available_parallelism()
        .map(|n| n.get() as i32)
        .unwrap_or(0)
}

fn read_temp(file_opt: &mut Option<File>, buf: &mut String) -> f64 {
    if let Some(file) = file_opt.as_mut()
        && let Some(temp) = read_fd_parsed::<f64>(file, buf)
    {
        return if temp > 1000.0 { temp / 1000.0 } else { temp };
    }
    0.0
}

pub fn get_core_frequency(core_id: i32, freq_type: &str) -> i64 {
    let core_idx = core_id as usize;
    let mut buf = String::with_capacity(32);

    let fds_mutex = get_cpu_fds();
    let mut fds = fds_mutex.lock().unwrap();

    let slot: Option<&mut Option<File>> = match freq_type {
        "max_info" => fds.max_freq.get_mut(core_idx),
        "min_info" => fds.min_freq.get_mut(core_idx),
        "cur" => fds.cur_freq.get_mut(core_idx),
        _ => None,
    };

    if let Some(Some(file)) = slot {
        return read_fd_parsed::<i64>(file, &mut buf).unwrap_or(0);
    }
    let file_name = match freq_type {
        "max_info" => "cpuinfo_max_freq",
        "min_info" => "cpuinfo_min_freq",
        "cur" => "scaling_cur_freq",
        _ => return 0,
    };
    let path = format!(
        "/sys/devices/system/cpu/cpu{}/cpufreq/{}",
        core_id, file_name
    );
    if let Some(val) = read_path_parsed::<i64>(&path, &mut buf) {
        return val;
    }
    // Fallback: cpufreq policy directory
    let policy_path = format!(
        "/sys/devices/system/cpu/cpufreq/policy{}/{}",
        core_id, file_name
    );
    read_path_parsed::<i64>(&policy_path, &mut buf).unwrap_or(0)
}

pub fn get_core_governor(core_id: i32) -> String {
    let core_idx = core_id as usize;
    let mut buf = String::with_capacity(32);

    let fds_mutex = get_cpu_fds();
    let mut fds = fds_mutex.lock().unwrap();

    if let Some(Some(file)) = fds.governor.get_mut(core_idx) {
        buf.clear();
        if file.seek(SeekFrom::Start(0)).is_ok() && file.read_to_string(&mut buf).is_ok() {
            let len = buf.trim_end().len();
            buf.truncate(len);
            return buf;
        }
    }

    // Fallback 1: direct path read (in case FD was stale)
    let path = format!(
        "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor",
        core_id
    );
    if let Ok(s) = fs::read_to_string(&path) {
        let trimmed = s.trim().to_string();
        if !trimmed.is_empty() {
            return trimmed;
        }
    }

    // Fallback 2: try reading from cpufreq policy directory
    // On some devices (MediaTek, UNISOC), per-core cpufreq dirs are symlinks
    // to shared policy directories which may have different SELinux labels
    let policy_path = format!(
        "/sys/devices/system/cpu/cpufreq/policy{}/scaling_governor",
        core_id
    );
    if let Ok(s) = fs::read_to_string(&policy_path) {
        let trimmed = s.trim().to_string();
        if !trimmed.is_empty() {
            return trimmed;
        }
    }

    "N/A".to_string()
}

pub fn get_cpu_temperature() -> f64 {
    let mut buf = String::with_capacity(16);
    let mut fd_mutex = get_cpu_thermal_fd().lock().unwrap();
    read_temp(&mut fd_mutex, &mut buf)
}

pub fn get_gpu_temperature() -> f64 {
    let mut buf = String::with_capacity(16);
    let mut fd_mutex = get_gpu_thermal_fd().lock().unwrap();
    read_temp(&mut fd_mutex, &mut buf)
}

pub fn get_core_temperature(core_id: i32) -> f64 {
    let mut buf = String::with_capacity(16);
    let mut fds_mutex = get_core_thermal_fds().lock().unwrap();
    if let Some(slot) = fds_mutex.get_mut(core_id as usize) {
        let temp = read_temp(slot, &mut buf);
        if temp != 0.0 {
            return temp;
        }
    }
    get_cpu_temperature()
}

#[derive(Default, Clone, Copy)]
struct CpuTicks {
    user: u64,
    nice: u64,
    system: u64,
    idle: u64,
    iowait: u64,
    irq: u64,
    softirq: u64,
    steal: u64,
}

impl CpuTicks {
    fn total(&self) -> u64 {
        self.user
            + self.nice
            + self.system
            + self.idle
            + self.iowait
            + self.irq
            + self.softirq
            + self.steal
    }

    fn idle_total(&self) -> u64 {
        self.idle + self.iowait
    }
}

static LAST_CPU_TICKS: OnceCell<Mutex<Vec<Option<CpuTicks>>>> = OnceCell::new();

pub fn calculate_cpu_load(proc_stat: &str) -> Vec<f64> {
    let cores = get_core_count() as usize;
    let mut current_ticks = vec![None; cores + 1];
    let mut results = Vec::with_capacity(cores + 1);

    if proc_stat.is_empty() {
        return results;
    }

    for line in proc_stat.lines() {
        if line.starts_with("cpu") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let name = parts[0];

                let idx = if name == "cpu" {
                    0
                } else if let Ok(core_id) = name[3..].parse::<usize>() {
                    core_id + 1
                } else {
                    continue;
                };

                if idx > cores {
                    continue;
                }

                let user = parts[1].parse::<u64>().unwrap_or(0);
                let nice = parts[2].parse::<u64>().unwrap_or(0);
                let system = parts[3].parse::<u64>().unwrap_or(0);
                let idle = parts[4].parse::<u64>().unwrap_or(0);
                let iowait = parts
                    .get(5)
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
                let irq = parts
                    .get(6)
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
                let softirq = parts
                    .get(7)
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
                let steal = parts
                    .get(8)
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);

                current_ticks[idx] = Some(CpuTicks {
                    user,
                    nice,
                    system,
                    idle,
                    iowait,
                    irq,
                    softirq,
                    steal,
                });
            }
        }
    }

    let last_ticks_mutex = LAST_CPU_TICKS.get_or_init(|| Mutex::new(vec![None; cores + 1]));
    let mut last_ticks = last_ticks_mutex.lock().unwrap();

    // Process each entry (0 is total, 1..=cores are individual cores)
    for i in 0..=cores {
        if let Some(curr) = current_ticks[i] {
            if let Some(prev) = last_ticks.get(i).and_then(|x| *x) {
                let total_diff = curr.total().saturating_sub(prev.total());
                let idle_diff = curr.idle_total().saturating_sub(prev.idle_total());

                if total_diff > 0 {
                    let load = (total_diff - idle_diff) as f64 * 100.0 / total_diff as f64;
                    results.push(load.clamp(0.0, 100.0));
                } else {
                    results.push(0.0);
                }
            } else {
                results.push(-1.0); // Signal: first poll
            }
        } else {
            results.push(0.0); // Core offline or total missing
        }
    }

    *last_ticks = current_ticks;
    results
}
