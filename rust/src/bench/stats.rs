use std::time::Duration;

pub struct LatencyStats {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub p50: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
    pub stddev: f64,
}

impl LatencyStats {
    pub fn from_durations(samples: &[Duration]) -> Self {
        if samples.is_empty() {
            return Self { min: 0.0, max: 0.0, mean: 0.0, p50: 0.0, p90: 0.0, p95: 0.0, p99: 0.0, stddev: 0.0 };
        }

        let mut us: Vec<f64> = samples.iter().map(|d| d.as_secs_f64() * 1_000_000.0).collect();
        us.sort_by(|a, b| a.partial_cmp(b).unwrap());

        Self {
            min: us[0],
            max: us[us.len() - 1],
            mean: mean(&us),
            p50: percentile(&us, 0.50),
            p90: percentile(&us, 0.90),
            p95: percentile(&us, 0.95),
            p99: percentile(&us, 0.99),
            stddev: stddev(&us),
        }
    }

    pub fn from_micros(samples: &[f64]) -> Self {
        if samples.is_empty() {
            return Self { min: 0.0, max: 0.0, mean: 0.0, p50: 0.0, p90: 0.0, p95: 0.0, p99: 0.0, stddev: 0.0 };
        }

        let mut sorted = samples.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        Self {
            min: sorted[0],
            max: sorted[sorted.len() - 1],
            mean: mean(&sorted),
            p50: percentile(&sorted, 0.50),
            p90: percentile(&sorted, 0.90),
            p95: percentile(&sorted, 0.95),
            p99: percentile(&sorted, 0.99),
            stddev: stddev(&sorted),
        }
    }
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = (p * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn mean(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    data.iter().sum::<f64>() / data.len() as f64
}

fn stddev(data: &[f64]) -> f64 {
    if data.len() < 2 {
        return 0.0;
    }
    let m = mean(data);
    let variance = data.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (data.len() - 1) as f64;
    variance.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats() {
        let samples: Vec<f64> = (1..=100).map(|x| x as f64).collect();
        let stats = LatencyStats::from_micros(&samples);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 100.0);
        assert!((stats.mean - 50.5).abs() < 0.01);
        assert!((stats.p50 - 50.0).abs() < 1.0);
        assert!((stats.p99 - 99.0).abs() < 2.0);
    }
}
