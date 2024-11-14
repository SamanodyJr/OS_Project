use procfs::process::all_processes;
mod overview;
pub use overview::print_process;
fn main() {
    print_process();
}