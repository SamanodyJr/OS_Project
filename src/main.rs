mod ctrl_proc;
pub use ctrl_proc::kill_process;
pub use ctrl_proc::terminate_process;
pub use ctrl_proc::killall;
pub use ctrl_proc::suspend_process;
pub use ctrl_proc::resume_process;

fn main() {
    match terminate_process(1760746) {
        Ok(_) => println!("Process killed successfully."),
        Err(err) => eprintln!("{}", err),
    }
}
