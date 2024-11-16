use sysinfo::{System, Disk};
use std::thread::sleep;
use std::time::Duration;
use std::path::Path;
use std::fs::File;
use std::io::{self, BufRead};



pub fn read_cpu_stat() -> io::Result<Vec<Vec<u64>>> {
    let path = Path::new("/proc/stat");
    let file = File::open(&path)?;
    let reader = io::BufReader::new(file);

    let mut cpu_stats = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.starts_with("cpu") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let values: Vec<u64> = parts[1..]
                .iter()
                .map(|v| v.parse().unwrap_or(0))
                .collect();
            cpu_stats.push(values);
        }
        else {
            break;
        }
    }
    Ok(cpu_stats)
}

pub fn calculate_cpu_usage(prev: &[u64], curr: &[u64]) -> f64 {
    let prev_idle = prev[3] + prev[4];
    let curr_idle = curr[3] + curr[4];

    let prev_total: u64 = prev.iter().sum();
    let curr_total: u64 = curr.iter().sum();

    let total_diff = curr_total - prev_total;
    let idle_diff = curr_idle - prev_idle;

    100.0 * (total_diff - idle_diff) as f64 / total_diff as f64
}

pub struct CpuUsage {
    pub cpu_usage: f64,
    pub core_number: i32,
}

pub fn cpu_result() -> Vec<CpuUsage> {
    let mut cpu_usages = Vec::new();
    let mut prev_cpu_stats = read_cpu_stat().unwrap();
    sleep(Duration::from_secs(1));
    let curr_cpu_stats = read_cpu_stat().unwrap();

    for (i, (prev, curr)) in prev_cpu_stats.iter().zip(curr_cpu_stats.iter()).enumerate().skip(1) {
        let cpu_usage = calculate_cpu_usage(prev, curr);
        cpu_usages.push(CpuUsage {
            cpu_usage,
            core_number: i as i32 - 1, // Adjust core number to start from 0 for the first core
        });
    }
    cpu_usages
}