use procfs::process::all_processes;
mod overview;
pub use overview::print_process;
fn main() {
    overview::print_process();
}