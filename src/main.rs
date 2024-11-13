mod ctrl_proc;
pub use ctrl_proc::kill_process;
pub use ctrl_proc::terminate_process;
pub use ctrl_proc::killall;
pub use ctrl_proc::suspend_process;
pub use ctrl_proc::resume_process;
pub use ctrl_proc::change_priority;

fn main() {
    if change_priority(187997, 9) {
        println!("Priority changed successfully");
    } 
    else {
        println!("Failed to change priority");
    }
}
