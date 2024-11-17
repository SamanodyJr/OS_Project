use procfs::process::all_processes;
use procfs::{ticks_per_second,Uptime};
use sysinfo::{System, SystemExt};
use users::get_user_by_uid;
use std::fmt::Write;
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Duration;


pub struct ProcessInfo {
    pid: i32,
    command: String,
    user: String,
    v_memory: f64, 
    rss_memory: f64,
    shared_memory: f64, 
    memory_uasge: f64,
    cpu_usage: f64,
    time: String,
    priority: i64,
    nice: i64,
    ppid: i32,
    state: String,
    threads: i64,
}

fn seconds_to_hhmmss(seconds: f64) -> String {
    let scnds = seconds as u64;
    let hours = scnds / 3600;
    let minutes = (scnds % 3600) / 60;
    let secs = scnds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, secs)
}

pub fn get_processes_info() -> Vec<ProcessInfo> {
    let mut processes_info = Vec::new();
    let mut system = System::new_all();
    system.refresh_all();

    match all_processes() {
        Ok(processes) => {
            let ticks_per_second = ticks_per_second().unwrap() as f64;
            let system_uptime = Uptime::new().unwrap().uptime; 
            // let total_cpu_time: f64 = stat().unwrap().cpu().cpu_time() as f64 / ticks_per_second;
            let total_memory = system.total_memory();

            for process in processes {
                if let Ok(proc) = process {
                    if let Ok(stat) = proc.stat(){
                        let pid: i32 = proc.pid as i32;
                        let uid: u32 = proc.uid().unwrap() as u32;
                        let user = get_user_by_uid(uid).unwrap().name().to_str().map(|s| s.to_string()).unwrap();
                        let command = stat.comm.clone();
                        
                        let page_size: f64 = procfs::page_size().unwrap() as f64;
                        let v_memory: f64 = stat.vsize as f64 / (1024.0 * 1024.0);
                        let rss_memory: f64 = (stat.rss as f64 * page_size) / (1024.0 * 1024.0);
                        // let shared_memory: f64 = (proc.statm().unwrap().shared as f64 * page_size) / (1024.0 * 1024.0);
                        let shared_memory: f64 = match proc.statm() {
                            Ok(statm) => {
                                if statm.shared < 0 {
                                    0.0 // If shared memory is negative, treat it as 0
                                } else {
                                    (statm.shared as f64 * page_size) / (1024.0 * 1024.0) // Convert shared memory to MB
                                }
                            }
                            Err(_) => {
                                0.0 // If statm fails, assume no shared memory available
                            }
                        };
                        // let disk_read: f64 = proc.io().unwrap().read_bytes as f64 / (1024.0 * 1024.0);
                        // let disk_write: f64 = proc.io().unwrap().write_bytes as f64 / (1024.0 * 1024.0);
                        

                        let memory_uasge: f64 = 100.0 * ((rss_memory * (1024.0 * 1024.0)) / total_memory as f64);

                        let utime = stat.utime as f64 / ticks_per_second; // User time
                        let stime = stat.stime as f64 / ticks_per_second; // Kernel time
                        let cutime = stat.cutime as f64 / ticks_per_second; // CPU time in user mode for waited-for children processes
                        let cstime = stat.cstime as f64 / ticks_per_second; // CPU time in kernel mode for waited-for children processes
                

                        let process_start_time = stat.starttime as f64 / ticks_per_second;
                        let elapsed_time = system_uptime - process_start_time; 
                        let proc_usage_time = utime + stime + cutime + cstime;

                        let cpu_usage: f64 = 100.0 * (proc_usage_time / elapsed_time);

                        let priority = stat.priority;
                        let nice = stat.nice;
                        let ppid = stat.ppid; 
                        let state = stat.state.to_string(); 
                        let threads = stat.num_threads; 

                        let time = seconds_to_hhmmss(proc_usage_time);

                        processes_info.push(ProcessInfo {
                            pid,
                            command,
                            user,
                            v_memory, 
                            rss_memory, 
                            shared_memory,
                            memory_uasge,
                            cpu_usage,
                            time,
                            priority,
                            nice,
                            ppid,
                            state,
                            threads,
                        });
                }
            }
            
        }
    }
    Err(e) => {
        eprintln!("Failed to get processes: {}", e);
    }

    }
        return processes_info;
}
#[derive(Clone)]
pub struct Process {
    pub pid: i32,
    pub user: String,
    pub command: String,
    pub v_memory: f64,
    pub rss_memory: f64,
    pub shared_memory: f64,
    pub memory_uasge: f64,
    pub cpu_usage: f64,
    pub time: String,
    pub priority: i64,
    pub nice: i64,
    pub ppid: i32,
    pub state: String,
    pub threads: i64,
}

impl From<&ProcessInfo> for Process {
    fn from(info: &ProcessInfo) -> Self {
        Process {
            pid: info.pid,
            user: info.user.clone(),
            command: info.command.clone(),
            v_memory: info.v_memory,
            rss_memory: info.rss_memory,
            shared_memory: info.shared_memory,
            memory_uasge: info.memory_uasge,
            cpu_usage: info.cpu_usage,
            time: info.time.clone(),
            priority: info.priority,
            nice: info.nice,
            ppid: info.ppid,
            state: info.state.clone(),
            threads: info.threads,
        }
    }
}

pub fn get_processes() -> Vec<Process> {
    let processes_info = get_processes_info();
    processes_info.iter().map(Process::from).collect()
}

pub fn print_process() -> String {
    let processes_info = get_processes_info();
    let mut output = String::new();

    for process in processes_info {
        let _ = write!(
            output,
            "PID: {} | User: {} | Command: {} | Virtual Memory: {:.2} MB | RSS Memory: {:.2} MB | Shared Memory: {:.2} MB | Memory Usage: {:.2}% | CPU Usage: {:.2}% | Time: {} | Priority: {} | Nice: {} | Parent PID: {} | State: {} | Threads: {}\n",
            process.pid,
            process.user,
            process.command,
            process.v_memory,
            process.rss_memory,
            process.shared_memory,
            process.memory_uasge,
            process.cpu_usage,
            process.time,
            process.priority,
            process.nice,
            process.ppid,
            process.state,
            process.threads
        );
    }

    output
}

pub fn start_background_update(process_data: Arc<Mutex<Vec<Process>>>) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        // Update process data every second

        // Lock ProcessData and update it
        let new_data = get_processes();

        // Lock the mutex and replace its contents
        let mut data = process_data.lock().unwrap();
        *data = new_data;

        
    });
}