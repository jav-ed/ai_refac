use std::path::Path;
use std::env;

fn main() {
    println!("--- Testing rust-analyzer detection logic ---");
    
    // 1. Check if in PATH
    let path_result = which::which("rust-analyzer");
    println!("which('rust-analyzer'): {:?}", path_result);

    // 2. Check standard cargo bin location
    if let Some(home) = env::var_os("HOME") {
        let home_path = Path::new(&home);
        let cargo_bin = home_path.join(".cargo").join("bin").join("rust-analyzer");
        println!("Checking cargo_bin: {:?} (exists: {})", cargo_bin, cargo_bin.exists());
    } else {
        println!("HOME env var not found");
    }
}
