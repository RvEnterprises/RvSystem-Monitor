//! # CPU Data Provider
//!
//! Provides functions to read and parse CPU information from the system.

use once_cell::sync::OnceCell;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
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

        let open_opt =
            |paths: &[String]| -> Option<File> { paths.iter().find_map(|p| File::open(p).ok()) };

        let mut cur_freq = Vec::with_capacity(cores);
        let mut max_freq = Vec::with_capacity(cores);
        let mut min_freq = Vec::with_capacity(cores);
        let mut governor = Vec::with_capacity(cores);

        for i in 0..cores {
            cur_freq.push(open_opt(&[
                format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_cur_freq", i),
                format!(
                    "/sys/devices/system/cpu/cpufreq/policy{}/scaling_cur_freq",
                    i
                ),
                format!("/sys/devices/system/cpu/cpu{}/cpufreq/cpuinfo_cur_freq", i),
            ]));
            max_freq.push(open_opt(&[
                format!("/sys/devices/system/cpu/cpu{}/cpufreq/cpuinfo_max_freq", i),
                format!(
                    "/sys/devices/system/cpu/cpufreq/policy{}/cpuinfo_max_freq",
                    i
                ),
                format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_max_freq", i),
                format!(
                    "/sys/devices/system/cpu/cpufreq/policy{}/scaling_max_freq",
                    i
                ),
            ]));
            min_freq.push(open_opt(&[
                format!("/sys/devices/system/cpu/cpu{}/cpufreq/cpuinfo_min_freq", i),
                format!(
                    "/sys/devices/system/cpu/cpufreq/policy{}/cpuinfo_min_freq",
                    i
                ),
                format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_min_freq", i),
                format!(
                    "/sys/devices/system/cpu/cpufreq/policy{}/scaling_min_freq",
                    i
                ),
            ]));
            governor.push(open_opt(&[
                format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor", i),
                format!(
                    "/sys/devices/system/cpu/cpufreq/policy{}/scaling_governor",
                    i
                ),
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
        if let Some(val) = read_path_parsed::<i64>(&path1, &mut buf)
            .or_else(|| read_path_parsed::<i64>(&path2, &mut buf))
        {
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

pub fn calculate_cpu_load(proc_stat: &str) -> Vec<f64> {
    let cores = get_core_count() as usize;

    if proc_stat.is_empty() {
        return Vec::new();
    }

    thread_local! {
        static TICKS_BUF: std::cell::RefCell<Vec<Option<CpuTicks>>> =
            std::cell::RefCell::new(Vec::new());
        static LAST_TICKS: std::cell::RefCell<Vec<Option<CpuTicks>>> =
            std::cell::RefCell::new(Vec::new());
    }

    TICKS_BUF.with(|buf| {
        let mut buf = buf.borrow_mut();
        buf.clear();
        buf.resize(cores + 1, None);

        LAST_TICKS.with(|last| {
            let mut last = last.borrow_mut();
            last.resize(cores + 1, None);

            for line in proc_stat.lines() {
                if !line.starts_with("cpu") {
                    continue;
                }

                let mut iter = line.split_whitespace();
                let name = match iter.next() {
                    Some(n) => n,
                    None => continue,
                };

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

                let mut ticks = CpuTicks::default();
                ticks.user = iter.next().and_then(|s| s.parse().ok()).unwrap_or(0);
                ticks.nice = iter.next().and_then(|s| s.parse().ok()).unwrap_or(0);
                ticks.system = iter.next().and_then(|s| s.parse().ok()).unwrap_or(0);
                ticks.idle = iter.next().and_then(|s| s.parse().ok()).unwrap_or(0);
                ticks.iowait = iter.next().and_then(|s| s.parse().ok()).unwrap_or(0);
                ticks.irq = iter.next().and_then(|s| s.parse().ok()).unwrap_or(0);
                ticks.softirq = iter.next().and_then(|s| s.parse().ok()).unwrap_or(0);
                ticks.steal = iter.next().and_then(|s| s.parse().ok()).unwrap_or(0);
                buf[idx] = Some(ticks);
            }

            let mut results = Vec::with_capacity(cores + 1);
            for i in 0..=cores {
                if let Some(curr) = buf[i] {
                    if let Some(prev) = last[i] {
                        let total_diff = curr.total().saturating_sub(prev.total());
                        let idle_diff = curr.idle_total().saturating_sub(prev.idle_total());
                        if total_diff > 0 {
                            let load = (total_diff - idle_diff) as f64 * 100.0 / total_diff as f64;
                            results.push(load.clamp(0.0, 100.0));
                        } else {
                            results.push(0.0);
                        }
                    } else {
                        results.push(-1.0);
                    }
                } else {
                    results.push(0.0);
                }
            }

            std::mem::swap(&mut *last, &mut *buf);
            results
        })
    })
}
