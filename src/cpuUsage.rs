use sysinfo::{System, Disk};
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let mut sys = System::new_all();

    // Initial refresh to populate data
    sys.refresh_all();

    // Sample CPU usage over 5 intervals
    let num_samples = 7;
    let interval = Duration::from_millis(300); // 200 ms interval between samples
    let mut total_cpu_usage = vec![0.0; sys.cpus().len()];

    for _ in 0..num_samples {
        // Refresh CPU usage
        sys.refresh_cpu_usage();
        
        // Accumulate CPU usage for each core
        for (i, cpu) in sys.cpus().iter().enumerate() {
            total_cpu_usage[i] += cpu.cpu_usage();
        }
        
        // Wait for the next sampling interval
        sleep(interval);
    }

    // Calculate and print the average CPU usage for each core
    for (i, cpu) in sys.cpus().iter().enumerate() {
        let avg_usage = total_cpu_usage[i] / num_samples as f32;
        println!("CPU: {} : {:.2}% (average over {} samples)", cpu.name(), avg_usage, num_samples);
    }
}








use sysinfo::{System, Disks};

use procfs::process::Process;


fn main() {
    let mut sys = System::new_all();
    sys.refresh_all();
    let process = Process::myself().expect("Count not get current process");
    let io = process.io().expect("Could not get I/O Stats");
    println!("Read MB: {}\n\n", io.rchar as f32 / 1024.0 / 1024.0); 
    println!("=> disks:");
    let disks = Disks::new_with_refreshed_list();
    for disk in &disks 
    {
        println!("{disk:?}");
    }


}


fn read_cpu_stat() -> io::Result<Vec<Vec<u64>>> {
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

fn calculate_cpu_usage(prev: &[u64], curr: &[u64]) -> f64 {
    let prev_idle = prev[3] + prev[4];
    let curr_idle = curr[3] + curr[4];

    let prev_total: u64 = prev.iter().sum();
    let curr_total: u64 = curr.iter().sum();

    let total_diff = curr_total - prev_total;
    let idle_diff = curr_idle - prev_idle;

    100.0 * (total_diff - idle_diff) as f64 / total_diff as f64
}
