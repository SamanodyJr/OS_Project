#![allow(non_snake_case)]

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread;

#[derive(Default)]
pub struct DiskUsage {
    pub device_name: String,
    pub reads_completed: u64,
    pub time_reading: u64,
    pub writes_completed: u64,
    pub time_writing: u64,
    pub io_in_progress: u64,
    pub time_io: u64,
}




pub fn Disk_Usage() -> DiskUsage 
{
    let root_disk = get_root_disk().unwrap();
    let root_disk_without_partition = strip_partition_suffix(&root_disk);

    let root_disk_name = root_disk_without_partition.strip_prefix("/dev/").unwrap_or(&root_disk_without_partition);

    let file_path = "/proc/diskstats";
    let path = Path::new(file_path);

    
        let file = File::open(path).unwrap();
        let reader = io::BufReader::new(file);
        let mut disk_usage = DiskUsage {
            device_name: root_disk_name.to_string(),
            reads_completed: 0,
            time_reading: 0,
            writes_completed: 0,
            time_writing: 0,
            io_in_progress: 0,
            time_io: 0,
        };

        for line in reader.lines() 
        {
            let line = line.unwrap();
            let fields: Vec<&str> = line.split_whitespace().collect();

            if fields.len() >= 14 
            {
                let device_name = fields[2];

                if device_name == root_disk_name 
                {
                    let reads_completed = fields[3];
                    let time_reading = fields[6];
                    let writes_completed = fields[7];
                    let time_writing = fields[10];
                    let io_in_progress = fields[11];
                    let time_io = fields[12];

                        disk_usage.device_name = device_name.to_string();
                        disk_usage.reads_completed = reads_completed.parse().unwrap();
                        disk_usage.time_reading = time_reading.parse().unwrap();
                        disk_usage.writes_completed = writes_completed.parse().unwrap();
                        disk_usage.time_writing = time_writing.parse().unwrap();
                        disk_usage.io_in_progress = io_in_progress.parse().unwrap();
                        disk_usage.time_io = time_io.parse().unwrap();
                    break;

                    // println!("Disk Stats for root device: {}", device_name);
                    // println!("------------------------------------");
                    // println!("  Reads Completed: {}", reads_completed);
                    // println!("  Time Reading (ms): {}", time_reading);
                    // println!("  Writes Completed: {}", writes_completed);
                    // println!("  Time Writing (ms): {}", time_writing);
                    // println!("  IO In Progress: {}", io_in_progress);
                    // println!("  Time IO (ms): {}", time_io);
                    // println!();
                }
            }
        }
        disk_usage
}

fn get_root_disk() -> io::Result<String> 
{
    let file = File::open("/proc/mounts")?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() 
    {
        let line = line?;
        let fields: Vec<&str> = line.split_whitespace().collect();

        if fields.len() >= 2 && fields[1] == "/" 
        {
            let root_disk = fields[0].to_string();
            // println!("Found root disk: {}", root_disk);
            return Ok(root_disk);
        }
    }

    Err(io::Error::new(io::ErrorKind::NotFound, "Root filesystem not found"))
}

fn strip_partition_suffix(device: &str) -> String 
{
    if let Some(pos) = device.rfind('p') 
    {
        if device[pos + 1..].chars().all(char::is_numeric) 
        {
            return device[..pos].to_string();
        }
    }
    device.to_string()
}

pub fn start_background_update_io(memory_usage: Arc<Mutex<DiskUsage>>) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        // Update process data every second

        // Lock ProcessData and update it
        let new_data = Disk_Usage();

        // Lock the mutex and replace its contents
        let mut data = memory_usage.lock().unwrap();
        *data = new_data;

        
    });
}