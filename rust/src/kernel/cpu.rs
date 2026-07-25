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

        let open_opt = |paths: &[String]| -> Option<File> {
            paths.iter().find_map(|p| File::open(p).ok())
        };

        let mut cur_freq = Vec::with_capacity(cores);
        let mut max_freq = Vec::with_capacity(cores);
        let mut min_freq = Vec::with_capacity(cores);
        let mut governor = Vec::with_capacity(cores);

        for i in 0..cores {
            cur_freq.push(open_opt(&[
                format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_cur_freq", i),
                format!("/sys/devices/system/cpu/cpufreq/policy{}/scaling_cur_freq", i),
                format!("/sys/devices/system/cpu/cpu{}/cpufreq/cpuinfo_cur_freq", i),
            ]));
            max_freq.push(open_opt(&[
                format!("/sys/devices/system/cpu/cpu{}/cpufreq/cpuinfo_max_freq", i),
                format!("/sys/devices/system/cpu/cpufreq/policy{}/cpuinfo_max_freq", i),
                format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_max_freq", i),
                format!("/sys/devices/system/cpu/cpufreq/policy{}/scaling_max_freq", i),
            ]));
            min_freq.push(open_opt(&[
                format!("/sys/devices/system/cpu/cpu{}/cpufreq/cpuinfo_min_freq", i),
                format!("/sys/devices/system/cpu/cpufreq/policy{}/cpuinfo_min_freq", i),
                format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_min_freq", i),
                format!("/sys/devices/system/cpu/cpufreq/policy{}/scaling_min_freq", i),
            ]));
            governor.push(open_opt(&[
                format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor", i),
                format!("/sys/devices/system/cpu/cpufreq/policy{}/scaling_governor", i),
            ]));
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
                    map.insert(tz_type.trim().to_lowercase(), temp_path);
                }
            }
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
        for zone in priority {
            if let Some((_, path)) = map.iter().find(|(k, _)| k.contains(*zone)) {
                best_path = Some(path.clone());
                break;
            }
        }
    }
    let file = best_path.and_then(|p| File::open(p).ok());
    Mutex::new(file)
}

fn get_cpu_thermal_fd() -> &'static Mutex<Option<File>> {
    CPU_THERMAL_FD.get_or_init(|| {
        get_thermal_fd_from_priority(
            get_thermal_map(),
            &[
                "soc_max",
                "soc_thermal",
                "soc-thermal",
                "cpu_max",
                "cpu-thermal",
                "msm_therm",
                "mtktsap",
                "ap_ntc",
                "cpu",
                "soc",
                "tsens_tz_sensor0",
                "thermal-cpufreq",
            ],
        )
    })
}

fn get_gpu_thermal_fd() -> &'static Mutex<Option<File>> {
    GPU_THERMAL_FD.get_or_init(|| {
        get_thermal_fd_from_priority(
            get_thermal_map(),
            &[
                "gpu-thermal",
                "gpu0-thermal",
                "gpuss-0-usr",
                "gpu",
                "tsens_tz_sensor9",
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

        let mut qc_zones: Vec<(i32, i32, String)> = Vec::new();
        for key in map.keys() {
            if key.starts_with("cpu-") {
                // Format: cpu-C-N-S or cpu-<type>-core<N>
                let parts: Vec<&str> = key.split('-').collect();
                if parts.len() >= 3 {
                    if let (Ok(c), Ok(n)) = (parts[1].parse::<i32>(), parts[2].parse::<i32>()) {
                        qc_zones.push((c, n, key.clone()));
                    } else {
                        // Dimensity format: cpu-little-core0
                        let cluster_type = parts[1];
                        let core_str = parts[2];
                        if core_str.starts_with("core") {
                            if let Ok(n) = core_str[4..].parse::<i32>() {
                                let c = match cluster_type {
                                    "little" => 0,
                                    "medium" => 1,
                                    "big" => 2,
                                    "prime" => 3,
                                    _ => -1,
                                };
                                if c != -1 {
                                    qc_zones.push((c, n, key.clone()));
                                }
                            }
                        }
                    }
                }
            } else if key.starts_with("cpu_") {
                // Format: cpu_littleN, cpu_bigN
                let is_little = key.starts_with("cpu_little");
                let is_big = key.starts_with("cpu_big");
                if is_little || is_big {
                    let prefix_len = if is_little { "cpu_little".len() } else { "cpu_big".len() };
                    if let Ok(n) = key[prefix_len..].parse::<i32>() {
                        let core_idx = n - 1;
                        let c = if is_little { 0 } else { 1 };
                        qc_zones.push((c, core_idx, key.clone()));
                    }
                }
            } else if key.starts_with("cpu") {
                // Format: cpuN-silver-S, cpuN-gold-S
                if let Some(dash_idx) = key.find('-') {
                    if let Ok(n) = key[3..dash_idx].parse::<i32>() {
                        let rest = &key[dash_idx + 1..];
                        let c = if rest.starts_with("silver") || rest.starts_with("little") {
                            0
                        } else if rest.starts_with("gold") || rest.starts_with("big") {
                            1
                        } else if rest.starts_with("prime") {
                            2
                        } else {
                            -1
                        };
                        if c != -1 {
                            qc_zones.push((c, n, key.clone()));
                        }
                    }
                }
            } else if key.starts_with("tsens_tz_sensor") {
                if let Ok(n) = key[15..].parse::<i32>() {
                    let core_idx = if n >= 1 && n <= 8 {
                        n - 1
                    } else if n == 0 {
                        99
                    } else {
                        n
                    };
                    qc_zones.push((99, core_idx, key.clone()));
                }
            }
        }

        let mut unique_cn: HashMap<(i32, i32), String> = HashMap::new();
        for (c, n, key) in qc_zones {
            if !unique_cn.contains_key(&(c, n)) 
                || key.ends_with("-0") 
                || key.ends_with("-0-0")
                || key.ends_with("-usr") 
            {
                unique_cn.insert((c, n), key);
            }
        }

        let mut sorted_cn: Vec<(&(i32, i32), &String)> = unique_cn.iter().collect();
        sorted_cn.sort_by(|a, b| {
            if a.0.0 != b.0.0 {
                a.0.0.cmp(&b.0.0)
            } else {
                a.0.1.cmp(&b.0.1)
            }
        });

        for i in 0..cores {
            if i < sorted_cn.len() {
                let key = sorted_cn[i].1;
                fds.push(map.get(key).and_then(|p| File::open(p).ok()));
            } else {
                fds.push(None);
            }
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
    let file_names = match freq_type {
        "max_info" => vec!["cpuinfo_max_freq", "scaling_max_freq"],
        "min_info" => vec!["cpuinfo_min_freq", "scaling_min_freq"],
        "cur" => vec!["scaling_cur_freq", "cpuinfo_cur_freq"],
        _ => return 0,
    };

    for name in file_names {
        let path1 = format!("/sys/devices/system/cpu/cpu{}/cpufreq/{}", core_id, name);
        let path2 = format!("/sys/devices/system/cpu/cpufreq/policy{}/{}", core_id, name);
        if let Some(val) = read_path_parsed::<i64>(&path1, &mut buf).or_else(|| read_path_parsed::<i64>(&path2, &mut buf)) {
            return val;
        }
    }
    
    0
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

    // Fallback
    let path1 = format!(
        "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor",
        core_id
    );
    let path2 = format!(
        "/sys/devices/system/cpu/cpufreq/policy{}/scaling_governor",
        core_id
    );
    fs::read_to_string(&path1)
        .or_else(|_| fs::read_to_string(&path2))
        .map(|mut s| {
            let l = s.trim_end().len();
            s.truncate(l);
            s
        })
        .unwrap_or_else(|_| "N/A".to_string())
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
