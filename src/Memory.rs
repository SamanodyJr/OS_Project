#![allow(non_snake_case)]

use sysinfo::{System, SystemExt, RefreshKind};

pub struct MemoryUsage {
    pub used: f64,
    pub free: f64,
    pub total: f64,
    pub used_swap: f64,
    pub free_swap: f64,
    pub total_swap: f64,
}

pub fn Mem_Usage() -> MemoryUsage {
    let mut sys = System::new_with_specifics(RefreshKind::new().with_memory());

    sys.refresh_memory();

    let total_memory = sys.total_memory() as f64;
    let used_memory = sys.used_memory() as f64;
    let free_memory = sys.free_memory() as f64;

    let total_swap = sys.total_swap() as f64;
    let used_swap = sys.used_swap() as f64;
    let free_swap = sys.free_swap() as f64;

    let memory_usage = MemoryUsage {
        used: used_memory/1024.0/1024.0/1024.0,
        free: free_memory/1024.0/1024.0/1024.0,
        total: total_memory/1024.0/1024.0/1024.0,
        used_swap: used_swap/1024.0/1024.0/1024.0,
        free_swap: free_swap/1024.0/1024.0/1024.0,
        total_swap: total_swap/1024.0/1024.0/1024.0,
    };

    memory_usage
}
// {
//     let mut sys = System::new_with_specifics(RefreshKind::new().with_memory());


//         sys.refresh_memory();

//         let total_memory = sys.total_memory() as f64;
//         let used_memory = sys.used_memory() as f64;
//         let free_memory = sys.free_memory() as f64;

//         let total_swap = sys.total_swap() as f64;
//         let used_swap = sys.used_swap() as f64;
//         let free_swap = sys.free_swap() as f64;

//         println!("Memory Usage:");

//         println!("  Used: {} %", (used_memory/total_memory)*100.0);
//         println!("  Free: {} %", (free_memory/total_memory)*100.0);

//         println!("  Total: {} KB", total_memory/1024.0);
//         println!("  Used: {} KB", used_memory/1024.0);
//         println!("  Free: {} KB", free_memory/1024.0);

//         println!("Swap Usage:");
        
//         println!("  Used: {} %", (used_swap/total_swap)*100.0);
//         println!("  Free: {} %", (free_swap/total_swap)*100.0);

//         println!("  Used: {} KB", used_swap/1024.0);
//         println!("  Free: {} KB", free_swap/1024.0);

    
// }