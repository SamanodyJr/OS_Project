use procfs::process::all_processes;

fn main() {
    match all_processes() {
        Ok(processes) => {
            for process in processes {
                if let Ok(proc) = process {
                    // Get the process ID and command
                    let pid = proc.pid;
                    let command = proc.stat().unwrap().comm;
                    let pageSize: f64 = procfs::page_size().unwrap() as f64;
                    let memory: f64 = proc.stat().unwrap().rss as f64 * pageSize / 1024.0 / 1024.0;
                    // let name = proc.stat().comm;

                    println!("PID: {}, Name: {}, Command: {}, Memory: {}", pid, name, command, memory);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to get processes: {}", e);
        }
    }
}
