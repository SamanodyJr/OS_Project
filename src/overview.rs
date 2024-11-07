use procfs::process::all_processes;
use procfs::{ticks_per_second,Uptime};


pub fn print_process() {
    match all_processes() {
        Ok(processes) => {
            let ticks_per_second = ticks_per_second().unwrap() as f64;
            let system_uptime = Uptime::new().unwrap().uptime; 

            for process in processes {
                if let Ok(proc) = process {
                    if let Ok(stat) = proc.stat(){
                        let pid = proc.pid;
                        let command = stat.comm.clone();
                        let page_size: f64 = procfs::page_size().unwrap() as f64;
                        let memory: f64 = stat.rss as f64 * page_size / 1024.0 / 1024.0;

                        let utime = stat.utime as f64; // User time
                        let stime = stat.stime as f64; // Kernel time
                        let cutime = stat.cutime as f64; // CPU time in user mode for waited-for children processes
                        let cstime = stat.cstime as f64; // CPU time in kernel mode for waited-for children processes

                        let time = (utime + stime + cutime + cstime) as f64;
                        let process_start_time = stat.starttime as f64 / ticks_per_second;
                        let elapsed_time = system_uptime - process_start_time; 
                        let cpu_usage_percentage = 100.0 * (time / ticks_per_second) / elapsed_time;
                        println!("PID: {}, Command: {}, Memory: {:.2} MB, CPU Usage: {:.2}%", pid, command, memory, cpu_usage_percentage);
                }
            }
            }
        }
        Err(e) => {
            eprintln!("Failed to get processes: {}", e);
        }
    }
}