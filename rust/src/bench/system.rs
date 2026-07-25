use std::fs;

pub struct SystemSnapshot {
    pub rss_kb: i64,
    pub ctx_voluntary: u64,
    pub ctx_involuntary: u64,
    pub core_affinity: String,
    pub thermal_throttle: bool,
}

impl SystemSnapshot {
    pub fn capture() -> Self {
        Self {
            rss_kb: read_rss_kb(),
            ctx_voluntary: read_ctx_switches().0,
            ctx_involuntary: read_ctx_switches().1,
            core_affinity: read_core_affinity(),
            thermal_throttle: false,
        }
    }

    pub fn delta(&self, prev: &SystemSnapshot) -> SystemDelta {
        SystemDelta {
            rss_delta_kb: self.rss_kb - prev.rss_kb,
            ctx_voluntary_delta: self.ctx_voluntary.saturating_sub(prev.ctx_voluntary),
            ctx_involuntary_delta: self.ctx_involuntary.saturating_sub(prev.ctx_involuntary),
        }
    }
}

pub struct SystemDelta {
    pub rss_delta_kb: i64,
    pub ctx_voluntary_delta: u64,
    pub ctx_involuntary_delta: u64,
}

fn read_rss_kb() -> i64 {
    let content = fs::read_to_string("/proc/self/status").unwrap_or_default();
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            let trimmed = rest.trim_start();
            let num_str: String = trimmed.chars().take_while(|c| c.is_ascii_digit()).collect();
            return num_str.parse().unwrap_or(0);
        }
    }
    0
}

fn read_ctx_switches() -> (u64, u64) {
    let content = fs::read_to_string("/proc/self/stat").unwrap_or_default();
    let fields: Vec<&str> = content.split_whitespace().collect();
    if fields.len() > 44 {
        let voluntary = fields[36].parse().unwrap_or(0);
        let involuntary = fields[37].parse().unwrap_or(0);
        return (voluntary, involuntary);
    }
    (0, 0)
}

fn read_core_affinity() -> String {
    let content = fs::read_to_string("/proc/self/status").unwrap_or_default();
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("Cpus_allowed_list:") {
            return rest.trim().to_string();
        }
    }
    "unknown".to_string()
}
