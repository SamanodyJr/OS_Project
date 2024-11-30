// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::thread;
use std::time::Duration;
use std::path::Path;
use std::fs::File;
use std::io::{self, BufRead};
use tauri::command;
use sysinfo::{System, SystemExt, RefreshKind};

#[derive(Serialize, Clone, Default, Debug)]
pub struct CpuUsage {
    cpu_usage: f64,
    core_number: i32,
}

#[derive(Serialize, Clone, Default, Debug)]
pub struct MemoryUsage {
    pub used: f64,
    pub free: f64,
    pub total: f64,
    pub used_swap: f64,
    pub free_swap: f64,
    pub total_swap: f64,
}

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
        } else {
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

#[command]
fn cpu_resultt() -> Vec<CpuUsage> {
    let mut cpu_usages = Vec::new();
    let prev_cpu_stats = read_cpu_stat().unwrap();
    thread::sleep(Duration::from_secs(1));
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

#[command]
fn Mem_Usage() -> MemoryUsage {
    let mut sys = System::new_with_specifics(RefreshKind::new().with_memory());

    sys.refresh_memory();

    let total_memory = sys.total_memory() as f64;
    let used_memory = sys.used_memory() as f64;
    let free_memory = sys.free_memory() as f64;

    let total_swap = sys.total_swap() as f64;
    let used_swap = sys.used_swap() as f64;
    let free_swap = sys.free_swap() as f64;

    MemoryUsage {
        used: used_memory / 1024.0 / 1024.0 / 1024.0,
        free: free_memory / 1024.0 / 1024.0 / 1024.0,
        total: total_memory / 1024.0 / 1024.0 / 1024.0,
        used_swap: used_swap / 1024.0 / 1024.0 / 1024.0,
        free_swap: free_swap / 1024.0 / 1024.0 / 1024.0,
        total_swap: total_swap / 1024.0 / 1024.0 / 1024.0,
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![cpu_resultt, Mem_Usage])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}