use scheduler::{set_priority, Which};

pub fn change_priority(pid: i32, priority: i32) -> Result<(), String> {
    
    if priority < -20 || priority > 19 {
        return Err(format!("Invalid priority value: {}. Priority must be between -20 and 19.", priority));
    }

    match set_priority(which::Process(pid), , priority) {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("Failed to set priority for process {}: {}", pid, err)),
    }
}
