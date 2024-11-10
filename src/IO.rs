#![allow(non_snake_case)]

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::{thread, time::Duration};

pub fn Disk_Usage() -> io::Result<()> 
{
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
                let major = fields[0];
                let minor = fields[1];
                let device_name = fields[2];
                let reads_completed = fields[3];
                let reads_merged = fields[4];
                let sectors_read = fields[5];
                let time_reading = fields[6];
                let writes_completed = fields[7];
                let writes_merged = fields[8];
                let sectors_written = fields[9];
                let time_writing = fields[10];
                let io_in_progress = fields[11];
                let time_io = fields[12];

                println!("Disk Stats for device: {}", device_name);
                println!("------------------------------------");
                println!("  Major ID: {}", major);
                println!("  Minor ID: {}", minor);
                println!("  Reads Completed: {}", reads_completed);
                println!("  Reads Merged: {}", reads_merged);
                println!("  Sectors Read: {}", sectors_read);
                println!("  Time Reading (ms): {}", time_reading);
                println!("  Writes Completed: {}", writes_completed);
                println!("  Writes Merged: {}", writes_merged);
                println!("  Sectors Written: {}", sectors_written);
                println!("  Time Writing (ms): {}", time_writing);
                println!("  IO In Progress: {}", io_in_progress);
                println!("  Time IO (ms): {}", time_io);
                println!();
            }
        }
        thread::sleep(Duration::from_secs(5));
    }
}

