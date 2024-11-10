#![allow(non_snake_case)]

use sysinfo::{System, SystemExt, RefreshKind};
use std::{thread, time::Duration};

pub fn Mem_Usage() 
{
    let mut sys = System::new_with_specifics(RefreshKind::new().with_memory());

    loop 
    {
        // Refresh memory usage data
        sys.refresh_memory();

        // Get total and used memory in bytes
        let total_memory = sys.total_memory() as f64;
        let used_memory = sys.used_memory() as f64;
        let free_memory = sys.free_memory() as f64;

        // Get total and used swap memory in bytes
        let total_swap = sys.total_swap() as f64;
        let used_swap = sys.used_swap() as f64;
        let free_swap = sys.free_swap() as f64;

        // Display memory and swap usage
        println!("Memory Usage:");

        println!("  Used: {} %", (used_memory/total_memory)*100.0);
        println!("  Free: {} %", (free_memory/total_memory)*100.0);

        println!("  Total: {} KB", total_memory/1024.0);
        println!("  Used: {} KB", used_memory/1024.0);
        println!("  Free: {} KB", free_memory/1024.0);

        println!("Swap Usage:");
        
        println!("  Used: {} %", (used_swap/total_swap)*100.0);
        println!("  Free: {} %", (free_swap/total_swap)*100.0);

        println!("  Used: {} KB", used_swap/1024.0);
        println!("  Free: {} KB", free_swap/1024.0);

        thread::sleep(Duration::from_secs(5));
    }
}
