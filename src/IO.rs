#![allow(non_snake_case)]

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::{thread, time::Duration};

pub fn Disk_Usage() -> io::Result<()> 
{
    let root_disk = get_root_disk()?;
    let root_disk_without_partition = strip_partition_suffix(&root_disk);

    let root_disk_name = root_disk_without_partition.strip_prefix("/dev/").unwrap_or(&root_disk_without_partition);

    let file_path = "/proc/diskstats";
    let path = Path::new(file_path);

    loop 
    {
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);

        for line in reader.lines() 
        {
            let line = line?;
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

                    println!("Disk Stats for root device: {}", device_name);
                    println!("------------------------------------");
                    println!("  Reads Completed: {}", reads_completed);
                    println!("  Time Reading (ms): {}", time_reading);
                    println!("  Writes Completed: {}", writes_completed);
                    println!("  Time Writing (ms): {}", time_writing);
                    println!("  IO In Progress: {}", io_in_progress);
                    println!("  Time IO (ms): {}", time_io);
                    println!();
                }
            }
        }
        thread::sleep(Duration::from_secs(5));
    }
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
            println!("Found root disk: {}", root_disk);
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
